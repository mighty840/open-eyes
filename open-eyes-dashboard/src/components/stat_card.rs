use dioxus::prelude::*;

#[component]
pub fn StatCard(label: String, value: String, #[props(default)] color: String) -> Element {
    let value_style = if color.is_empty() {
        String::new()
    } else {
        format!("color: {color}")
    };

    rsx! {
        div { class: "stat-card",
            div { class: "label", "{label}" }
            div { class: "value", style: value_style, "{value}" }
        }
    }
}
