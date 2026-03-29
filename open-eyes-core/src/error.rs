use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenEyesError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Ingestion error: {0}")]
    Ingestion(String),

    #[error("{0}")]
    Other(String),
}
