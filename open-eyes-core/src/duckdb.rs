use std::path::Path;
use std::sync::{Arc, Mutex};

use arrow::util::display::{ArrayFormatter, FormatOptions};
use duckdb::Connection;

use crate::error::OpenEyesError;
use crate::models::{ColumnSchema, TableInfo};

/// Thread-safe DuckDB connection pool (single connection behind a mutex).
/// DuckDB is embedded so this is the standard approach.
#[derive(Clone)]
pub struct DuckDbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DuckDbPool {
    /// Open or create a DuckDB database at the given path.
    pub fn open(path: &Path) -> Result<Self, OpenEyesError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OpenEyesError::Other(format!("Failed to create data dir: {e}")))?;
        }
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize the metadata schema (oe_datasets, oe_resources, oe_chat_messages).
    pub fn init_schema(&self) -> Result<(), OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS oe_datasets (
                id VARCHAR PRIMARY KEY,
                title VARCHAR NOT NULL,
                title_de VARCHAR,
                description VARCHAR,
                description_de VARCHAR,
                organization VARCHAR,
                categories VARCHAR[],
                tags VARCHAR[],
                license VARCHAR,
                source_portal VARCHAR DEFAULT 'govdata.de',
                ckan_url VARCHAR,
                created_at TIMESTAMP,
                modified_at TIMESTAMP,
                ingested_at TIMESTAMP DEFAULT now()
            );

            CREATE TABLE IF NOT EXISTS oe_resources (
                id VARCHAR PRIMARY KEY,
                dataset_id VARCHAR NOT NULL REFERENCES oe_datasets(id),
                name VARCHAR,
                format VARCHAR,
                url VARCHAR NOT NULL,
                table_name VARCHAR,
                row_count BIGINT,
                column_names VARCHAR[],
                download_status VARCHAR DEFAULT 'pending',
                error_message VARCHAR,
                ingested_at TIMESTAMP DEFAULT now()
            );

            CREATE SEQUENCE IF NOT EXISTS oe_chat_messages_seq;

            CREATE TABLE IF NOT EXISTS oe_chat_messages (
                id BIGINT PRIMARY KEY DEFAULT nextval('oe_chat_messages_seq'),
                session_id VARCHAR NOT NULL,
                role VARCHAR NOT NULL,
                content VARCHAR NOT NULL,
                sql_query VARCHAR,
                chart_spec VARCHAR,
                created_at TIMESTAMP DEFAULT now()
            );
            "#,
        )?;
        Ok(())
    }

    /// Execute a read-only SQL query and return results as JSON rows.
    pub fn query_json(&self, sql: &str) -> Result<Vec<serde_json::Value>, OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;
        let mut stmt = conn.prepare(sql)?;

        // Use query_arrow to get schema + data in one shot
        let arrow_result = stmt.query_arrow([])?;
        let schema = arrow_result.get_schema();
        let column_names: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();

        let batches: Vec<arrow::array::RecordBatch> = arrow_result.collect();

        let mut results = Vec::new();
        for batch in &batches {
            for row_idx in 0..batch.num_rows() {
                let mut map = serde_json::Map::new();
                for (col_idx, name) in column_names.iter().enumerate() {
                    let col = batch.column(col_idx);
                    let json_val = arrow_column_to_json(col, row_idx);
                    map.insert(name.clone(), json_val);
                }
                results.push(serde_json::Value::Object(map));
            }
        }
        Ok(results)
    }

    /// Execute a non-query SQL statement (INSERT, CREATE TABLE, etc.)
    pub fn execute(&self, sql: &str) -> Result<usize, OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;
        let count = conn.execute(sql, [])?;
        Ok(count)
    }

    /// Execute a batch of SQL statements.
    pub fn execute_batch(&self, sql: &str) -> Result<(), OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;
        conn.execute_batch(sql)?;
        Ok(())
    }

    /// Get all loaded tables with their metadata for LLM context.
    pub fn get_table_infos(&self) -> Result<Vec<TableInfo>, OpenEyesError> {
        let sql = r#"
            SELECT r.table_name, d.title, d.description, r.row_count,
                   CAST(r.column_names AS VARCHAR) AS column_names_str
            FROM oe_resources r
            JOIN oe_datasets d ON r.dataset_id = d.id
            WHERE r.table_name IS NOT NULL AND r.download_status = 'loaded'
            ORDER BY r.row_count DESC
        "#;
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            let table_name: String = row.get(0)?;
            let dataset_title: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let row_count: i64 = row.get::<_, Option<i64>>(3)?.unwrap_or(0);
            let cols_str: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
            let column_names = parse_duckdb_array_string(&cols_str);
            Ok(TableInfo {
                table_name,
                dataset_title,
                description,
                row_count,
                column_names,
            })
        })?;

        let mut infos = Vec::new();
        for row in rows {
            infos.push(row?);
        }
        Ok(infos)
    }

    /// Get column schemas for specific tables.
    pub fn get_column_schemas(
        &self,
        table_names: &[String],
    ) -> Result<Vec<(String, Vec<ColumnSchema>)>, OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;

        let mut result = Vec::new();
        for table_name in table_names {
            let sql = format!("DESCRIBE {table_name}");
            match conn.prepare(&sql) {
                Ok(mut stmt) => {
                    let rows = stmt.query_map([], |row| {
                        let name: String = row.get(0)?;
                        let data_type: String = row.get(1)?;
                        Ok(ColumnSchema { name, data_type })
                    })?;
                    let schemas: Vec<ColumnSchema> = rows.filter_map(|r| r.ok()).collect();
                    result.push((table_name.clone(), schemas));
                }
                Err(_) => continue,
            }
        }
        Ok(result)
    }
}

/// Parse a DuckDB array string like "[col1, col2, col3]" into a Vec<String>.
fn parse_duckdb_array_string(s: &str) -> Vec<String> {
    let s = s.trim();
    let s = s.strip_prefix('[').unwrap_or(s);
    let s = s.strip_suffix(']').unwrap_or(s);
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|part| part.trim().trim_matches('\'').trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn arrow_column_to_json(col: &arrow::array::ArrayRef, row: usize) -> serde_json::Value {
    use arrow::array::*;
    use arrow::datatypes::DataType;

    if col.is_null(row) {
        return serde_json::Value::Null;
    }

    match col.data_type() {
        DataType::Boolean => {
            let arr = col.as_any().downcast_ref::<BooleanArray>();
            arr.map(|a| serde_json::Value::Bool(a.value(row)))
                .unwrap_or(serde_json::Value::Null)
        }
        DataType::Int8 => col
            .as_any()
            .downcast_ref::<Int8Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as i64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::Int16 => col
            .as_any()
            .downcast_ref::<Int16Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as i64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::Int32 => col
            .as_any()
            .downcast_ref::<Int32Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as i64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::Int64 => col
            .as_any()
            .downcast_ref::<Int64Array>()
            .map(|a| serde_json::Value::Number(a.value(row).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::UInt8 => col
            .as_any()
            .downcast_ref::<UInt8Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as u64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::UInt16 => col
            .as_any()
            .downcast_ref::<UInt16Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as u64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::UInt32 => col
            .as_any()
            .downcast_ref::<UInt32Array>()
            .map(|a| serde_json::Value::Number((a.value(row) as u64).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::UInt64 => col
            .as_any()
            .downcast_ref::<UInt64Array>()
            .map(|a| serde_json::Value::Number(a.value(row).into()))
            .unwrap_or(serde_json::Value::Null),
        DataType::Float32 => {
            let arr = col.as_any().downcast_ref::<Float32Array>();
            arr.and_then(|a| serde_json::Number::from_f64(a.value(row) as f64))
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        DataType::Float64 => {
            let arr = col.as_any().downcast_ref::<Float64Array>();
            arr.and_then(|a| serde_json::Number::from_f64(a.value(row)))
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        DataType::Utf8 => {
            let arr = col.as_any().downcast_ref::<StringArray>();
            arr.map(|a| serde_json::Value::String(a.value(row).to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        DataType::LargeUtf8 => {
            let arr = col.as_any().downcast_ref::<LargeStringArray>();
            arr.map(|a| serde_json::Value::String(a.value(row).to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        DataType::Decimal128(_, scale) => {
            let arr = col.as_any().downcast_ref::<Decimal128Array>();
            arr.and_then(|a| {
                let raw = a.value(row) as f64;
                let divisor = 10f64.powi(*scale as i32);
                serde_json::Number::from_f64(raw / divisor)
            })
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
        }
        DataType::Decimal256(_, _scale) => {
            // Best-effort: format as string, parse to f64
            let formatter = ArrayFormatter::try_new(col.as_ref(), &FormatOptions::default());
            match formatter {
                Ok(f) => {
                    let s = f.value(row).to_string();
                    s.parse::<f64>()
                        .ok()
                        .and_then(serde_json::Number::from_f64)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::String(s))
                }
                Err(_) => serde_json::Value::Null,
            }
        }
        _ => {
            // Fallback: try to format as string
            let formatter = ArrayFormatter::try_new(col.as_ref(), &FormatOptions::default());
            match formatter {
                Ok(f) => serde_json::Value::String(f.value(row).to_string()),
                Err(_) => serde_json::Value::String(format!("<{}>", col.data_type())),
            }
        }
    }
}

/// Build an ECharts option JSON string from chart config and data.
pub fn build_echart_option(
    chart_type: &crate::models::ChartType,
    config: &crate::models::ChartConfig,
    data: &[serde_json::Value],
) -> String {
    use crate::models::ChartType;

    match chart_type {
        ChartType::Bar | ChartType::Line => build_cartesian_chart(chart_type, config, data),
        ChartType::Pie => build_pie_chart(config, data),
        ChartType::Scatter => build_scatter_chart(config, data),
        ChartType::Table | ChartType::None => "{}".to_string(),
    }
}

fn build_cartesian_chart(
    chart_type: &crate::models::ChartType,
    config: &crate::models::ChartConfig,
    data: &[serde_json::Value],
) -> String {
    let chart_type_str = match chart_type {
        crate::models::ChartType::Line => "line",
        _ => "bar",
    };

    let x_values: Vec<String> = data
        .iter()
        .filter_map(|row| {
            row.get(&config.x_field).map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string())
            })
        })
        .collect();

    let y_values: Vec<f64> = data
        .iter()
        .filter_map(|row| {
            row.get(&config.y_field).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
        })
        .collect();

    serde_json::json!({
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#e0e0e0" } },
        "tooltip": { "trigger": "axis" },
        "xAxis": { "type": "category", "data": x_values, "axisLabel": { "color": "#aaa", "rotate": 30 } },
        "yAxis": { "type": "value", "axisLabel": { "color": "#aaa" } },
        "series": [{ "type": chart_type_str, "data": y_values, "itemStyle": { "color": "#5470c6" } }],
        "backgroundColor": "transparent",
        "grid": { "left": "10%", "right": "5%", "bottom": "15%" }
    })
    .to_string()
}

fn build_pie_chart(config: &crate::models::ChartConfig, data: &[serde_json::Value]) -> String {
    let pie_data: Vec<serde_json::Value> = data
        .iter()
        .filter_map(|row| {
            let name = row.get(&config.x_field).map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string())
            });
            let value = row.get(&config.y_field).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
            match (name, value) {
                (Some(n), Some(v)) => Some(serde_json::json!({ "name": n, "value": v })),
                _ => None,
            }
        })
        .collect();

    serde_json::json!({
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#e0e0e0" } },
        "tooltip": { "trigger": "item" },
        "series": [{ "type": "pie", "radius": "60%", "data": pie_data, "label": { "color": "#ccc" } }],
        "backgroundColor": "transparent"
    })
    .to_string()
}

fn build_scatter_chart(config: &crate::models::ChartConfig, data: &[serde_json::Value]) -> String {
    let scatter_data: Vec<Vec<f64>> = data
        .iter()
        .filter_map(|row| {
            let x = row.get(&config.x_field).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
            let y = row.get(&config.y_field).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
            match (x, y) {
                (Some(x), Some(y)) => Some(vec![x, y]),
                _ => None,
            }
        })
        .collect();

    serde_json::json!({
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#e0e0e0" } },
        "tooltip": { "trigger": "item" },
        "xAxis": { "type": "value", "axisLabel": { "color": "#aaa" } },
        "yAxis": { "type": "value", "axisLabel": { "color": "#aaa" } },
        "series": [{ "type": "scatter", "data": scatter_data, "itemStyle": { "color": "#5470c6" } }],
        "backgroundColor": "transparent",
        "grid": { "left": "10%", "right": "5%", "bottom": "10%" }
    })
    .to_string()
}
