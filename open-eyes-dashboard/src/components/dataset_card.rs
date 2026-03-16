use dioxus::prelude::*;

use crate::app::Route;
use crate::infrastructure::datasets::DatasetSummary;

#[component]
pub fn DatasetCard(dataset: DatasetSummary) -> Element {
    let lang_signal: Signal<String> = use_context();
    let language = lang_signal();

    let desc = if dataset.description.len() > 150 {
        format!("{}...", &dataset.description[..150])
    } else {
        dataset.description.clone()
    };

    let files_label = if language == "de" {
        format!("{} Dateien", dataset.resource_count)
    } else {
        format!("{} files", dataset.resource_count)
    };

    rsx! {
        Link {
            to: Route::DatasetDetailPage { dataset_id: dataset.id.clone() },
            class: "dataset-card",
            h4 { "{dataset.title}" }
            if !desc.is_empty() {
                p { class: "description", "{desc}" }
            }
            div { class: "meta",
                if !dataset.organization.is_empty() {
                    span { "{dataset.organization}" }
                }
                span { "{files_label}" }
            }
            if !dataset.tags.is_empty() {
                div { class: "tags",
                    for tag in dataset.tags.iter().take(5) {
                        span { class: "tag", "{tag}" }
                    }
                }
            }
        }
    }
}
