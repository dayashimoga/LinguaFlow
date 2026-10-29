# LinguaFlow — High-Performance Multilingual Communication Platform

[![CI Quality Gates](https://github.com/dayashimoga/LinguaFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/dayashimoga/LinguaFlow/actions/workflows/ci.yml)
[![Mobile Builds](https://github.com/dayashimoga/LinguaFlow/actions/workflows/mobile.yml/badge.svg)](https://github.com/dayashimoga/LinguaFlow/actions/workflows/mobile.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust: 1.80+](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Svelte: 5](https://img.shields.io/badge/Svelte-5-ff3e00.svg)](https://svelte.dev)
[![Tauri: 2](https://img.shields.io/badge/Tauri-2-24c8db.svg)](https://tauri.app)

LinguaFlow is an open, high-performance, offline-first multilingual communication and translation platform designed for South Asian and global languages. LinguaFlow launches with primary native support for **Kannada (`kn-IN`)**, **Tamil (`ta-IN`)**, and **English (`en-US`)**, and is architected to scale to other Indic and international languages.

> For an in-depth architectural breakdown, crate specifications, and performance audit, refer to the **[Complete Project Walkthrough](docs/PROJECT_WALKTHROUGH.md)**.

---

## Key Features

- **100% Offline-First Architecture**: Fast bilingual lexicon lookup, neural model translation, audio capture, speech synthesis, and spaced-repetition learning operate natively on-device with zero network dependency.
- **Direct Native IPC**: The desktop application packages production UI assets directly into the binary (`custom-protocol`), eliminating any reliance on a local development web server.
- **Authentic Capability Detection**: Features report real hardware and model readiness—no simulated translations, hardcoded completions, or fake progress bars.
- **Full-Duplex Speech & Conversation**: 16 kHz audio ring buffer, real-time Energy/ZCR Voice Activity Detection (VAD), streaming ASR, formant TTS, and acoustic echo ducking.
- **Translation Intelligence**: Register adaptation (formal vs. colloquial), named entity protection, phonetic Indic transliteration, and objective quality scoring (chrF, WER, CER).
- **Media, Subtitles & OCR**: Chunked media transcription with WebVTT/SRT export, borderless desktop live caption overlay, and image OCR with Otsu binarization.
- **Spaced-Repetition Learning**: SuperMemo SM-2 flashcard scheduler, acoustic pronunciation scoring, grammar modules, and zero-log GDPR/DISHA compliant data export and purge.
- **Enterprise Multi-Tenancy**: Organization workspaces, role-based access control (RBAC), shared domain glossaries, token-bucket rate limiting, and inference worker pool backpressure.

---

## Architecture Overview

```
                      ┌──────────────────────┐
                      │    Svelte 5 / TS     │ (Glassmorphic Accessible UI)
                      └──────────┬───────────┘
                                 │ Direct Native IPC / Custom Protocol (tauri://localhost)
                      ┌──────────▼───────────┐
                      │      src-tauri       │ (Tauri 2 Native Host)
                      └──────────┬───────────┘
                                 │
           ┌─────────────────────┼─────────────────────┐
           │                     │                     │
           ▼                     ▼                     ▼
    ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
    │  lingua-cli  │      │lingua-server │      │lingua-storage│ (SQLite WAL + FTS5)
    └──────┬───────┘      └──────┬───────┘      └──────┬───────┘
           │                     │                     │
           └─────────────────────┼─────────────────────┘
                                 │
                                 ▼
                          ┌──────────────┐
                          │lingua-engine │ (Audio, VAD, ASR, TTS, NLP, OCR, SM-2, RBAC)
                          └──────┬───────┘
                                 │
                                 ▼
                          ┌──────────────┐
                          │ lingua-core  │ (Domain Types, BCP-47, Script Detection, Errors)
                          └──────────────┘
```

---

## Workspace Crates

| Crate | Purpose | Key Modules |
| :--- | :--- | :--- |
| **[`lingua-core`](crates/lingua-core)** | Zero-I/O Domain Foundation | BCP-47 codes, Dravidian script detection, NFC normalization, typed IPC envelopes, system error taxonomy. |
| **[`lingua-storage`](crates/lingua-storage)** | Local Persistence Engine | SQLite in WAL mode, 9 automated migrations, FTS5 full-text search, settings, history, model catalogue, and multi-tenant DB. |
| **[`lingua-engine`](crates/lingua-engine)** | Inference & NLP Coordinator | 16 kHz audio buffer, Energy/ZCR VAD, streaming ASR, TTS, Otsu OCR, document translator, SM-2 algorithm, worker pool. |
| **[`lingua-server`](crates/lingua-server)** | Edge & Cloud Gateway | Axum REST endpoints and WebSocket duplex streaming gateway for server deployments. |
| **[`lingua-cli`](crates/lingua-cli)** | Companion CLI & Tooling | Diagnostics, status reporting, database maintenance, and 28 empirical microbenchmarks. |
| **[`src-tauri`](src-tauri)** | Desktop & Mobile Host | Direct native IPC handlers (`commands.rs`), embedded offline assets (`tauri://localhost`), and startup resilience guards. |

---

## Empirical Benchmark Highlights

Measured on Windows 11 / x86_64 across 500 continuous iterations via `lingua bench`:

| Metric | Target Budget | Measured Latency | Safety Margin |
| :--- | :--- | :--- | :--- |
| **Typed IPC Serialization** | < 5.0 ms | **23.42 µs** | **213x faster** |
| **SQLite User Settings Read** | < 20.0 ms | **39.39 µs** | **507x faster** |
| **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20.0 ms | **0.480 ms (480 µs)** | **41.6x faster** |
| **Offline Lexicon Translation** | < 10.0 ms | **0.304 ms (304 µs)** | **32.8x faster** |
| **16 kHz Audio Buffer Soak** | < 0.1 ms | **0.005 ms (5.0 µs)** | **20.0x faster** |
| **VAD Energy & ZCR Frame Check** | < 0.1 ms | **0.004 ms (4.0 µs)** | **25.0x faster** |
| **Multi-Formant Speech Synthesis** | < 50.0 ms | **4.752 ms** | **10.5x faster** |
| **Acoustic Echo Ducking Switch** | < 0.1 ms | **0.00005 ms (0.05 µs)** | **2,000x faster** |
| **Romanized Indic Transliteration** | < 0.5 ms | **0.003 ms (3.0 µs)** | **166x faster** |
| **SM-2 Interval Calculation** | < 10.0 µs | **0.330 µs** | **30.3x faster** |
| **Multi-Tenant RBAC Evaluation** | < 0.01 ms | **0.00008 ms (0.08 µs)** | **125x faster** |
| **API Key Rate Limit Check** | < 0.05 ms | **0.0002 ms (0.2 µs)** | **250x faster** |

**Resource Footprints**:
- **Idle Memory**: **< 59 MB RAM** (Limit: < 120 MB)
- **Peak Audio Buffer**: **< 1.92 MB** bounded memory
- **Compiled Web Assets (`dist/`)**: **347.8 kB total** (92.4 kB gzip)

---

## Quickstart & Verification

### 1. Development Mode
```powershell
# Install frontend dependencies
npm ci

# Run native desktop application with live reload
npm run tauri dev
```

### 2. Standalone Production Build
```powershell
# Compile production web bundle
npm run build

# Compile all workspace binaries
cargo build --release --workspace

# The desktop binary is generated at target/release/lingua-tauri(.exe)
```

### 3. Run Quality Gates & Tests
```powershell
# Svelte frontend typecheck
npm run check

# Strict Rust formatting & Clippy checks (0 warnings enforced)
cargo fmt --check
cargo clippy --workspace -- -D warnings

# Execute the 120 automated unit and integration tests
cargo test --workspace

# Execute the 28 empirical microbenchmarks
cargo run -p lingua-cli --release -- bench --iterations 500
```

---

## Multi-Platform Support

| Platform | Architecture | Distribution Format | Embedded Offline Assets |
| :--- | :--- | :--- | :---: |
| **Windows** | x86_64, aarch64 | NSIS Installer, Portable `.exe` | :white_check_mark: Yes (`tauri://localhost`) |
| **Linux** | x86_64, aarch64 | AppImage, `.deb` | :white_check_mark: Yes (`tauri://localhost`) |
| **macOS** | Universal (Intel + Apple Silicon) | `.app`, `.dmg` | :white_check_mark: Yes (`tauri://localhost`) |
| **Android** | aarch64, armv7, x86_64 | `.apk`, `.aab` | :white_check_mark: Yes |
| **iOS** | arm64 | Xcode Archive (`.xcarchive`) | :white_check_mark: Yes |
| **Web** | WebAssembly / Browsers | Static PWA Bundle (`dist/`) | :white_check_mark: Yes |

---

## Documentation Index

- **[Complete Project Walkthrough](docs/PROJECT_WALKTHROUGH.md)**: Exhaustive architectural, subsystem, and code-level walkthrough.
- **[Product Specification](docs/architecture/product-specification.md)**: Personas, stages, and requirements.
- **[Empirical Benchmarks](docs/BENCHMARKS.md)**: Full latency, throughput, and memory measurements.
- **[Architectural Decision Records](docs/adr/)**: ADR-0001 through ADR-0004.
- **[Requirements Traceability Matrix](docs/requirements-traceability-matrix.md)**: Complete mapping across Stages 0–8.
- **[Runbook](docs/RUNBOOK.md)** & **[Troubleshooting Guide](docs/TROUBLESHOOTING.md)**: Operational guides.
- **[Security Policy](SECURITY.md)** & **[SBOM](docs/SBOM.json)**: Vulnerability disclosure, privacy guarantees, and software bill of materials.

---

## License

Licensed under the Apache License, Version 2.0 or the MIT License at your option. See [LICENSE](LICENSE) for details.
