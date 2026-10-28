# LinguaFlow — Development Environment Setup Guide

## 1. Prerequisites

### Host Environment
- **Rust Toolchain**: 1.80+ (stable). Verify with `rustc --version` and `cargo --version`.
- **Node.js**: v20+ or v22+ LTS. Verify with `node --version` and `npm --version`.
- **Git**: 2.40+.
- **Podman**: 4.5+ (recommended for containerized Linux dev services).

### Platform Specifics
- **Windows**: Windows 10/11 x64 or ARM64. WebView2 is pre-installed on Windows 10 (version 1803+) and Windows 11.
- **Linux (Ubuntu / Debian)**:
  ```bash
  sudo apt-get update && sudo apt-get install -y \
    libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```
- **macOS**: macOS 11.0+ (Big Sur or newer), Xcode Command Line Tools (`xcode-select --install`).
- **Android**: Android Studio with SDK API 34, NDK (26.x+), and Java 17.

---

## 2. Quickstart: Building & Running

### 2.1 Workspace Validation
```bash
# Verify all workspace crates compile
cargo check --workspace

# Run complete unit and integration test suite
cargo test --workspace

# Run strict linter and formatting checks
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### 2.2 Running the Frontend (Vite)
```bash
# Install dependencies
npm ci

# Launch dev server on http://localhost:5173
npm run dev

# Compile production web PWA bundle
npm run build
```

### 2.3 Running the Axum Gateway Server
```bash
cargo run -p lingua-cli -- serve --port 8080
```
Visit `http://localhost:8080/api/v1/health` and `http://localhost:8080/api/v1/capabilities`.

### 2.4 Running the Tauri Desktop Application
```bash
# Install Tauri CLI if not already installed
cargo install tauri-cli --version "^2.0.0" --locked

# Launch desktop app with hot reload
cargo tauri dev
```

---

## 3. Podman Containerized Bring-Up
For isolated, reproducible Linux daemon execution:
- **Windows (PowerShell)**: `.\scripts\podman-up.ps1`
- **Linux / macOS (Bash)**: `./scripts/podman-up.sh`

To clean up all containers, images, and volumes:
- **Windows (PowerShell)**: `.\scripts\podman-clean.ps1`
- **Linux / macOS (Bash)**: `./scripts/podman-clean.sh`
