use open_eyes_core::AppConfig;

use super::error::DashboardError;

pub fn load_config() -> Result<AppConfig, DashboardError> {
    let config_path =
        std::env::var("OPEN_EYES_CONFIG").unwrap_or_else(|_| "config.toml".to_string());
    AppConfig::load(std::path::Path::new(&config_path))
        .map_err(|e| DashboardError::Config(e.to_string()))
}
