use open_eyes_core::error::OpenEyesError;
use open_eyes_core::DuckDbPool;

/// Download pending resources and load them into DuckDB as tables.
pub async fn load_pending(db: &DuckDbPool, max_size_mb: u64) -> Result<u64, OpenEyesError> {
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
                update_resource_status(db, id, &table_name, row_count, None);
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
    db: &DuckDbPool,
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

    // Download to temp file
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

    let ext = match format.to_uppercase().as_str() {
        "CSV" => "csv",
        "JSON" => "json",
        _ => {
            return Err(OpenEyesError::Ingestion(format!(
                "Unsupported format: {format}"
            )))
        }
    };

    let tmp_dir = tempfile::tempdir().map_err(|e| OpenEyesError::Other(format!("tmpdir: {e}")))?;
    let tmp_path = tmp_dir.path().join(format!("resource.{ext}"));
    std::fs::write(&tmp_path, &bytes)
        .map_err(|e| OpenEyesError::Other(format!("write tmp: {e}")))?;

    load_file_to_duckdb(db, &tmp_path, table_name, ext)?;

    // Get row count
    let count_result = db.query_json(&format!("SELECT COUNT(*) AS cnt FROM {table_name}"))?;
    let row_count = count_result
        .first()
        .and_then(|r| r["cnt"].as_i64())
        .unwrap_or(0);

    Ok(row_count)
}

fn load_file_to_duckdb(
    db: &DuckDbPool,
    path: &std::path::Path,
    table_name: &str,
    ext: &str,
) -> Result<(), OpenEyesError> {
    let path_str = path.to_string_lossy().replace('\'', "''");
    let sql = match ext {
        "csv" => format!(
            "CREATE OR REPLACE TABLE {table_name} AS SELECT * FROM read_csv_auto('{path_str}', ignore_errors=true)"
        ),
        "json" => format!(
            "CREATE OR REPLACE TABLE {table_name} AS SELECT * FROM read_json_auto('{path_str}', ignore_errors=true)"
        ),
        _ => return Err(OpenEyesError::Ingestion(format!("Unsupported: {ext}"))),
    };

    db.execute(&sql)?;
    Ok(())
}

fn update_resource_status(
    db: &DuckDbPool,
    id: &str,
    table_name: &str,
    row_count: i64,
    _err: Option<&str>,
) {
    // Get column names
    let cols = db
        .query_json(&format!(
            "SELECT column_name FROM information_schema.columns WHERE table_name = '{table_name}'"
        ))
        .unwrap_or_default();
    let col_names: Vec<String> = cols
        .iter()
        .filter_map(|c| c["column_name"].as_str().map(|s| s.to_string()))
        .collect();
    let cols_sql = format!(
        "[{}]",
        col_names
            .iter()
            .map(|c| format!("'{}'", c.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let sql = format!(
        "UPDATE oe_resources SET download_status = 'loaded', table_name = '{table_name}', row_count = {row_count}, column_names = {cols_sql} WHERE id = '{id}'",
        table_name = table_name.replace('\'', "''"),
        id = id.replace('\'', "''"),
    );
    if let Err(e) = db.execute(&sql) {
        tracing::warn!("Failed to update resource status for {id}: {e}");
    }
}

fn update_resource_status_error(db: &DuckDbPool, id: &str, error: &str) {
    let sql = format!(
        "UPDATE oe_resources SET download_status = 'error', error_message = '{err}' WHERE id = '{id}'",
        err = error.replace('\'', "''"),
        id = id.replace('\'', "''"),
    );
    if let Err(e) = db.execute(&sql) {
        tracing::warn!("Failed to update error status for {id}: {e}");
    }
}
