use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatResult {
    pub summary: String,
    pub chart_type: String,
    pub chart_option_json: String,
    pub data: Vec<serde_json::Value>,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryMessage {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub sql_query: String,
    pub chart_spec: String,
}

#[server]
pub async fn ask_question(
    question: String,
    session_id: String,
    language: String,
) -> Result<ChatResult, ServerFnError> {
    use super::duckdb_state::DuckDbState;
    use super::llm_state::LlmState;

    let db: DuckDbState = dioxus_fullstack::FullstackContext::extract().await?;
    let llm: LlmState = dioxus_fullstack::FullstackContext::extract().await?;

    // Save user message
    let q_escaped = question.replace('\'', "''");
    let sid = session_id.replace('\'', "''");
    db.0.execute(&format!(
        "INSERT INTO oe_chat_messages (session_id, role, content) VALUES ('{sid}', 'user', '{q_escaped}')"
    )).map_err(|e| ServerFnError::new(e.to_string()))?;

    // Phase 1: Get available tables
    let table_infos = tokio::task::spawn_blocking({
        let db = db.0.clone();
        move || db.get_table_infos()
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if table_infos.is_empty() {
        return Ok(ChatResult {
            summary: if language == "de" {
                "Keine Datentabellen verfügbar. Bitte führen Sie zuerst die Datenaufnahme aus."
                    .to_string()
            } else {
                "No data tables available. Please run the ingestion pipeline first.".to_string()
            },
            ..Default::default()
        });
    }

    // Phase 1: Select relevant tables
    let selected_tables = llm
        .0
        .select_tables(&question, &table_infos, &language)
        .await
        .map_err(|e| ServerFnError::new(format!("Table selection failed: {e}")))?;

    if selected_tables.is_empty() {
        return Ok(ChatResult {
            summary: if language == "de" {
                "Keine relevanten Tabellen für Ihre Frage gefunden.".to_string()
            } else {
                "No relevant tables found for your question.".to_string()
            },
            ..Default::default()
        });
    }

    // Phase 2: Get schemas and generate SQL
    let schemas = tokio::task::spawn_blocking({
        let db = db.0.clone();
        let tables = selected_tables.clone();
        move || db.get_column_schemas(&tables)
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let sql = llm
        .0
        .generate_sql(&question, &schemas, &language)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL generation failed: {e}")))?;

    // Execute SQL
    let data = tokio::task::spawn_blocking({
        let db = db.0.clone();
        let sql = sql.clone();
        move || db.query_json(&sql)
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .map_err(|e| ServerFnError::new(format!("Query failed: {e}")))?;

    // Phase 3: Summarize and get chart config
    let (summary, chart_type, chart_config) = llm
        .0
        .summarize(&question, &sql, &data, &language)
        .await
        .map_err(|e| ServerFnError::new(format!("Summarization failed: {e}")))?;

    // Build ECharts option JSON
    let chart_option_json = if let Some(ref config) = chart_config {
        open_eyes_core::build_echart_option(&chart_type, config, &data)
    } else {
        "{}".to_string()
    };

    let chart_type_str = serde_json::to_string(&chart_type).unwrap_or_else(|_| "\"none\"".into());

    // Save assistant message
    let summary_escaped = summary.replace('\'', "''");
    let sql_escaped = sql.replace('\'', "''");
    let chart_escaped = chart_option_json.replace('\'', "''");
    db.0.execute(&format!(
        "INSERT INTO oe_chat_messages (session_id, role, content, sql_query, chart_spec) VALUES ('{sid}', 'assistant', '{summary_escaped}', '{sql_escaped}', '{chart_escaped}')"
    )).map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(ChatResult {
        summary,
        chart_type: chart_type_str,
        chart_option_json,
        data,
        sql,
    })
}

#[server]
pub async fn fetch_chat_history(
    session_id: String,
) -> Result<Vec<ChatHistoryMessage>, ServerFnError> {
    use super::duckdb_state::DuckDbState;
    let db: DuckDbState = dioxus_fullstack::FullstackContext::extract().await?;
    let sid = session_id.replace('\'', "''");

    let rows = db.0.query_json(&format!(
        "SELECT id, role, content, COALESCE(sql_query, '') AS sql_query, COALESCE(chart_spec, '') AS chart_spec FROM oe_chat_messages WHERE session_id = '{sid}' ORDER BY created_at ASC"
    )).map_err(|e| ServerFnError::new(e.to_string()))?;

    let messages = rows
        .iter()
        .map(|r| ChatHistoryMessage {
            id: r["id"].as_i64().unwrap_or(0),
            role: r["role"].as_str().unwrap_or("").to_string(),
            content: r["content"].as_str().unwrap_or("").to_string(),
            sql_query: r["sql_query"].as_str().unwrap_or("").to_string(),
            chart_spec: r["chart_spec"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    Ok(messages)
}
