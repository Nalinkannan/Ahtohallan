#!/bin/bash

# Ollama Diagnostic Script for Ahtohallan RAG Chatbot
# This script checks if Ollama is properly configured with GPU support

echo "======================================"
echo "  Ollama Diagnostic Script"
echo "======================================"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check functions
check_passed() {
    echo -e "${GREEN}✓${NC} $1"
}

check_failed() {
    echo -e "${RED}✗${NC} $1"
}

check_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# 1. Check if Ollama is installed
echo "1. Checking Ollama installation..."
if command -v ollama &> /dev/null; then
    OLLAMA_VERSION=$(ollama --version 2>&1 | head -n 1)
    check_passed "Ollama is installed: $OLLAMA_VERSION"
else
    check_failed "Ollama is not installed"
    echo ""
    echo "Install Ollama:"
    echo "  Linux/macOS: curl -fsSL https://ollama.ai/install.sh | sh"
    echo "  Windows: Download from https://ollama.ai/download"
    exit 1
fi
echo ""

# 2. Check if Ollama service is running
echo "2. Checking Ollama service..."
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    check_passed "Ollama is running on port 11434"
else
    check_failed "Ollama is not running"
    echo ""
    echo "Start Ollama with: ollama serve"
    exit 1
fi
echo ""

# 3. Check GPU availability
echo "3. Checking GPU..."
if command -v nvidia-smi &> /dev/null; then
    GPU_INFO=$(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>&1 | head -n 1)
    if [ $? -eq 0 ]; then
        check_passed "NVIDIA GPU detected: $GPU_INFO"

        # Check GPU memory usage
        GPU_USED=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -n 1)
        GPU_TOTAL=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -n 1)
        check_info "GPU Memory: ${GPU_USED}MB / ${GPU_TOTAL}MB used"

        # Check CUDA version
        CUDA_VERSION=$(nvidia-smi | grep "CUDA Version" | awk '{print $9}')
        if [ ! -z "$CUDA_VERSION" ]; then
            check_info "CUDA Version: $CUDA_VERSION"
        fi
    else
        check_warning "nvidia-smi found but GPU not accessible"
    fi
elif command -v rocm-smi &> /dev/null; then
    GPU_INFO=$(rocm-smi --showproductname 2>&1 | grep "Card series" | head -n 1)
    if [ $? -eq 0 ]; then
        check_passed "AMD GPU detected: $GPU_INFO"
    else
        check_warning "rocm-smi found but GPU not accessible"
    fi
elif [ "$(uname)" == "Darwin" ]; then
    CHIP=$(sysctl -n machdep.cpu.brand_string 2>/dev/null)
    if [[ "$CHIP" == *"Apple"* ]]; then
        check_passed "Apple Silicon detected: $CHIP"
        check_info "Metal acceleration available"
    else
        check_warning "No GPU acceleration detected"
    fi
else
    check_warning "No GPU detected (will use CPU only)"
fi
echo ""

# 4. Check if phi3 model is downloaded
echo "4. Checking phi3 model..."
if ollama list | grep -q "phi3"; then
    MODEL_SIZE=$(ollama list | grep "phi3" | awk '{print $2}')
    check_passed "phi3 model is available (Size: $MODEL_SIZE)"
else
    check_failed "phi3 model not found"
    echo ""
    echo "Download phi3 with: ollama pull phi3"
    exit 1
fi
echo ""

# 5. Test Ollama API
echo "5. Testing Ollama API..."
API_RESPONSE=$(curl -s http://localhost:11434/api/tags)
if [ $? -eq 0 ]; then
    check_passed "Ollama API is responding"
    MODEL_COUNT=$(echo "$API_RESPONSE" | grep -o '"name"' | wc -l)
    check_info "Available models: $MODEL_COUNT"
else
    check_failed "Ollama API is not responding"
    exit 1
fi
echo ""

# 6. Performance test
echo "6. Running quick performance test..."
echo "   (This may take 10-30 seconds on first run)"

START_TIME=$(date +%s%N)

# Send a test query
TEST_RESPONSE=$(curl -s http://localhost:11434/api/generate -d '{
  "model": "phi3",
  "prompt": "Say only the word: OK",
  "stream": false,
  "options": {
    "num_predict": 10
  }
}')

END_TIME=$(date +%s%N)
ELAPSED_TIME=$(( ($END_TIME - $START_TIME) / 1000000 ))

if echo "$TEST_RESPONSE" | grep -q "response"; then
    RESPONSE_TEXT=$(echo "$TEST_RESPONSE" | grep -o '"response":"[^"]*"' | cut -d'"' -f4)
    check_passed "Model inference working (${ELAPSED_TIME}ms)"
    check_info "Response: $RESPONSE_TEXT"

    # Performance assessment
    if [ $ELAPSED_TIME -lt 2000 ]; then
        check_passed "Excellent performance (likely using GPU)"
    elif [ $ELAPSED_TIME -lt 5000 ]; then
        check_info "Good performance"
    elif [ $ELAPSED_TIME -lt 10000 ]; then
        check_warning "Moderate performance (GPU may not be used)"
    else
        check_warning "Slow performance (likely using CPU only)"
    fi
else
    check_failed "Model inference failed"
    check_info "Response: $TEST_RESPONSE"
fi
echo ""

# 7. Check backend requirements
echo "7. Checking backend requirements..."
if command -v cargo &> /dev/null; then
    check_passed "Rust/Cargo is installed"
else
    check_failed "Rust/Cargo not found"
    echo "Install from: https://rustup.rs"
fi

if [ -f "Cargo.toml" ]; then
    check_passed "Cargo.toml found (in project directory)"
else
    check_warning "Not in project directory (Cargo.toml not found)"
fi
echo ""

# 8. Environment variables
echo "8. Checking environment variables..."
if [ ! -z "$OLLAMA_NUM_GPU" ]; then
    check_info "OLLAMA_NUM_GPU=$OLLAMA_NUM_GPU"
else
    check_info "OLLAMA_NUM_GPU not set (default will be used)"
fi

if [ ! -z "$OLLAMA_MAX_VRAM" ]; then
    check_info "OLLAMA_MAX_VRAM=$OLLAMA_MAX_VRAM MB"
fi

if [ ! -z "$OLLAMA_HOST" ]; then
    check_info "OLLAMA_HOST=$OLLAMA_HOST"
fi
echo ""

# Summary
echo "======================================"
echo "  Summary"
echo "======================================"
echo ""
check_passed "Ollama is properly configured!"
echo ""
echo "Next steps:"
echo "  1. Start backend: cargo run --bin backend --features backend"
echo "  2. Start frontend: dx serve --platform web"
echo "  3. Open: http://localhost:8080"
echo ""
echo "To monitor GPU usage:"
if command -v nvidia-smi &> /dev/null; then
    echo "  nvidia-smi -l 1"
elif command -v rocm-smi &> /dev/null; then
    echo "  watch -n 1 rocm-smi"
fi
echo ""
echo "For more help, see: OLLAMA_SETUP.md"
echo ""
