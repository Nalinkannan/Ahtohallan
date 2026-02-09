# ⚡ QUICKSTART - Hackathon Demo Guide

> **Get your RAG chatbot running in 2 minutes!**

## 🎯 Prerequisites (Install Once)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Ollama
# Download from: https://ollama.com/download

# 3. Install Dioxus CLI
cargo install dioxus-cli

# 4. Pull the phi3 model
ollama pull phi3
```

## 🚀 Run the App (Every Time)

### Option 1: Automated (Linux/Mac)
```bash
chmod +x run.sh
./run.sh
```

### Option 2: Automated (Windows)
```cmd
run.bat
```

### Option 3: Manual (3 Terminals)

**Terminal 1: Start Ollama**
```bash
ollama serve
```

**Terminal 2: Start Backend**
```bash
cargo run --bin backend
# Wait for: "🎯 Backend running at http://127.0.0.1:3000"
```

**Terminal 3: Start Frontend**
```bash
dx serve
# Open: http://localhost:8080
```

## 🎪 Demo Script (5 minutes)

### Step 1: Upload Documents (30 seconds)
1. Click **"Choose Files"**
2. Select files from `sample-docs/` folder
3. Wait for **"✅ Uploaded X file(s)"**
4. Note the chunk count

### Step 2: Ask Questions (2 minutes)
```
✅ "What is Rust?"
✅ "What are the key features of Rust?"
✅ "What companies use Rust?"
✅ "What is the ownership system?"
```

### Step 3: Show Grounding (1 minute)
```
❌ "What is Python?" → "I don't know based on the provided documents."
❌ "What's the weather?" → "I don't know based on the provided documents."
```

### Step 4: Show Sources (30 seconds)
- Point out the **"📚 Sources"** tag under responses
- Show it references the uploaded files

## 🐛 Quick Fixes

| Problem | Solution |
|---------|----------|
| Backend won't start | `cargo clean && cargo build --bin backend` |
| Ollama not responding | `ollama serve` in separate terminal |
| Frontend 404 errors | Check backend is on port 3000: `curl http://localhost:3000/health` |
| Model not found | `ollama pull phi3` |
| Port already in use | Kill process: `lsof -ti:3000 \| xargs kill -9` (Mac/Linux) |

## 📊 Key Talking Points

1. **Zero Setup Friction**
   - "Everything runs locally - no API keys needed"
   - "In-memory storage - no database setup"

2. **Grounded Responses**
   - "Answers ONLY from uploaded documents"
   - "Prevents hallucination with strict prompting"

3. **Technology Stack**
   - "Pure Rust - frontend to backend"
   - "fastembed for 384-dim embeddings"
   - "Cosine similarity search in-memory"
   - "Ollama phi3 for generation"

4. **Performance**
   - "Embeddings: ~100 docs/sec"
   - "Search: O(n) but fast for < 10K chunks"
   - "Response: 2-5 seconds"

## 🎨 Customization Cheat Sheet

### Change Chunk Size
**File**: `src/bin/backend.rs` (line ~251)
```rust
let chunks = chunk_text(&text, 512, 50);  // 512 words, 50 overlap
```

### Change Top-K Results
**File**: `src/bin/backend.rs` (line ~369)
```rust
let results = store.search(&query_embedding, 3);  // Top 3
```

### Change LLM Model
**File**: `src/bin/backend.rs` (line ~421)
```rust
"model": "phi3"  // Try: "llama3.2", "mistral", "qwen2.5"
```

### Modify System Prompt
**File**: `src/bin/backend.rs` (lines ~394-408)
```rust
let prompt = format!(r#"You are a helpful assistant..."#);
```

## 📁 Test Files Included

- `sample-docs/rust-overview.md` - Comprehensive Rust guide
- Add your own `.md` or `.pdf` files for testing

## 🏆 Winning Tips

1. **Prepare Demo Files**: Use short, focused PDFs (2-5 pages)
2. **Script Questions**: Know what's in your documents
3. **Show Failures**: Demo out-of-scope questions getting rejected
4. **Emphasize Speed**: "Setup in < 2 mins, no accounts needed"
5. **Highlight Safety**: "Local-first, no data leaves your machine"

## ⏱️ Time Breakdown

| Phase | Time | Task |
|-------|------|------|
| Setup | 5 min | Install Rust, Ollama, download phi3 |
| Build | 3 min | `cargo build --release --bin backend` |
| First Run | 2 min | Download embedding model (~100MB) |
| Upload | 10 sec | Process 5-10 documents |
| Query | 3 sec | Get answer from RAG pipeline |
| **Total** | **~10 min** | **First-time setup** |
| **Subsequent** | **< 1 min** | **Start services** |

## 🔗 Quick Links

- **Frontend**: http://localhost:8080
- **Backend**: http://localhost:3000
- **Health Check**: http://localhost:3000/health
- **Ollama**: http://localhost:11434

## 📸 Screenshot Checklist

For your hackathon submission, capture:
- [ ] Upload interface with file selection
- [ ] Successful upload confirmation
- [ ] Chat with multiple Q&A exchanges
- [ ] Sources displayed under answers
- [ ] "I don't know" response for out-of-scope query
- [ ] Terminal showing backend logs

## 🎤 Elevator Pitch (30 seconds)

> "Ahtohallan is a production-ready RAG chatbot built entirely in Rust. It runs completely locally with zero external dependencies - just upload your documents and ask questions. The system uses fastembed for vector embeddings, an in-memory vector store for similarity search, and Ollama's phi3 model for generation. Unlike traditional chatbots, it ONLY answers from your documents, preventing hallucinations. Setup takes under 2 minutes with a single `cargo run` command."

## 🚨 Emergency Commands

```bash
# Kill all services
pkill -f backend
pkill -f "dx serve"
pkill -f ollama

# Clean rebuild
cargo clean
rm -rf target/
cargo build --release --bin backend

# Reset Ollama
ollama stop
rm -rf ~/.ollama/models
ollama pull phi3

# Check ports
lsof -i :3000  # Backend
lsof -i :8080  # Frontend
lsof -i :11434 # Ollama
```

---

**Ready to win? Let's go! 🏆**