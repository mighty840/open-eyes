use dioxus::prelude::*;

use crate::components::chart_container::ChartContainer;
use crate::components::page_header::PageHeader;
use crate::infrastructure::analytics::fetch_saved_charts;

#[component]
pub fn AnalyticsPage() -> Element {
    let charts = use_server_future(fetch_saved_charts)?;

    let lang_signal: Signal<String> = use_context();
    let language = lang_signal();

    let title = if language == "de" { "Auswertungen" } else { "Analytics" };
    let desc = if language == "de" {
        "Gespeicherte Visualisierungen aus Ihren Fragen"
    } else {
        "Saved visualizations from your questions"
    };

    rsx! {
        PageHeader {
            title: title.to_string(),
            description: desc.to_string(),
        }

        match charts() {
            Some(Ok(saved)) => {
                if saved.is_empty() {
                    rsx! {
                        div { class: "empty-state",
                            div { class: "empty-state-icon", "?" }
                            h4 {
                                if language == "de" { "Noch keine Auswertungen" }
                                else { "No visualizations yet" }
                            }
                            p {
                                if language == "de" {
                                    "Stellen Sie Fragen im Chat, um automatisch Diagramme und Visualisierungen zu erstellen."
                                } else {
                                    "Ask questions in the chat to automatically create charts and visualizations."
                                }
                            }
                        }
                    }
                } else {
                    rsx! {
                        for chart in saved {
                            div { class: "card",
                                p { "{chart.content}" }
                                if !chart.sql_query.is_empty() {
                                    div { class: "sql-preview", "{chart.sql_query}" }
                                }
                                ChartContainer { option_json: chart.chart_spec }
                                p {
                                    style: "font-size: 11px; color: var(--text-muted); margin-top: 8px;",
                                    "{chart.created_at}"
                                }
                            }
                        }
                    }
                }
            },
            Some(Err(_)) => rsx! {
                div { class: "card",
                    p {
                        if language == "de" { "Fehler beim Laden der Auswertungen." }
                        else { "Error loading analytics." }
                    }
                }
            },
            None => rsx! {
                div { class: "loading",
                    if language == "de" { "Lade Auswertungen..." } else { "Loading analytics..." }
                }
            },
        }
    }
}
