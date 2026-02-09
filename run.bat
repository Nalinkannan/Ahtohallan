@echo off
REM Ahtohallan RAG Chatbot - Quick Start Script (Windows)
REM ========================================================

echo.
echo ❄️  Ahtohallan RAG Chatbot - Quick Start
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust is not installed!
    echo 📥 Install from: https://rustup.rs/
    pause
    exit /b 1
)

REM Check if Ollama is installed
where ollama >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ⚠️  Ollama is not installed!
    echo 📥 Install from: https://ollama.com/download
    echo.
    set /p CONTINUE="Continue anyway? (y/n): "
    if /i not "%CONTINUE%"=="y" exit /b 1
) else (
    echo ✅ Ollama is installed

    REM Check if Ollama is running
    curl -s http://localhost:11434/api/tags >nul 2>nul
    if %ERRORLEVEL% NEQ 0 (
        echo 🚀 Starting Ollama...
        start /B ollama serve
        timeout /t 3 /nobreak >nul
        echo ✅ Ollama started
    ) else (
        echo ✅ Ollama is already running
    )

    REM Check if phi3 model is available
    ollama list | findstr /C:"phi3" >nul
    if %ERRORLEVEL% NEQ 0 (
        echo 📥 Downloading phi3 model (this may take a few minutes)...
        ollama pull phi3
        echo ✅ phi3 model downloaded
    ) else (
        echo ✅ phi3 model is available
    )
)

echo.
echo 📦 Building project...
echo.

REM Build the project
cargo build --release --bin backend
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed!
    pause
    exit /b 1
)

echo.
echo ✅ Build complete!
echo.
echo 🚀 Starting services...
echo.

REM Start backend in background
echo 🔧 Starting backend on http://localhost:3000...
start /B cargo run --release --bin backend > backend.log 2>&1
echo ✅ Backend started

REM Wait for backend to be ready
echo ⏳ Waiting for backend to be ready...
set /a counter=0
:wait_loop
curl -s http://localhost:3000/health >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo ✅ Backend is ready!
    goto backend_ready
)
set /a counter+=1
if %counter% GEQ 30 (
    echo ❌ Backend failed to start. Check backend.log for details.
    pause
    exit /b 1
)
timeout /t 1 /nobreak >nul
goto wait_loop

:backend_ready
echo.
echo 🎨 Starting frontend...
echo.

REM Check if dx is installed
where dx >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo 📥 Installing dioxus-cli...
    cargo install dioxus-cli
)

REM Start frontend
dx serve

REM Note: On Windows, pressing Ctrl+C will terminate both processes
