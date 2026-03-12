use dioxus::prelude::*;

use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::infrastructure::stats::fetch_ingestion_status;

#[component]
pub fn SettingsPage() -> Element {
    let status = use_server_future(fetch_ingestion_status)?;

    rsx! {
        PageHeader {
            title: "Settings",
            description: "Configuration and ingestion status",
        }

        div { class: "card",
            h3 { "Ingestion Status" }
            div { class: "stats-grid",
                style: "margin-top: 12px;",
                match status() {
                    Some(Ok(s)) => rsx! {
                        StatCard { label: "Total Datasets".to_string(), value: s.total_datasets.to_string() }
                        StatCard { label: "Loaded".to_string(), value: s.loaded_resources.to_string(), color: "var(--success)".to_string() }
                        StatCard { label: "Pending".to_string(), value: s.pending_resources.to_string(), color: "var(--warning)".to_string() }
                        StatCard { label: "Errors".to_string(), value: s.error_resources.to_string(), color: "var(--error)".to_string() }
                    },
                    Some(Err(e)) => rsx! { p { "Error: {e}" } },
                    None => rsx! { p { "Loading..." } },
                }
            }
        }

        div { class: "card",
            h3 { "Configuration" }
            p {
                style: "font-size: 13px; color: var(--text-muted); margin-bottom: 12px;",
                "Configuration is loaded from config.toml and environment variables."
            }
            div { class: "config-display",
                "OPEN_EYES_LLM_BASE_URL  - Override LLM endpoint\n\
                 OPEN_EYES_LLM_API_KEY   - Set API key\n\
                 OPEN_EYES_LLM_MODEL     - Override model\n\
                 OPEN_EYES_DUCKDB_PATH   - Override database path\n\
                 OPEN_EYES_PORT          - Override server port\n\
                 OPEN_EYES_CKAN_BASE_URL - Override CKAN endpoint"
            }
        }
    }
}
