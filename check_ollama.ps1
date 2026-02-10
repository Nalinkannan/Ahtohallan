# Ollama Diagnostic Script for Ahtohallan RAG Chatbot (Windows)
# This script checks if Ollama is properly configured with GPU support

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "  Ollama Diagnostic Script (Windows)" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Helper functions
function Check-Passed {
    param($message)
    Write-Host "✓ " -ForegroundColor Green -NoNewline
    Write-Host $message
}

function Check-Failed {
    param($message)
    Write-Host "✗ " -ForegroundColor Red -NoNewline
    Write-Host $message
}

function Check-Warning {
    param($message)
    Write-Host "⚠ " -ForegroundColor Yellow -NoNewline
    Write-Host $message
}

function Check-Info {
    param($message)
    Write-Host "ℹ " -ForegroundColor Blue -NoNewline
    Write-Host $message
}

# 1. Check if Ollama is installed
Write-Host "1. Checking Ollama installation..." -ForegroundColor Cyan
$ollamaPath = Get-Command ollama -ErrorAction SilentlyContinue
if ($ollamaPath) {
    $version = & ollama --version 2>&1 | Select-Object -First 1
    Check-Passed "Ollama is installed: $version"
} else {
    Check-Failed "Ollama is not installed"
    Write-Host ""
    Write-Host "Install Ollama from: https://ollama.ai/download" -ForegroundColor Yellow
    exit 1
}
Write-Host ""

# 2. Check if Ollama service is running
Write-Host "2. Checking Ollama service..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -UseBasicParsing -TimeoutSec 5 -ErrorAction Stop
    Check-Passed "Ollama is running on port 11434"
} catch {
    Check-Failed "Ollama is not running"
    Write-Host ""
    Write-Host "Start Ollama:" -ForegroundColor Yellow
    Write-Host "  Option 1: Restart Ollama Service (Services -> Ollama -> Restart)" -ForegroundColor Yellow
    Write-Host "  Option 2: Run 'ollama serve' in a terminal" -ForegroundColor Yellow
    exit 1
}
Write-Host ""

# 3. Check GPU availability
Write-Host "3. Checking GPU..." -ForegroundColor Cyan
$nvidiaSmi = Get-Command nvidia-smi -ErrorAction SilentlyContinue
if ($nvidiaSmi) {
    try {
        $gpuInfo = & nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0) {
            Check-Passed "NVIDIA GPU detected: $gpuInfo"

            # Check GPU memory usage
            $gpuUsed = & nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | Select-Object -First 1
            $gpuTotal = & nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | Select-Object -First 1
            Check-Info "GPU Memory: ${gpuUsed}MB / ${gpuTotal}MB used"

            # Check CUDA version
            $cudaVersion = & nvidia-smi | Select-String "CUDA Version" | ForEach-Object { $_ -replace '.*CUDA Version:\s*([0-9.]+).*', '$1' }
            if ($cudaVersion) {
                Check-Info "CUDA Version: $cudaVersion"
            }
        } else {
            Check-Warning "nvidia-smi found but GPU not accessible"
        }
    } catch {
        Check-Warning "Error checking NVIDIA GPU: $_"
    }
} else {
    Check-Warning "No NVIDIA GPU detected (will use CPU only)"
    Check-Info "For GPU acceleration, install NVIDIA drivers and CUDA toolkit"
}
Write-Host ""

# 4. Check if phi3 model is downloaded
Write-Host "4. Checking phi3 model..." -ForegroundColor Cyan
$models = & ollama list 2>&1
if ($models -match "phi3") {
    $modelLine = $models | Select-String "phi3" | Select-Object -First 1
    Check-Passed "phi3 model is available"
    Check-Info "$modelLine"
} else {
    Check-Failed "phi3 model not found"
    Write-Host ""
    Write-Host "Download phi3 with: ollama pull phi3" -ForegroundColor Yellow
    exit 1
}
Write-Host ""

# 5. Test Ollama API
Write-Host "5. Testing Ollama API..." -ForegroundColor Cyan
try {
    $apiResponse = Invoke-RestMethod -Uri "http://localhost:11434/api/tags" -Method Get -TimeoutSec 10
    Check-Passed "Ollama API is responding"
    $modelCount = $apiResponse.models.Count
    Check-Info "Available models: $modelCount"
} catch {
    Check-Failed "Ollama API is not responding: $_"
    exit 1
}
Write-Host ""

# 6. Performance test
Write-Host "6. Running quick performance test..." -ForegroundColor Cyan
Write-Host "   (This may take 10-30 seconds on first run)" -ForegroundColor Gray

$startTime = Get-Date

$testRequest = @{
    model = "phi3"
    prompt = "Say only the word: OK"
    stream = $false
    options = @{
        num_predict = 10
    }
} | ConvertTo-Json

try {
    $testResponse = Invoke-RestMethod -Uri "http://localhost:11434/api/generate" -Method Post -Body $testRequest -ContentType "application/json" -TimeoutSec 60
    $endTime = Get-Date
    $elapsed = ($endTime - $startTime).TotalMilliseconds

    if ($testResponse.response) {
        Check-Passed "Model inference working (${elapsed}ms)"
        Check-Info "Response: $($testResponse.response)"

        # Performance assessment
        if ($elapsed -lt 2000) {
            Check-Passed "Excellent performance (likely using GPU)"
        } elseif ($elapsed -lt 5000) {
            Check-Info "Good performance"
        } elseif ($elapsed -lt 10000) {
            Check-Warning "Moderate performance (GPU may not be used)"
        } else {
            Check-Warning "Slow performance (likely using CPU only)"
        }
    } else {
        Check-Failed "Model inference returned no response"
    }
} catch {
    Check-Failed "Model inference failed: $_"
}
Write-Host ""

# 7. Check backend requirements
Write-Host "7. Checking backend requirements..." -ForegroundColor Cyan
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargo) {
    Check-Passed "Rust/Cargo is installed"
    $rustVersion = & cargo --version
    Check-Info "$rustVersion"
} else {
    Check-Failed "Rust/Cargo not found"
    Write-Host "Install from: https://rustup.rs" -ForegroundColor Yellow
}

if (Test-Path "Cargo.toml") {
    Check-Passed "Cargo.toml found (in project directory)"
} else {
    Check-Warning "Not in project directory (Cargo.toml not found)"
}
Write-Host ""

# 8. Environment variables
Write-Host "8. Checking environment variables..." -ForegroundColor Cyan
$ollamaNumGpu = $env:OLLAMA_NUM_GPU
if ($ollamaNumGpu) {
    Check-Info "OLLAMA_NUM_GPU=$ollamaNumGpu"
} else {
    Check-Info "OLLAMA_NUM_GPU not set (default will be used)"
}

$ollamaMaxVram = $env:OLLAMA_MAX_VRAM
if ($ollamaMaxVram) {
    Check-Info "OLLAMA_MAX_VRAM=$ollamaMaxVram MB"
}

$ollamaHost = $env:OLLAMA_HOST
if ($ollamaHost) {
    Check-Info "OLLAMA_HOST=$ollamaHost"
}
Write-Host ""

# 9. Check ports
Write-Host "9. Checking network ports..." -ForegroundColor Cyan
$ollamaPort = Get-NetTCPConnection -LocalPort 11434 -ErrorAction SilentlyContinue
if ($ollamaPort) {
    Check-Passed "Port 11434 (Ollama) is in use"
}

$backendPort = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue
if ($backendPort) {
    Check-Passed "Port 3000 (Backend) is in use"
} else {
    Check-Info "Port 3000 (Backend) is available - ready to start backend"
}

$frontendPort = Get-NetTCPConnection -LocalPort 8080 -ErrorAction SilentlyContinue
if ($frontendPort) {
    Check-Passed "Port 8080 (Frontend) is in use"
} else {
    Check-Info "Port 8080 (Frontend) is available - ready to start frontend"
}
Write-Host ""

# Summary
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "  Summary" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Check-Passed "Ollama is properly configured!"
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Green
Write-Host "  1. Start backend:  " -NoNewline -ForegroundColor Green
Write-Host "cargo run --bin backend --features backend" -ForegroundColor White
Write-Host "  2. Start frontend: " -NoNewline -ForegroundColor Green
Write-Host "dx serve --platform web" -ForegroundColor White
Write-Host "  3. Open browser:   " -NoNewline -ForegroundColor Green
Write-Host "http://localhost:8080" -ForegroundColor White
Write-Host ""
Write-Host "To monitor GPU usage:" -ForegroundColor Cyan
if ($nvidiaSmi) {
    Write-Host "  nvidia-smi -l 1" -ForegroundColor White
}
Write-Host ""
Write-Host "To set GPU environment variables permanently:" -ForegroundColor Cyan
Write-Host "  [System.Environment]::SetEnvironmentVariable('OLLAMA_NUM_GPU', '1', 'User')" -ForegroundColor White
Write-Host ""
Write-Host "For more help, see: OLLAMA_SETUP.md" -ForegroundColor Yellow
Write-Host ""
