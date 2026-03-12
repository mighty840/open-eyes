use open_eyes_core::models::chat::{ChartType, ChatResponse, SummarizationResponse};
use open_eyes_core::models::ckan::{CkanPackage, CkanResponse, CkanSearchResult};
use open_eyes_core::models::dataset::{Dataset, OverviewStats};

#[test]
fn test_chart_type_serde() {
    let json = r#""bar""#;
    let ct: ChartType = serde_json::from_str(json).unwrap();
    assert_eq!(ct, ChartType::Bar);

    let json = r#""none""#;
    let ct: ChartType = serde_json::from_str(json).unwrap();
    assert_eq!(ct, ChartType::None);

    let serialized = serde_json::to_string(&ChartType::Pie).unwrap();
    assert_eq!(serialized, r#""pie""#);
}

#[test]
fn test_chart_type_default() {
    let ct = ChartType::default();
    assert_eq!(ct, ChartType::None);
}

#[test]
fn test_chat_response_default() {
    let resp = ChatResponse::default();
    assert!(resp.summary.is_empty());
    assert_eq!(resp.chart_type, ChartType::None);
    assert!(resp.data.is_empty());
    assert!(resp.sql.is_none());
}

#[test]
fn test_overview_stats_default() {
    let stats = OverviewStats::default();
    assert_eq!(stats.total_datasets, 0);
    assert_eq!(stats.total_tables, 0);
    assert_eq!(stats.total_rows, 0);
}

#[test]
fn test_dataset_default() {
    let ds = Dataset::default();
    assert!(ds.id.is_empty());
    assert!(ds.title.is_empty());
    assert!(ds.categories.is_empty());
    assert!(ds.tags.is_empty());
}

#[test]
fn test_ckan_package_deserialize() {
    let json = r#"{
        "id": "pkg-1",
        "title": "Test Package",
        "notes": "Some description",
        "organization": {"name": "org1", "title": "Organization 1"},
        "groups": [{"name": "health", "title": "Health"}],
        "tags": [{"name": "covid"}, {"name": "data"}],
        "license_title": "CC-BY",
        "url": "https://example.com",
        "metadata_created": "2024-01-01T00:00:00",
        "metadata_modified": "2024-06-01T00:00:00",
        "resources": [
            {"id": "r1", "name": "data.csv", "format": "CSV", "url": "https://example.com/data.csv"}
        ]
    }"#;

    let pkg: CkanPackage = serde_json::from_str(json).unwrap();
    assert_eq!(pkg.id, "pkg-1");
    assert_eq!(pkg.title.unwrap(), "Test Package");
    assert_eq!(pkg.organization.unwrap().name, "org1");
    assert_eq!(pkg.tags.unwrap().len(), 2);
    assert_eq!(pkg.resources.unwrap().len(), 1);
}

#[test]
fn test_ckan_response_deserialize() {
    let json = r#"{
        "success": true,
        "result": {
            "count": 1,
            "results": [{"id": "p1", "title": "T1"}]
        }
    }"#;

    let resp: CkanResponse<CkanSearchResult> = serde_json::from_str(json).unwrap();
    assert!(resp.success);
    assert_eq!(resp.result.count, 1);
    assert_eq!(resp.result.results.len(), 1);
}

#[test]
fn test_summarization_response_deserialize() {
    let json = r#"{
        "summary": "The data shows...",
        "chart_type": "bar",
        "chart_config": {"title": "Chart", "x_field": "x", "y_field": "y", "series_field": null}
    }"#;

    let resp: SummarizationResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.summary, "The data shows...");
    assert_eq!(resp.chart_type, ChartType::Bar);
    assert!(resp.chart_config.is_some());
    assert_eq!(resp.chart_config.unwrap().x_field, "x");
}
