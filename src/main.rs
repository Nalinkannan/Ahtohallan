use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "Ahtohallan - RAG Chatbot" }
        ChatApp {}
    }
}

#[derive(Clone, PartialEq)]
struct Message {
    role: String,
    content: String,
    sources: Vec<String>,
}

#[component]
fn ChatApp() -> Element {
    let mut messages = use_signal(|| Vec::<Message>::new());
    let mut input_value = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);

    // Chat handler
    let handle_send = move || {
        spawn(async move {
            let query = input_value.read().trim().to_string();

            if query.is_empty() {
                return;
            }

            // Add user message
            messages.write().push(Message {
                role: "user".to_string(),
                content: query.clone(),
                sources: vec![],
            });

            input_value.set(String::new());
            is_loading.set(true);

            // Send to backend using gloo-net
            let response = gloo_net::http::Request::post("http://localhost:3000/chat")
                .json(&serde_json::json!({
                    "query": query
                }))
                .unwrap()
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.ok() {
                        match resp.json::<serde_json::Value>().await {
                            Ok(data) => {
                                let answer = data
                                    .get("answer")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("No response")
                                    .to_string();

                                let sources: Vec<String> = data
                                    .get("sources")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default();

                                messages.write().push(Message {
                                    role: "assistant".to_string(),
                                    content: answer,
                                    sources,
                                });
                            }
                            Err(_) => {
                                messages.write().push(Message {
                                    role: "error".to_string(),
                                    content: "Failed to parse response".to_string(),
                                    sources: vec![],
                                });
                            }
                        }
                    } else {
                        let error_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| format!("Status: {}", resp.status()));
                        messages.write().push(Message {
                            role: "error".to_string(),
                            content: format!("Server error: {}", error_text),
                            sources: vec![],
                        });
                    }
                }
                Err(_) => {
                    messages.write().push(Message {
                        role: "error".to_string(),
                        content: "Failed to connect to backend. Is it running on port 3000?"
                            .to_string(),
                        sources: vec![],
                    });
                }
            }

            is_loading.set(false);
        });
    };

    let handle_keydown = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter {
            evt.prevent_default();
            handle_send();
        }
    };

    rsx! {
        div {
            class: "app-container",

            // Header
            header {
                class: "header",
                h1 { "❄️ Ahtohallan" }
                p { class: "subtitle", "RAG-Powered Document Chat" }
            }

            // Main content
            div {
                class: "main-content",

                // Upload section (placeholder - use curl to upload)
                div {
                    class: "upload-section",
                    h2 { "📁 Upload Documents" }
                    p { class: "status", "Use the backend API to upload documents:" }
                    pre {
                        class: "code-block",
                        "curl -X POST http://localhost:3000/upload \\\n  -F \"files=@document.pdf\""
                    }
                    p { class: "hint", "Upload .md or .pdf files via curl or API client" }
                }

                // Chat section
                div {
                    class: "chat-section",
                    h2 { "💬 Chat" }

                    // Messages
                    div {
                        class: "messages",

                        if messages().is_empty() {
                            div {
                                class: "empty-state",
                                p { "👋 Upload documents via API and start chatting!" }
                                p { class: "hint", "Your questions will be answered using only the uploaded documents." }
                            }
                        }

                        for msg in messages() {
                            div {
                                class: "message {msg.role}",
                                div {
                                    class: "message-content",
                                    if msg.role == "user" {
                                        strong { "You: " }
                                    } else if msg.role == "assistant" {
                                        strong { "🤖 Assistant: " }
                                    } else {
                                        strong { "⚠️ Error: " }
                                    }
                                    span { "{msg.content}" }
                                }

                                if !msg.sources.is_empty() {
                                    div {
                                        class: "sources",
                                        strong { "📚 Sources: " }
                                        for source in msg.sources.iter() {
                                            span { class: "source-tag", "{source}" }
                                        }
                                    }
                                }
                            }
                        }

                        if is_loading() {
                            div {
                                class: "message assistant loading",
                                div {
                                    class: "message-content",
                                    strong { "🤖 Assistant: " }
                                    span { "Thinking..." }
                                }
                            }
                        }
                    }

                    // Input
                    div {
                        class: "input-area",
                        textarea {
                            class: "chat-input",
                            placeholder: "Ask a question about your documents...",
                            value: "{input_value}",
                            disabled: is_loading(),
                            rows: 2,
                            oninput: move |evt| input_value.set(evt.value()),
                            onkeydown: handle_keydown,
                        }

                        button {
                            class: "send-button",
                            disabled: is_loading() || input_value().trim().is_empty(),
                            onclick: move |_| handle_send(),
                            if is_loading() {
                                "⏳"
                            } else {
                                "🚀 Send"
                            }
                        }
                    }
                }
            }

            // Footer
            footer {
                class: "footer",
                p { "Built with Dioxus 🦀 | Powered by Ollama (phi3) + fastembed" }
                p { class: "hint", "Backend: http://localhost:3000 | Upload via: curl -X POST http://localhost:3000/upload -F 'files=@file.pdf'" }
            }
        }
    }
}
