# 🏗️ Architecture Documentation

> Technical deep-dive into Ahtohallan RAG Chatbot architecture

## Table of Contents

- [System Overview](#system-overview)
- [Component Architecture](#component-architecture)
- [Data Flow](#data-flow)
- [Vector Store Implementation](#vector-store-implementation)
- [Embedding Pipeline](#embedding-pipeline)
- [RAG Pipeline](#rag-pipeline)
- [API Design](#api-design)
- [Performance Characteristics](#performance-characteristics)
- [Design Decisions](#design-decisions)
- [Scaling Considerations](#scaling-considerations)

---

## System Overview

Ahtohallan is a Retrieval-Augmented Generation (RAG) chatbot built entirely in Rust. It follows a classic client-server architecture with these key components:

```
┌────────────────────────────────────────────────────────────────┐
│                        User Browser                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │          Dioxus Web Frontend (WASM)                       │  │
│  │  - File upload UI                                         │  │
│  │  - Chat interface                                         │  │
│  │  - Real-time updates                                      │  │
│  └───────────────┬──────────────────────────────────────────┘  │
└──────────────────┼─────────────────────────────────────────────┘
                   │ HTTP/JSON
                   │
┌──────────────────▼─────────────────────────────────────────────┐
│               Axum Backend Server                               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  HTTP Handlers                                            │  │
│  │  - /upload (multipart)                                    │  │
│  │  - /chat (JSON)                                           │  │
│  │  - /health                                                │  │
│  └───────┬────────────────────────────────────────┬──────────┘  │
│          │                                        │             │
│  ┌───────▼──────────┐                  ┌─────────▼──────────┐  │
│  │  Document        │                  │  Query             │  │
│  │  Processor       │                  │  Handler           │  │
│  │  - PDF extract   │                  │  - Embedding       │  │
│  │  - MD parsing    │                  │  - Vector search   │  │
│  │  - Chunking      │                  │  - LLM call        │  │
│  └───────┬──────────┘                  └─────────┬──────────┘  │
│          │                                       │             │
│  ┌───────▼───────────────────────────────────────▼──────────┐  │
│  │            In-Memory Vector Store                         │  │
│  │  - ChunkData: Vec<{text, embedding, source}>             │  │
│  │  - Cosine similarity search                              │  │
│  │  - Thread-safe (RwLock)                                  │  │
│  └───────┬────────────────────────────────────────┬──────────┘  │
└──────────┼────────────────────────────────────────┼─────────────┘
           │                                        │
    ┌──────▼──────┐                        ┌────────▼────────┐
    │  fastembed  │                        │  Ollama (phi3)  │
    │  all-MiniLM │                        │  HTTP API       │
    │  384-dim    │                        │  localhost:11434│
    └─────────────┘                        └─────────────────┘
```

---

## Component Architecture

### 1. Frontend (Dioxus Web)

**File**: `src/main.rs`

**Key Components**:
- `App`: Root component with CSS and document setup
- `ChatApp`: Main application state and logic
  - `messages`: Signal<Vec<Message>> - Chat history
  - `input_value`: Signal<String> - Current input
  - `is_loading`: Signal<bool> - Query in progress
  - `is_uploading`: Signal<bool> - Upload in progress
  - `upload_status`: Signal<String> - Upload feedback
  - `total_chunks`: Signal<usize> - Indexed chunk count

**Event Handlers**:
- `handle_upload`: Async file upload with FormData
- `handle_send`: Async query submission with JSON
- `handle_keypress`: Enter key handling

**Tech Stack**:
- Dioxus 0.7.1 (reactive UI)
- wasm-bindgen (JS interop)
- web-sys (Browser APIs)

### 2. Backend (Axum)

**File**: `src/bin/backend.rs`

**Key Structures**:

```rust
struct AppState {
    vector_store: Arc<RwLock<VectorStore>>,
    embedding_model: Arc<TextEmbedding>,
    ollama_client: reqwest::Client,
}

struct VectorStore {
    chunks: Vec<ChunkData>,
}

struct ChunkData {
    text: String,
    embedding: Vec<f32>,  // 384-dim vector
    source: String,        // Filename
}
```

**Request/Response Types**:

```rust
// Chat
struct ChatRequest { query: String }
struct ChatResponse { answer: String, sources: Vec<String> }

// Ollama
struct OllamaResponse { response: String }
```

**Handlers**:
- `health_check()`: Simple health endpoint
- `upload_handler()`: Multipart file processing
- `chat_handler()`: RAG query pipeline

---

## Data Flow

### Upload Flow

```
1. User selects files (.md/.pdf)
   │
2. Frontend: Create FormData with Blob objects
   │
3. POST /upload (multipart/form-data)
   │
4. Backend: Extract file bytes
   │
5. Parse based on extension:
   ├─ .md → String::from_utf8()
   └─ .pdf → lopdf OR pdf-extract
   │
6. Chunk text (512 words, 50 overlap)
   │
7. Generate embeddings: fastembed.embed(chunks)
   │
8. Store in VectorStore: {text, embedding, source}
   │
9. Return 200 OK with metadata
   │
10. Frontend: Update UI with status
```

### Query Flow

```
1. User types question + presses Enter
   │
2. Frontend: Add message to UI (optimistic update)
   │
3. POST /chat {"query": "..."}
   │
4. Backend: Generate query embedding
   │
5. Vector search: cosine_similarity(query_emb, all_chunks)
   │
6. Sort by similarity, take top 3
   │
7. Build context from top chunks
   │
8. Construct strict grounding prompt
   │
9. POST to Ollama: /api/generate
   │
10. Parse response text
   │
11. Return {answer, sources}
   │
12. Frontend: Display answer with sources
```

---

## Vector Store Implementation

### Core Algorithm

```rust
fn search(&self, query_embedding: &[f32], top_k: usize) 
    -> Vec<(String, String, f32)> 
{
    // 1. Convert query to ndarray
    let query_vec = Array1::from_vec(query_embedding.to_vec());
    
    // 2. Compute cosine similarity with all chunks
    let mut scores: Vec<(usize, f32)> = self.chunks
        .iter()
        .enumerate()
        .map(|(idx, chunk)| {
            let chunk_vec = Array1::from_vec(chunk.embedding.clone());
            let similarity = cosine_similarity(query_vec.view(), chunk_vec.view());
            (idx, similarity)
        })
        .collect();
    
    // 3. Sort descending by similarity
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // 4. Return top-k with text, source, score
    scores.into_iter()
        .take(top_k)
        .map(|(idx, score)| {
            let chunk = &self.chunks[idx];
            (chunk.text.clone(), chunk.source.clone(), score)
        })
        .collect()
}
```

### Cosine Similarity

```rust
fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot = a.dot(&b);              // Dot product
    let norm_a = a.dot(&a).sqrt();    // L2 norm of a
    let norm_b = b.dot(&b).sqrt();    // L2 norm of b
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)           // Cosine = dot / (||a|| * ||b||)
}
```

**Range**: [-1, 1] where:
- 1.0 = identical vectors
- 0.0 = orthogonal (unrelated)
- -1.0 = opposite vectors

### Thread Safety

- `Arc<RwLock<VectorStore>>` for concurrent access
- Multiple readers OR single writer
- No data races at compile time

---

## Embedding Pipeline

### Model: all-MiniLM-L6-v2

**Specs**:
- Dimension: 384
- Max tokens: 512
- Speed: ~100 docs/sec (CPU)
- Size: ~90MB

**Why this model?**:
- ✅ Fast inference on CPU
- ✅ Good semantic understanding
- ✅ Balanced accuracy/speed
- ✅ Small memory footprint

### Initialization

```rust
let model = TextEmbedding::try_new(
    InitOptions::new(EmbeddingModel::AllMiniLML6V2)
        .with_show_download_progress(true)
)?;
```

**First run**: Downloads model to `~/.cache/fastembed/`

### Batch Processing

```rust
// Embed multiple chunks at once
let embeddings: Vec<Vec<f32>> = model.embed(chunks, None)?;

// Returns: Vec of 384-dim vectors
assert_eq!(embeddings[0].len(), 384);
```

---

## RAG Pipeline

### 1. Chunking Strategy

```rust
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) 
    -> Vec<String> 
{
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    
    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk = words[start..end].join(" ");
        chunks.push(chunk);
        
        if end >= words.len() { break; }
        start += chunk_size - overlap;  // Sliding window
    }
    
    chunks
}
```

**Default**: 512 words with 50-word overlap (10%)

**Why overlap?**:
- Prevents splitting concepts across chunks
- Improves retrieval of boundary information
- Trade-off: 10% storage increase

### 2. Retrieval (Top-K)

```rust
let results = store.search(&query_embedding, 3);
// Returns top 3 most similar chunks
```

**Why top-3?**:
- Balance context size vs relevance
- Fits within Ollama context window
- Fast enough for real-time

### 3. Context Construction

```rust
let context: String = results
    .iter()
    .enumerate()
    .map(|(i, (text, _source, score))| {
        format!("[Chunk {}] (relevance: {:.2})\n{}\n", i+1, score, text)
    })
    .collect::<Vec<_>>()
    .join("\n---\n\n");
```

**Format**:
```
[Chunk 1] (relevance: 0.85)
<text content>

---

[Chunk 2] (relevance: 0.78)
<text content>

---

[Chunk 3] (relevance: 0.72)
<text content>
```

### 4. Prompt Engineering

```rust
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
```

**Key design choices**:
- Explicit grounding rules
- Exact fallback phrase
- No room for hallucination

### 5. LLM Generation

```rust
let ollama_request = serde_json::json!({
    "model": "phi3",
    "prompt": prompt,
    "stream": false,
    "options": {
        "temperature": 0.1,      // Low = more deterministic
        "num_predict": 512,      // Max tokens to generate
    }
});
```

**Model: phi3**
- Size: 2.3GB
- Speed: 20-30 tokens/sec (CPU)
- Context: 128K tokens
- Quality: Strong reasoning

---

## API Design

### Endpoint: POST /upload

**Request**:
```http
POST /upload HTTP/1.1
Content-Type: multipart/form-data; boundary=----WebKitFormBoundary

------WebKitFormBoundary
Content-Disposition: form-data; name="files"; filename="doc.pdf"
Content-Type: application/pdf

<binary data>
------WebKitFormBoundary--
```

**Response**:
```json
{
  "status": "success",
  "processed_files": ["doc.pdf"],
  "total_chunks": 42,
  "errors": []
}
```

**Error Response**:
```json
{
  "status": "error",
  "errors": ["Failed to parse doc.pdf: Invalid UTF-8"]
}
```

### Endpoint: POST /chat

**Request**:
```json
{
  "query": "What is the main topic?"
}
```

**Response**:
```json
{
  "answer": "The main topic is...",
  "sources": ["doc.pdf", "notes.md"]
}
```

**Error Scenarios**:
- Empty query → `"Ask something!"`
- No docs → `"Upload documents first!"`
- Ollama down → `"Start Ollama first: \`ollama serve\`"`

---

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Text extraction | O(n) | n = file size |
| Chunking | O(n) | n = text length |
| Embedding | O(k) | k = chunks, ~10ms/chunk |
| Vector search | O(m*d) | m = total chunks, d = 384 |
| LLM generation | O(t) | t = tokens, ~30 tokens/sec |

### Space Complexity

| Component | Memory |
|-----------|--------|
| Base embedding model | ~90MB |
| Per chunk (avg) | ~1KB (text + 384*4 bytes) |
| 1000 chunks | ~1MB |
| 10000 chunks | ~10MB |
| Ollama (phi3) | ~2.3GB |

### Benchmarks (Typical Hardware)

**Upload (10-page PDF)**:
- Parse: 0.5s
- Chunk: 0.1s
- Embed: 2-5s (50 chunks)
- Total: **3-6 seconds**

**Query**:
- Embed: 50ms
- Search: 10ms (1000 chunks)
- LLM: 2-5s
- Total: **2-5 seconds**

---

## Design Decisions

### 1. In-Memory Vector Store

**Why not Qdrant/Chroma/Pinecone?**
- ✅ Zero setup friction (hackathon goal)
- ✅ No external dependencies
- ✅ Fast for < 10K chunks
- ✅ Thread-safe with RwLock
- ❌ No persistence
- ❌ O(n) search (no indexing)

**When to switch?**:
- Production with > 10K chunks
- Need persistence across restarts
- Multiple backend instances

### 2. Ollama vs OpenAI/Anthropic

**Why Ollama?**
- ✅ Local-first (no API keys)
- ✅ Free (no rate limits)
- ✅ Privacy (data stays local)
- ✅ Offline capable
- ❌ Slower than cloud APIs
- ❌ Lower quality than GPT-4

**When to switch?**:
- Need GPT-4 level reasoning
- Want faster responses
- Cloud deployment

### 3. Pure Rust (No Python)

**Why not LangChain/LlamaIndex?**
- ✅ Single language (easier debugging)
- ✅ Better performance
- ✅ Type safety at compile time
- ✅ Smaller binary
- ❌ Less mature ecosystem
- ❌ Fewer examples

### 4. Dioxus Web (Not React)

**Why Dioxus?**
- ✅ Share code with backend
- ✅ Type-safe props
- ✅ Compile-time guarantees
- ✅ No JS bundler needed
- ❌ Smaller community
- ❌ Fewer UI libraries

---

## Scaling Considerations

### Horizontal Scaling

**Current limitation**: In-memory state

**Solutions**:
1. **Shared vector DB**: Redis, Qdrant
2. **Stateless backend**: Offload storage
3. **Load balancer**: Round-robin requests

### Vertical Scaling

**Bottlenecks**:
1. **Embedding**: CPU-bound → Use GPU
2. **LLM**: Memory-bound → Better hardware
3. **Search**: O(n) → Use HNSW/IVF index

### Optimization Ideas

1. **Caching**:
```rust
// Cache query embeddings
let mut query_cache: HashMap<String, Vec<f32>> = HashMap::new();
```

2. **Async processing**:
```rust
// Upload returns immediately, process in background
tokio::spawn(async move {
    process_document(file).await;
});
```

3. **Approximate search**:
```rust
// Use HNSW index for O(log n) search
use hnswlib::Hnsw;
```

4. **Quantization**:
```rust
// Store embeddings as i8 instead of f32 (4x smaller)
let quantized: Vec<i8> = embedding.iter()
    .map(|&x| (x * 127.0) as i8)
    .collect();
```

---

## Security Considerations

1. **Input validation**: Limited to .md and .pdf
2. **Rate limiting**: Not implemented (add for production)
3. **File size limits**: Not enforced (add max 10MB)
4. **CORS**: Permissive (restrict in production)
5. **API authentication**: None (add JWT/OAuth)

---

## Future Enhancements

### High Priority
- [ ] Persistent storage (SQLite + Qdrant)
- [ ] Multi-query support (conversation history)
- [ ] Document deletion
- [ ] Better error messages

### Medium Priority
- [ ] Support more file types (DOCX, TXT, HTML)
- [ ] Streaming responses
- [ ] Chunk highlighting in sources
- [ ] Query history

### Low Priority
- [ ] User authentication
- [ ] Multi-user support
- [ ] Document versioning
- [ ] Analytics dashboard

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_cosine_similarity() {
        let a = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let b = Array1::from_vec(vec![0.0, 1.0, 0.0]);
        assert_eq!(cosine_similarity(a.view(), b.view()), 0.0);
    }
    
    #[test]
    fn test_chunking() {
        let text = "word ".repeat(1000);
        let chunks = chunk_text(&text, 512, 50);
        assert!(chunks.len() > 1);
    }
}
```

### Integration Tests
```bash
# Backend API tests
curl -X POST http://localhost:3000/upload -F "files=@test.pdf"
curl -X POST http://localhost:3000/chat -d '{"query":"test"}'
```

### Load Tests
```bash
# Using wrk
wrk -t4 -c100 -d30s --latency http://localhost:3000/health
```

---

## Monitoring & Observability

### Logging
```rust
// Current: tracing subscriber
tracing_subscriber::fmt()
    .with_target(false)
    .compact()
    .init();

// Add structured logging
info!("Document processed", chunks = chunks.len(), source = filename);
```

### Metrics (Future)
- Queries per second
- Average latency
- Embedding cache hit rate
- Vector store size

---

## References

- [Dioxus Documentation](https://dioxuslabs.com/learn/0.7)
- [Axum Documentation](https://docs.rs/axum)
- [fastembed GitHub](https://github.com/Anush008/fastembed-rs)
- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [RAG Paper](https://arxiv.org/abs/2005.11401)

---

**Document Version**: 1.0  
**Last Updated**: 2024  
**Maintained By**: Ahtohallan Team