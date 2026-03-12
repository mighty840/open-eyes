// Server function modules (compiled for both web and server;
// the #[server] macro generates client stubs for the web target)
pub mod analytics;
pub mod chat;
pub mod datasets;
pub mod stats;

// Server-only modules
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod duckdb_state;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod llm_state;
#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
pub use server::server_start;
