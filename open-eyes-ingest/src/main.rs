#![allow(non_snake_case)]

use std::path::Path;

use clap::{Parser, Subcommand};

mod crawler;
mod downloader;

#[derive(Parser)]
#[command(name = "open-eyes-ingest", about = "CKAN data ingestion for Open Eyes")]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crawl CKAN portal and store dataset metadata
    Crawl {
        /// Maximum number of datasets to crawl
        #[arg(long, default_value = "10000")]
        max_datasets: u64,
    },
    /// Download and load pending resources into SQLite
    Load,
    /// Run crawl + load on a recurring schedule
    Daemon,
}

#[allow(clippy::expect_used)]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let config =
        open_eyes_core::AppConfig::load(Path::new(&cli.config)).expect("Failed to load config");

    let db = open_eyes_core::DbPool::open(Path::new(&config.db.path))
        .expect("Failed to open SQLite");
    db.init_schema().expect("Failed to init schema");

    match cli.command {
        Commands::Crawl { max_datasets } => {
            match crawler::crawl(&db, &config.ckan, max_datasets).await {
                Ok(n) => tracing::info!("Crawled {n} datasets"),
                Err(e) => tracing::error!("Crawl failed: {e}"),
            }
        }
        Commands::Load => {
            match downloader::load_pending(&db, config.db.max_resource_size_mb).await {
                Ok(n) => tracing::info!("Loaded {n} resources"),
                Err(e) => tracing::error!("Load failed: {e}"),
            }
        }
        Commands::Daemon => {
            tracing::info!(
                "Starting daemon (interval: {}h)",
                config.ckan.crawl_interval_hours
            );
            let interval = std::time::Duration::from_secs(config.ckan.crawl_interval_hours * 3600);
            loop {
                tracing::info!("Running scheduled crawl + load");
                if let Err(e) = crawler::crawl(&db, &config.ckan, config.ckan.max_datasets).await {
                    tracing::error!("Crawl error: {e}");
                }
                if let Err(e) =
                    downloader::load_pending(&db, config.db.max_resource_size_mb).await
                {
                    tracing::error!("Load error: {e}");
                }
                tokio::time::sleep(interval).await;
            }
        }
    }
}
