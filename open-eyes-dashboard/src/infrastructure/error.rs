use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Database error: {0}")]
    Db(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}
