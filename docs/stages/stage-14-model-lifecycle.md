# LinguaFlow — Stage 14: Model Pack Lifecycle, Dynamic Learning Decks & Enterprise API Hardening

Execute the Global Engineering Contract. Ensure `ModelsView`, `LearnView`, and `EnterpriseView` operate on real persisted state and end-to-end cryptography rather than static UI mocks.

---

## Phase 14.1 — Authentic Resumable Model Manager (`ModelsView.svelte`)

### Subphase 14.1.1 — Real HTTP Range Downloads, SHA-256 Verification & Disk Accounting
- Connect `ModelsView.svelte` to real downloadable artifact manifests (hosted on HuggingFace, GitHub Releases, or local mirrors) for:
  - `lingua-lexicon-kn-ta-en-v1.sqlite.zst` (Bundled / Core Offline Dictionary).
  - `indictrans2-en-kn-ta-int8-onnx` (Offline Neural Translation Pack).
  - `sherpa-onnx-asr-kn-ta-en-int8` (Offline Streaming Speech Recognition Pack).
  - `vits-mms-tts-kn-ta-en-int8` (Offline Neural Voice Synthesis Pack).
  - `tesseract-ocr-kan-tam-eng` (Offline Unicode OCR Pack).
- Support HTTP Range resumable chunked downloads with real-time progress events over Tauri IPC.
- Perform cryptographic SHA-256 verification before model activation, atomic directory swap upon successful validation, and real `std::fs` disk usage reporting.

---

## Phase 14.2 — Dictionary-Driven Spaced Repetition & Pronunciation Scoring (`LearnView.svelte`)

### Subphase 14.2.1 — Auto-Generate Flashcard Decks from Saved Favorites & Lexicon Domains
- Wire the Favorite (☆/★) button in `TranslateView.svelte` and `DictionaryView.svelte` directly into `lingua-storage`'s SM-2 flashcard deck tables (`008_learning_and_privacy.sql`).
- Pre-seed curated domain decks tailored for target personas:
  - **Rural Healthcare & Medical Triage** (Dr. Ananya).
  - **Interstate Logistics & Freight** (Murugan).
  - **Kannada & Tamil Literature & Grammar** (Vikram).

### Subphase 14.2.2 — Real Microphone Pronunciation Practice
- Connect `LearnView.svelte`'s pronunciation drill to the Stage 11 `cpal` microphone capture + ASR engine.
- Compare the learner's spoken Kannada/Tamil phoneme sequence against the target phrase using real CER/chrF scoring and provide visual acoustic feedback.

---

## Phase 14.3 — Enterprise Gateway & Privacy Compliance (`EnterpriseView.svelte` & `lingua-server`)

### Subphase 14.3.1 — End-to-End Axum Middleware Enforcement
- Wire `rbac.rs` and `api_gateway.rs` into actual Axum tower middleware layers in `crates/lingua-server`.
- Ensure all `/api/v1/*` requests validate `Authorization: Bearer lf_live_...` headers against SQLite `009_enterprise_platform` tables.
- Decrement token-bucket quotas in real time and write immutable audit rows visible in `EnterpriseView.svelte`.

---

## Acceptance Criteria
1. `ModelsView.svelte` downloads model packs with resumable HTTP range requests and validates SHA-256 checksums before enabling neural inference.
2. Saving words in `TranslateView` or `DictionaryView` immediately creates reviewable cards in `LearnView.svelte`.
3. Pronunciation assessment evaluates spoken audio against target texts via real ASR phonemes and CER/chrF metrics.
4. `lingua-server` enforces RBAC policies, API key bearer authentication, and token-bucket quotas with tamper-evident audit logs.

