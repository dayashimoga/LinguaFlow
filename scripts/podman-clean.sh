#!/usr/bin/env bash
# LinguaFlow — Idempotent Podman Cleanup (Bash)
set -uo pipefail

echo ">>> Cleaning LinguaFlow Podman Containers, Images, and Volumes..."

podman stop -t 2 linguaflow-gateway 2>/dev/null || true
podman rm -f linguaflow-gateway 2>/dev/null || true
podman volume rm -f linguaflow-data 2>/dev/null || true
podman rmi -f linguaflow:stage-0 2>/dev/null || true

echo ">>> Cleaned all LinguaFlow Podman containers, volumes, and temporary images."
