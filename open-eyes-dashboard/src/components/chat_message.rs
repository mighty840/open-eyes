use dioxus::prelude::*;

use crate::components::chart_container::ChartContainer;

#[component]
pub fn ChatMessageBubble(
    role: String,
    content: String,
    #[props(default)] sql: String,
    #[props(default)] chart_spec: String,
) -> Element {
    let class = if role == "user" {
        "chat-message user"
    } else {
        "chat-message assistant"
    };

    rsx! {
        div { class: class,
            p { "{content}" }
            if !sql.is_empty() {
                div { class: "sql-preview", "{sql}" }
            }
            if !chart_spec.is_empty() && chart_spec != "{}" {
                ChartContainer { option_json: chart_spec }
            }
        }
    }
}

#[component]
pub fn ThinkingIndicator() -> Element {
    rsx! {
        div { class: "chat-message assistant",
            div { class: "thinking",
                div { class: "thinking-dot" }
                div { class: "thinking-dot" }
                div { class: "thinking-dot" }
            }
        }
    }
}
