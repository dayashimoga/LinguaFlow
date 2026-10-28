#!/usr/bin/env bash
# LinguaFlow — Idempotent Podman Bring-Down (Bash)
set -uo pipefail

echo ">>> Stopping LinguaFlow Podman Services..."

podman stop -t 2 linguaflow-gateway 2>/dev/null || true
podman rm -f linguaflow-gateway 2>/dev/null || true

echo ">>> LinguaFlow Podman Services cleanly stopped."
