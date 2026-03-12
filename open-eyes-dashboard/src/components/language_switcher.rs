use dioxus::prelude::*;

#[component]
pub fn LanguageSwitcher(current: String, on_change: EventHandler<String>) -> Element {
    let de_class = if current == "de" {
        "lang-btn active"
    } else {
        "lang-btn"
    };
    let en_class = if current == "en" {
        "lang-btn active"
    } else {
        "lang-btn"
    };

    rsx! {
        div { class: "language-switcher",
            button {
                class: de_class,
                onclick: move |_| on_change("de".to_string()),
                "DE"
            }
            button {
                class: en_class,
                onclick: move |_| on_change("en".to_string()),
                "EN"
            }
        }
    }
}
