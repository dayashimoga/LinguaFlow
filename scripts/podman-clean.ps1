# LinguaFlow — Idempotent Podman Cleanup (PowerShell)
$ErrorActionPreference = "SilentlyContinue"

Write-Host ">>> Cleaning LinguaFlow Podman Containers, Images, and Volumes..." -ForegroundColor Yellow

podman stop -t 2 linguaflow-gateway
podman rm -f linguaflow-gateway
podman volume rm -f linguaflow-data
podman rmi -f linguaflow:stage-0

Write-Host ">>> Cleaned all LinguaFlow Podman containers, volumes, and temporary images." -ForegroundColor Green
