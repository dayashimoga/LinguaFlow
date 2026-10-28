# LinguaFlow — Idempotent Podman Bring-Down (PowerShell)
$ErrorActionPreference = "SilentlyContinue"

Write-Host ">>> Stopping LinguaFlow Podman Services..." -ForegroundColor Cyan

podman stop -t 2 linguaflow-gateway
podman rm -f linguaflow-gateway

Write-Host ">>> LinguaFlow Podman Services cleanly stopped." -ForegroundColor Green
