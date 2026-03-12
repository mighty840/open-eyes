use dioxus::prelude::*;

use crate::app::Route;
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::infrastructure::stats::{
    fetch_overview_stats, fetch_popular_categories, fetch_recent_datasets,
};

#[component]
pub fn HomePage() -> Element {
    let stats = use_server_future(fetch_overview_stats)?;
    let categories = use_server_future(fetch_popular_categories)?;
    let recent = use_server_future(fetch_recent_datasets)?;

    rsx! {
        PageHeader {
            title: "Dashboard",
            description: "Explore German open government data",
        }

        div { class: "stats-grid",
            match stats() {
                Some(Ok(s)) => rsx! {
                    StatCard { label: "Datasets".to_string(), value: s.total_datasets.to_string() }
                    StatCard { label: "Loaded Tables".to_string(), value: s.total_tables.to_string() }
                    StatCard { label: "Total Rows".to_string(), value: format_number(s.total_rows) }
                    StatCard { label: "Resources".to_string(), value: s.total_resources.to_string() }
                },
                Some(Err(_)) => rsx! {
                    StatCard { label: "Datasets".to_string(), value: "—".to_string() }
                    StatCard { label: "Tables".to_string(), value: "—".to_string() }
                    StatCard { label: "Rows".to_string(), value: "—".to_string() }
                    StatCard { label: "Resources".to_string(), value: "—".to_string() }
                },
                None => rsx! {
                    div { class: "loading", "Loading stats..." }
                },
            }
        }

        div { class: "card",
            h3 { "Popular Categories" }
            match categories() {
                Some(Ok(cats)) => rsx! {
                    div { class: "categories-list",
                        for cat in cats {
                            span { class: "category-badge",
                                "{cat.name}"
                                span { class: "count", "{cat.count}" }
                            }
                        }
                    }
                },
                Some(Err(_)) => rsx! { p { "Failed to load categories" } },
                None => rsx! { p { "Loading..." } },
            }
        }

        div { class: "card",
            h3 { "Recently Ingested" }
            match recent() {
                Some(Ok(datasets)) => rsx! {
                    div { class: "recent-list",
                        for ds in datasets {
                            Link {
                                to: Route::DatasetDetailPage { dataset_id: ds.id.clone() },
                                class: "recent-item",
                                span { "{ds.title}" }
                                span { class: "org", "{ds.organization}" }
                            }
                        }
                    }
                },
                Some(Err(_)) => rsx! { p { "Failed to load recent datasets" } },
                None => rsx! { p { "Loading..." } },
            }
        }
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
