use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::*;
use dioxus_free_icons::Icon;

use crate::components::chat_message::{ChatMessageBubble, ThinkingIndicator};
use crate::infrastructure::chat::{ask_question, ChatResult};

#[derive(Clone)]
struct MessageEntry {
    role: String,
    content: String,
    sql: String,
    chart_spec: String,
}

#[component]
pub fn ChatPanel(visible: bool, on_close: EventHandler<()>, language: String) -> Element {
    let mut messages = use_signal(Vec::<MessageEntry>::new);
    let mut input_text = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let session_id = use_signal(|| format!("session-{}", chrono::Utc::now().timestamp_millis()));

    let panel_class = if visible {
        "chat-panel"
    } else {
        "chat-panel hidden"
    };

    rsx! {
        div { class: panel_class,
            div { class: "chat-header",
                h3 { "Chat" }
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| on_close(()),
                    Icon { icon: BsXLg, width: 16, height: 16 }
                }
            }

            div { class: "chat-messages",
                if messages().is_empty() {
                    div { class: "chat-message assistant",
                        p { "Ask me anything about German open government data. I'll find relevant datasets, query them, and visualize the results." }
                    }
                }

                for msg in messages() {
                    ChatMessageBubble {
                        role: msg.role.clone(),
                        content: msg.content.clone(),
                        sql: msg.sql.clone(),
                        chart_spec: msg.chart_spec.clone(),
                    }
                }

                if loading() {
                    ThinkingIndicator {}
                }
            }

            div { class: "chat-input",
                input {
                    r#type: "text",
                    placeholder: "Ask a question about the data...",
                    value: "{input_text}",
                    disabled: loading(),
                    oninput: move |e: FormEvent| input_text.set(e.value()),
                    onkeypress: {
                        let lang = language.clone();
                        move |e: KeyboardEvent| {
                            if e.key() == Key::Enter && !loading() {
                                let q = input_text();
                                if !q.trim().is_empty() {
                                    let lang = lang.clone();
                                    let sid = session_id();
                                    input_text.set(String::new());
                                    messages.write().push(MessageEntry {
                                        role: "user".into(),
                                        content: q.clone(),
                                        sql: String::new(),
                                        chart_spec: String::new(),
                                    });
                                    loading.set(true);
                                    spawn(async move {
                                        handle_response(&mut messages, ask_question(q, sid, lang).await);
                                        loading.set(false);
                                    });
                                }
                            }
                        }
                    },
                }
                button {
                    disabled: loading(),
                    onclick: {
                        let lang = language.clone();
                        move |_| {
                            let q = input_text();
                            if !q.trim().is_empty() && !loading() {
                                let lang = lang.clone();
                                let sid = session_id();
                                input_text.set(String::new());
                                messages.write().push(MessageEntry {
                                    role: "user".into(),
                                    content: q.clone(),
                                    sql: String::new(),
                                    chart_spec: String::new(),
                                });
                                loading.set(true);
                                spawn(async move {
                                    handle_response(&mut messages, ask_question(q, sid, lang).await);
                                    loading.set(false);
                                });
                            }
                        }
                    },
                    Icon { icon: BsSend, width: 16, height: 16 }
                }
            }
        }
    }
}

fn handle_response(
    messages: &mut Signal<Vec<MessageEntry>>,
    result: Result<ChatResult, dioxus::prelude::ServerFnError>,
) {
    match result {
        Ok(resp) => {
            messages.write().push(MessageEntry {
                role: "assistant".into(),
                content: resp.summary,
                sql: resp.sql,
                chart_spec: resp.chart_option_json,
            });
        }
        Err(e) => {
            messages.write().push(MessageEntry {
                role: "assistant".into(),
                content: format!("Error: {e}"),
                sql: String::new(),
                chart_spec: String::new(),
            });
        }
    }
}
