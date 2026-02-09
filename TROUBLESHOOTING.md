# 🔧 Troubleshooting Guide

> Comprehensive solutions for common issues with Ahtohallan RAG Chatbot

## Table of Contents

- [Installation Issues](#installation-issues)
- [Backend Issues](#backend-issues)
- [Frontend Issues](#frontend-issues)
- [Ollama Issues](#ollama-issues)
- [Upload Issues](#upload-issues)
- [Query Issues](#query-issues)
- [Performance Issues](#performance-issues)
- [Network Issues](#network-issues)

---

## Installation Issues

### 🔴 "cargo: command not found"

**Problem**: Rust is not installed or not in PATH

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify
cargo --version
```

**Windows**:
- Download installer from https://rustup.rs/
- Restart terminal after installation

---

### 🔴 "dx: command not found"

**Problem**: Dioxus CLI not installed

**Solution**:
```bash
cargo install dioxus-cli

# Verify
dx --version
```

**Alternative**: Run without dx
```bash
cargo build --target wasm32-unknown-unknown --release
# Then use trunk or a static file server
```

---

### 🔴 Compilation Errors During Build

**Problem**: Missing dependencies or wrong Rust version

**Solution**:
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
rm -rf target/
cargo build --release --bin backend

# Check Rust version (should be 1.70+)
rustc --version
```

---

## Backend Issues

### 🔴 "Failed to load embedding model"

**Problem**: First-time download of fastembed model (~100MB)

**Solution**:
1. Wait for download to complete (watch terminal output)
2. Check internet connection
3. If stuck, delete cache and retry:
```bash
rm -rf ~/.cache/fastembed/
cargo run --bin backend
```

**Expected behavior**: First run takes 2-3 minutes for model download

---

### 🔴 "Address already in use (port 3000)"

**Problem**: Another process is using port 3000

**Solution**:

**Linux/Mac**:
```bash
# Find process
lsof -i :3000

# Kill process
kill -9 <PID>

# Or kill all
lsof -ti:3000 | xargs kill -9
```

**Windows**:
```cmd
# Find process
netstat -ano | findstr :3000

# Kill process
taskkill /PID <PID> /F
```

**Alternative**: Change port in `src/bin/backend.rs` line 538:
```rust
let addr = "127.0.0.1:3001";  // Use different port
```

---

### 🔴 Backend Crashes with "thread panicked"

**Problem**: Various runtime errors

**Solution**:
```bash
# Enable detailed logs
RUST_BACKTRACE=full cargo run --bin backend

# Check backend.log
tail -f backend.log

# Common causes:
# 1. Out of memory - reduce document size
# 2. Corrupted PDF - try different file
# 3. Missing Ollama - ensure ollama is running
```

---

### 🔴 "Failed to bind to address"

**Problem**: Permission issue or port blocked

**Solution**:
1. Try with sudo (Linux/Mac): `sudo cargo run --bin backend`
2. Check firewall settings
3. Use port > 1024: Change to 8000 or 8888

---

## Frontend Issues

### 🔴 "Failed to compile" (Frontend)

**Problem**: WASM compilation errors

**Solution**:
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Update dependencies
cargo update

# Clean build
cargo clean
dx serve
```

---

### 🔴 White Screen / Nothing Loads

**Problem**: JavaScript errors or build issues

**Solution**:
1. Open browser DevTools (F12)
2. Check Console for errors
3. Hard refresh: Ctrl+Shift+R (or Cmd+Shift+R)
4. Clear browser cache
5. Rebuild:
```bash
dx clean
dx serve
```

---

### 🔴 "WebSocket connection failed"

**Problem**: Dev server not running properly

**Solution**:
```bash
# Kill dx serve
pkill -f "dx serve"

# Restart
dx serve --hot-reload

# Or without hot reload
dx serve --no-hot-reload
```

---

### 🔴 Frontend Shows Port 8080 but Browser Won't Connect

**Problem**: Firewall or localhost resolution

**Solution**:
1. Try `127.0.0.1:8080` instead of `localhost:8080`
2. Check firewall allows port 8080
3. Try different port:
```bash
dx serve --port 8888
```

---

## Ollama Issues

### 🔴 "Start Ollama first: `ollama serve`"

**Problem**: Ollama is not running

**Solution**:

**Linux/Mac**:
```bash
# Start Ollama
ollama serve

# Or as background service
ollama serve > /dev/null 2>&1 &

# Verify
curl http://localhost:11434/api/tags
```

**Windows**:
```cmd
# Start Ollama
start /B ollama serve

# Or just run ollama.exe from Start menu
```

---

### 🔴 "model 'phi3' not found"

**Problem**: Model not downloaded

**Solution**:
```bash
# Pull phi3
ollama pull phi3

# Verify
ollama list

# Should show:
# NAME    ID              SIZE    MODIFIED
# phi3:latest  xxxxx           2.3 GB  X minutes ago
```

---

### 🔴 Ollama Responses Are Too Slow (> 30 seconds)

**Problem**: Model is too large for your hardware

**Solution**:
```bash
# Try smaller model
ollama pull phi3:mini

# Or faster alternatives
ollama pull tinyllama
ollama pull qwen2.5:0.5b

# Update backend.rs line 421:
"model": "phi3:mini"
```

---

### 🔴 "Connection refused: localhost:11434"

**Problem**: Ollama port blocked or wrong address

**Solution**:
1. Check Ollama is running: `ps aux | grep ollama`
2. Verify port: `lsof -i :11434`
3. Test connection: `curl http://localhost:11434/api/tags`
4. Restart Ollama:
```bash
pkill ollama
ollama serve
```

---

## Upload Issues

### 🔴 "Failed to parse PDF"

**Problem**: PDF is scanned/image-based or corrupted

**Solution**:
1. Check if PDF has selectable text (not scanned images)
2. Try OCR tool first:
```bash
# Using Tesseract (if installed)
tesseract document.pdf output.txt
# Then convert to markdown
```
3. Use text-based PDFs or markdown files instead
4. Try `pdf2txt.py` (Python tool) to extract text first

---

### 🔴 "No text extracted from file"

**Problem**: File is empty or encrypted

**Solution**:
1. Open file manually to verify content
2. Remove PDF password protection
3. Convert to plain text/markdown
4. Check file size > 0 bytes

---

### 🔴 Upload Hangs / Never Completes

**Problem**: File too large or backend crashed

**Solution**:
1. Check file size (keep under 10MB for demos)
2. Check backend.log for errors
3. Restart backend
4. Split large documents into smaller files

---

### 🔴 "Failed to generate embeddings"

**Problem**: Model issue or out of memory

**Solution**:
```bash
# Check available RAM
free -h  # Linux
vm_stat  # Mac

# Restart backend to clear memory
pkill -f backend
cargo run --bin backend

# Process smaller files
# Split large PDFs into chapters
```

---

## Query Issues

### 🔴 "Upload documents first!"

**Problem**: No documents in vector store

**Solution**:
1. Upload at least one document
2. Wait for "✅ Uploaded" confirmation
3. Verify chunk count > 0

---

### 🔴 Answers Are Not Relevant

**Problem**: Poor similarity matching or chunk size issues

**Solution**:
1. **Increase top-k** (backend.rs line 369):
```rust
let results = store.search(&query_embedding, 5);  // Try 5 instead of 3
```

2. **Adjust chunk size** (backend.rs line 251):
```rust
let chunks = chunk_text(&text, 256, 25);  // Smaller chunks
```

3. **Rephrase question** to match document language

---

### 🔴 Always Says "I don't know based on the provided documents"

**Problem**: Over-strict grounding or poor retrieval

**Solution**:
1. Check if question is actually in documents
2. Lower similarity threshold (requires code change)
3. Increase chunk overlap (backend.rs line 251):
```rust
let chunks = chunk_text(&text, 512, 100);  // More overlap
```

---

### 🔴 Responses Include External Knowledge

**Problem**: Prompt not strict enough

**Solution**:
Enhance system prompt (backend.rs lines 394-408):
```rust
let prompt = format!(
    r#"STRICT RULES:
1. Answer ONLY from the context below
2. If answer is NOT in context, say: "I don't know based on the provided documents."
3. Do NOT use external knowledge
4. Do NOT make assumptions

Context:
{}

Question: {}

Answer:"#,
    context, query
);
```

---

## Performance Issues

### 🔴 Embedding Generation Is Slow

**Problem**: CPU-bound operation

**Solution**:
1. Use smaller documents
2. Reduce chunk count (larger chunk size)
3. Batch process in background
4. Consider GPU-enabled fastembed (requires CUDA)

**Expected times**:
- 1 page PDF: 1-3 seconds
- 10 page PDF: 5-15 seconds
- 100 page PDF: 30-90 seconds

---

### 🔴 Vector Search Is Slow (> 1 second)

**Problem**: Too many chunks (> 10,000)

**Solution**:
1. Limit document size
2. Implement approximate nearest neighbor (ANN) search
3. Use external vector DB (Qdrant, Milvus) instead of in-memory

**Current limitations**:
- O(n) linear search
- Fast for < 10K chunks
- Slow for > 100K chunks

---

### 🔴 High Memory Usage

**Problem**: Many documents in memory

**Solution**:
```bash
# Monitor memory
top -p $(pgrep backend)  # Linux
top -pid $(pgrep backend)  # Mac

# Restart backend to clear
pkill -f backend
cargo run --bin backend

# In production: implement cleanup or persistence
```

---

## Network Issues

### 🔴 "Failed to connect to backend"

**Problem**: Backend not running or wrong URL

**Solution**:
1. Verify backend is running:
```bash
curl http://localhost:3000/health
# Should return: OK
```

2. Check frontend is pointing to correct URL (main.rs):
```rust
// Line ~55 and ~130
"http://localhost:3000/upload"
"http://localhost:3000/chat"
```

3. Disable browser extensions (ad blockers)
4. Check CORS is enabled (backend.rs line 532)

---

### 🔴 CORS Errors in Browser Console

**Problem**: Cross-origin request blocked

**Solution**:
Backend already has permissive CORS (line 532):
```rust
.layer(CorsLayer::permissive())
```

If still failing:
1. Check both servers are running
2. Use same protocol (http://, not mixing with https://)
3. Clear browser cache
4. Try different browser

---

### 🔴 Request Timeout

**Problem**: Ollama taking too long

**Solution**:
Increase timeout (backend.rs line 419):
```rust
.timeout(Duration::from_secs(60))  // Increase to 60s
```

And Ollama client (line 510):
```rust
let ollama_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()
```

---

## Advanced Debugging

### Enable Verbose Logging

```bash
# Backend with full logs
RUST_LOG=debug cargo run --bin backend

# Or even more verbose
RUST_LOG=trace cargo run --bin backend
```

### Check System Resources

```bash
# CPU usage
top

# Memory usage
free -h

# Disk space
df -h

# Network connections
netstat -tuln | grep -E '3000|8080|11434'
```

### Test Backend Manually

```bash
# Health check
curl http://localhost:3000/health

# Upload test
curl -X POST http://localhost:3000/upload \
  -F "files=@sample-docs/rust-overview.md"

# Chat test
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{"query": "What is Rust?"}'
```

### Inspect Ollama

```bash
# List models
ollama list

# Show model info
ollama show phi3

# Test directly
ollama run phi3 "Hello, how are you?"

# Check Ollama logs
journalctl -u ollama -f  # If running as service
```

---

## Getting Help

If none of these solutions work:

1. **Check logs**:
   - Backend: `backend.log`
   - Browser: DevTools Console (F12)
   - Ollama: System logs

2. **Collect info**:
   ```bash
   # System info
   uname -a
   rustc --version
   cargo --version
   ollama --version
   
   # Backend status
   curl -v http://localhost:3000/health
   ```

3. **Create minimal reproduction**:
   - Use sample-docs/rust-overview.md
   - Try single simple query
   - Note exact error message

4. **Open an issue** with:
   - Operating system
   - Rust version
   - Exact error message
   - Steps to reproduce
   - Relevant logs

---

## Prevention Tips

1. **Always check**:
   - Ollama is running before starting backend
   - Backend is ready before using frontend
   - Sufficient RAM (4GB+ recommended)

2. **Best practices**:
   - Start with small test files
   - Keep documents under 50 pages for demos
   - Monitor resource usage
   - Restart services if sluggish

3. **Quick health check**:
   ```bash
   # All services up?
   curl http://localhost:11434/api/tags  # Ollama
   curl http://localhost:3000/health     # Backend
   curl http://localhost:8080            # Frontend
   ```

---

**Still stuck? Check QUICKSTART.md for demo-specific tips or README.md for architecture details.**