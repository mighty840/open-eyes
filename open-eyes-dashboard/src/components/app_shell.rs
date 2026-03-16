use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::*;
use dioxus_free_icons::Icon;

use crate::app::Route;
use crate::components::chat_panel::ChatPanel;
use crate::components::sidebar::Sidebar;

#[component]
pub fn AppShell() -> Element {
    let mut chat_visible = use_signal(|| true);
    let mut language = use_signal(|| "de".to_string());

    // Provide language as context so pages can read it
    use_context_provider(|| language);

    // Also provide a callback for opening the chat with a pre-filled question
    let mut chat_prefill = use_signal(String::new);
    use_context_provider(|| chat_prefill);

    rsx! {
        div { class: "app-shell",
            Sidebar {
                language: language(),
                on_language_change: move |lang: String| language.set(lang),
            }
            main { class: "main-content",
                Outlet::<Route> {}
            }
            ChatPanel {
                visible: chat_visible(),
                on_close: move |_| chat_visible.set(false),
                language: language(),
                prefill_question: chat_prefill(),
                on_prefill_consumed: move |_| chat_prefill.set(String::new()),
            }
            if !chat_visible() {
                button {
                    class: "chat-fab",
                    title: "Ask a question about government data",
                    onclick: move |_| chat_visible.set(true),
                    Icon { icon: BsChatDots, width: 24, height: 24 }
                }
            }
        }
    }
}
