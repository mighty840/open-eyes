use std::io::Write;

use open_eyes_core::AppConfig;

#[test]
fn test_load_valid_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut f = std::fs::File::create(&path).unwrap();
    write!(
        f,
        r#"
[dashboard]
port = 9090
title = "Test"
default_language = "en"

[duckdb]
path = "./test.duckdb"
max_resource_size_mb = 50

[llm]
base_url = "http://localhost:11434/v1"
model = "test-model"
api_key = "test-key"
max_tokens = 2048
temperature = 0.5

[ckan]
base_url = "https://example.com/api/3/action"
max_datasets = 100
formats = ["CSV"]
crawl_interval_hours = 12
"#
    )
    .unwrap();

    let config = AppConfig::load(&path).unwrap();
    assert_eq!(config.dashboard.port, 9090);
    assert_eq!(config.dashboard.title, "Test");
    assert_eq!(config.dashboard.default_language, "en");
    assert_eq!(config.duckdb.path, "./test.duckdb");
    assert_eq!(config.duckdb.max_resource_size_mb, 50);
    assert_eq!(config.llm.model, "test-model");
    assert_eq!(config.llm.temperature, 0.5);
    assert_eq!(config.ckan.max_datasets, 100);
    assert_eq!(config.ckan.formats, vec!["CSV"]);
    assert_eq!(config.ckan.crawl_interval_hours, 12);
}

#[test]
fn test_load_minimal_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("minimal.toml");
    let mut f = std::fs::File::create(&path).unwrap();
    write!(
        f,
        r#"
[dashboard]
[duckdb]
[llm]
[ckan]
"#
    )
    .unwrap();

    let config = AppConfig::load(&path).unwrap();
    assert_eq!(config.dashboard.port, 8080);
    assert_eq!(config.dashboard.default_language, "de");
    assert_eq!(config.llm.model, "gpt-4o-mini");
}

#[test]
fn test_load_missing_config_fails() {
    let result = AppConfig::load(std::path::Path::new("/nonexistent/config.toml"));
    assert!(result.is_err());
}

// Note: env var override tests are not included here because
// std::env::set_var is unsafe and env vars are process-global,
// making parallel test execution unreliable.
