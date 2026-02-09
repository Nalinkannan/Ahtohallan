# 🚀 START - Quick Launch Guide

## ✅ Prerequisites Check

Before starting, ensure you have:
- ✅ Rust installed: `rustc --version`
- ✅ Ollama installed: `ollama --version`
- ✅ Dioxus CLI: `dx --version` (or install: `cargo install dioxus-cli`)
- ✅ phi3 model: `ollama list` (or pull: `ollama pull phi3`)

---

## 🎯 Launch Commands (3 Terminals)

### Terminal 1: Ollama Server
```bash
ollama serve
```
**Wait for**: `Listening on 127.0.0.1:11434`

---

### Terminal 2: Backend Server
```bash
cargo run --bin backend --features backend
```
**Wait for**: `🎯 Backend running at http://127.0.0.1:3000`

---

### Terminal 3: Frontend Server
```bash
dx serve
```
**Wait for**: `Serving ahtohallan 🚀`
**Open**: http://localhost:8080

---

## 📤 Upload Documents

In a **4th terminal** or before starting the frontend:

```bash
# Upload sample document
curl -X POST http://localhost:3000/upload \
  -F "files=@sample-docs/rust-overview.md"

# Expected response:
# {"status":"success","processed_files":["rust-overview.md"],"total_chunks":42}
```

**Upload your own files**:
```bash
curl -X POST http://localhost:3000/upload \
  -F "files=@/path/to/your/document.pdf"
```

---

## 💬 Start Chatting

1. Open browser: **http://localhost:8080**
2. Type a question: _"What is Rust?"_
3. Get answer with sources!

---

## 🛑 Stop All Services

Press `Ctrl+C` in each terminal:
1. Terminal 3 (Frontend) - Ctrl+C
2. Terminal 2 (Backend) - Ctrl+C  
3. Terminal 1 (Ollama) - Ctrl+C

---

## ⚡ One-Command Start (Optional)

**Windows (PowerShell)**:
```powershell
# Start all in background (requires 3 separate terminals)
Start-Process powershell -ArgumentList "ollama serve"
Start-Sleep 3
Start-Process powershell -ArgumentList "cargo run --bin backend --features backend"
Start-Sleep 5
dx serve
```

**Linux/Mac (tmux)**:
```bash
# Install tmux if needed: sudo apt install tmux
tmux new-session -d -s rag 'ollama serve'
tmux split-window -h -t rag 'sleep 3 && cargo run --bin backend --features backend'
tmux split-window -v -t rag 'sleep 8 && dx serve'
tmux attach -t rag
```

---

## 🔍 Verify Everything Works

### Test Backend
```bash
curl http://localhost:3000/health
# Expected: OK
```

### Test Frontend
```bash
curl http://localhost:8080
# Expected: HTML content
```

### Test Upload
```bash
curl -X POST http://localhost:3000/upload \
  -F "files=@sample-docs/rust-overview.md"
# Expected: {"status":"success",...}
```

### Test Chat (after upload)
```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"query":"What is Rust?"}'
# Expected: {"answer":"...","sources":["rust-overview.md"]}
```

---

## 🐛 Quick Troubleshooting

| Problem | Solution |
|---------|----------|
| "Ollama connection failed" | Run `ollama serve` first |
| "model phi3 not found" | Run `ollama pull phi3` |
| "Port 3000 in use" | Kill process: `lsof -ti:3000 \| xargs kill -9` |
| "dx: command not found" | Run `cargo install dioxus-cli` |
| "Upload documents first!" | Upload via curl (see above) |
| "Backend won't compile" | Add `--features backend` flag |

---

## 📊 Expected Output

**Terminal 1 (Ollama)**:
```
Listening on 127.0.0.1:11434 (version 0.x.x)
```

**Terminal 2 (Backend)**:
```
🚀 Starting RAG Chatbot Backend
📚 Loading embedding model (all-MiniLM-L6-v2)...
✅ Embedding model loaded
🔍 Testing Ollama connection...
✅ Ollama is running
🎯 Backend running at http://127.0.0.1:3000
📖 Endpoints:
   - GET  /health
   - POST /upload (multipart/form-data)
   - POST /chat (JSON)
```

**Terminal 3 (Frontend)**:
```
Build completed successfully, launching app! 💫
╭─────────────────────────────────╮
│  Status:  Serving ahtohallan 🚀 │
╰─────────────────────────────────╯
```

---

## ✨ Success Indicators

You know everything works when:
- ✅ Ollama shows: `Listening on 127.0.0.1:11434`
- ✅ Backend shows: `🎯 Backend running at http://127.0.0.1:3000`
- ✅ Frontend shows: `Serving ahtohallan 🚀`
- ✅ Browser at http://localhost:8080 shows chat UI
- ✅ Upload returns success with chunk count
- ✅ Chat returns answers with sources

---

## 🎯 First Demo

```bash
# 1. Upload sample doc
curl -X POST http://localhost:3000/upload \
  -F "files=@sample-docs/rust-overview.md"

# 2. Open browser
# http://localhost:8080

# 3. Ask questions:
# - "What is Rust?"
# - "What companies use Rust?"
# - "Explain the ownership system"

# 4. Test grounding:
# - "What is Python?" → Should say "I don't know based on the provided documents."
```

---

**Ready to chat with your documents! 🎉**