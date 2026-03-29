use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedChart {
    pub id: i64,
    pub session_id: String,
    pub content: String,
    pub sql_query: String,
    pub chart_spec: String,
    pub created_at: String,
}

#[server]
pub async fn fetch_saved_charts() -> Result<Vec<SavedChart>, ServerFnError> {
    use super::db_state::DbState;
    let db: DbState = dioxus_fullstack::FullstackContext::extract().await?;

    let rows = db.0.query_json(
        "SELECT id, session_id, content, COALESCE(sql_query, '') AS sql_query, chart_spec, COALESCE(created_at, '') AS created_at FROM oe_chat_messages WHERE chart_spec IS NOT NULL AND chart_spec != '' AND chart_spec != '{}' ORDER BY created_at DESC LIMIT 50"
    ).map_err(|e| ServerFnError::new(e.to_string()))?;

    let charts = rows
        .iter()
        .map(|r| SavedChart {
            id: r["id"].as_i64().unwrap_or(0),
            session_id: r["session_id"].as_str().unwrap_or("").to_string(),
            content: r["content"].as_str().unwrap_or("").to_string(),
            sql_query: r["sql_query"].as_str().unwrap_or("").to_string(),
            chart_spec: r["chart_spec"].as_str().unwrap_or("").to_string(),
            created_at: r["created_at"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(charts)
}
