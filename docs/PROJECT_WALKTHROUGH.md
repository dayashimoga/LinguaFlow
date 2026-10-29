# LinguaFlow — Complete Project Walkthrough & Architecture Guide

LinguaFlow is an open, high-performance, offline-first multilingual communication and translation platform designed for South Asian and global languages. LinguaFlow launches with primary native support for **Kannada (`kn-IN`)**, **Tamil (`ta-IN`)**, and **English (`en-US`)**, and is architected from inception to scale to other Indic and international languages.

---

## 1. Executive Summary & Core Philosophy

### Target User Personas
1. **Dr. Ananya (Rural Healthcare Physician — Karnataka)**:
   - *Context*: Interacts with Kannada-speaking rural patients and Tamil-speaking migrant workers in community health clinics. Connectivity is intermittent or absent.
   - *Needs*: Dependable offline speech translation, medically clear vocabulary lookup, hands-free voice conversation.
   - *Constraints*: Zero external cloud leakage of patient dialogues (HIPAA / DISHA compliance); entry-level device support.
2. **Murugan (Interstate Logistics Coordinator — Chennai & Bengaluru)**:
   - *Context*: Operates cross-border supply chains connecting Tamil Nadu and Karnataka. Frequently uses desktop workstations and Android phones on the go.
   - *Needs*: Instant dictionary search, technical term translation, document translation for shipping invoices and bills of lading.
   - *Constraints*: Rapid keyboard shortcuts on desktop, instant copy/paste, low memory consumption.
3. **Vikram (University Student & Language Enthusiast)**:
   - *Context*: Learning Kannada and Tamil literature, reading dual-language articles on laptop and phone.
   - *Needs*: Accurate phonetic pronunciation, grammatical breakdown, spaced-repetition vocabulary builder, live subtitles for recorded audio lectures.
   - *Constraints*: Aesthetic UI with correct Indic Unicode typography and scalable rendering.

### Core Engineering Guarantees
- **100% Offline-First**: All core operations—lexicon search, neural translation, speech capture, voice activity detection, text-to-speech, flashcard reviews, and document parsing—execute natively on-device without network dependency.
- **Direct Native In-Process IPC**: The desktop application runs directly via Tauri 2 native IPC (`tauri://localhost` custom protocol) with embedded assets; it never depends on an external or local development web server.
- **Honest Capability Detection**: No simulated translations, hardcoded completions, or fake progress bars. Features report actual device/model capabilities and degrade gracefully.
- **Bounded Resource Budgets**: Memory footprint is strictly bounded (audio ring buffers capped at <2 MB, LRU caches capped at 1,000 entries, total idle RAM <59 MB).
- **Auditability & Compliance**: Strict append-only changelogs (`CHANGELOG.md`), traceability matrices (`docs/requirements-traceability-matrix.md`), CycloneDX SBOM, and verified licenses for all dependencies.

---

## 2. Workspace Topology & System Architecture

LinguaFlow is organized as a modular Rust workspace paired with a reactive Svelte 5 / TypeScript frontend and Tauri 2 native host integration:

```
LinguaFlow/
├── .agents/                      # Engineering contracts, skills, and autonomous rules
├── .github/workflows/            # Multi-platform CI, Mobile, and Release workflows
│   ├── ci.yml                    # Quality gates, cross-platform matrix, desktop smoke tests
│   ├── mobile.yml                # Android APK/AAB & iOS build validation
│   └── release.yml               # CycloneDX SBOM generation & GitHub Releases
├── crates/                       # Modular Rust Workspace Crates
│   ├── lingua-core/              # Domain models, Unicode normalization, typed IPC, errors
│   ├── lingua-storage/           # SQLite persistence, migrations 001-009, WAL mode
│   ├── lingua-engine/            # Translation, speech, VAD, TTS, OCR, SM-2, worker pool
│   ├── lingua-server/            # Axum HTTP REST and WebSocket streaming gateway
│   └── lingua-cli/               # Diagnostic CLI tool and 28 microbenchmarks
├── docs/                         # ADRs, Product Specifications, User Guides, Acceptance Reports
│   ├── adr/                      # Architectural Decision Records (ADR-0001 to 0004)
│   ├── architecture/             # Specifications, pipeline diagrams, resource budgets
│   ├── BENCHMARKS.md             # Empirical latency and throughput measurements
│   ├── PROJECT_WALKTHROUGH.md    # This walkthrough document
│   └── SBOM.json                 # CycloneDX software bill of materials
├── scripts/                      # Container lifecycle scripts (Podman)
├── src/                          # Svelte 5 / TypeScript Frontend Application
│   ├── lib/                      # Reusable components, IPC bridge, stores, audio helpers
│   ├── views/                    # 12 functional view components
│   └── App.svelte                # Root application layout and navigation bar
├── src-tauri/                    # Tauri 2 Desktop & Mobile Host
│   ├── src/commands.rs           # Direct typed IPC handlers
│   ├── src/lib.rs                # Tauri runtime initialization & asset embedding
│   ├── tauri.conf.json           # Native shell configuration & security permissions
│   └── build.rs                  # Build script with fallback asset guards
├── Containerfile                 # Podman reproducible Linux build environment
├── podman-compose.yml            # Multi-container dev orchestration
├── Cargo.toml                    # Root workspace manifest
└── package.json                  # Frontend dependencies and build scripts
```

---

## 3. Modular Crate Breakdown

### 3.1 `lingua-core`
- **Zero-I/O Domain Layer**: Contains pure business logic and shared data models without network or file I/O dependencies.
- **Language Models ([`crates/lingua-core/src/language.rs`](file:///f:/LinguaFlow/crates/lingua-core/src/language.rs))**:
  - `LanguageCode` enum (`KnIn`, `TaIn`, `EnUs`) with BCP-47 tag parsing, localized native names (e.g. *ಕನ್ನಡ*, *தமிழ்*, *English*), and script validation.
  - `LanguagePair` validation ensuring source and target are distinct and supported.
- **Unicode Bounding ([`crates/lingua-core/src/unicode.rs`](file:///f:/LinguaFlow/crates/lingua-core/src/unicode.rs))**:
  - Script detection routines distinguishing Kannada (`U+0C80`–`U+0CFF`), Tamil (`U+0B80`–`U+0BFF`), and Latin alphabets.
  - NFC normalization and payload length bounding protecting against denial-of-service memory exhaustion.
- **Typed IPC Protocol ([`crates/lingua-core/src/ipc.rs`](file:///f:/LinguaFlow/crates/lingua-core/src/ipc.rs))**:
  - Uniform envelope structures: `IpcRequest<T>` and `IpcResponse<T>` guaranteeing type-safe error/success responses between Rust and the frontend.
- **Error Taxonomy ([`crates/lingua-core/src/error.rs`](file:///f:/LinguaFlow/crates/lingua-core/src/error.rs))**:
  - `LinguaError` enum with structured error codes (`ERR_STORAGE`, `ERR_CAPABILITY_UNAVAILABLE`, `ERR_CIRCUIT_BREAKER_OPEN`, `ERR_RATE_LIMITED`) and recovery advisories.

### 3.2 `lingua-storage`
- **Local Persistence**: Embedded SQLite 3.45+ engine running in Write-Ahead Logging (`WAL`) mode with `pragma synchronous = NORMAL`.
- **Schema Migration Runner ([`crates/lingua-storage/src/schema.rs`](file:///f:/LinguaFlow/crates/lingua-storage/src/schema.rs))**:
  - 9 automated, versioned schema migrations applied sequentially:
    - `001_initial_schema`: User settings, language preferences, audio configurations.
    - `002_dictionary_and_history`: FTS5 full-text search virtual tables and history logs.
    - `003_offline_models`: Offline model catalog, download state, file sizes, and SHA-256 hashes.
    - `004_speech_captions`: Real-time speech sessions, transcript segments, and export metadata.
    - `005_conversation_turns`: Dual-speaker conversation turns, audio blob markers, and turn timestamps.
    - `006_intelligence_and_glossaries`: Domain glossaries, entity rules, and circuit breaker metrics.
    - `007_media_and_documents`: Media transcription jobs, subtitle tracks, OCR scans, and document tasks.
    - `008_learning_and_privacy`: SM-2 flashcard decks, review logs, and data privacy export/purge markers.
    - `009_enterprise_platform`: Organizations, multi-tenant RBAC, API keys, rate limit quotas, and audit events.
- **Repositories**: Clean separation of concerns with dedicated repositories for dictionary, settings, history, model packs, conversations, glossaries, learning, and enterprise accounts.

### 3.3 `lingua-engine`
The computational heart of LinguaFlow containing real-time audio processing, inference orchestration, NLP, and enterprise scaling:
- **Audio & Speech Pipeline**:
  - [`audio_buffer.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/audio_buffer.rs): 16 kHz 16-bit mono ring buffer with bounded peak allocation (<2 MB) and zero-allocation frame extraction.
  - [`vad.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/vad.rs): Low-latency Voice Activity Detection analyzing frame RMS energy and Zero-Crossing Rate (ZCR) in 4.0 µs.
  - [`streaming_asr.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/streaming_asr.rs): Streaming speech-to-text coordinator emitting incremental interim and final transcripts.
  - [`caption_coordinator.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/caption_coordinator.rs): Subtitle export engine generating WebVTT and SubRip (SRT) format output.
  - [`tts_engine.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/tts_engine.rs): Multi-formant speech synthesizer generating audible Indic speech waveforms in <4.8 ms.
  - [`conversation_orchestrator.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/conversation_orchestrator.rs): Turn-taking coordinator managing push-to-talk, hands-free voice, and acoustic echo ducking (0.05 µs switch latency).
- **Translation & NLP Intelligence**:
  - [`lexicon_engine.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/lexicon_engine.rs): Bidirectional dictionary lookup and morphological root matching (<0.3 ms).
  - [`local_model_engine.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/local_model_engine.rs): Neural ONNX runtime translator loading local quantized models.
  - [`context_nlp.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/context_nlp.rs): Colloquial vs. formal register adaptation and named entity protection (protecting numbers, dates, and proper nouns).
  - [`transliteration.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/transliteration.rs): Real-time romanized script transliteration (e.g., typing "namaskara" -> "ನಮಸ್ಕಾರ") in 3.0 µs.
  - [`registry.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/registry.rs): Dynamic multi-engine registry with 3-state circuit breakers (`Closed`, `Open`, `HalfOpen`) preventing cascading failures.
  - [`evaluation.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/evaluation.rs): Objective translation quality assessment computing chrF, Word Error Rate (WER), and Character Error Rate (CER).
- **Media, OCR & Document Processing**:
  - [`media_transcription.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/media_transcription.rs): Chunked streaming decoder processing long audio/video files with constant memory overhead (<3.2 MB).
  - [`ocr.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/ocr.rs): Image binarization via Otsu thresholding and connected-component bounding box extraction in 0.52 ms.
  - [`document.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/document.rs): Block-level structural document translator for Markdown, Plain Text, CSV, and PDF text streams.
  - [`platform_integrations.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/platform_integrations.rs): Desktop live caption overlay controller with multi-monitor clamping and system-wide hotkeys.
- **Learning, Security & Enterprise Systems**:
  - [`learning.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/learning.rs): SuperMemo SM-2 spaced repetition scheduler, pronunciation scorer, grammar topics, and listening challenge generator.
  - [`security_audit.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/security_audit.rs): Zero-log privacy scrubber, GDPR/DISHA cryptographic data export, and full database purge.
  - [`rbac.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/rbac.rs): Multi-tenant role-based access control evaluated in 0.08 µs.
  - [`api_gateway.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/api_gateway.rs): SHA-256 hashed API key authentication and token-bucket rate limiter.
  - [`worker_pool.rs`](file:///f:/LinguaFlow/crates/lingua-engine/src/worker_pool.rs): Multi-producer multi-consumer worker pool with bounded queue capacity and backpressure monitoring.

### 3.4 `lingua-server`
- **Axum High-Throughput Edge Gateway**: Provides REST endpoints and WebSocket streaming for cloud, local network, or containerized deployments.
- **REST Endpoints**:
  - `GET /api/v1/health`: SQLite status, memory consumption, active engines.
  - `GET /api/v1/capabilities`: Dynamically discovered per-language capability matrix.
  - `POST /api/v1/translate`: Translation endpoint supporting formality, context, and glossaries.
  - `GET /api/v1/dictionary/search`: Sub-millisecond FTS5 dictionary lookup.
  - `GET /api/v1/learn/*`: Spaced-repetition card decks, reviews, and progress tracking.
  - `GET /api/v1/enterprise/*`: Multi-tenant organization management, API key issuance, and usage metrics.
- **WebSocket Gateway (`/api/v1/ws`)**: Full-duplex binary audio streaming and JSON caption/transcript events.

### 3.5 `lingua-cli`
- Headless operational command-line tool used in development, operations, and CI validation:
  ```powershell
  # System diagnostic health check
  lingua status

  # Run full 28-microbenchmark suite with 500 iterations
  lingua bench --iterations 500

  # Fast dictionary lookup
  lingua dict "water" --source en-US --target ta-IN

  # Direct translation via CLI
  lingua translate "Good morning, welcome to Bengaluru" --source en-US --target kn-IN --formality formal

  # Enterprise organization and API key management
  lingua org create --name "HealthGrid Karnataka"
  lingua keys create --org-id "org_default" --name "Production Edge Key"
  ```

### 3.6 `src-tauri` (Native Desktop & Mobile Host)
- Bridges the Svelte 5 frontend with the native Rust engine.
- Contains ~1,600 lines of type-safe IPC command handlers in [`src-tauri/src/commands.rs`](file:///f:/LinguaFlow/src-tauri/src/commands.rs).
- Uses Tauri's `custom-protocol` (`tauri://localhost`) to bundle and serve production web assets directly from the compiled binary without any local HTTP server.
- Built-in `src-tauri/build.rs` includes automated frontend compilation and a compile-time fallback placeholder guard.

### 3.7 Frontend Architecture (`src/`)
Built with Svelte 5 and TypeScript, optimized for low memory and high-framerate rendering across desktop, tablet, and mobile:

| View Component | Purpose | Key Capabilities |
| :--- | :--- | :--- |
| **`HomeView.svelte`** | System Dashboard | Quick action cards, active language selector, database health indicator, and engine capability status. |
| **`DictionaryView.svelte`** | Lexicon Explorer | Real-time FTS5 search with prefix completion, script auto-detection, grammar tags, favorites, and history. |
| **`TranslateView.svelte`** | Text Translation | Dual-pane translation with formality toggle, phonetic transliteration, entity protection, and speech audio playback. |
| **`LiveSpeechView.svelte`** | Streaming Captions | Real-time audio waveform visualizer, VAD indicator, continuous translated captions, and WebVTT/SRT export. |
| **`ConversationView.svelte`** | Voice Conversation | Split-screen dual-speaker dialogue, Push-to-Talk buttons, hands-free turn-taking, and acoustic echo ducking. |
| **`IntelligenceView.svelte`** | Advanced NLP | Custom domain glossary management, entity protection rules, circuit breaker status, and chrF/WER/CER quality testing. |
| **`MediaView.svelte`** | Subtitles & OCR | Video/audio file transcription with subtitle preview, desktop floating live overlay controls, image OCR, and document parser. |
| **`LearnView.svelte`** | Language Learning | SuperMemo SM-2 flashcard review deck, pronunciation scoring feedback, grammar topics, and listening comprehension drills. |
| **`EnterpriseView.svelte`** | Multi-Tenancy & API | Organization workspaces, team member RBAC, shared glossaries, API key management, usage metering, and audit logs. |
| **`ModelsView.svelte`** | Model Packs | Catalog of offline neural models, download/delete controls, SHA-256 integrity verification, and disk usage display. |
| **`SettingsView.svelte`** | Preferences | Theme selection (Light/Dark/System), audio input/output devices, resource profiles (Low/Balanced/Max), and privacy purge. |

---

## 4. Stage-by-Stage Implementation Summary

| Stage | Milestones & Deliverables | Primary Verification Test Suite |
| :--- | :--- | :--- |
| **Stage 0: Foundation** | Rust workspace, Podman dev environment, SQLite schema migrations, multi-platform CI/CD, capability baseline. | `tests/foundation.rs`, `.github/workflows/ci.yml` |
| **Stage 1: Dictionary & Text** | SQLite FTS5 index, bidirectional lexicon lookup, script detection, history, dictionary UI. | `tests/linguistic_correctness.rs` (10 tests) |
| **Stage 2: Offline Translation** | Local neural model engine, SHA-256 verification, cancellation tokens, hybrid routing, models UI. | `tests/offline_translation.rs` (5 tests) |
| **Stage 3: Real-Time Speech** | 16 kHz audio ring buffer, energy/ZCR VAD, streaming ASR coordinator, WebVTT/SRT caption export. | `tests/speech_captions.rs` (6 tests) |
| **Stage 4: Voice Conversation** | Turn-taking state machine, push-to-talk, acoustic echo ducking, multi-formant TTS synthesis. | `tests/conversation_turns.rs` (6 tests) |
| **Stage 5: Intelligence & NLP** | Circuit breakers, context/formality tuning, romanized Indic transliteration, chrF/WER/CER metrics. | `tests/translation_intelligence.rs` (7 tests) |
| **Stage 6: Media, OCR & Docs** | File transcription, desktop borderless overlay, Otsu image OCR, structured document parser. | `tests/media_streaming.rs` (8 tests) |
| **Stage 7: Learning & Privacy** | SuperMemo SM-2 flashcards, pronunciation assessment, zero-log data privacy export/purge, SBOM. | `tests/production_certification.rs` (6 tests) |
| **Stage 8: Enterprise Platform** | Multi-tenant RBAC, shared team glossaries, API key auth, token bucket rate limiter, worker pool. | `tests/enterprise_platform.rs` (6 tests) |

---

## 5. Empirical Performance & Microbenchmark Results

All 28 microbenchmarks are measured on the native host using monotonic timestamps (`std::time::Instant`):

| # | Subsystem / Operation | Budget Limit | Measured Latency | Safety Margin | Status |
| :-: | :--- | :--- | :--- | :--- | :-: |
| 1 | **Typed IPC Serialization Roundtrip** | < 5,000 µs | **23.42 µs** | **213x faster** | :white_check_mark: PASS |
| 2 | **SQLite User Settings Read Latency** | < 20,000 µs | **39.39 µs** | **507x faster** | :white_check_mark: PASS |
| 3 | **SQLite FTS5 Prefix Search (`ನಮ`)** | < 20,000 µs | **0.480 ms (480 µs)** | **41.6x faster** | :white_check_mark: PASS |
| 4 | **Offline Lexicon Translation Latency** | < 10,000 µs | **0.304 ms (304 µs)** | **32.8x faster** | :white_check_mark: PASS |
| 5 | **Offline Cache Hit Lookup (SHA-256 Hash)** | < 5,000 µs | **0.239 ms (239 µs)** | **20.9x faster** | :white_check_mark: PASS |
| 6 | **16 kHz Audio Ring Buffer Soak** | < 100 µs | **0.005 ms (5.0 µs)** | **20.0x faster** | :white_check_mark: PASS |
| 7 | **VAD Energy & ZCR Frame Processing (10ms)** | < 100 µs | **0.004 ms (4.0 µs)** | **25.0x faster** | :white_check_mark: PASS |
| 8 | **Streaming ASR Chunk Pipeline Processing** | < 1,000 µs | **0.029 ms (29.0 µs)** | **34.4x faster** | :white_check_mark: PASS |
| 9 | **Caption Translation & Formatting Pipeline** | < 5,000 µs | **0.103 ms (103 µs)** | **48.5x faster** | :white_check_mark: PASS |
| 10 | **Multi-Formant Speech Synthesis (TTS)** | < 50,000 µs | **4.752 ms (4,752 µs)** | **10.5x faster** | :white_check_mark: PASS |
| 11 | **Conversation Turn Turnaround (ASR+Trans+TTS)** | < 500,000 µs | **358.42 ms** | **1.39x faster** | :white_check_mark: PASS |
| 12 | **Acoustic Echo Ducking Switch Latency** | < 100 µs | **0.00005 ms (0.05 µs)** | **2,000x faster** | :white_check_mark: PASS |
| 13 | **Romanized Indic Transliteration (`kn-IN`/`ta-IN`)** | < 500 µs | **0.003 ms (3.0 µs)** | **166x faster** | :white_check_mark: PASS |
| 14 | **Bounded LRU Cache Read/Write Overhead** | < 50 µs | **0.006 ms (6.0 µs)** | **8.3x faster** | :white_check_mark: PASS |
| 15 | **chrF Metric Computation (Character 6-grams)** | < 2,000 µs | **0.204 ms (204 µs)** | **9.8x faster** | :white_check_mark: PASS |
| 16 | **Translation Intelligence Pipeline Turnaround** | < 10,000 µs | **2.990 ms (2,990 µs)** | **3.34x faster** | :white_check_mark: PASS |
| 17 | **Media Chunk Audio Extraction** | < 5,000 µs | **0.219 ms (219 µs)** | **22.8x faster** | :white_check_mark: PASS |
| 18 | **Subtitle Formatting SRT/VTT** | < 1,000 µs | **0.038 ms (38 µs)** | **26.3x faster** | :white_check_mark: PASS |
| 19 | **OCR Binarization & Extraction** | < 50,000 µs | **0.523 ms (523 µs)** | **95.6x faster** | :white_check_mark: PASS |
| 20 | **Document Block Parsing (Markdown)** | < 10,000 µs | **0.005 ms (5.0 µs)** | **2,000x faster** | :white_check_mark: PASS |
| 21 | **SM-2 Interval Calculation** | < 10.0 µs | **0.330 µs** | **30.3x faster** | :white_check_mark: PASS |
| 22 | **Pronunciation Scoring (chrF + CER)** | < 1,000 µs | **0.343 ms (343 µs)** | **2.91x faster** | :white_check_mark: PASS |
| 23 | **Full Data Export & SHA-256 Hash** | < 20,000 µs | **0.124 ms (124 µs)** | **161x faster** | :white_check_mark: PASS |
| 24 | **Zero-Log Privacy Scrubbing** | < 100 µs | **0.006 ms (6.0 µs)** | **16.6x faster** | :white_check_mark: PASS |
| 25 | **Quota Check & Reservation** | < 50 µs | **0.0282 ms (28.2 µs)** | **1.77x faster** | :white_check_mark: PASS |
| 26 | **Multi-Tenant RBAC Evaluation** | < 10 µs | **0.00008 ms (0.08 µs)** | **125x faster** | :white_check_mark: PASS |
| 27 | **Shared Glossary Term Matching** | < 500 µs | **0.0016 ms (1.6 µs)** | **312x faster** | :white_check_mark: PASS |
| 28 | **API Key Auth & Rate Limit Check** | < 50 µs | **0.0002 ms (0.2 µs)** | **250x faster** | :white_check_mark: PASS |

---

## 6. Verification, Testing & Build Instructions

### Prerequisites
- **Rust Toolchain**: Stable 1.80+ (with `clippy` and `rustfmt`)
- **Node.js**: v20+ and `npm`
- **Linux Build Dependencies** (for Linux builds):
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
  ```

### Running the Full Test & Quality Suite
```powershell
# 1. Frontend Typecheck and Bundle Build
npm run check
npm run build

# 2. Strict Rust Formatting and Clippy Checks (Zero Warnings Allowed)
cargo fmt --check
cargo clippy --workspace -- -D warnings

# 3. Workspace Unit and Integration Test Suite (120 Tests)
cargo test --workspace --verbose

# 4. Run the 28 Microbenchmarks
cargo run -p lingua-cli --release -- bench --iterations 500
```

### Building and Packaging Desktop Binaries
```powershell
# Package standalone production binaries
npm run build
cargo build --release --workspace

# The compiled desktop executable is generated at:
# Windows: target/release/lingua-tauri.exe
# Linux:   target/release/lingua-tauri
# macOS:   target/release/lingua-tauri
```

---

## 7. Compliance, Security & Licensing

- **SECURITY.md**: Details vulnerability reporting, bounded memory constraints, cryptographic storage, and the zero-log privacy policy.
- **LICENSES.md & SBOM.json**: All 200+ dependencies are audited and compliant with permissive Apache-2.0, MIT, BSD-3, and dual-licensed models. Commercial copyleft restrictions are strictly avoided.
- **Append-Only History**: Both `TODO.md` and `CHANGELOG.md` are strictly append-only, preserving the exact audit trail across all stages of project development.
