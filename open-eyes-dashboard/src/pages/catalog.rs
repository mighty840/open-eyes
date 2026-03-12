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

    let q = query();
    let p = page();
    let search_result = use_resource(move || {
        let q = q.clone();
        async move { search_datasets(q, p * PAGE_SIZE, PAGE_SIZE).await }
    });

    let result = search_result.read();

    rsx! {
        PageHeader {
            title: "Data Catalog",
            description: "Browse and search government datasets",
        }

        SearchBar {
            placeholder: "Search datasets...".to_string(),
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
                        div { class: "card", p { "No datasets found" } }
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
                                "Previous"
                            }
                            {
                                let p_display = p + 1;
                                rsx! { span { "Page {p_display} of {total_pages}" } }
                            }
                            button {
                                disabled: p + 1 >= total_pages,
                                onclick: move |_| page.set(p + 1),
                                "Next"
                            }
                        }
                    }
                }
            },
            Some(Err(e)) => rsx! { div { class: "card", p { "Error: {e}" } } },
            None => rsx! { div { class: "loading", "Loading catalog..." } },
        }
    }
}
