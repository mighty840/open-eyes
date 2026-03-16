use dioxus::prelude::*;

use crate::components::dataset_card::DatasetCard;
use crate::components::page_header::PageHeader;
use crate::components::search_bar::SearchBar;
use crate::infrastructure::datasets::search_datasets;

const PAGE_SIZE: u64 = 20;

#[component]
pub fn CatalogPage() -> Element {
    let mut query = use_signal(String::new);
    let mut page = use_signal(|| 0u64);

    let lang_signal: Signal<String> = use_context();
    let language = lang_signal();

    let q = query();
    let p = page();
    let search_result = use_resource(move || {
        let q = q.clone();
        async move { search_datasets(q, p * PAGE_SIZE, PAGE_SIZE).await }
    });

    let result = search_result.read();

    let title = if language == "de" { "Datenkatalog" } else { "Data Catalog" };
    let desc = if language == "de" {
        "Durchsuchen Sie öffentliche Datensätze der deutschen Regierung"
    } else {
        "Browse public datasets from the German government"
    };
    let search_placeholder = if language == "de" {
        "Datensätze durchsuchen...".to_string()
    } else {
        "Search datasets...".to_string()
    };

    rsx! {
        PageHeader {
            title: title.to_string(),
            description: desc.to_string(),
        }

        SearchBar {
            placeholder: search_placeholder,
            value: query(),
            on_input: move |val: String| {
                query.set(val);
                page.set(0);
            },
        }

        match &*result {
            Some(Ok(data)) => {
                let total_pages = data.total.div_ceil(PAGE_SIZE);
                rsx! {
                    if data.datasets.is_empty() {
                        div { class: "empty-state",
                            div { class: "empty-state-icon", "?" }
                            h4 {
                                if language == "de" { "Keine Ergebnisse gefunden" }
                                else { "No results found" }
                            }
                            p {
                                if language == "de" { "Versuchen Sie andere Suchbegriffe oder stöbern Sie ohne Filter." }
                                else { "Try different search terms or browse without a filter." }
                            }
                        }
                    } else {
                        div { class: "dataset-grid",
                            for ds in &data.datasets {
                                DatasetCard { key: "{ds.id}", dataset: ds.clone() }
                            }
                        }
                        div { class: "pagination",
                            button {
                                disabled: p == 0,
                                onclick: move |_| page.set(p.saturating_sub(1)),
                                if language == "de" { "Zurück" } else { "Previous" }
                            }
                            {
                                let p_display = p + 1;
                                let page_label = if language == "de" {
                                    format!("Seite {p_display} von {total_pages}")
                                } else {
                                    format!("Page {p_display} of {total_pages}")
                                };
                                rsx! { span { "{page_label}" } }
                            }
                            button {
                                disabled: p + 1 >= total_pages,
                                onclick: move |_| page.set(p + 1),
                                if language == "de" { "Weiter" } else { "Next" }
                            }
                        }
                    }
                }
            },
            Some(Err(_e)) => rsx! {
                div { class: "card",
                    p {
                        if language == "de" { "Fehler beim Laden der Daten. Bitte versuchen Sie es erneut." }
                        else { "Error loading data. Please try again." }
                    }
                }
            },
            None => rsx! {
                div { class: "loading",
                    if language == "de" { "Lade Katalog..." } else { "Loading catalog..." }
                }
            },
        }
    }
}
