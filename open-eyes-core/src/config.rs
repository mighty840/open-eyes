use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::OpenEyesError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub dashboard: DashboardConfig,
    pub db: DbConfig,
    pub llm: LlmConfig,
    pub ckan: CkanConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_language")]
    pub default_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
    #[serde(default = "default_max_resource_size")]
    pub max_resource_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_llm_base_url")]
    pub base_url: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanConfig {
    #[serde(default = "default_ckan_base_url")]
    pub base_url: String,
    #[serde(default = "default_max_datasets")]
    pub max_datasets: u64,
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,
    #[serde(default = "default_crawl_interval")]
    pub crawl_interval_hours: u64,
}

fn default_port() -> u16 {
    8080
}
fn default_title() -> String {
    "Open Eyes".into()
}
fn default_language() -> String {
    "de".into()
}
fn default_db_path() -> String {
    "./data/open-eyes.sqlite".into()
}
fn default_max_resource_size() -> u64 {
    100
}
fn default_llm_base_url() -> String {
    "https://api.openai.com/v1".into()
}
fn default_llm_model() -> String {
    "gpt-4o-mini".into()
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_temperature() -> f32 {
    0.1
}
fn default_ckan_base_url() -> String {
    "https://ckan.govdata.de/api/3/action".into()
}
fn default_max_datasets() -> u64 {
    10000
}
fn default_formats() -> Vec<String> {
    vec!["CSV".into(), "JSON".into()]
}
fn default_crawl_interval() -> u64 {
    24
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, OpenEyesError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OpenEyesError::Config(format!("Failed to read config: {e}")))?;
        let mut config: AppConfig =
            toml::from_str(&content).map_err(|e| OpenEyesError::Config(e.to_string()))?;

        // Apply env var overrides
        if let Ok(val) = std::env::var("OPEN_EYES_LLM_API_KEY") {
            config.llm.api_key = val;
        }
        if let Ok(val) = std::env::var("OPEN_EYES_LLM_BASE_URL") {
            config.llm.base_url = val;
        }
        if let Ok(val) = std::env::var("OPEN_EYES_LLM_MODEL") {
            config.llm.model = val;
        }
        if let Ok(val) = std::env::var("OPEN_EYES_DB_PATH") {
            config.db.path = val;
        }
        if let Ok(val) = std::env::var("OPEN_EYES_PORT") {
            if let Ok(port) = val.parse() {
                config.dashboard.port = port;
            }
        }
        if let Ok(val) = std::env::var("OPEN_EYES_CKAN_BASE_URL") {
            config.ckan.base_url = val;
        }

        Ok(config)
    }
}
