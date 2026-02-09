# ❄️ Ahtohallan - RAG Chatbot

> **Production-ready RAG chatbot built in Rust for 6-hour hackathons**

A complete Retrieval-Augmented Generation (RAG) chatbot that answers questions based **ONLY** on your uploaded documents. Built with Dioxus, Axum, fastembed, and Ollama.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Dioxus](https://img.shields.io/badge/dioxus-0.7.1-blue?style=for-the-badge)

## ✨ Features

- 📄 **Document Upload**: Supports `.md` and `.pdf` files
- 🧠 **Grounded Responses**: Answers based ONLY on uploaded documents
- 🚀 **Fast Embeddings**: Uses `fastembed` with all-MiniLM-L6-v2 (384-dim)
- 💾 **In-Memory Vector Store**: Zero external dependencies
- 🤖 **Ollama Integration**: Local LLM with `phi3` model
- ⚡ **Real-time Chat**: Modern web UI with Dioxus

## 🎯 Quick Start (< 2 minutes)

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Ollama**:
   - Download from: https://ollama.com/download
   - Or on Linux/Mac: `curl -fsSL https://ollama.com/install.sh | sh`

3. **Pull the phi3 model**:
   ```bash
   ollama pull phi3
   ```

### Run the App

```bash
# Clone the repository
git clone <your-repo-url>
cd ahtohallan

# Terminal 1: Start Ollama (if not running as service)
ollama serve

# Terminal 2: Start the backend
cargo run --bin backend --features backend

# Terminal 3: Start the frontend
dx serve
# Or if dx is not installed:
# cargo install dioxus-cli
# dx serve
```

**That's it!** Open your browser to `http://localhost:8080` (or the URL shown by `dx serve`).

## 📚 How It Works

### 1. Document Processing Pipeline
```
Upload (.md/.pdf) → Text Extraction → Chunking (512 tokens, 50 overlap) 
   → Embedding Generation → In-Memory Vector Store
```

### 2. Query Pipeline
```
User Query → Query Embedding → Cosine Similarity Search (top 3) 
   → Context Construction → Ollama (phi3) → Grounded Answer
```

### 3. Architecture

```
┌─────────────────┐         ┌──────────────────┐
│  Dioxus Web UI  │ ◄─────► │  Axum Backend    │
│  (Port 8080)    │  HTTP   │  (Port 3000)     │
└─────────────────┘         └──────────────────┘
                                     │
                         ┌───────────┼───────────┐
                         │           │           │
                    ┌────▼───┐  ┌────▼────┐  ┌───▼─────┐
                    │fastembed│  │ Vector  │  │ Ollama  │
                    │(384-dim)│  │  Store  │  │ (phi3)  │
                    └─────────┘  └─────────┘  └─────────┘
```

## 🔧 API Endpoints

### Backend (http://localhost:3000)

- **GET** `/health` - Health check
- **POST** `/upload` - Upload documents (multipart/form-data)
- **POST** `/chat` - Send chat query (JSON: `{"query": "your question"}`)

### Example cURL Commands

```bash
# Upload a document
curl -X POST http://localhost:3000/upload \
  -F "files=@document.pdf"

# Ask a question
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"query": "What is this document about?"}'
```

## 🛠️ Project Structure

```
ahtohallan/
├── src/
│   ├── main.rs              # Dioxus web frontend
│   └── bin/
│       └── backend.rs       # Axum backend server
├── assets/
│   └── main.css             # UI styling
├── Cargo.toml               # Dependencies
└── README.md
```

## 📦 Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `dioxus` | 0.7.1 | Web frontend framework |
| `axum` | 0.8.1 | Backend HTTP server |
| `fastembed` | 4.2.0 | Embedding generation |
| `ndarray` | 0.16 | Cosine similarity |
| `lopdf` | 0.35 | PDF parsing (primary) |
| `pdf-extract` | 0.7 | PDF parsing (fallback) |
| `reqwest` | 0.12 | Ollama HTTP client |

## 🐛 Troubleshooting

### "Failed to connect to backend"
- **Solution**: Make sure the backend is running on port 3000
  ```bash
  cargo run --bin backend
  ```

### "Start Ollama first: `ollama serve`"
- **Solution**: Ollama is not running
  ```bash
  ollama serve
  ```

### "model 'phi3' not found"
- **Solution**: Pull the model first
  ```bash
  ollama pull phi3
  ```

### "Failed to parse PDF"
- **Solution**: Some PDFs have complex layouts or are scanned images
  - Try converting to text first using OCR tools
  - Or use markdown files instead

### Backend fails to start with "Failed to load embedding model"
- **Solution**: The model downloads on first run (~100MB). Wait for completion.
- Check internet connection for the initial download.

### Frontend CORS errors
- **Solution**: The backend has permissive CORS enabled. Make sure both servers are running.

## 🚀 Demo Workflow

1. **Start the services** (backend + frontend)
2. **Upload documents**:
   - Click "Choose Files"
   - Select `.md` or `.pdf` files
   - Wait for "✅ Uploaded" confirmation
3. **Ask questions**:
   - Type in the chat input
   - Press Enter or click "🚀 Send"
   - Get answers grounded in your documents

### Example Questions

If you upload a research paper:
- ✅ "What is the main contribution of this paper?"
- ✅ "What methodology was used?"
- ✅ "What are the key findings?"
- ❌ "What is the weather today?" → "I don't know based on the provided documents."

## 🎓 Hackathon Tips

### Time-Saving Features
- **In-memory storage**: No database setup needed
- **Local LLM**: No API keys or rate limits
- **Pure Rust**: Single language, no polyglot complexity
- **One command run**: `cargo run --bin backend`

### Demo Tips
1. Prepare sample documents beforehand
2. Use short PDFs (< 10 pages) for fast demos
3. Ask questions you KNOW are in the documents
4. Show the "I don't know" response for out-of-scope queries

### Customization Ideas
- Change chunk size (line 251 in `backend.rs`)
- Adjust top-k results (line 369 in `backend.rs`)
- Swap Ollama model (line 421 in `backend.rs`)
- Modify system prompt (lines 394-408 in `backend.rs`)

## 📝 Configuration

### Change Embedding Model
Edit `backend.rs` line 519:
```rust
EmbeddingModel::AllMiniLML6V2  // Default (384-dim)
// Other options:
// EmbeddingModel::BGESmallENV15  // (384-dim)
// EmbeddingModel::AllMiniLML12V2 // (384-dim)
```

### Change LLM Model
Edit `backend.rs` line 421:
```rust
"model": "phi3"
// Other options (must be pulled first):
// "llama3.2"
// "mistral"
// "qwen2.5"
```

### Change Ports
- **Backend**: Edit `backend.rs` line 538: `"127.0.0.1:3000"`
- **Frontend**: Edit `Dioxus.toml` or use `dx serve --port 8080`

## 🧪 Testing

```bash
# Build all binaries
cargo build --release --features backend

# Run backend tests
cargo test --bin backend --features backend

# Check code quality
cargo clippy

# Format code
cargo fmt
```

## 📈 Performance Notes

- **Embedding speed**: ~100 docs/sec (CPU)
- **Vector search**: O(n) linear scan (sufficient for < 10K chunks)
- **Memory usage**: ~50MB base + ~1KB per chunk
- **Ollama response**: 2-5 seconds for phi3

## 🔒 Security Notes

- **Local only**: No external API calls (except Ollama)
- **No persistence**: Data cleared on restart
- **CORS**: Permissive for development (restrict in production)
- **File uploads**: Limited to .md and .pdf

## 📄 License

MIT License - feel free to use in hackathons and projects!

## 🙏 Acknowledgments

Built with:
- [Dioxus](https://dioxuslabs.com/) - Rust UI framework
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [fastembed](https://github.com/Anush008/fastembed-rs) - Fast embeddings
- [Ollama](https://ollama.com/) - Local LLM runtime

---

**Built with 🦀 Rust by [@Nalinkannan](https://github.com/Nalinkannan)**

*Perfect for hackathons, demos, and learning RAG systems!*