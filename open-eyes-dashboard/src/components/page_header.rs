use dioxus::prelude::*;

#[component]
pub fn PageHeader(title: String, #[props(default)] description: String) -> Element {
    rsx! {
        div { class: "page-header",
            h2 { "{title}" }
            if !description.is_empty() {
                p { "{description}" }
            }
        }
    }
}
