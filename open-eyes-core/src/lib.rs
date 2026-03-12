pub mod config;
pub mod duckdb;
pub mod error;
pub mod llm;
pub mod models;

pub use config::AppConfig;
pub use duckdb::{build_echart_option, DuckDbPool};
pub use error::OpenEyesError;
pub use llm::LlmClient;
