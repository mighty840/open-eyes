use std::sync::Arc;

use axum::Extension;
use dioxus::prelude::*;

use open_eyes_core::{DuckDbPool, LlmClient};

use super::config;
use super::duckdb_state::DuckDbState;
use super::error::DashboardError;
use super::llm_state::LlmState;

pub fn server_start(app: fn() -> Element) -> Result<(), DashboardError> {
    tokio::runtime::Runtime::new()
        .map_err(|e| DashboardError::Other(e.to_string()))?
        .block_on(async move {
            let config = config::load_config()?;

            let db = DuckDbPool::open(std::path::Path::new(&config.duckdb.path))
                .map_err(|e| DashboardError::DuckDb(e.to_string()))?;
            db.init_schema()
                .map_err(|e| DashboardError::DuckDb(e.to_string()))?;

            let llm = LlmClient::new(&config.llm);

            let port = dioxus_cli_config::server_port().unwrap_or(config.dashboard.port);
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| DashboardError::Other(format!("Failed to bind: {e}")))?;

            tracing::info!("Open Eyes dashboard listening on {addr}");

            let router = axum::Router::new()
                .serve_dioxus_application(ServeConfig::new(), app)
                .layer(Extension(DuckDbState(db)))
                .layer(Extension(LlmState(Arc::new(llm))));

            axum::serve(listener, router.into_make_service())
                .await
                .map_err(|e| DashboardError::Other(format!("Server error: {e}")))?;

            Ok(())
        })
}
