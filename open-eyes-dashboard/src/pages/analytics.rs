use dioxus::prelude::*;

use crate::components::chart_container::ChartContainer;
use crate::components::page_header::PageHeader;
use crate::infrastructure::analytics::fetch_saved_charts;

#[component]
pub fn AnalyticsPage() -> Element {
    let charts = use_server_future(fetch_saved_charts)?;

    rsx! {
        PageHeader {
            title: "Analytics",
            description: "Saved visualizations from your queries",
        }

        match charts() {
            Some(Ok(saved)) => {
                if saved.is_empty() {
                    rsx! {
                        div { class: "card",
                            p { "No saved charts yet. Ask questions in the chat to generate visualizations." }
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
            Some(Err(e)) => rsx! { div { class: "card", p { "Error: {e}" } } },
            None => rsx! { div { class: "loading", "Loading charts..." } },
        }
    }
}
