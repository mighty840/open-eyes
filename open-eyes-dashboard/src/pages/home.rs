use dioxus::prelude::*;

use crate::app::Route;
use crate::components::stat_card::StatCard;
use crate::infrastructure::stats::{
    fetch_overview_stats, fetch_popular_categories, fetch_recent_datasets,
};

const SUGGESTED_QUESTIONS_DE: &[(&str, &str)] = &[
    ("Welche Kategorien haben die meisten Daten?", "Entdecken Sie die beliebtesten Themenbereiche"),
    ("Zeige die neuesten Veröffentlichungen", "Sehen Sie, was gerade veröffentlicht wurde"),
    ("Welche Organisationen veröffentlichen am meisten?", "Erfahren Sie, wer am aktivsten ist"),
    ("Welche Datenformate sind verfügbar?", "CSV, JSON und mehr auf einen Blick"),
];

const SUGGESTED_QUESTIONS_EN: &[(&str, &str)] = &[
    ("Which categories have the most data?", "Discover the most popular topic areas"),
    ("Show the latest publications", "See what was just published"),
    ("Which organizations publish the most?", "Find out who is most active"),
    ("What data formats are available?", "CSV, JSON and more at a glance"),
];

#[component]
pub fn HomePage() -> Element {
    let stats = use_server_future(fetch_overview_stats)?;
    let categories = use_server_future(fetch_popular_categories)?;
    let recent = use_server_future(fetch_recent_datasets)?;

    let lang_signal: Signal<String> = use_context();
    let language = lang_signal();

    let suggestions = if language == "de" {
        SUGGESTED_QUESTIONS_DE
    } else {
        SUGGESTED_QUESTIONS_EN
    };

    // Get the chat prefill signal from context
    let mut chat_prefill: Signal<String> = use_context();

    rsx! {
        // Hero section
        div { class: "hero-section",
            h2 {
                if language == "de" {
                    "Offene Daten, verständlich erklärt"
                } else {
                    "Open data, clearly explained"
                }
            }
            p {
                if language == "de" {
                    "Stellen Sie Fragen zu öffentlichen Regierungsdaten in einfacher Sprache. Open Eyes findet die passenden Datensätze und erstellt Visualisierungen für Sie."
                } else {
                    "Ask questions about public government data in plain language. Open Eyes finds the right datasets and creates visualizations for you."
                }
            }
            div { class: "suggested-questions",
                for (question, hint) in suggestions.iter() {
                    {
                        let q = question.to_string();
                        rsx! {
                            button {
                                class: "suggestion-card",
                                onclick: move |_| {
                                    chat_prefill.set(q.clone());
                                },
                                span { class: "suggestion-icon", "?" }
                                strong { "{question}" }
                                br {}
                                span { style: "opacity: 0.7; font-size: 13px;", "{hint}" }
                            }
                        }
                    }
                }
            }
        }

        // Quick stats
        div { class: "stats-grid",
            match stats() {
                Some(Ok(s)) => rsx! {
                    StatCard {
                        label: if language == "de" { "Datensätze".to_string() } else { "Data Collections".to_string() },
                        value: format_number_friendly(s.total_datasets as u64),
                    }
                    StatCard {
                        label: if language == "de" { "Analysierbare Tabellen".to_string() } else { "Queryable Tables".to_string() },
                        value: s.total_tables.to_string(),
                    }
                    StatCard {
                        label: if language == "de" { "Datenpunkte".to_string() } else { "Data Points".to_string() },
                        value: format_number_friendly(s.total_rows),
                    }
                    StatCard {
                        label: if language == "de" { "Dateien".to_string() } else { "Files".to_string() },
                        value: s.total_resources.to_string(),
                    }
                },
                Some(Err(_)) => rsx! {
                    StatCard { label: "—".to_string(), value: "—".to_string() }
                },
                None => rsx! {
                    div { class: "loading",
                        if language == "de" { "Lade Übersicht..." } else { "Loading overview..." }
                    }
                },
            }
        }

        // Popular categories — clickable
        div { class: "card",
            h3 {
                if language == "de" { "Beliebte Themenbereiche" } else { "Popular Topics" }
            }
            match categories() {
                Some(Ok(cats)) => rsx! {
                    div { class: "categories-list",
                        for cat in cats {
                            span {
                                class: "category-badge",
                                "{cat.name}"
                                span { class: "count", "{cat.count}" }
                            }
                        }
                    }
                },
                Some(Err(_)) => rsx! {
                    p {
                        if language == "de" { "Themenbereiche konnten nicht geladen werden" }
                        else { "Could not load topics" }
                    }
                },
                None => rsx! {
                    p {
                        if language == "de" { "Wird geladen..." } else { "Loading..." }
                    }
                },
            }
        }

        // Recently ingested
        div { class: "card",
            h3 {
                if language == "de" { "Kürzlich hinzugefügt" } else { "Recently Added" }
            }
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
                Some(Err(_)) => rsx! {
                    p {
                        if language == "de" { "Neueste Datensätze konnten nicht geladen werden" }
                        else { "Could not load recent datasets" }
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
}

fn format_number_friendly(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
