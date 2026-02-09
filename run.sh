#!/bin/bash

# Ahtohallan RAG Chatbot - Quick Start Script
# ============================================

set -e

echo "❄️  Ahtohallan RAG Chatbot - Quick Start"
echo "========================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "📥 Install from: https://rustup.rs/"
    exit 1
fi

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "⚠️  Ollama is not installed!"
    echo "📥 Install from: https://ollama.com/download"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✅ Ollama is installed"

    # Check if Ollama is running
    if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        echo "🚀 Starting Ollama..."
        ollama serve > /dev/null 2>&1 &
        OLLAMA_PID=$!
        sleep 2
        echo "✅ Ollama started (PID: $OLLAMA_PID)"
    else
        echo "✅ Ollama is already running"
    fi

    # Check if phi3 model is available
    if ! ollama list | grep -q "phi3"; then
        echo "📥 Downloading phi3 model (this may take a few minutes)..."
        ollama pull phi3
        echo "✅ phi3 model downloaded"
    else
        echo "✅ phi3 model is available"
    fi
fi

echo ""
echo "📦 Building project..."
echo ""

# Build the project
cargo build --release --bin backend

echo ""
echo "✅ Build complete!"
echo ""
echo "🚀 Starting services..."
echo ""

# Start backend in background
echo "🔧 Starting backend on http://localhost:3000..."
cargo run --release --bin backend > backend.log 2>&1 &
BACKEND_PID=$!
echo "✅ Backend started (PID: $BACKEND_PID)"

# Wait for backend to be ready
echo "⏳ Waiting for backend to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo "✅ Backend is ready!"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo "❌ Backend failed to start. Check backend.log for details."
        kill $BACKEND_PID 2>/dev/null
        exit 1
    fi
done

echo ""
echo "🎨 Starting frontend..."
echo ""

# Check if dx is installed
if ! command -v dx &> /dev/null; then
    echo "📥 Installing dioxus-cli..."
    cargo install dioxus-cli
fi

# Start frontend
dx serve

# Cleanup on exit
trap "echo ''; echo '🛑 Stopping services...'; kill $BACKEND_PID 2>/dev/null; echo '✅ Services stopped'; exit" INT TERM
