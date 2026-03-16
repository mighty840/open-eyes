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

const SUGGESTED_QUESTIONS_DE: &[&str] = &[
    "Welche Kategorien haben die meisten Datensätze?",
    "Zeige die neuesten Datensätze",
    "Welche Organisationen veröffentlichen am meisten?",
    "Welche Datenformate werden am häufigsten genutzt?",
];

const SUGGESTED_QUESTIONS_EN: &[&str] = &[
    "Which categories have the most data?",
    "Show the latest published datasets",
    "Which organizations publish the most?",
    "What data formats are most common?",
];

#[component]
pub fn ChatPanel(
    visible: bool,
    on_close: EventHandler<()>,
    language: String,
    #[props(default)] prefill_question: String,
    #[props(default)] on_prefill_consumed: EventHandler<()>,
) -> Element {
    let mut messages = use_signal(Vec::<MessageEntry>::new);
    let mut input_text = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let session_id = use_signal(|| format!("session-{}", chrono::Utc::now().timestamp_millis()));

    // Handle prefilled question from homepage suggestion cards
    let lang_clone = language.clone();
    use_effect(move || {
        let q = prefill_question.clone();
        if !q.is_empty() && !loading() {
            let lang = lang_clone.clone();
            let sid = session_id();
            messages.write().push(MessageEntry {
                role: "user".into(),
                content: q.clone(),
                sql: String::new(),
                chart_spec: String::new(),
            });
            loading.set(true);
            on_prefill_consumed(());
            spawn(async move {
                handle_response(&mut messages, ask_question(q, sid, lang).await);
                loading.set(false);
            });
        }
    });

    let panel_class = if visible {
        "chat-panel"
    } else {
        "chat-panel hidden"
    };

    let suggestions = if language == "de" {
        SUGGESTED_QUESTIONS_DE
    } else {
        SUGGESTED_QUESTIONS_EN
    };

    let header_title = if language == "de" {
        "Daten-Assistent"
    } else {
        "Data Assistant"
    };

    let placeholder = if language == "de" {
        "Stellen Sie eine Frage zu den Daten..."
    } else {
        "Ask a question about the data..."
    };

    rsx! {
        div { class: panel_class,
            div { class: "chat-header",
                h3 {
                    Icon { icon: BsChatDots, width: 16, height: 16 }
                    "{header_title}"
                }
                button {
                    class: "btn btn-ghost",
                    onclick: move |_| on_close(()),
                    Icon { icon: BsXLg, width: 16, height: 16 }
                }
            }

            div { class: "chat-messages",
                if messages().is_empty() {
                    div { class: "chat-welcome",
                        div { class: "chat-welcome-icon",
                            Icon { icon: BsEye, width: 24, height: 24 }
                        }
                        h4 {
                            if language == "de" { "Willkommen bei Open Eyes" } else { "Welcome to Open Eyes" }
                        }
                        p {
                            if language == "de" {
                                "Stellen Sie Fragen zu deutschen Regierungsdaten in natürlicher Sprache. Ich suche die passenden Datensätze, analysiere sie und erstelle Visualisierungen."
                            } else {
                                "Ask questions about German government data in plain language. I'll find relevant datasets, analyze them, and create visualizations."
                            }
                        }
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

            // Show suggestion chips when chat is empty
            if messages().is_empty() && !loading() {
                div { class: "chat-suggestions",
                    for suggestion in suggestions.iter() {
                        {
                            let q = suggestion.to_string();
                            let lang = language.clone();
                            rsx! {
                                button {
                                    class: "suggestion-chip",
                                    onclick: move |_| {
                                        let q = q.clone();
                                        let lang = lang.clone();
                                        let sid = session_id();
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
                                    },
                                    "{suggestion}"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "chat-input",
                input {
                    r#type: "text",
                    placeholder: placeholder,
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
            let friendly_msg = format!(
                "Es tut mir leid, bei der Verarbeitung Ihrer Anfrage ist ein Fehler aufgetreten. Bitte versuchen Sie es mit einer anderen Formulierung.\n\nTechnische Details: {e}"
            );
            messages.write().push(MessageEntry {
                role: "assistant".into(),
                content: friendly_msg,
                sql: String::new(),
                chart_spec: String::new(),
            });
        }
    }
}
