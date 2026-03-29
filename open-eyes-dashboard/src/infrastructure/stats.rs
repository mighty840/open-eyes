use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverviewStats {
    pub total_datasets: u64,
    pub total_tables: u64,
    pub total_rows: u64,
    pub total_resources: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDataset {
    pub id: String,
    pub title: String,
    pub organization: String,
    pub ingested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IngestionStatus {
    pub total_datasets: u64,
    pub loaded_resources: u64,
    pub pending_resources: u64,
    pub error_resources: u64,
}

#[server]
pub async fn fetch_overview_stats() -> Result<OverviewStats, ServerFnError> {
    use super::db_state::DbState;
    let db: DbState = dioxus_fullstack::FullstackContext::extract().await?;

    let datasets =
        db.0.query_json("SELECT COUNT(*) AS cnt FROM oe_datasets")
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let total_datasets = datasets
        .first()
        .and_then(|r| r["cnt"].as_u64())
        .unwrap_or(0);

    let tables = db.0.query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE table_name IS NOT NULL AND download_status = 'loaded'").map_err(|e| ServerFnError::new(e.to_string()))?;
    let total_tables = tables.first().and_then(|r| r["cnt"].as_u64()).unwrap_or(0);

    let rows = db.0.query_json("SELECT COALESCE(SUM(row_count), 0) AS cnt FROM oe_resources WHERE download_status = 'loaded'").map_err(|e| ServerFnError::new(e.to_string()))?;
    let total_rows = rows.first().and_then(|r| r["cnt"].as_u64()).unwrap_or(0);

    let resources =
        db.0.query_json("SELECT COUNT(*) AS cnt FROM oe_resources")
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let total_resources = resources
        .first()
        .and_then(|r| r["cnt"].as_u64())
        .unwrap_or(0);

    Ok(OverviewStats {
        total_datasets,
        total_tables,
        total_rows,
        total_resources,
    })
}

#[server]
pub async fn fetch_popular_categories() -> Result<Vec<CategoryCount>, ServerFnError> {
    use super::db_state::DbState;
    let db: DbState = dioxus_fullstack::FullstackContext::extract().await?;

    let rows = db.0.query_json(
        "SELECT j.value AS name, COUNT(*) AS count FROM oe_datasets d, json_each(d.categories) j GROUP BY j.value ORDER BY count DESC LIMIT 10"
    ).map_err(|e| ServerFnError::new(e.to_string()))?;

    let cats = rows
        .iter()
        .map(|r| CategoryCount {
            name: r["name"].as_str().unwrap_or("").to_string(),
            count: r["count"].as_u64().unwrap_or(0),
        })
        .collect();

    Ok(cats)
}

#[server]
pub async fn fetch_recent_datasets() -> Result<Vec<RecentDataset>, ServerFnError> {
    use super::db_state::DbState;
    let db: DbState = dioxus_fullstack::FullstackContext::extract().await?;

    let rows = db.0.query_json(
        "SELECT id, title, COALESCE(organization, '') AS organization, COALESCE(ingested_at, '') AS ingested_at FROM oe_datasets ORDER BY ingested_at DESC LIMIT 10"
    ).map_err(|e| ServerFnError::new(e.to_string()))?;

    let datasets = rows
        .iter()
        .map(|r| RecentDataset {
            id: r["id"].as_str().unwrap_or("").to_string(),
            title: r["title"].as_str().unwrap_or("").to_string(),
            organization: r["organization"].as_str().unwrap_or("").to_string(),
            ingested_at: r["ingested_at"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(datasets)
}

#[server]
pub async fn fetch_ingestion_status() -> Result<IngestionStatus, ServerFnError> {
    use super::db_state::DbState;
    let db: DbState = dioxus_fullstack::FullstackContext::extract().await?;

    let datasets =
        db.0.query_json("SELECT COUNT(*) AS cnt FROM oe_datasets")
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let total_datasets = datasets
        .first()
        .and_then(|r| r["cnt"].as_u64())
        .unwrap_or(0);

    let loaded = db
        .0
        .query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'loaded'")
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let loaded_resources = loaded.first().and_then(|r| r["cnt"].as_u64()).unwrap_or(0);

    let pending = db
        .0
        .query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'pending'")
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let pending_resources = pending.first().and_then(|r| r["cnt"].as_u64()).unwrap_or(0);

    let errors =
        db.0.query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'error'")
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let error_resources = errors.first().and_then(|r| r["cnt"].as_u64()).unwrap_or(0);

    Ok(IngestionStatus {
        total_datasets,
        loaded_resources,
        pending_resources,
        error_resources,
    })
}
