# Developer Guide - Ahtohallan RAG Chatbot

## Quick Start

### Running the Application

**Terminal 1 - Backend:**
```bash
cd ahtohallan
cargo run --bin backend --features backend
```

**Terminal 2 - Frontend:**
```bash
cd ahtohallan
dx serve --platform web
```

Access the app at: http://localhost:8080

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Frontend (WASM)                 │
│  ┌─────────────┐      ┌──────────────────────┐ │
│  │   Upload    │      │   Chat Interface     │ │
│  │   Section   │──────│   (Messages, Input)  │ │
│  └─────────────┘      └──────────────────────┘ │
└────────────────┬────────────────────────────────┘
                 │ HTTP (localhost:3000)
┌────────────────┴────────────────────────────────┐
│              Backend (Axum Server)               │
│  ┌─────────────────┐  ┌──────────────────────┐ │
│  │ File Processing │  │   Vector Search      │ │
│  │ (PDF, Markdown) │──│   (fastembed)        │ │
│  └─────────────────┘  └──────────────────────┘ │
│           │                      │               │
│           └──────────┬───────────┘               │
│                 ┌────▼────┐                      │
│                 │ Ollama  │                      │
│                 │  (LLM)  │                      │
│                 └─────────┘                      │
└─────────────────────────────────────────────────┘
```

---

## Key Components

### Frontend (`src/main.rs`)

#### Main Components
- **`App`** - Root component with document head
- **`ChatApp`** - Main application logic and UI

#### State Management
```rust
// Messages and chat
let mut messages = use_signal(|| Vec::<Message>::new());
let mut input_value = use_signal(|| String::new());
let mut is_loading = use_signal(|| false);
let mut deep_think = use_signal(|| false);

// Document management
let mut documents = use_signal(|| Vec::<DocumentInfo>::new());
let mut upload_status = use_signal(|| String::new());
let mut is_uploading = use_signal(|| false);

// Voice input
let mut is_listening = use_signal(|| false);
```

#### Key Functions

**File Upload:**
```rust
async fn upload_files_formdata(form_data: FormData) -> Result<String, String>
```
- Takes FormData with files
- Uploads to backend via fetch API
- Returns success/error message

**Send Message:**
```rust
async fn send_message(
    mut messages: Signal<Vec<Message>>,
    mut input_value: Signal<String>,
    mut is_loading: Signal<bool>,
    deep_think: Signal<bool>,
)
```
- Sends query to backend
- Updates messages list
- Handles errors

---

### Backend (`src/bin/backend.rs`)

#### Core Structures
```rust
struct AppState {
    vector_store: Arc<RwLock<VectorStore>>,
    embedding_model: Arc<Mutex<TextEmbedding>>,
    ollama_client: Client,
}

struct VectorStore {
    chunks: Vec<ChunkData>,
}

struct ChunkData {
    text: String,
    embedding: Vec<f32>,
    source: String,
}
```

#### API Endpoints

**`GET /health`**
- Health check endpoint

**`POST /upload`**
- Accepts multipart/form-data with files
- Extracts text from PDF/Markdown
- Chunks text (512 tokens, 50 overlap)
- Generates embeddings
- Stores in vector database

**`POST /chat`**
- Request: `{ query: string, deep_think: bool }`
- Searches vector store for relevant chunks
- Builds context from top results
- Queries Ollama with context
- Response: `{ answer: string, sources: string[] }`

**`POST /delete`**
- Request: `{ filename: string }`
- Removes all chunks from that file
- Response: `{ status: string, message: string }`

---

## File Upload Flow

### Frontend Process
1. User clicks "Choose Files" button
2. Hidden `<input type="file">` is triggered
3. User selects files (.md or .pdf)
4. Files are read from HTML input element
5. FormData is created
6. Files appended to FormData with `append_with_blob()`
7. Fetch request to `/upload` endpoint
8. Response handled, UI updated

### Backend Process
1. Receive multipart form data
2. Extract each file
3. Determine file type (.md or .pdf)
4. Extract text content
5. Chunk text (512 tokens with 50 overlap)
6. Generate embeddings for each chunk
7. Store in vector store with filename
8. Return success response

---

## RAG (Retrieval Augmented Generation) Flow

```
1. User Query
   │
   ▼
2. Generate Query Embedding
   │
   ▼
3. Search Vector Store (cosine similarity)
   │
   ▼
4. Retrieve Top K Chunks (default: 5)
   │
   ▼
5. Build Context from Chunks
   │
   ▼
6. Create Prompt with Context
   │
   ▼
7. Send to Ollama (phi3)
   │
   ▼
8. Stream Response
   │
   ▼
9. Return Answer + Sources
```

---

## Development Tips

### Adding New File Types

**Frontend:**
```rust
// Update file input accept attribute
accept: ".md,.pdf,.txt"  // Add .txt
```

**Backend:**
```rust
// Add extraction logic
} else if filename.ends_with(".txt") {
    // Extract plain text
    String::from_utf8_lossy(&content).to_string()
}
```

### Adjusting Chunk Size

In `backend.rs`:
```rust
// Current: 512 tokens with 50 overlap
let chunks = chunk_text(&text, 512, 50);

// For larger context: 1024 tokens with 100 overlap
let chunks = chunk_text(&text, 1024, 100);

// For smaller chunks: 256 tokens with 25 overlap
let chunks = chunk_text(&text, 256, 25);
```

### Changing Number of Retrieved Chunks

In `backend.rs`, `chat_handler_impl`:
```rust
// Current: top 5 chunks
let results = store.search(&query_embedding, 5);

// For more context: top 10
let results = store.search(&query_embedding, 10);
```

### Using Different Embedding Model

In `backend.rs`, `main`:
```rust
// Current: all-MiniLM-L6-v2
let embedding_model = TextEmbedding::try_new(InitOptions {
    model_name: EmbeddingModel::AllMiniLML6V2,
    ..Default::default()
})?;

// For better quality (slower):
// EmbeddingModel::BGEBaseENV15
// EmbeddingModel::BGESmallENV15
```

### Changing LLM Model

In `backend.rs`:
```rust
// Current: phi3
let model = "phi3";

// Try other models:
// let model = "llama3";
// let model = "mistral";
// let model = "codellama";
```

---

## Styling Guide

### CSS Variables

Located in `assets/main.css`:
```css
:root {
    --primary-color: #4a90e2;      /* Blue - main actions */
    --secondary-color: #50c878;    /* Green - secondary actions */
    --accent-purple: #667eea;      /* Purple gradient start */
    --accent-pink: #764ba2;        /* Purple gradient end */
    --background: #f5f7fa;         /* Page background */
    --surface: #ffffff;            /* Card background */
    --text-primary: #2c3e50;       /* Main text */
    --text-secondary: #7f8c8d;     /* Secondary text */
    --error-color: #e74c3c;        /* Errors */
}
```

### Adding Custom Animations

```css
@keyframes myAnimation {
    from { opacity: 0; }
    to { opacity: 1; }
}

.my-element {
    animation: myAnimation 0.3s ease;
}
```

---

## Common Tasks

### Adding a New Feature

1. **Add UI in `src/main.rs`:**
```rust
rsx! {
    button {
        onclick: move |_| handle_my_feature(),
        "My Feature"
    }
}
```

2. **Add backend endpoint in `src/bin/backend.rs`:**
```rust
async fn my_feature_handler(/* params */) -> impl IntoResponse {
    // Logic here
}

// In main():
.route("/my-feature", post(my_feature_handler))
```

3. **Style in `assets/main.css`:**
```css
.my-feature-button {
    padding: 12px 24px;
    background: var(--primary-color);
    /* ... */
}
```

### Debugging

**Frontend:**
```rust
// Add logging
log::info!("Debug: {}", my_value);

// Check browser console (F12)
```

**Backend:**
```rust
// Logs to terminal
info!("Processing file: {}", filename);
error!("Error occurred: {}", e);
```

### Testing Endpoints

**Using curl:**
```bash
# Health check
curl http://localhost:3000/health

# Chat
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"query":"What is this about?","deep_think":false}'

# Delete
curl -X POST http://localhost:3000/delete \
  -H "Content-Type: application/json" \
  -d '{"filename":"test.pdf"}'
```

---

## Performance Optimization

### Frontend
- Use `use_memo` for expensive computations
- Avoid unnecessary signal reads
- Batch state updates when possible

### Backend
- Adjust chunk size vs retrieval count trade-off
- Consider caching embeddings
- Use async where beneficial
- Monitor memory usage for large documents

---

## Troubleshooting

### "Failed to connect to backend"
- Ensure backend is running on port 3000
- Check CORS settings in backend
- Verify network connectivity

### "Upload failed"
- Check file type is .md or .pdf
- Verify file isn't corrupted
- Check backend logs for errors

### "No answer from LLM"
- Ensure Ollama is running
- Check if phi3 model is installed
- Verify network connection to Ollama

### Frontend won't build
- Clear target: `cargo clean`
- Update Dioxus: `cargo update -p dioxus`
- Check Rust version: `rustc --version`

---

## Dependencies

### Frontend
```toml
dioxus = "0.7.3"
gloo-net = "*"
serde_json = "*"
wasm-bindgen = "*"
web-sys = { features = [...] }
```

### Backend
```toml
axum = "*"
tokio = { features = ["full"] }
fastembed = "*"
ollama-rs = "*"
lopdf = "*"
```

---

## Resources

- [Dioxus Docs](https://dioxuslabs.com/learn/0.7)
- [Axum Docs](https://docs.rs/axum)
- [Ollama](https://ollama.ai)
- [fastembed](https://github.com/Anush008/fastembed-rs)

---

## License

Project-specific - refer to LICENSE file.