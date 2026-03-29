use open_eyes_core::error::OpenEyesError;
use open_eyes_core::DbPool;

/// Download pending resources and load them into SQLite as tables.
pub async fn load_pending(db: &DbPool, max_size_mb: u64) -> Result<u64, OpenEyesError> {
    let pending = db.query_json(
        "SELECT id, dataset_id, format, url FROM oe_resources WHERE download_status = 'pending'",
    )?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let mut loaded = 0u64;

    for row in &pending {
        let id = row["id"].as_str().unwrap_or_default();
        let dataset_id = row["dataset_id"].as_str().unwrap_or_default();
        let format = row["format"].as_str().unwrap_or_default();
        let url = row["url"].as_str().unwrap_or_default();

        if id.is_empty() || url.is_empty() {
            continue;
        }

        let table_name = make_table_name(dataset_id, id);

        match download_and_load(&client, db, id, url, format, &table_name, max_size_mb).await {
            Ok(row_count) => {
                tracing::info!("Loaded {table_name}: {row_count} rows");
                loaded += 1;
                update_resource_status(db, id, &table_name, row_count);
            }
            Err(e) => {
                tracing::warn!("Failed to load resource {id}: {e}");
                update_resource_status_error(db, id, &e.to_string());
            }
        }
    }

    tracing::info!("Load complete: {loaded} resources loaded");
    Ok(loaded)
}

fn make_table_name(dataset_id: &str, resource_id: &str) -> String {
    let ds = &dataset_id[..dataset_id.len().min(8)];
    let rs = &resource_id[..resource_id.len().min(8)];
    format!("data_{ds}_{rs}")
}

async fn download_and_load(
    client: &reqwest::Client,
    db: &DbPool,
    _resource_id: &str,
    url: &str,
    format: &str,
    table_name: &str,
    max_size_mb: u64,
) -> Result<i64, OpenEyesError> {
    // Check content-length first
    let head = client.head(url).send().await;
    if let Ok(resp) = &head {
        if let Some(len) = resp.content_length() {
            if len > max_size_mb * 1024 * 1024 {
                return Err(OpenEyesError::Ingestion(format!(
                    "Resource too large: {} MB (max {max_size_mb} MB)",
                    len / (1024 * 1024)
                )));
            }
        }
    }

    // Download
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(OpenEyesError::Ingestion(format!(
            "Download failed: HTTP {}",
            resp.status()
        )));
    }

    let bytes = resp.bytes().await?;
    if bytes.len() as u64 > max_size_mb * 1024 * 1024 {
        return Err(OpenEyesError::Ingestion("Resource too large".into()));
    }

    let normalized_fmt = format.rsplit('/').next().unwrap_or(format).to_uppercase();
    match normalized_fmt.as_str() {
        "CSV" => load_csv_to_sqlite(db, &bytes, table_name),
        "JSON" => load_json_to_sqlite(db, &bytes, table_name),
        _ => Err(OpenEyesError::Ingestion(format!(
            "Unsupported format: {format}"
        ))),
    }
}

fn load_csv_to_sqlite(db: &DbPool, bytes: &[u8], table_name: &str) -> Result<i64, OpenEyesError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .from_reader(bytes);

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| OpenEyesError::Ingestion(format!("CSV header error: {e}")))?
        .iter()
        .map(|h| sanitize_column_name(h))
        .collect();

    if headers.is_empty() {
        return Err(OpenEyesError::Ingestion("CSV has no headers".into()));
    }

    // Drop existing table and create new one with all TEXT columns
    let col_defs = headers
        .iter()
        .map(|h| format!("\"{h}\" TEXT"))
        .collect::<Vec<_>>()
        .join(", ");
    db.execute_batch(&format!(
        "DROP TABLE IF EXISTS {table_name}; CREATE TABLE {table_name} ({col_defs});"
    ))?;

    // Build parameterized INSERT
    let placeholders = headers.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {table_name} VALUES ({placeholders})");

    // Batch insert rows
    let mut row_count = 0i64;
    db.execute("BEGIN")?;

    for result in reader.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue, // skip bad rows
        };
        let values: Vec<String> = (0..headers.len())
            .map(|i| record.get(i).unwrap_or("").to_string())
            .collect();
        let params: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
        if db.execute_with_params(&insert_sql, &params).is_ok() {
            row_count += 1;
        }
    }

    db.execute("COMMIT")?;
    Ok(row_count)
}

fn load_json_to_sqlite(
    db: &DbPool,
    bytes: &[u8],
    table_name: &str,
) -> Result<i64, OpenEyesError> {
    let text = String::from_utf8_lossy(bytes);
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| OpenEyesError::Ingestion(format!("JSON parse error: {e}")))?;

    // Handle both array of objects and single object
    let rows: Vec<&serde_json::Map<String, serde_json::Value>> = match &parsed {
        serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_object()).collect(),
        serde_json::Value::Object(obj) => vec![obj],
        _ => {
            return Err(OpenEyesError::Ingestion(
                "JSON must be an array or object".into(),
            ))
        }
    };

    if rows.is_empty() {
        return Err(OpenEyesError::Ingestion("JSON has no rows".into()));
    }

    // Collect all unique keys across all rows
    let mut headers: Vec<String> = Vec::new();
    for row in &rows {
        for key in row.keys() {
            let col = sanitize_column_name(key);
            if !headers.contains(&col) {
                headers.push(col);
            }
        }
    }

    let col_defs = headers
        .iter()
        .map(|h| format!("\"{h}\" TEXT"))
        .collect::<Vec<_>>()
        .join(", ");
    db.execute_batch(&format!(
        "DROP TABLE IF EXISTS {table_name}; CREATE TABLE {table_name} ({col_defs});"
    ))?;

    let placeholders = headers.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {table_name} VALUES ({placeholders})");

    db.execute("BEGIN")?;
    let mut row_count = 0i64;

    for row in &rows {
        let values: Vec<String> = headers
            .iter()
            .map(|h| {
                // Find the original key that sanitizes to this header
                row.iter()
                    .find(|(k, _)| sanitize_column_name(k) == *h)
                    .map(|(_, v)| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        let params: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
        if db.execute_with_params(&insert_sql, &params).is_ok() {
            row_count += 1;
        }
    }

    db.execute("COMMIT")?;
    Ok(row_count)
}

fn sanitize_column_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        "col".to_string()
    } else if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{sanitized}")
    } else {
        sanitized
    }
}

fn update_resource_status(db: &DbPool, id: &str, table_name: &str, row_count: i64) {
    // Get column names via PRAGMA
    let cols = db
        .query_json(&format!("PRAGMA table_info({table_name})"))
        .unwrap_or_default();
    let col_names: Vec<String> = cols
        .iter()
        .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
        .collect();
    let cols_json = serde_json::to_string(&col_names).unwrap_or_else(|_| "[]".to_string());

    let sql = format!(
        "UPDATE oe_resources SET download_status = 'loaded', table_name = '{table_name}', row_count = {row_count}, column_names = '{cols_json}' WHERE id = '{id}'",
        table_name = table_name.replace('\'', "''"),
        cols_json = cols_json.replace('\'', "''"),
        id = id.replace('\'', "''"),
    );
    if let Err(e) = db.execute(&sql) {
        tracing::warn!("Failed to update resource status for {id}: {e}");
    }
}

fn update_resource_status_error(db: &DbPool, id: &str, error: &str) {
    let sql = format!(
        "UPDATE oe_resources SET download_status = 'error', error_message = '{err}' WHERE id = '{id}'",
        err = error.replace('\'', "''"),
        id = id.replace('\'', "''"),
    );
    if let Err(e) = db.execute(&sql) {
        tracing::warn!("Failed to update error status for {id}: {e}");
    }
}
