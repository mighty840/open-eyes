use open_eyes_core::config::CkanConfig;
use open_eyes_core::error::OpenEyesError;
use open_eyes_core::models::{CkanPackage, CkanResponse, CkanSearchResult};
use open_eyes_core::DuckDbPool;

/// Crawl CKAN portal and upsert datasets + resources into DuckDB.
pub async fn crawl(
    db: &DuckDbPool,
    config: &CkanConfig,
    max_datasets: u64,
) -> Result<u64, OpenEyesError> {
    let client = reqwest::Client::new();
    let mut offset = 0u64;
    let page_size = 100u64;
    let mut total_inserted = 0u64;
    let limit = max_datasets.min(config.max_datasets);

    tracing::info!(
        "Starting CKAN crawl from {} (max {})",
        config.base_url,
        limit
    );

    loop {
        if offset >= limit {
            break;
        }

        let rows = page_size.min(limit - offset);
        let url = format!(
            "{}/package_search?rows={rows}&start={offset}",
            config.base_url
        );

        tracing::info!("Fetching {url}");
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            tracing::warn!("CKAN API returned {status}, stopping crawl");
            break;
        }

        let body: CkanResponse<CkanSearchResult> = resp.json().await?;
        if !body.success || body.result.results.is_empty() {
            break;
        }

        for pkg in &body.result.results {
            if let Err(e) = upsert_package(db, pkg, &config.formats) {
                tracing::warn!("Failed to upsert package {}: {e}", pkg.id);
            } else {
                total_inserted += 1;
            }
        }

        offset += body.result.results.len() as u64;
        tracing::info!("Crawled {offset}/{} datasets", body.result.count.min(limit));

        if body.result.results.len() < page_size as usize {
            break;
        }
    }

    tracing::info!("Crawl complete: {total_inserted} datasets upserted");
    Ok(total_inserted)
}

fn upsert_package(
    db: &DuckDbPool,
    pkg: &CkanPackage,
    allowed_formats: &[String],
) -> Result<(), OpenEyesError> {
    let title = pkg.title.as_deref().unwrap_or("Untitled");
    let description = pkg.notes.as_deref().unwrap_or("");
    let org = pkg
        .organization
        .as_ref()
        .map(|o| o.title.as_deref().unwrap_or(&o.name))
        .unwrap_or("");
    let categories: Vec<&str> = pkg
        .groups
        .as_ref()
        .map(|g| g.iter().map(|g| g.name.as_str()).collect())
        .unwrap_or_default();
    let tags: Vec<&str> = pkg
        .tags
        .as_ref()
        .map(|t| t.iter().map(|t| t.name.as_str()).collect())
        .unwrap_or_default();
    let license = pkg.license_title.as_deref().unwrap_or("");
    let ckan_url = pkg.url.as_deref().unwrap_or("");
    let created = pkg.metadata_created.as_deref().unwrap_or("");
    let modified = pkg.metadata_modified.as_deref().unwrap_or("");

    let cats_sql = format!(
        "[{}]",
        categories
            .iter()
            .map(|c| format!("'{}'", c.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let tags_sql = format!(
        "[{}]",
        tags.iter()
            .map(|t| format!("'{}'", t.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let sql = format!(
        r#"INSERT OR REPLACE INTO oe_datasets (id, title, title_de, description, description_de, organization, categories, tags, license, ckan_url, created_at, modified_at)
        VALUES ('{id}', '{title}', '{title}', '{desc}', '{desc}', '{org}', {cats}, {tags}, '{license}', '{url}',
                CASE WHEN '{created}' = '' THEN NULL ELSE TRY_CAST('{created}' AS TIMESTAMP) END,
                CASE WHEN '{modified}' = '' THEN NULL ELSE TRY_CAST('{modified}' AS TIMESTAMP) END)"#,
        id = pkg.id.replace('\'', "''"),
        title = title.replace('\'', "''"),
        desc = description.replace('\'', "''"),
        org = org.replace('\'', "''"),
        cats = cats_sql,
        tags = tags_sql,
        license = license.replace('\'', "''"),
        url = ckan_url.replace('\'', "''"),
        created = created.replace('\'', "''"),
        modified = modified.replace('\'', "''"),
    );

    db.execute(&sql)?;

    // Insert resources
    if let Some(resources) = &pkg.resources {
        for res in resources {
            let fmt = res.format.as_deref().unwrap_or("").to_uppercase();
            if !allowed_formats.iter().any(|f| f.to_uppercase() == fmt) {
                continue;
            }
            let res_name = res.name.as_deref().unwrap_or("");
            let res_sql = format!(
                r#"INSERT OR REPLACE INTO oe_resources (id, dataset_id, name, format, url, download_status)
                VALUES ('{id}', '{did}', '{name}', '{fmt}', '{url}', 'pending')"#,
                id = res.id.replace('\'', "''"),
                did = pkg.id.replace('\'', "''"),
                name = res_name.replace('\'', "''"),
                fmt = fmt.replace('\'', "''"),
                url = res.url.replace('\'', "''"),
            );
            if let Err(e) = db.execute(&res_sql) {
                tracing::warn!("Failed to insert resource {}: {e}", res.id);
            }
        }
    }

    Ok(())
}
