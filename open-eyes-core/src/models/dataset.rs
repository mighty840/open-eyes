use serde::{Deserialize, Serialize};

/// Dataset metadata stored in oe_datasets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dataset {
    pub id: String,
    pub title: String,
    pub title_de: Option<String>,
    pub description: Option<String>,
    pub description_de: Option<String>,
    pub organization: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub license: Option<String>,
    pub source_portal: String,
    pub ckan_url: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub ingested_at: Option<String>,
}

/// Resource metadata stored in oe_resources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resource {
    pub id: String,
    pub dataset_id: String,
    pub name: Option<String>,
    pub format: Option<String>,
    pub url: String,
    pub table_name: Option<String>,
    pub row_count: Option<i64>,
    pub column_names: Vec<String>,
    pub download_status: String,
    pub error_message: Option<String>,
    pub ingested_at: Option<String>,
}

/// Overview statistics for the home page
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverviewStats {
    pub total_datasets: u64,
    pub total_tables: u64,
    pub total_rows: u64,
    pub total_resources: u64,
}

/// Category count for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCount {
    pub name: String,
    pub count: u64,
}

/// Table info for LLM table selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub dataset_title: String,
    pub description: Option<String>,
    pub row_count: i64,
    pub column_names: Vec<String>,
}

/// Column schema for SQL generation prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
}
