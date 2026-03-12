use dioxus::prelude::*;

#[component]
pub fn SearchBar(placeholder: String, value: String, on_input: EventHandler<String>) -> Element {
    rsx! {
        div { class: "search-bar",
            input {
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{value}",
                oninput: move |e: FormEvent| on_input(e.value()),
            }
        }
    }
}
