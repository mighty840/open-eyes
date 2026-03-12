use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("DuckDB error: {0}")]
    DuckDb(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}
