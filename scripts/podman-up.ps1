# LinguaFlow — Idempotent Podman Bring-Up (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host ">>> Starting LinguaFlow Podman Services..." -ForegroundColor Cyan

# Check if podman is installed
if (-not (Get-Command podman -ErrorAction SilentlyContinue)) {
    Write-Error "Podman is not installed or not in PATH."
    exit 1
}

# Stop and remove existing container if running
Write-Host ">>> Ensuring clean state for container 'linguaflow-gateway'..." -ForegroundColor Yellow
podman stop -t 2 linguaflow-gateway 2>$null
podman rm -f linguaflow-gateway 2>$null

# Build container image if needed
Write-Host ">>> Building image 'linguaflow:stage-0'..." -ForegroundColor Cyan
podman build -t linguaflow:stage-0 -f Containerfile .

# Ensure volume exists
Write-Host ">>> Ensuring volume 'linguaflow-data'..." -ForegroundColor Cyan
podman volume create linguaflow-data 2>$null

# Run container with bounded memory and health check
Write-Host ">>> Launching container 'linguaflow-gateway'..." -ForegroundColor Green
podman run -d `
    --name linguaflow-gateway `
    -p 8080:8080 `
    --memory 512m `
    -v linguaflow-data:/data `
    -e RUST_LOG=info `
    -e LINGUAFLOW_DATA_DIR=/data `
    linguaflow:stage-0

Write-Host ">>> Waiting for health check on http://localhost:8080/api/v1/health..." -ForegroundColor Cyan
$maxRetries = 15
$attempt = 0
$healthy = $false

while ($attempt -lt $maxRetries) {
    Start-Sleep -Seconds 1
    $attempt++
    try {
        $resp = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/health" -Method Get -TimeoutSec 2
        if ($resp.status -eq "healthy") {
            $healthy = $true
            break
        }
    } catch {
        # continue waiting
    }
}

if ($healthy) {
    Write-Host ">>> LinguaFlow Gateway is HEALTHY and listening on http://localhost:8080" -ForegroundColor Green
} else {
    Write-Host ">>> Gateway launched but healthcheck timed out. Inspect logs with: podman logs linguaflow-gateway" -ForegroundColor Yellow
}
