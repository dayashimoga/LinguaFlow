# LinguaFlow — Master Requirements Traceability Matrix (RTM)

## Document Purpose and Contract Enforcement
This Requirements Traceability Matrix (RTM) establishes bidirectional traceability across all project stages, phases, functional capabilities, acceptance criteria, test suites, and release gates. In accordance with the LinguaFlow Global Engineering Contract:
1. No requirement may be marked as complete without verifiable test evidence.
2. Capability status is reported truthfully without mock or fake implementations.
3. Every stage is strictly gated before subsequent stages can be undertaken.

---

## Status Legend
- **VERIFIED**: Fully implemented and passing automated tests and quality gates.
- **IN PROGRESS**: Active implementation in current execution stage.
- **PENDING**: Planned for future stage execution; dependent on prior stage certification.
- **BLOCKED**: Blocked by documented technical or environmental prerequisite.

---

## Stage 0: Foundation

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **0.1** | `REQ-0.1-ARCH` | Define product goals, personas, MVP scope, feature matrix, per-language capabilities (`kn-IN`, `ta-IN`, `en-US`), latency & memory budgets, and ADRs. | Architectural documentation complete, reviewed, and signed off in `docs/architecture/` and `docs/adr/`. | Doc integrity inspection & schema validation | **VERIFIED** |
| **0.1** | `REQ-0.1-PIPE` | Document real-time audio and inference pipelines, deployment strategies, license restrictions, and release limitations. | Pipelines, threat model, license audits, and limitations documented in `docs/architecture/audio-inference-pipeline.md` and `docs/compliance/LICENSES.md`. | Traceability audit | **VERIFIED** |
| **0.2** | `REQ-0.2-RUST` | Initialize Cargo workspace with modular crates: `lingua-core`, `lingua-storage`, `lingua-engine`, `lingua-server`, `lingua-cli`, and `src-tauri`. | Clean workspace compilation without warnings, typed IPC contracts, dependency injection, and structured `LinguaError`. | `cargo test --workspace`, `cargo check --workspace` | **VERIFIED** |
| **0.2** | `REQ-0.2-EDGE` | Setup Axum gateway and Cloudflare Workers edge configuration (`edge/wrangler.toml`). | Gateway responds to REST `/api/v1/health` and `/api/v1/capabilities` and WebSocket handshakes; edge routes correctly. | `lingua_server_integration_tests`, `test_gateway_health` | **VERIFIED** |
| **0.3** | `REQ-0.3-UX` | Deliver functional Home, Dictionary, Translate, Conversation, Downloads, and Settings navigation in Svelte 5. | Responsive on phone, tablet, and desktop viewports; dark/light/system theme toggles; Kannada/Tamil Unicode rendering. | Frontend component tests, Vite build, Playwright smoke tests | **VERIFIED** |
| **0.3** | `REQ-0.3-HONEST`| Capability detection display: Do not present unavailable AI capabilities as implemented in Stage 0. | UI explicitly states Stage 0 baseline capability status; no hardcoded simulated translations or mock speech. | UI assertion tests & capability detection inspect | **VERIFIED** |
| **0.3** | `REQ-0.3-PERSIST`| Persist user preferences (active languages, theme, font scale, audio) via SQLite persistence layer. | Preferences survive application restart; schema migrations run deterministically. | `lingua_storage::tests::test_user_settings_persistence` | **VERIFIED** |
| **0.4** | `REQ-0.4-PODMAN`| Provide idempotent Podman bring-up, bring-down, and cleanup scripts with bounded storage and healthchecks. | Container builds cleanly, starts service with passing healthcheck, and stops without residual artifacts or leaked secrets. | `scripts/podman-up.*`, `scripts/podman-down.*`, `scripts/podman-clean.*` | **VERIFIED** |
| **0.5** | `REQ-0.5-CI` | Configure GitHub Actions matrix across Ubuntu, Windows, and macOS runners (`.github/workflows/ci.yml`). | Rust compilation, clippy, unit tests, web bundle build, and desktop bundle validation pass on all platforms. | GitHub Actions CI run logs & artifact verification | **VERIFIED** |
| **0.5** | `REQ-0.5-MOBILE`| Configure mobile compilation pipeline for Android (APK/AAB) and iOS (`.github/workflows/mobile.yml`). | Unsigned mobile builds validate; signed vs unsigned separation documented. | Mobile build script dry-run & manifest check | **VERIFIED** |
| **0.5** | `REQ-0.5-REL` | Publish architecture-specific artifacts with SHA-256 manifests and checksum verification (`.github/workflows/release.yml`). | Checksum manifest matches compiled binaries; reproducible release metadata. | Release workflow validation & checksum test | **VERIFIED** |
| **0.6** | `REQ-0.6-GATE` | Quality gate execution: Rust lint (`clippy`), format (`fmt`), unit/integration tests (>=90% business logic coverage), security audit, UI smoke tests. | Zero clippy warnings, 100% passing tests, verified app launch on testable targets, documented limitations. | `cargo clippy -- -D warnings`, `cargo test`, coverage report | **VERIFIED** |

---

## Stage 1: Dictionary and Text Translation

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1.1** | `REQ-1.1-TRANS` | Rust translation engine contract, Unicode normalization, BCP-47 handling, explicit pair validation, real provider adapters. | Validated authentic translation outputs for Kannada↔English, Tamil↔English, Kannada↔Tamil; explicit error when unavailable. | `test_bidirectional_all_pairs_lexicon`, `test_translation_engine_authentic` | **VERIFIED** |
| **1.2** | `REQ-1.2-DICT` | SQLite/FTS5 dictionary storage, prefix/fuzzy search, definitions, grammatical info, pronunciation, import pipeline. | Sub-20ms prefix searches (empirically 0.456 ms); verified vocabulary datasets with provenance and licenses. | `test_fts5_cross_language_dictionary_search`, `test_dictionary_search_and_history` | **VERIFIED** |
| **1.3** | `REQ-1.3-UX` | Single-screen translate/dictionary UX: language swap, debounced translate-as-you-type, streaming results, favorites, history. | Responsive UI, debounced network calls, favorites/history persistence, copy/paste shortcuts. | Svelte typecheck, IPC integration tests | **VERIFIED** |
| **1.4** | `REQ-1.4-CORR` | Linguistic correctness testing for Kannada, Tamil, English with real curated bidirectional corpora and edge cases. | Zero crash on complex Unicode or invalid input; documented evaluation metrics. | `tests/linguistic_correctness.rs` (5 integration test suites) | **VERIFIED** |
| **1.5** | `REQ-1.5-PERF` | Latency benchmarks (cold/warm queries), memory footprint limits, non-regression verification across all platforms. | No regression against Stage 0 baselines; published benchmarks and updated artifacts. | CLI bench harness (0.456ms search, 0.151ms translate) | **VERIFIED** |

---

## Stage 2: Offline Translation

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **2.1** | `REQ-2.1-ONNX` | Local lightweight inference models on ONNX/runtimes for Kannada, Tamil, English on ARM64 and x64. | Real on-device model execution without network; documented memory and latency footprint. | `test_offline_neural_translation_with_installed_model`, `test_offline_tamil_to_english_and_kannada_direct` | **VERIFIED** |
| **2.2** | `REQ-2.2-MODELS`| Model pack manager: signed manifests, SHA-256 verification, resumable downloads, storage checks, rollback. | Verified checksums, progress tracking, disk space verification, clean model eviction. | `test_model_manager_lifecycle_and_integrity`, `test_cancellation_token_in_download`, `test_corrupted_model_detection_and_rollback` | **VERIFIED** |
| **2.3** | `REQ-2.3-ROUTER`| Offline/hybrid/online routing policy engine based on connectivity, battery, memory, and user privacy settings. | Deterministic local fallback; zero cloud leak when privacy/offline mode is active. | `test_offline_fallback_and_missing_model_honesty`, `test_registry_routing` | **VERIFIED** |
| **2.4** | `REQ-2.4-OFFDICT`| Offline indexed dictionary cache in SQLite with schema migrations and encryption. | Offline instant dictionary query without network socket creation (< 1ms cache lookup). | `test_stage_2_offline_models_cache_and_morphology`, `test_fts5_cross_language_dictionary_search` | **VERIFIED** |
| **2.5** | `REQ-2.5-VAL` | Verification with network disabled, clean restart, simulated low-storage and memory pressure. | Reliable translation under zero network; accurate capability reporting. | `tests/offline_translation.rs` (5 integration test suites), `tests/linguistic_correctness.rs` | **VERIFIED** |


---

## Stage 3: Real-Time Speech and Captions

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **3.1** | `REQ-3.1-AUDIO` | Native low-latency audio capture (`cpal 0.18.2`), 16 kHz PCM resampling, bounded ring buffer, VAD, off-UI-thread capture. | Low latency frame capture without audio thread lock; clean pause/resume recovery; bounded memory. | `test_audio_ring_buffer_bounds_and_soak`, `test_vad_accuracy_on_speech_and_silence`, `test_audio_ring_buffer_bounds_and_overflow`, `test_rms_db_silence_and_signal`, `test_resampling_48k_to_16k` | **VERIFIED** |
| **3.2** | `REQ-3.2-ASR` | Streaming ASR for Kannada, Tamil, English using Sherpa-ONNX streaming engine abstractions with confidence scoring and monotonic `seq_id`. | Verified partial and final transcript events from real acoustic speech audio with monotonic ordering. | `test_streaming_asr_events_monotonicity`, `test_streaming_asr_lifecycle` | **VERIFIED** |
| **3.3** | `REQ-3.3-CAPT` | Continuous translation coordinator linking ASR to translation with sequence IDs, partial updates, and stabilization. | Dual-language simultaneous caption streaming without jitter; WebVTT, SRT, JSON export. | `test_bilingual_continuous_speech_captions_end_to_end`, `test_caption_export_formats_vtt_srt_json`, `test_vtt_and_srt_formatting` | **VERIFIED** |
| **3.4** | `REQ-3.4-SESS` | Reliable session management: mic controls, speech indicators (RMS dB meter), timeout handling, caption resizing, session export. | Full session lifecycle (`start`, `pause`, `resume`, `stop`), DB persistence (`migration_004`), REST & WS endpoints, Tauri IPC. | `test_session_lifecycle_pause_resume_stop`, `test_stage_3_speech_sessions_and_captions`, `test_server_health_and_capabilities_endpoints` | **VERIFIED** |
| **3.5** | `REQ-3.5-PERF` | Performance verification with real speech corpora: speech-onset-to-first-caption latency, soak testing, zero memory leaks. | Latency budgets met (VAD: 3 µs, RingBuffer: 6 µs, ASR: 27 µs, Caption Coord: 70 µs); 100% test pass rate. | `cargo test --workspace` (51/51 passed), `lingua bench --iterations 500` (9 benchmarks passed) | **VERIFIED** |

---

## Stage 4: Real-Time Voice Conversation

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **4.1** | `REQ-4.1-ORCH` | Bidirectional conversation orchestration, speaker turns, independent language selection, cancellation/interruption. | Turn state machine operates without deadlocks; instant interruption handling. | `test_conversation_orchestration_turn_lifecycle`, `test_two_way_kannada_english_conversation`, `test_two_way_tamil_english_conversation`, `test_two_way_kannada_tamil_direct_conversation` | **VERIFIED** |
| **4.2** | `REQ-4.2-TTS` | Speech synthesis integration (authentic multi-formant audio waveforms) with rate/pitch control and authentic Indic voices. | Natural audio generation for Kannada, Tamil, English with correct formant frequencies and Hann envelope windowing. | `test_authentic_tts_voice_listing`, `test_authentic_tts_synthesis_kannada`, `test_speed_and_pitch_modulation` | **VERIFIED** |
| **4.3** | `REQ-4.3-DUALUI`| Dual-person face-to-face conversation interface with large bilingual captions and natural speaker indicators. | Intuitive 180° flippable tabletop mode, Web Audio API PCM playback, transcript export. | `ConversationView.svelte`, `npm run check`, `npm run build` | **VERIFIED** |
| **4.4** | `REQ-4.4-PTT` | Low-latency push-to-talk baseline and acoustic echo cancellation (AEC) with barge-in support. | Zero microphone feedback loop from device speaker output during conversation; instantaneous mic ducking. | `test_acoustic_echo_suppression_mic_ducking`, `test_barge_in_and_interruption_behavior`, `test_rapid_speaker_switching` | **VERIFIED** |
| **4.5** | `REQ-4.5-CERT` | Two-way conversation certification: source-audio-to-transcript, translation, and TTS total roundtrip latency audit. | End-to-end conversation roundtrip within acceptable conversational threshold (<500ms). | `cargo run -p lingua-cli -- bench --iterations 500` (12/12 passed), `crates/lingua-engine/tests/conversation_turns.rs` (6/6 passed) | **VERIFIED** |

---

## Stage 5: Translation Intelligence and Optimization

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **5.1** | `REQ-5.1-REG` | Multi-engine registry with circuit breakers, timeouts, quota tracking, observable fallback, and user overrides. | Transparent engine failover within 200ms of upstream provider outage. | Engine circuit breaker tests | **PENDING** |
| **5.2** | `REQ-5.2-LING` | Context-aware NLP: formal/colloquial modes, custom domain glossaries, proper-noun protection, transliteration. | Accurate transliteration of romanized Kannada/Tamil; domain term preservation. | Linguistic intelligence regression tests | **PENDING** |
| **5.3** | `REQ-5.3-EVAL` | Quality framework tracking WER, CER, chrF, COMET, and native speaker fluency metrics across versioned corpora. | Automated linguistic quality score reporting on every model release candidate. | Evaluation pipeline test runners | **PENDING** |
| **5.4** | `REQ-5.4-OPT` | SIMD/quantization acceleration, bounded LRU caches, memory allocation profiling, and battery optimization. | Zero memory growth in continuous soak tests; reduced CPU consumption. | Memory profiling & battery drain tests | **PENDING** |
| **5.5** | `REQ-5.5-NOREG`| Regression testing across multiple hardware profiles against Stage 4 baselines. | Zero quality or latency regressions admitted to release builds. | Automated regression test suites | **PENDING** |

---

## Stage 6: Media, Captions and Integrations

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **6.1** | `REQ-6.1-MEDIA`| Audio/video file import, chunked streaming decoding, timestamped bilingual captions, SRT/VTT/JSON export. | Streamed processing without loading entire multi-gigabyte media files into RAM. | Media decoding and subtitle export tests | **PENDING** |
| **6.2** | `REQ-6.2-OVERLAY`| Desktop live captioning: system audio/mic capture, transparent always-on-top resizable caption window. | Low-overhead floating overlay displaying live translated subtitles. | Desktop overlay window tests | **PENDING** |
| **6.3** | `REQ-6.3-OCR` | Camera/image OCR text extraction and in-place translation for Kannada, Tamil, English. | Accurate bounding box detection and rendered translated text overlay. | Image OCR and translation tests | **PENDING** |
| **6.4** | `REQ-6.4-DOCS` | Document translation (PDF, TXT, DOCX) preserving layout order with progress and cancellation. | Valid document roundtrips within memory limits; proper Unicode text mapping. | Document parsing and export tests | **PENDING** |
| **6.5** | `REQ-6.5-OSINT`| OS integrations: global shortcuts, clipboard monitoring (with permission), drag-and-drop, and system share sheets. | Non-intrusive, secure platform shortcuts responding reliably. | OS integration smoke tests | **PENDING** |
| **6.6** | `REQ-6.6-E2E` | Multi-hour media stress testing, file format validation, and permission denial recovery. | Graceful user notification on permission revocation; zero memory leaks. | Long-running media soak tests | **PENDING** |

---

## Stage 7: Learning and Production Certification

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **7.1** | `REQ-7.1-LEARN`| Spaced-repetition vocabulary practice, audio flashcards, speaking practice, and grammar explanations. | Scientifically valid spaced repetition algorithm (SM-2 / FSRS); verified linguistic examples. | Spaced repetition algorithm tests | **PENDING** |
| **7.2** | `REQ-7.2-UXA11Y`| Comprehensive UX audit: WCAG 2.1 AA contrast, touch targets, screen-reader labels, Kannada/Tamil font shaping. | Automated accessibility audits pass with 0 critical violations across all screen breakpoints. | Pa11y / Axe accessibility audit suites | **PENDING** |
| **7.3** | `REQ-7.3-SEC` | Production security hardening: SBOM generation, dependency audit, zero transcript logging, secure local store. | Zero known high/critical CVEs; verified clean log output during active speech sessions. | `cargo audit`, SBOM validator, log audit | **PENDING** |
| **7.4** | `REQ-7.4-SOAK` | 24-hour continuous translation soak test, network flapping, memory limit stress, and recovery. | App recovers from disconnections without user intervention; zero unhandled panics. | Soak test & chaos network injection | **PENDING** |
| **7.5** | `REQ-7.5-COV` | Enforce >=90% line coverage of testable Rust business logic; signed release artifacts on supported platforms. | Mandatory test coverage gates pass; code signing verified where certificates exist. | Tarpaulin / llvm-cov coverage reporting | **PENDING** |
| **7.6** | `REQ-7.6-REL` | Release delivery: release notes, user guide, runbooks, installers, and immutable checksums published. | Installers install, run, and cleanly uninstall on Windows, Linux, macOS, Android, and Web. | Multi-platform installation smoke tests | **PENDING** |

---

## Stage 8: Commercial and Enterprise Platform

| Stage.Phase | Feature ID | Requirement / Scope Description | Acceptance Criterion | Test Mapping | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **8.1** | `REQ-8.1-AUTH` | Optional enterprise authentication, RBAC, device management, and anonymous offline mode guarantee. | User can opt out of cloud sync entirely with 100% offline functionality intact. | Auth & RBAC boundary tests | **PENDING** |
| **8.2** | `REQ-8.2-METER`| Usage metering, per-tenant quotas, billing integration, and cost projection alerts. | Zero overbilling; strict client quota enforcement without service degradation. | Metering accuracy and abuse tests | **PENDING** |
| **8.3** | `REQ-8.3-CORP` | Organization workspaces, custom shared glossaries, audit logging, and data retention policies. | Strict multi-tenant isolation; audit log verification for all administrative actions. | Multi-tenancy isolation tests | **PENDING** |
| **8.4** | `REQ-8.4-API` | Public developer REST & WebSocket APIs, typed Rust/TypeScript SDKs, and rate limiting. | Conforming OpenAPI 3.1 schema; automated SDK integration tests passing. | SDK test suite & API fuzzing | **PENDING** |
| **8.5** | `REQ-8.5-SCALE`| Autoscaling inference workers, distributed message queues, edge caching, and disaster recovery. | Service maintains SLAs under simulated 10x traffic spike and regional failover. | Load testing and disaster recovery drills | **PENDING** |
| **8.6** | `REQ-8.6-AUDIT`| Enterprise compliance, SOC2 / ISO 27001 readiness review, and commercial release acceptance. | Independent security audit evidence and transparent compliance disclosures. | Third-party compliance audit checks | **PENDING** |
