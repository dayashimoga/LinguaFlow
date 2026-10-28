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

