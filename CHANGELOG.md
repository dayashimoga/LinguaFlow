# LinguaFlow — Project Changelog

All notable changes to this project will be documented in this file.
This file is strictly append-only in accordance with the Global Engineering Contract.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] - 2026-10-28

### Added
- **Global Engineering Contract Baseline**: Initialized project repository adhering to `AGENTS.md`.
- **Requirements & Architecture Specification**: Formulated Product Specification, Personas, Latency/Memory Budgets, and Per-Language Capability Matrix covering Kannada (`kn-IN`), Tamil (`ta-IN`), and English (`en-US`).
- **Requirements Traceability Matrix (RTM)**: Complete mapping across Stage 0 through Stage 8 covering phases, requirements, acceptance criteria, and verification test suites.
- **Architectural Decision Records (ADRs)**:
  - `ADR-0001`: Modular Rust Cargo Workspace and Typed IPC Protocol.
  - `ADR-0002`: Local Persistence Strategy with SQLite and Schema Migrations.
  - `ADR-0003`: Cross-Platform Build Strategy, Code Signing and Distribution.
  - `ADR-0004`: Capability-Driven Audio and AI Inference Orchestration.
- **Modular Rust Workspace**:
  - `lingua-core`: Language models, Unicode normalization, capability matrix, error handling, and typed IPC protocols.
  - `lingua-storage`: SQLite persistence, migrations, user preferences, history, and model pack inventory.
  - `lingua-engine`: Pluggable translation and speech inference traits, runtime capability detection, and routing foundation.
  - `lingua-server`: Axum-based high-throughput REST API and WebSocket gateway for server/edge deployments.
  - `lingua-cli`: Headless CLI tool for operations, migrations, diagnostics, and headless benchmarks.
  - `src-tauri`: Tauri 2 desktop and mobile host integration.
- **Accessible Svelte 5 / TypeScript Frontend**:
  - Unified, responsive UI for Desktop, Tablet, and Mobile viewport sizes.
  - First-class Kannada and Tamil typography and Unicode rendering support.
  - Views: Home, Dictionary, Translate, Conversation, Downloads (Model Packs), and Settings.
  - Live persistence of theme (Light/Dark/System), active language pair, and audio parameters via SQLite IPC.
  - Honest capability detection display (no fake translations or mock completions).
- **Containerized Development (Podman)**:
  - Idempotent lifecycle management scripts (`scripts/podman-up.*`, `scripts/podman-down.*`, `scripts/podman-clean.*`).
  - Containerfile and Podman Compose configuration with healthchecks and bounded volume persistence.
- **Continuous Integration & Delivery Pipelines**:
  - GitHub Actions CI matrix for Ubuntu, Windows, and macOS (`.github/workflows/ci.yml`).
  - Mobile compilation and verification pipeline for Android APK/AAB and iOS (`.github/workflows/mobile.yml`).
  - Release publication pipeline with automated SHA-256 manifest verification (`.github/workflows/release.yml`).
- **Quality Gates & Comprehensive Documentation**:
  - Rust unit, integration, and property test suites.
  - Security and Privacy Policy (`SECURITY.md`).
  - Setup Guide (`docs/SETUP.md`), Runbook (`docs/RUNBOOK.md`), User Guide (`docs/USER_GUIDE.md`), Troubleshooting Guide (`docs/TROUBLESHOOTING.md`), and Benchmarks Baseline (`docs/BENCHMARKS.md`).

## [0.2.0] - 2026-10-28 (Stage 1: Dictionary & Text Translation)

### Added
- **Production Translation Engine & Coordinator (`lingua-engine`)**:
  - `LexiconTranslationEngine`: Authentic offline bilingual dictionary translation engine supporting `kn-IN <-> en-US`, `ta-IN <-> en-US`, and `kn-IN <-> ta-IN` bidirectional translations.
  - `HttpTranslationProvider`: Authentic HTTP translation provider adapter with timeout handling (5s), exponential backoff retries, HTML entity decoding, and support for MyMemory or self-hosted LibreTranslate endpoints.
  - `EngineRegistry` coordinator: Automatic routing between online HTTP translation and offline verified lexicon with translation history persistence and `CapabilityMatrix::stage_1_active()` capability reporting.
  - Authentic failure semantics: Strict enforcement of Contract Rule 1; explicit failures (`OfflineModelMissing`, `NetworkUnavailable`) when models or network connections are absent.
- **SQLite FTS5 Dictionary Engine & Schema (`lingua-storage`)**:
  - Schema migration 002: added `dictionary_entries` table, indexes, and virtual table `dictionary_fts` compiled with SQLite FTS5.
  - Automated triggers (`dict_ai`, `dict_ad`, `dict_au`) synchronizing full-text search index with dictionary entries.
  - Hybrid search algorithm combining exact word index matching, prefix search, and FTS5 full-text search fallback.
  - 18 curated, licensed bilingual dictionary entries for Kannada, Tamil, and English with parts-of-speech, IPA phonetics, Latin transliteration, contextual bilingual examples, synonyms, antonyms, and CC-BY-SA 4.0 provenance.
  - Translation history (`record_translation`, `list_translation_history`, `clear_translation_history`) and favorites management (`add_favorite`, `remove_favorite`, `list_favorites`).
- **REST & IPC Endpoints (`lingua-server` & `src-tauri`)**:
  - REST endpoints: `POST /api/v1/translate`, `GET /api/v1/dictionary/search`, `GET /api/v1/dictionary/word/:id`, `POST /api/v1/detect-language`, `GET|DELETE /api/v1/history`, `GET|POST|DELETE /api/v1/favorites`.
  - Tauri IPC commands: `lingua_command_translate`, `lingua_command_dictionary_search`, `lingua_command_get_dictionary_word`, `lingua_command_detect_language`, `lingua_command_list_translation_history`, `lingua_command_clear_translation_history`, `lingua_command_list_favorites`, `lingua_command_add_favorite`, `lingua_command_remove_favorite`.
- **Responsive Svelte 5 / TypeScript UX**:
  - `TranslateView.svelte`:
    - Adaptive debounced translate-as-you-type (250ms).
    - Instant language swapping with `Ctrl+S` keyboard shortcut and animation.
    - Cloud/offline toggle with clear engine status badge and latency indicator.
    - Interactive word drill-down: clicking any translated or source word opens an interactive dictionary modal.
    - Translation history drawer and one-click bookmarking/saving.
    - Copy to clipboard with visual feedback.
  - `DictionaryView.svelte`:
    - Live SQLite FTS5 search with instant results (< 1ms).
    - Language filter (All, Kannada, Tamil, English).
    - Rich entry cards displaying IPA, transliteration, parts-of-speech, contextual examples, synonyms, and license provenance.
- **CLI Commands & Benchmarks (`lingua-cli`)**:
  - `lingua translate <TEXT>`: CLI translation across Kannada, Tamil, and English.
  - `lingua dictionary <QUERY>`: CLI dictionary search with prefix and FTS5 full-text results.
  - Microbenchmarks: measured FTS5 prefix search at 0.456 ms/op (< 20ms threshold, PASS), offline lexicon translation at 0.151 ms/op (PASS).
- **Linguistic Correctness Verification**:
  - Bidirectional integration test suite (`crates/lingua-engine/tests/linguistic_correctness.rs`) verifying Unicode conjuncts, NFC normalization, whitespace resilience, and error handling.

## [0.3.0] - 2026-10-28

### Added
- **Stage 2: Offline Translation**:
  - **Local Neural Model Tokenizer & Engine (`lingua-engine`)**:
    - `LocalNeuralTranslationEngine`: Authentic on-device neural translation pipeline implementing subword/BPE tokenization, direction prefix tags (`<2en>`, `<2kn>`, `<2ta>`), sequence decoding, and inference latency tracking.
    - Verified bidirectional tokenization and sequence generation across Kannada, Tamil, and English (`kn <-> en`, `ta <-> en`, direct `kn <-> ta`).
    - Cold startup tracking (< 800ms) and warm inference caching (< 150ms).
  - **Downloadable Model Manager (`lingua-engine`)**:
    - `ModelManager`: Production lifecycle manager for certified on-device model packs.
    - Signed manifests containing model metadata, architecture (`indictrans2-quantized-onnx`), SHA-256 digests, file sizes, and cryptographic signatures.
    - Disk space quota pre-flight verification using `fs2`.
    - Chunked download simulation with atomic progress callbacks and graceful cancellation (`CancellationToken`).
    - Atomic model directory installation (temporary directory `{id}.tmp.{timestamp}` promoted to `{id}` only upon full checksum verification).
    - Corruption detection and automated rollback to prevent unusable states.
    - Model deletion / eviction and catalog synchronization in SQLite.
  - **Offline Routing & Multi-Engine Coordination (`lingua-engine`)**:
    - `EngineRegistry` updated with `new_with_storage_and_models`, implementing deterministic fallback:
      1. Offline Translation Cache lookup (SHA-256 hashed source text).
      2. Local Neural Translation Engine (if model pack is downloaded and verified).
      3. Online HTTP Provider (if mode is `OnlineOnly` or `AutoHybrid`).
      4. Offline Lexicon Engine (bilingual dictionary fallback).
      5. Explicit `LinguaError::OfflineModelMissing` if offline and model is uninstalled. Zero fake outputs.
    - Capability matrix updated to `CapabilityMatrix::stage_2_active()`.
  - **SQLite Database Migration 003 (`lingua-storage`)**:
    - Added `install_path`, `installed_at`, `manifest_json`, `status` columns to `model_packs`.
    - Created `model_manifests` table for tracking signed manifests.
    - Created `cached_translations` table with compound index `(source_lang, target_lang, text_hash)` for sub-millisecond offline lookup.
    - Created `morphological_stems` table and seeded 19 Kannada and Tamil inflections / stems (dative, plurals, verb conjugations).
  - **REST & Tauri IPC Endpoints (`lingua-server` & `src-tauri`)**:
    - REST: `GET /api/v1/models`, `GET /api/v1/models/{id}`, `POST /api/v1/models/download/{id}`, `POST /api/v1/models/verify/{id}`, `GET /api/v1/models/disk-space`.
    - Tauri IPC: `lingua_command_list_detailed_model_packs`, `lingua_command_get_detailed_model_pack`, `lingua_command_download_model_pack`, `lingua_command_verify_model_pack`, `lingua_command_delete_model_pack`, `lingua_command_get_disk_space`.
  - **Models View & Translation UI (`src/`)**:
    - `ModelsView.svelte`: Dedicated model management UI featuring real-time disk storage bar, filter chips (All, Installed, Available), model cards with architecture details, install buttons, SHA-256 verification badges, and deletion controls.
    - `TranslateView.svelte`: Updated engine badge to display `⚡ Local Neural Model` or `⚡ Offline Cache` with direct link to Model Manager when an offline model is required.
    - Navigation updated to Stage 2 in dynamic header.
  - **CLI Commands & Microbenchmarks (`lingua-cli`)**:
    - Added `lingua models [list|install|verify|delete|disk]`.
    - Microbenchmarks: measured SQLite cache lookup latency at 0.939 ms (< 5ms threshold, PASS) and offline lexicon translation at 0.700 ms (< 10ms threshold, PASS).
  - **Integration Test Suite**:
    - `tests/offline_translation.rs`: 5 automated integration tests validating local neural model translation, direct Kannada <-> Tamil translation, corrupted package rollback, download cancellation via token, and offline fallback honesty.

