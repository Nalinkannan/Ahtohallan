# Quick Fix Guide - Ollama Connection Issues

## 🚨 Common Error Messages and Solutions

### Error: "Failed to connect to Ollama: error sending request"

**Cause:** Ollama service is not running

**Quick Fix:**
```bash
# Start Ollama
ollama serve
```

**Windows:** Check if Ollama service is running
1. Press `Win + R`
2. Type `services.msc`
3. Find "Ollama Service"
4. Right-click → Restart

---

### Error: "Model 'phi3' not found"

**Cause:** phi3 model not downloaded

**Quick Fix:**
```bash
# Download the model
ollama pull phi3

# Verify it's downloaded
ollama list
```

---

### Error: "Connection refused" or "Connection timeout"

**Cause:** Ollama not running or firewall blocking

**Quick Fix:**
```bash
# 1. Check if Ollama is running
curl http://localhost:11434/api/tags

# 2. If not running, start it
ollama serve

# 3. Test the model
ollama run phi3 "test"
```

---

## ⚡ Enable GPU Acceleration

### Quick GPU Setup

**1. Check if GPU is detected:**
```bash
# NVIDIA
nvidia-smi

# AMD
rocm-smi
```

**2. Ollama automatically uses GPU if available!**
- No configuration needed for most setups
- Just ensure drivers are up to date

**3. Verify GPU is being used:**
```bash
# Terminal 1: Monitor GPU
nvidia-smi -l 1

# Terminal 2: Run a query through your app
# You should see GPU memory and utilization increase
```

---

## 🔧 Advanced GPU Configuration

### Set GPU Environment Variables

**Linux/macOS:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export OLLAMA_NUM_GPU=1
export OLLAMA_NUM_GPU_LAYERS=99
```

**Windows (PowerShell):**
```powershell
# Set permanently
[System.Environment]::SetEnvironmentVariable('OLLAMA_NUM_GPU', '1', 'User')

# Or temporarily
$env:OLLAMA_NUM_GPU = "1"
```

---

## 📊 Verify Everything is Working

### Complete Verification Steps

```bash
# 1. Check Ollama is installed
ollama --version

# 2. Start Ollama
ollama serve

# 3. Pull model (if not already done)
ollama pull phi3

# 4. Test the model
ollama run phi3 "Say hello"

# 5. Test the API
curl http://localhost:11434/api/tags

# 6. Start backend
cargo run --bin backend --features backend

# 7. Start frontend (in another terminal)
dx serve --platform web

# 8. Open browser
# http://localhost:8080
```

---

## 🐛 Run Diagnostic Script

**Linux/macOS:**
```bash
chmod +x check_ollama.sh
./check_ollama.sh
```

**Windows:**
```powershell
.\check_ollama.ps1
```

---

## 💡 Performance Tips

### Speed Up Responses

1. **Keep Ollama Running**: Don't restart between queries
2. **First Query is Slow**: Model loads into memory (10-30s)
3. **Subsequent Queries**: Much faster (2-5s)
4. **Use GPU**: 5-10x faster than CPU
5. **Close Other GPU Apps**: Free up VRAM

### Reduce Timeout Errors

The backend now has:
- ✅ 60s timeout for quick mode
- ✅ 120s timeout for deep think mode
- ✅ Automatic retry (3 attempts)
- ✅ Better error messages

If still timing out:
- First run takes longer (model loading)
- Try simpler questions first
- Consider using a smaller model: `ollama pull tinyllama`

---

## 🎯 Quick Start (From Scratch)

```bash
# 1. Install Ollama
# Linux/macOS:
curl -fsSL https://ollama.ai/install.sh | sh
# Windows: Download from https://ollama.ai/download

# 2. Start Ollama
ollama serve

# 3. Download phi3
ollama pull phi3

# 4. Test it
ollama run phi3 "Explain RAG in one sentence"

# 5. Start your backend
cd ahtohallan
cargo run --bin backend --features backend

# 6. Start frontend (new terminal)
dx serve --platform web

# 7. Open http://localhost:8080

# Done! 🎉
```

---

## 📝 Common Questions

**Q: Why is my first query so slow?**
A: Model is loading into GPU memory. Subsequent queries will be faster.

**Q: How do I know if GPU is being used?**
A: Run `nvidia-smi -l 1` and watch memory/utilization increase during queries.

**Q: Can I use a different model?**
A: Yes! Try: `ollama pull mistral` or `ollama pull llama3`
   Then update `backend.rs` line with model name.

**Q: My GPU has low VRAM, what can I do?**
A: Try smaller models:
   - `ollama pull phi3:mini`
   - `ollama pull tinyllama`

**Q: Backend says "Upload documents first"**
A: Upload at least one PDF or Markdown file before asking questions.

---

## 🆘 Still Having Issues?

1. **Check logs:**
   ```bash
   # Backend logs (in terminal running backend)
   # Look for error messages
   
   # Ollama logs
   OLLAMA_DEBUG=1 ollama serve
   ```

2. **Test each component separately:**
   ```bash
   # Test Ollama directly
   curl -X POST http://localhost:11434/api/generate \
     -d '{"model":"phi3","prompt":"test","stream":false}'
   ```

3. **Read detailed guides:**
   - `OLLAMA_SETUP.md` - Complete GPU setup guide
   - `DEVELOPER_GUIDE.md` - Development reference
   - `IMPROVEMENTS_SUMMARY.md` - What changed

4. **Check GitHub:**
   - Ollama issues: https://github.com/ollama/ollama/issues
   - Dioxus issues: https://github.com/DioxusLabs/dioxus/issues

---

## ✅ Success Checklist

Before asking questions, make sure:

- [ ] Ollama is installed: `ollama --version`
- [ ] Ollama is running: `curl http://localhost:11434/api/tags`
- [ ] phi3 is downloaded: `ollama list | grep phi3`
- [ ] GPU is detected (if applicable): `nvidia-smi`
- [ ] Backend is running on port 3000
- [ ] Frontend is running on port 8080
- [ ] At least one document is uploaded
- [ ] Browser console shows no errors (F12)

If all checked, you're good to go! 🚀

---

**Last Updated:** 2024
**Works With:** Ollama 0.1.x+, phi3, CUDA 11.x+/12.x+