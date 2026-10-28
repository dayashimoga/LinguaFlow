#!/usr/bin/env bash
# LinguaFlow — Idempotent Podman Bring-Up (Bash)
set -euo pipefail

echo ">>> Starting LinguaFlow Podman Services..."

if ! command -v podman &>/dev/null; then
    echo "ERROR: Podman is not installed or not in PATH."
    exit 1
fi

echo ">>> Ensuring clean state for container 'linguaflow-gateway'..."
podman stop -t 2 linguaflow-gateway 2>/dev/null || true
podman rm -f linguaflow-gateway 2>/dev/null || true

echo ">>> Building image 'linguaflow:stage-0'..."
podman build -t linguaflow:stage-0 -f Containerfile .

echo ">>> Ensuring volume 'linguaflow-data'..."
podman volume create linguaflow-data 2>/dev/null || true

echo ">>> Launching container 'linguaflow-gateway'..."
podman run -d \
    --name linguaflow-gateway \
    -p 8080:8080 \
    --memory 512m \
    -v linguaflow-data:/data \
    -e RUST_LOG=info \
    -e LINGUAFLOW_DATA_DIR=/data \
    linguaflow:stage-0

echo ">>> Waiting for health check on http://localhost:8080/api/v1/health..."
retries=15
count=0
healthy=0

while [ "$count" -lt "$retries" ]; do
    sleep 1
    count=$((count + 1))
    if curl -s -f http://localhost:8080/api/v1/health >/dev/null 2>&1; then
        healthy=1
        break
    fi
done

if [ "$healthy" -eq 1 ]; then
    echo ">>> LinguaFlow Gateway is HEALTHY and listening on http://localhost:8080"
else
    echo ">>> Gateway launched but healthcheck timed out. Check: podman logs linguaflow-gateway"
fi
