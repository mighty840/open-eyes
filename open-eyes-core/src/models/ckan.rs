use serde::{Deserialize, Serialize};

/// CKAN API response wrapper
#[derive(Debug, Deserialize)]
pub struct CkanResponse<T> {
    pub success: bool,
    pub result: T,
}

/// CKAN package_search result
#[derive(Debug, Deserialize)]
pub struct CkanSearchResult {
    pub count: u64,
    pub results: Vec<CkanPackage>,
}

/// CKAN package (dataset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanPackage {
    pub id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub organization: Option<CkanOrganization>,
    pub groups: Option<Vec<CkanGroup>>,
    pub tags: Option<Vec<CkanTag>>,
    pub license_title: Option<String>,
    pub url: Option<String>,
    pub metadata_created: Option<String>,
    pub metadata_modified: Option<String>,
    pub resources: Option<Vec<CkanResource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanOrganization {
    pub name: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanGroup {
    pub name: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanTag {
    pub name: String,
}

/// CKAN resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanResource {
    pub id: String,
    pub name: Option<String>,
    pub format: Option<String>,
    pub url: String,
}
