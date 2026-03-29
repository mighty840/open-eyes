use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::OpenEyesError;
use crate::models::{ColumnSchema, TableInfo};

/// Thread-safe SQLite connection pool (single connection behind a mutex).
#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    /// Open or create a SQLite database at the given path.
    pub fn open(path: &Path) -> Result<Self, OpenEyesError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OpenEyesError::Other(format!("Failed to create data dir: {e}")))?;
        }
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
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
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                title_de TEXT,
                description TEXT,
                description_de TEXT,
                organization TEXT,
                categories TEXT DEFAULT '[]',
                tags TEXT DEFAULT '[]',
                license TEXT,
                source_portal TEXT DEFAULT 'govdata.de',
                ckan_url TEXT,
                created_at TEXT,
                modified_at TEXT,
                ingested_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS oe_resources (
                id TEXT PRIMARY KEY,
                dataset_id TEXT NOT NULL REFERENCES oe_datasets(id),
                name TEXT,
                format TEXT,
                url TEXT NOT NULL,
                table_name TEXT,
                row_count INTEGER,
                column_names TEXT DEFAULT '[]',
                download_status TEXT DEFAULT 'pending',
                error_message TEXT,
                ingested_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS oe_chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                sql_query TEXT,
                chart_spec TEXT,
                created_at TEXT DEFAULT (datetime('now'))
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

        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rows = stmt.query_map([], |row| {
            let mut map = serde_json::Map::new();
            for (i, name) in column_names.iter().enumerate() {
                let val = sqlite_value_to_json(row, i);
                map.insert(name.clone(), val);
            }
            Ok(serde_json::Value::Object(map))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
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
                   r.column_names AS column_names_str
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
            let cols_str: String = row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "[]".to_string());
            let column_names = parse_json_array_string(&cols_str);
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

    /// Get column schemas for specific tables using PRAGMA table_info.
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
            // Validate table name to prevent injection
            if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let sql = format!("PRAGMA table_info({table_name})");
            match conn.prepare(&sql) {
                Ok(mut stmt) => {
                    let rows = stmt.query_map([], |row| {
                        let name: String = row.get(1)?; // column name
                        let data_type: String = row.get(2)?; // column type
                        Ok(ColumnSchema { name, data_type })
                    })?;
                    let schemas: Vec<ColumnSchema> = rows.filter_map(|r| r.ok()).collect();
                    if !schemas.is_empty() {
                        result.push((table_name.clone(), schemas));
                    }
                }
                Err(_) => continue,
            }
        }
        Ok(result)
    }

    /// Execute a parameterized insert with values.
    pub fn execute_with_params(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
    ) -> Result<usize, OpenEyesError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| OpenEyesError::Other(e.to_string()))?;
        let count = conn.execute(sql, params)?;
        Ok(count)
    }
}

/// Parse a JSON array string like '["col1", "col2"]' into a Vec<String>.
fn parse_json_array_string(s: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(s).unwrap_or_else(|_| {
        // Fallback: try the old bracket format [col1, col2]
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
    })
}

fn sqlite_value_to_json(row: &rusqlite::Row, idx: usize) -> serde_json::Value {
    // Try types in order: integer, real, text, null
    if let Ok(val) = row.get::<_, i64>(idx) {
        return serde_json::Value::Number(val.into());
    }
    if let Ok(val) = row.get::<_, f64>(idx) {
        return serde_json::Number::from_f64(val)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null);
    }
    if let Ok(val) = row.get::<_, String>(idx) {
        return serde_json::Value::String(val);
    }
    serde_json::Value::Null
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
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#1a1a2e" } },
        "tooltip": { "trigger": "axis" },
        "xAxis": { "type": "category", "data": x_values, "axisLabel": { "color": "#4a4a5e", "rotate": 30 } },
        "yAxis": { "type": "value", "axisLabel": { "color": "#4a4a5e" } },
        "series": [{ "type": chart_type_str, "data": y_values, "itemStyle": { "color": "#1e3a5f" } }],
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
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#1a1a2e" } },
        "tooltip": { "trigger": "item" },
        "series": [{ "type": "pie", "radius": "60%", "data": pie_data, "label": { "color": "#4a4a5e" } }],
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
        "title": { "text": config.title, "left": "center", "textStyle": { "color": "#1a1a2e" } },
        "tooltip": { "trigger": "item" },
        "xAxis": { "type": "value", "axisLabel": { "color": "#4a4a5e" } },
        "yAxis": { "type": "value", "axisLabel": { "color": "#4a4a5e" } },
        "series": [{ "type": "scatter", "data": scatter_data, "itemStyle": { "color": "#1e3a5f" } }],
        "backgroundColor": "transparent",
        "grid": { "left": "10%", "right": "5%", "bottom": "10%" }
    })
    .to_string()
}
