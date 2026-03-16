use dioxus::prelude::*;

use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::infrastructure::stats::fetch_ingestion_status;

#[component]
pub fn SettingsPage() -> Element {
    let status = use_server_future(fetch_ingestion_status)?;

    let lang_signal: Signal<String> = use_context();
    let language = lang_signal();

    let title = if language == "de" { "Einstellungen" } else { "Settings" };
    let desc = if language == "de" {
        "Datenstand und Systemstatus"
    } else {
        "Data status and system information"
    };

    rsx! {
        PageHeader {
            title: title.to_string(),
            description: desc.to_string(),
        }

        div { class: "card",
            h3 {
                if language == "de" { "Datenstand" } else { "Data Status" }
            }
            div { class: "stats-grid",
                style: "margin-top: 12px;",
                match status() {
                    Some(Ok(s)) => rsx! {
                        StatCard {
                            label: if language == "de" { "Datensätze".to_string() } else { "Data Collections".to_string() },
                            value: s.total_datasets.to_string(),
                        }
                        StatCard {
                            label: if language == "de" { "Geladen".to_string() } else { "Loaded".to_string() },
                            value: s.loaded_resources.to_string(),
                            color: "var(--success)".to_string(),
                        }
                        StatCard {
                            label: if language == "de" { "In Bearbeitung".to_string() } else { "Processing".to_string() },
                            value: s.pending_resources.to_string(),
                            color: "var(--warning)".to_string(),
                        }
                        StatCard {
                            label: if language == "de" { "Fehler".to_string() } else { "Errors".to_string() },
                            value: s.error_resources.to_string(),
                            color: "var(--error)".to_string(),
                        }
                    },
                    Some(Err(_)) => rsx! {
                        p {
                            if language == "de" { "Fehler beim Laden des Status." }
                            else { "Error loading status." }
                        }
                    },
                    None => rsx! {
                        p {
                            if language == "de" { "Wird geladen..." } else { "Loading..." }
                        }
                    },
                }
            }
        }

        div { class: "card",
            h3 {
                if language == "de" { "Über Open Eyes" } else { "About Open Eyes" }
            }
            p {
                style: "font-size: 14px; color: var(--text-secondary); line-height: 1.7;",
                if language == "de" {
                    "Open Eyes macht öffentliche Regierungsdaten von GovData.de für alle zugänglich. \
                     Stellen Sie einfach eine Frage im Chat und erhalten Sie Antworten mit \
                     automatischen Visualisierungen — ganz ohne technische Vorkenntnisse."
                } else {
                    "Open Eyes makes public government data from GovData.de accessible to everyone. \
                     Simply ask a question in the chat and receive answers with automatic \
                     visualizations — no technical knowledge required."
                }
            }
        }
    }
}
