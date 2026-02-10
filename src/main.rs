#![allow(unused_mut)]

use dioxus::prelude::*;
use gloo_net::http::Request;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{FormData, HtmlInputElement};

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

#[derive(Clone, PartialEq)]
struct DocumentInfo {
    filename: String,
}

#[component]
fn ChatApp() -> Element {
    let mut messages = use_signal(|| Vec::<Message>::new());
    let mut input_value = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut is_listening = use_signal(|| false);
    let mut documents = use_signal(|| Vec::<DocumentInfo>::new());
    let mut upload_status = use_signal(|| String::new());
    let mut is_uploading = use_signal(|| false);

    // Auto-scroll to latest message
    use_effect(move || {
        if !messages().is_empty() {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(messages_div) = document.query_selector(".messages").ok().flatten()
                    {
                        let _ = messages_div.set_scroll_top(messages_div.scroll_height());
                    }
                }
            }
        }
    });

    // TTS function using Web Speech API
    let speak_text = move |text: String| {
        if let Some(window) = web_sys::window() {
            let synthesis = window.speech_synthesis().unwrap();
            if let Ok(utterance) = web_sys::SpeechSynthesisUtterance::new_with_text(&text) {
                synthesis.speak(&utterance);
            }
        }
    };

    // STT function with Web Speech Recognition
    let mut start_listening = move || {
        let mut is_listening_clone = is_listening.clone();
        let mut input_value_clone = input_value.clone();

        is_listening_clone.set(true);

        spawn(async move {
            if let Some(window) = web_sys::window() {
                // Try webkit prefixed version
                let recognition_result =
                    js_sys::Reflect::get(&window, &"webkitSpeechRecognition".into())
                        .or_else(|_| js_sys::Reflect::get(&window, &"SpeechRecognition".into()));

                if let Ok(recognition_class) = recognition_result {
                    if let Ok(func) = recognition_class.dyn_into::<js_sys::Function>() {
                        if let Ok(recognition) =
                            js_sys::Reflect::construct(&func, &js_sys::Array::new())
                        {
                            // Set properties
                            let _ = js_sys::Reflect::set(
                                &recognition,
                                &"continuous".into(),
                                &false.into(),
                            );
                            let _ = js_sys::Reflect::set(
                                &recognition,
                                &"interimResults".into(),
                                &false.into(),
                            );

                            // Create onresult callback
                            let mut input_clone = input_value_clone.clone();
                            let mut listening_clone = is_listening_clone.clone();
                            let onresult = wasm_bindgen::closure::Closure::wrap(Box::new(
                                move |event: web_sys::Event| {
                                    if let Ok(results) =
                                        js_sys::Reflect::get(&event, &"results".into())
                                    {
                                        if let Ok(result) =
                                            js_sys::Reflect::get(&results, &0.into())
                                        {
                                            if let Ok(alternative) =
                                                js_sys::Reflect::get(&result, &0.into())
                                            {
                                                if let Ok(transcript) = js_sys::Reflect::get(
                                                    &alternative,
                                                    &"transcript".into(),
                                                ) {
                                                    if let Some(text) = transcript.as_string() {
                                                        input_clone.set(text);
                                                        listening_clone.set(false);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                            )
                                as Box<dyn FnMut(web_sys::Event)>);

                            let _ = js_sys::Reflect::set(
                                &recognition,
                                &"onresult".into(),
                                onresult.as_ref(),
                            );

                            // Create onerror callback
                            let mut listening_clone2 = is_listening_clone.clone();
                            let onerror = wasm_bindgen::closure::Closure::wrap(Box::new(
                                move |_event: web_sys::Event| {
                                    listening_clone2.set(false);
                                },
                            )
                                as Box<dyn FnMut(web_sys::Event)>);

                            let _ = js_sys::Reflect::set(
                                &recognition,
                                &"onerror".into(),
                                onerror.as_ref(),
                            );

                            // Start recognition
                            if let Ok(start_fn) =
                                js_sys::Reflect::get(&recognition, &"start".into())
                            {
                                let _ = js_sys::Reflect::apply(
                                    &start_fn.dyn_into::<js_sys::Function>().unwrap(),
                                    &recognition,
                                    &js_sys::Array::new(),
                                );
                            }

                            // Keep closures alive
                            onresult.forget();
                            onerror.forget();
                        }
                    }
                } else {
                    // Fallback if speech recognition not available
                    is_listening_clone.set(false);
                }
            }
        });
    };

    // Chat handler
    let handle_send = move || {
        spawn({
            let mut messages = messages.clone();
            let mut input_value = input_value.clone();
            let mut is_loading = is_loading.clone();
            async move {
                send_message(messages, input_value, is_loading).await;
            }
        });
    };

    // Delete document handler
    let handle_delete = move |filename: String| {
        spawn({
            let mut documents = documents.clone();
            let mut upload_status = upload_status.clone();
            async move {
                match Request::post("http://localhost:3000/delete")
                    .json(&json!({ "filename": filename }))
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            documents.write().retain(|doc| doc.filename != filename);
                            upload_status.set(format!("✅ Removed {}", filename));
                        } else {
                            upload_status.set(format!("❌ Failed to delete {}", filename));
                        }
                    }
                    Err(_) => {
                        upload_status.set("❌ Failed to connect to backend".to_string());
                    }
                }
            }
        });
    };

    let handle_keydown = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter && !evt.modifiers().shift() {
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

                // Upload section with document management
                div {
                    class: "upload-section",
                    h2 { "📁 Document Upload" }

                    div {
                        class: "upload-controls",
                        input {
                            r#type: "file",
                            accept: ".md,.pdf",
                            multiple: true,
                            id: "file-upload",
                            style: "display: none;",
                            onchange: move |_| {
                                let mut documents_clone = documents.clone();
                                let mut upload_status_clone = upload_status.clone();
                                let mut is_uploading_clone = is_uploading.clone();

                                spawn(async move {
                                    is_uploading_clone.set(true);
                                    upload_status_clone.set("📤 Uploading files...".to_string());

                                    // Get the file input element
                                    if let Some(window) = web_sys::window() {
                                        if let Some(document) = window.document() {
                                            if let Some(element) = document.get_element_by_id("file-upload") {
                                                if let Ok(input) = element.dyn_into::<HtmlInputElement>() {
                                                    if let Some(files) = input.files() {
                                                        let file_count = files.length();

                                                        if file_count == 0 {
                                                            upload_status_clone.set("❌ No files selected".to_string());
                                                            is_uploading_clone.set(false);
                                                            return;
                                                        }

                                                        // Create FormData with all files
                                                        if let Ok(form_data) = FormData::new() {
                                                            let mut filenames = Vec::new();

                                                            for i in 0..file_count {
                                                                if let Some(file) = files.get(i) {
                                                                    let filename = file.name();
                                                                    filenames.push(filename.clone());
                                                                    let _ = form_data.append_with_blob("files", &file);
                                                                }
                                                            }

                                                            // Upload using fetch API
                                                            match upload_files_formdata(form_data).await {
                                                                Ok(_response) => {
                                                                    // Add uploaded files to documents list
                                                                    for filename in &filenames {
                                                                        if !documents_clone.read().iter().any(|d| &d.filename == filename) {
                                                                            documents_clone.write().push(DocumentInfo {
                                                                                filename: filename.clone(),
                                                                            });
                                                                        }
                                                                    }
                                                                    upload_status_clone.set(format!("✅ Successfully uploaded {} file(s)", filenames.len()));
                                                                }
                                                                Err(e) => {
                                                                    upload_status_clone.set(format!("❌ Upload failed: {}", e));
                                                                }
                                                            }

                                                            // Clear the input
                                                            input.set_value("");
                                                        } else {
                                                            upload_status_clone.set("❌ Failed to create form data".to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    is_uploading_clone.set(false);
                                });
                            }
                        }
                        button {
                            class: "upload-button",
                            disabled: is_uploading(),
                            onclick: move |_| {
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Some(el) = doc.get_element_by_id("file-upload") {
                                        if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                                            let _ = input.click();
                                        }
                                    }
                                }
                            },
                            if is_uploading() {
                                "⏳ Uploading..."
                            } else {
                                "📁 Choose Files (.md, .pdf)"
                            }
                        }
                    }

                    if !upload_status().is_empty() {
                        div {
                            class: if is_uploading() { "status uploading" } else { "status" },
                            "{upload_status}"
                        }
                    }

                    div {
                        class: "documents-list",
                        h3 { "📚 Uploaded Documents" }

                        if documents().is_empty() {
                            p {
                                class: "hint",
                                style: "text-align: center; padding: 20px; color: var(--text-secondary);",
                                "No documents uploaded yet. Upload .md or .pdf files to get started!"
                            }
                        } else {
                            div {
                                class: "documents-grid",
                                for doc in documents() {
                                    div {
                                        class: "document-card",
                                        div {
                                            class: "document-icon",
                                            if doc.filename.ends_with(".pdf") {
                                                "📄"
                                            } else {
                                                "📝"
                                            }
                                        }
                                        div {
                                            class: "document-details",
                                            div {
                                                class: "document-name",
                                                title: "{doc.filename}",
                                                "{doc.filename}"
                                            }
                                        }
                                        button {
                                            class: "delete-button",
                                            title: "Remove document",
                                            onclick: {
                                                let filename = doc.filename.clone();
                                                move |_| handle_delete(filename.clone())
                                            },
                                            "🗑️"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Chat section
                div {
                    class: "chat-section",
                    h2 { "💬 Chat" }

                    // Messages
                    div {
                        class: "messages",
                        id: "messages-container",

                        if messages().is_empty() {
                            div {
                                class: "empty-state",
                                p { "👋 Upload documents and start chatting!" }
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

                                    // TTS button for assistant messages
                                    if msg.role == "assistant" {
                                        button {
                                            class: "tts-button",
                                            onclick: {
                                                let content = msg.content.clone();
                                                move |_| speak_text(content.clone())
                                            },
                                            "📢"
                                        }
                                    }
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
                                    span { "💭 Thinking..." }
                                }
                            }
                        }
                    }

                    // Input area with Deep Think toggle
                    div {
                        class: "input-area",

                        div {
                            class: "input-row",
                            // Microphone button
                            button {
                                class: "mic-button",
                                disabled: is_listening() || is_loading(),
                                onclick: move |_| start_listening(),
                                title: "Voice input",
                                if is_listening() {
                                    "🎤 Listening..."
                                } else {
                                    "🎤"
                                }
                            }

                            textarea {
                                class: "chat-input",
                                placeholder: "Ask a question about your documents...",
                                value: "{input_value}",
                                disabled: is_loading() || is_listening(),
                                rows: 2,
                                oninput: move |evt| input_value.set(evt.value()),
                                onkeydown: handle_keydown,
                            }
                        }

                        div {
                            class: "controls-row",
                            button {
                                class: "send-button",
                                disabled: is_loading() || input_value().trim().is_empty() || is_listening(),
                                onclick: move |_| handle_send(),
                                if is_loading() {
                                    "⏳ Sending..."
                                } else {
                                    "🚀 Send"
                                }
                            }
                        }
                    }
                }
            }

            // Footer
            footer {
                class: "footer",
                p { "Built with Dioxus 🦀 | Powered by Ollama (phi3) + fastembed" }
                p { class: "hint", "Backend: http://localhost:3000" }
            }
        }
    }
}

async fn send_message(
    mut messages: Signal<Vec<Message>>,
    mut input_value: Signal<String>,
    mut is_loading: Signal<bool>,
) {
    let query = input_value().trim().to_string();
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

    // Send to backend
    match Request::post("http://localhost:3000/chat")
        .json(&json!({
            "query": query,
            "deep_think": false
        }))
        .unwrap()
        .send()
        .await
    {
        Ok(resp) => {
            if resp.ok() {
                match resp.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let answer = data["answer"].as_str().unwrap_or("No answer").to_string();
                        let sources: Vec<String> = data["sources"]
                            .as_array()
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
                content: "Failed to connect to backend. Is it running on port 3000?".to_string(),
                sources: vec![],
            });
        }
    }

    is_loading.set(false);
}

async fn upload_files_formdata(form_data: FormData) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;

    // Create request
    let mut opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    let form_data_value: &JsValue = form_data.as_ref();
    opts.set_body(form_data_value);

    let request = web_sys::Request::new_with_str_and_init("http://localhost:3000/upload", &opts)
        .map_err(|_| "Failed to create request")?;

    // Send request
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Invalid response")?;

    if resp.ok() {
        Ok("Upload successful".to_string())
    } else {
        Err(format!("Upload failed with status {}", resp.status()))
    }
}
