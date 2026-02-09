use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use ndarray::{Array1, ArrayView1};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Clone)]
struct AppState {
    vector_store: Arc<RwLock<VectorStore>>,
    embedding_model: Arc<Mutex<TextEmbedding>>,
    ollama_client: reqwest::Client,
}

struct VectorStore {
    chunks: Vec<ChunkData>,
}

#[derive(Clone)]
struct ChunkData {
    text: String,
    embedding: Vec<f32>,
    source: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    query: String,
}

#[derive(Serialize)]
struct ChatResponse {
    answer: String,
    sources: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

// ============================================================================
// VECTOR STORE IMPLEMENTATION
// ============================================================================

impl VectorStore {
    fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    fn add(&mut self, text: String, embedding: Vec<f32>, source: String) {
        self.chunks.push(ChunkData {
            text,
            embedding,
            source,
        });
    }

    fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(String, String, f32)> {
        if self.chunks.is_empty() {
            return Vec::new();
        }

        let query_vec = Array1::from_vec(query_embedding.to_vec());
        let mut scores: Vec<(usize, f32)> = self
            .chunks
            .iter()
            .enumerate()
            .map(|(idx, chunk)| {
                let chunk_vec = Array1::from_vec(chunk.embedding.clone());
                let similarity = cosine_similarity(query_vec.view(), chunk_vec.view());
                (idx, similarity)
            })
            .collect();

        // Sort by similarity (highest first)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scores
            .into_iter()
            .take(top_k)
            .map(|(idx, score)| {
                let chunk = &self.chunks[idx];
                (chunk.text.clone(), chunk.source.clone(), score)
            })
            .collect()
    }

    fn count(&self) -> usize {
        self.chunks.len()
    }
}

fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

// ============================================================================
// TEXT PROCESSING
// ============================================================================

fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk = words[start..end].join(" ");
        chunks.push(chunk);

        if end >= words.len() {
            break;
        }

        start += chunk_size - overlap;
    }

    chunks
}

fn extract_text_from_md(content: &[u8]) -> Result<String, String> {
    String::from_utf8(content.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))
}

fn extract_text_from_pdf(content: &[u8]) -> Result<String, String> {
    // Try lopdf first
    match lopdf::Document::load_mem(content) {
        Ok(doc) => {
            let mut text = String::new();
            let pages = doc.get_pages();

            for (page_num, _) in pages.iter() {
                if let Ok(page_text) = doc.extract_text(&[*page_num]) {
                    text.push_str(&page_text);
                    text.push('\n');
                }
            }

            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
        Err(e) => {
            warn!("lopdf failed: {}, trying pdf-extract", e);
        }
    }

    // Fallback to pdf-extract
    match pdf_extract::extract_text_from_mem(content) {
        Ok(text) => {
            if text.trim().is_empty() {
                Err("PDF contains no extractable text".to_string())
            } else {
                Ok(text)
            }
        }
        Err(e) => Err(format!("Failed to extract PDF text: {}", e)),
    }
}

// ============================================================================
// HANDLERS
// ============================================================================

async fn health_check() -> &'static str {
    "OK"
}

async fn upload_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut processed_files = Vec::new();
    let mut errors = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let filename = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let content = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                errors.push(format!("Failed to read {}: {}", filename, e));
                continue;
            }
        };

        info!("Processing file: {} ({} bytes)", filename, content.len());

        // Extract text based on file type
        let text = if filename.ends_with(".md") {
            match extract_text_from_md(&content) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("Failed to parse {}: {}", filename, e));
                    continue;
                }
            }
        } else if filename.ends_with(".pdf") {
            match extract_text_from_pdf(&content) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("Failed to parse {}: {}", filename, e));
                    continue;
                }
            }
        } else {
            errors.push(format!("Unsupported file type: {}", filename));
            continue;
        };

        if text.trim().is_empty() {
            errors.push(format!("No text extracted from {}", filename));
            continue;
        }

        info!("Extracted {} characters from {}", text.len(), filename);

        // Chunk text (512 tokens ~= 512 words with 50 word overlap)
        let chunks = chunk_text(&text, 512, 50);
        info!("Created {} chunks from {}", chunks.len(), filename);

        // Generate embeddings and store
        let texts_for_embedding: Vec<String> = chunks.clone();

        let embedding_model = state.embedding_model.clone();
        let embeddings_result = tokio::task::spawn_blocking(move || {
            let model = embedding_model.lock().unwrap();
            model.embed(texts_for_embedding, None)
        })
        .await
        .unwrap();

        match embeddings_result {
            Ok(embeddings) => {
                let mut store = state.vector_store.write().unwrap();

                for (chunk_text, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
                    store.add(chunk_text, embedding, filename.clone());
                }

                processed_files.push(filename.clone());
                info!("Successfully processed {}", filename);
            }
            Err(e) => {
                errors.push(format!(
                    "Failed to generate embeddings for {}: {}",
                    filename, e
                ));
            }
        }
    }

    let store = state.vector_store.read().unwrap();
    let total_chunks = store.count();
    drop(store);

    if processed_files.is_empty() && !errors.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "errors": errors,
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "processed_files": processed_files,
            "total_chunks": total_chunks,
            "errors": errors,
        })),
    )
}

fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    Box::pin(async move { chat_handler_impl(state, payload).await })
}

async fn chat_handler_impl(state: AppState, payload: ChatRequest) -> Response {
    let query = payload.query.trim();
    info!("Received chat query: {}", query);

    if query.is_empty() {
        return (
            StatusCode::OK,
            Json(ChatResponse {
                answer: "Ask something!".to_string(),
                sources: vec![],
            }),
        )
            .into_response();
    }

    // Check if documents are uploaded
    let has_docs = {
        let store = state.vector_store.read().unwrap();
        store.count() > 0
    };

    if !has_docs {
        return (
            StatusCode::OK,
            Json(ChatResponse {
                answer: "Upload documents first!".to_string(),
                sources: vec![],
            }),
        )
            .into_response();
    }

    // Generate query embedding
    info!("Generating query embedding...");
    let query_string = query.to_string();
    let embedding_model = state.embedding_model.clone();
    let embedding_result = tokio::task::spawn_blocking(move || {
        let model = embedding_model.lock().unwrap();
        model.embed(vec![query_string], None)
    })
    .await
    .unwrap();
    info!("Query embedding generated");

    let query_embedding = match embedding_result {
        Ok(mut embeddings) => {
            if embeddings.is_empty() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ChatResponse {
                        answer: "Failed to generate query embedding".to_string(),
                        sources: vec![],
                    }),
                )
                    .into_response();
            }
            embeddings.remove(0)
        }
        Err(e) => {
            error!("Embedding generation failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatResponse {
                    answer: format!("Failed to generate query embedding: {}", e),
                    sources: vec![],
                }),
            )
                .into_response();
        }
    };

    // Search vector store
    info!("Searching vector store...");
    let results = {
        let store = state.vector_store.read().unwrap();
        store.search(&query_embedding, 3)
    };
    info!("Found {} results", results.len());

    if results.is_empty() {
        return (
            StatusCode::OK,
            Json(ChatResponse {
                answer: "No relevant information found in the documents.".to_string(),
                sources: vec![],
            }),
        )
            .into_response();
    }

    // Build context from top results
    let context: String = results
        .iter()
        .enumerate()
        .map(|(i, (text, _source, score))| {
            format!("[Chunk {}] (relevance: {:.2})\n{}\n", i + 1, score, text)
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n");

    let sources: Vec<String> = results
        .iter()
        .map(|(_, source, _)| source.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    // Build prompt with strict grounding
    let prompt = format!(
        r#"You are a helpful assistant that answers questions based ONLY on the provided context.

CRITICAL RULES:
1. Answer ONLY using information from the context below
2. If the answer is not in the context, respond EXACTLY with: "I don't know based on the provided documents."
3. Do not use external knowledge or make assumptions
4. Be concise and direct

Context:
{}

Question: {}

Answer:"#,
        context, query
    );

    // Call Ollama
    let ollama_request = serde_json::json!({
        "model": "phi3",
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.1,
            "num_predict": 512,
        }
    });

    info!("Sending request to Ollama...");
    match state
        .ollama_client
        .post("http://localhost:11434/api/generate")
        .json(&ollama_request)
        .timeout(Duration::from_secs(60))
        .send()
        .await
    {
        Ok(response) => {
            info!("Received response from Ollama: {}", response.status());
            if response.status().is_success() {
                match response.json::<OllamaResponse>().await {
                    Ok(ollama_resp) => {
                        let answer = ollama_resp.response.trim().to_string();
                        info!("Successfully generated answer: {} chars", answer.len());
                        return (StatusCode::OK, Json(ChatResponse { answer, sources }))
                            .into_response();
                    }
                    Err(e) => {
                        error!("Failed to parse Ollama response: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ChatResponse {
                                answer: format!("Failed to parse Ollama response: {}", e),
                                sources: vec![],
                            }),
                        )
                            .into_response();
                    }
                }
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                error!("Ollama returned error {}: {}", status, error_text);
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ChatResponse {
                        answer: format!("Ollama error: {} - {}", status, error_text),
                        sources: vec![],
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            error!("Failed to connect to Ollama: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                Json(ChatResponse {
                    answer: "Start Ollama first: `ollama serve`".to_string(),
                    sources: vec![],
                }),
            )
                .into_response();
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("🚀 Starting RAG Chatbot Backend");

    // Initialize embedding model
    info!("📚 Loading embedding model (all-MiniLM-L6-v2)...");
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
    )
    .expect("Failed to load embedding model");
    info!("✅ Embedding model loaded");

    // Initialize HTTP client for Ollama
    let ollama_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Test Ollama connection
    info!("🔍 Testing Ollama connection...");
    match ollama_client
        .get("http://localhost:11434/api/tags")
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(_) => info!("✅ Ollama is running"),
        Err(_) => {
            warn!("⚠️  Ollama is not running. Start it with: ollama serve");
            warn!("⚠️  Then run: ollama pull phi3");
        }
    }

    // Create app state
    let state = AppState {
        vector_store: Arc::new(RwLock::new(VectorStore::new())),
        embedding_model: Arc::new(Mutex::new(model)),
        ollama_client,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(upload_handler))
        .route("/chat", post(chat_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    info!("🎯 Backend running at http://{}", addr);
    info!("📖 Endpoints:");
    info!("   - GET  /health");
    info!("   - POST /upload (multipart/form-data)");
    info!("   - POST /chat (JSON)");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
