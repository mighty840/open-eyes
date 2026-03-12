use serde::{Deserialize, Serialize};

/// Chat message stored in oe_chat_messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub sql_query: Option<String>,
    pub chart_spec: Option<String>,
    pub created_at: Option<String>,
}

/// Chart type for visualization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
    Table,
    #[default]
    None,
}

/// Chart configuration from LLM
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartConfig {
    pub title: String,
    pub x_field: String,
    pub y_field: String,
    pub series_field: Option<String>,
}

/// Full chat response from the pipeline
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatResponse {
    pub summary: String,
    pub chart_type: ChartType,
    pub chart_config: Option<ChartConfig>,
    pub data: Vec<serde_json::Value>,
    pub sql: Option<String>,
}

/// LLM table selection response
#[derive(Debug, Deserialize)]
pub struct TableSelectionResponse {
    pub tables: Vec<String>,
}

/// LLM SQL generation response
#[derive(Debug, Deserialize)]
pub struct SqlGenerationResponse {
    pub sql: String,
}

/// LLM summarization response
#[derive(Debug, Deserialize)]
pub struct SummarizationResponse {
    pub summary: String,
    pub chart_type: ChartType,
    pub chart_config: Option<ChartConfig>,
}
