use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DatasetSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub organization: String,
    pub license: String,
    pub tags: Vec<String>,
    pub resource_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetDetail {
    pub id: String,
    pub title: String,
    pub description: String,
    pub organization: String,
    pub license: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ckan_url: String,
    pub created_at: String,
    pub modified_at: String,
    pub resources: Vec<ResourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceInfo {
    pub id: String,
    pub name: String,
    pub format: String,
    pub table_name: String,
    pub row_count: i64,
    pub download_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetSearchResult {
    pub datasets: Vec<DatasetSummary>,
    pub total: u64,
}

#[server]
pub async fn search_datasets(
    query: String,
    offset: u64,
    limit: u64,
) -> Result<DatasetSearchResult, ServerFnError> {
    use super::duckdb_state::DuckDbState;
    let db: DuckDbState = dioxus_fullstack::FullstackContext::extract().await?;

    let (count_sql, data_sql);
    if query.is_empty() {
        count_sql = "SELECT COUNT(*) AS cnt FROM oe_datasets".to_string();
        data_sql = format!(
            "SELECT d.id, d.title, COALESCE(d.description, '') AS description, COALESCE(d.organization, '') AS organization, COALESCE(d.license, '') AS license, CAST(d.tags AS VARCHAR) AS tags_str, (SELECT COUNT(*) FROM oe_resources r WHERE r.dataset_id = d.id) AS resource_count FROM oe_datasets d ORDER BY d.ingested_at DESC LIMIT {limit} OFFSET {offset}"
        );
    } else {
        let q = query.replace('\'', "''");
        let where_clause = format!("WHERE d.title ILIKE '%{q}%' OR d.description ILIKE '%{q}%' OR d.organization ILIKE '%{q}%'");
        count_sql = format!("SELECT COUNT(*) AS cnt FROM oe_datasets d {where_clause}");
        data_sql = format!(
            "SELECT d.id, d.title, COALESCE(d.description, '') AS description, COALESCE(d.organization, '') AS organization, COALESCE(d.license, '') AS license, CAST(d.tags AS VARCHAR) AS tags_str, (SELECT COUNT(*) FROM oe_resources r WHERE r.dataset_id = d.id) AS resource_count FROM oe_datasets d {where_clause} ORDER BY d.ingested_at DESC LIMIT {limit} OFFSET {offset}"
        );
    }

    let count_rows =
        db.0.query_json(&count_sql)
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let total = count_rows
        .first()
        .and_then(|r| r["cnt"].as_u64())
        .unwrap_or(0);

    let rows =
        db.0.query_json(&data_sql)
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    let datasets = rows
        .iter()
        .map(|r| {
            let tags_str = r["tags_str"].as_str().unwrap_or("");
            let tags = parse_array_str(tags_str);
            DatasetSummary {
                id: r["id"].as_str().unwrap_or("").to_string(),
                title: r["title"].as_str().unwrap_or("").to_string(),
                description: r["description"].as_str().unwrap_or("").to_string(),
                organization: r["organization"].as_str().unwrap_or("").to_string(),
                license: r["license"].as_str().unwrap_or("").to_string(),
                tags,
                resource_count: r["resource_count"].as_u64().unwrap_or(0),
            }
        })
        .collect();

    Ok(DatasetSearchResult { datasets, total })
}

#[server]
pub async fn fetch_dataset_detail(dataset_id: String) -> Result<DatasetDetail, ServerFnError> {
    use super::duckdb_state::DuckDbState;
    let db: DuckDbState = dioxus_fullstack::FullstackContext::extract().await?;
    let did = dataset_id.replace('\'', "''");

    let rows = db.0.query_json(&format!(
        "SELECT id, title, COALESCE(description, '') AS description, COALESCE(organization, '') AS organization, COALESCE(license, '') AS license, CAST(categories AS VARCHAR) AS categories_str, CAST(tags AS VARCHAR) AS tags_str, COALESCE(ckan_url, '') AS ckan_url, COALESCE(CAST(created_at AS VARCHAR), '') AS created_at, COALESCE(CAST(modified_at AS VARCHAR), '') AS modified_at FROM oe_datasets WHERE id = '{did}'"
    )).map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = rows
        .first()
        .ok_or_else(|| ServerFnError::new("Dataset not found"))?;

    let categories = parse_array_str(row["categories_str"].as_str().unwrap_or(""));
    let tags = parse_array_str(row["tags_str"].as_str().unwrap_or(""));

    let res_rows = db.0.query_json(&format!(
        "SELECT id, COALESCE(name, '') AS name, COALESCE(format, '') AS format, COALESCE(table_name, '') AS table_name, COALESCE(row_count, 0) AS row_count, download_status FROM oe_resources WHERE dataset_id = '{did}'"
    )).map_err(|e| ServerFnError::new(e.to_string()))?;

    let resources = res_rows
        .iter()
        .map(|r| ResourceInfo {
            id: r["id"].as_str().unwrap_or("").to_string(),
            name: r["name"].as_str().unwrap_or("").to_string(),
            format: r["format"].as_str().unwrap_or("").to_string(),
            table_name: r["table_name"].as_str().unwrap_or("").to_string(),
            row_count: r["row_count"].as_i64().unwrap_or(0),
            download_status: r["download_status"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(DatasetDetail {
        id: row["id"].as_str().unwrap_or("").to_string(),
        title: row["title"].as_str().unwrap_or("").to_string(),
        description: row["description"].as_str().unwrap_or("").to_string(),
        organization: row["organization"].as_str().unwrap_or("").to_string(),
        license: row["license"].as_str().unwrap_or("").to_string(),
        categories,
        tags,
        ckan_url: row["ckan_url"].as_str().unwrap_or("").to_string(),
        created_at: row["created_at"].as_str().unwrap_or("").to_string(),
        modified_at: row["modified_at"].as_str().unwrap_or("").to_string(),
        resources,
    })
}

#[server]
pub async fn fetch_table_preview(
    table_name: String,
) -> Result<Vec<serde_json::Value>, ServerFnError> {
    use super::duckdb_state::DuckDbState;
    let db: DuckDbState = dioxus_fullstack::FullstackContext::extract().await?;

    if !table_name.starts_with("data_") || table_name.contains(';') || table_name.contains('\'') {
        return Err(ServerFnError::new("Invalid table name"));
    }

    let rows =
        db.0.query_json(&format!("SELECT * FROM {table_name} LIMIT 20"))
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows)
}

fn parse_array_str(s: &str) -> Vec<String> {
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
