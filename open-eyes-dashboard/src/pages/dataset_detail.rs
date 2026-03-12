use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::*;
use dioxus_free_icons::Icon;

use crate::components::page_header::PageHeader;
use crate::infrastructure::datasets::{fetch_dataset_detail, fetch_table_preview};

#[component]
pub fn DatasetDetailPage(dataset_id: String) -> Element {
    let did = dataset_id.clone();
    let detail = use_resource(move || {
        let did = did.clone();
        async move { fetch_dataset_detail(did).await }
    });

    rsx! {
        div { class: "back-nav",
            button {
                class: "btn btn-ghost btn-back",
                onclick: move |_| { navigator().go_back(); },
                Icon { icon: BsArrowLeft, width: 16, height: 16 }
                "Back"
            }
        }

        match &*detail.read() {
            Some(Ok(ds)) => rsx! {
                PageHeader { title: ds.title.clone() }

                if !ds.description.is_empty() {
                    div { class: "card",
                        p { "{ds.description}" }
                    }
                }

                div { class: "card",
                    div { class: "detail-meta",
                        div { class: "detail-meta-item",
                            span { class: "label", "Organization" }
                            span { "{ds.organization}" }
                        }
                        div { class: "detail-meta-item",
                            span { class: "label", "License" }
                            span { "{ds.license}" }
                        }
                        div { class: "detail-meta-item",
                            span { class: "label", "Created" }
                            span { "{ds.created_at}" }
                        }
                        div { class: "detail-meta-item",
                            span { class: "label", "Modified" }
                            span { "{ds.modified_at}" }
                        }
                    }
                    if !ds.tags.is_empty() {
                        div { class: "tags",
                            for tag in &ds.tags {
                                span { class: "tag", "{tag}" }
                            }
                        }
                    }
                }

                div { class: "card",
                    h3 { "Resources" }
                    for res in &ds.resources {
                        div { class: "card",
                            style: "margin: 8px 0; padding: 12px;",
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                span { strong { "{res.name}" } " ({res.format})" }
                                span {
                                    style: "font-size: 12px; color: var(--text-muted);",
                                    "{res.download_status} • {res.row_count} rows"
                                }
                            }
                            if !res.table_name.is_empty() {
                                DataPreview { table_name: res.table_name.clone() }
                            }
                        }
                    }
                }
            },
            Some(Err(e)) => rsx! { div { class: "card", p { "Error: {e}" } } },
            None => rsx! { div { class: "loading", "Loading dataset..." } },
        }
    }
}

#[component]
fn DataPreview(table_name: String) -> Element {
    let tn = table_name.clone();
    let preview = use_resource(move || {
        let tn = tn.clone();
        async move { fetch_table_preview(tn).await }
    });

    let preview_read = preview.read();
    match &*preview_read {
        Some(Ok(rows)) if !rows.is_empty() => {
            let columns: Vec<String> = rows
                .first()
                .and_then(|r| r.as_object())
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default();

            // Clone data out so we don't hold the borrow across rsx!
            let rows_cloned: Vec<serde_json::Value> = rows.clone();
            let cols_cloned = columns.clone();
            drop(preview_read);

            rsx! {
                div { class: "data-table-wrapper",
                    table { class: "data-table",
                        thead {
                            tr {
                                for col in &cols_cloned {
                                    th { "{col}" }
                                }
                            }
                        }
                        tbody {
                            for row in &rows_cloned {
                                tr {
                                    for col in &cols_cloned {
                                        td {
                                            {
                                                let val = row.get(col).cloned().unwrap_or(serde_json::Value::Null);
                                                let display = match &val {
                                                    serde_json::Value::String(s) => s.clone(),
                                                    serde_json::Value::Null => "\u{2014}".to_string(),
                                                    other => other.to_string(),
                                                };
                                                rsx! { "{display}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Ok(_)) => {
            drop(preview_read);
            rsx! { p { style: "color: var(--text-muted); font-size: 13px; margin-top: 8px;", "No data rows" } }
        }
        Some(Err(_)) => {
            drop(preview_read);
            rsx! { p { style: "color: var(--error); font-size: 13px; margin-top: 8px;", "Preview error" } }
        }
        None => {
            drop(preview_read);
            rsx! { p { style: "color: var(--text-muted); font-size: 13px; margin-top: 8px;", "Loading preview..." } }
        }
    }
}
