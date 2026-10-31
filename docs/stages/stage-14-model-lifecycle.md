# LinguaFlow — Stage 14: Model Pack Lifecycle, Dynamic Learning Decks, Security Hardening & Free/Paid Tiers

Execute the Global Engineering Contract (`.antigravity/contract.md`). Ensure `ModelsView`, `LearnView`, and `EnterpriseView` operate on real persisted state and end-to-end cryptography rather than static UI mocks.

---

## Phase 14.1: Resumable Model Manager, Dual Cryptographic Verification & mmap Eviction (`ModelsView.svelte`)

### Subphase 14.1.1 — HTTP Range Resumable Chunked Downloader & Disk Accounting
- **Minute Details**:
  - Implement chunked `HTTP Range: bytes=start-end` downloads with pause, resume, cancellation, and `.part` file recovery for NMT, ASR, TTS, and OCR model packs in `crates/lingua-engine/src/model_manager.rs`.
  - Query real available disk space via `std::fs` / `sysinfo` before starting downloads.

### Subphase 14.1.2 — Dual SHA-256 + Ed25519 Signature Check, Atomic Swap & 90s Idle mmap Unloader
- **Minute Details**:
  - Verify both the `SHA-256` digest and `Ed25519` signature before performing an atomic directory swap into `.active/`.
  - Wire `MmapModelPool` with a 90-second idle timer that automatically unloads inactive neural graphs from memory, displaying real-time RAM usage in `src/views/ModelsView.svelte`.

---

## Phase 14.2: Dictionary-Driven Spaced Repetition & Acoustic Pronunciation Practice (`LearnView.svelte`)

### Subphase 14.2.1 — Bookmark-to-SM2 Flashcard Pipeline & Persona Seed Decks
- **Minute Details**:
  - Wire dictionary favorite bookmarks (`☆/★`) in `src/views/DictionaryView.svelte` directly into SM-2 flashcard deck generation in `crates/lingua-storage` and `src/views/LearnView.svelte`.
  - Pre-seed the 3 persona decks via SQLite migration/seed:
    - **Healthcare & Medical Triage** (Dr. Ananya).
    - **Interstate Logistics & Freight** (Murugan).
    - **Kannada & Tamil Literature & Grammar** (Vikram).

### Subphase 14.2.2 — Real Microphone Pronunciation Assessment (`cpal` + ASR + CER/chrF)
- **Minute Details**:
  - Connect `src/views/LearnView.svelte` pronunciation practice to live `cpal` microphone capture $\rightarrow$ resampler $\rightarrow$ Silero VAD $\rightarrow$ ASR transcription.
  - Compute real CER (Character Error Rate) and chrF acoustic match feedback against the target Indic word/sentence.

---

## Phase 14.3: Enterprise Gateway Hardening, SQLCipher Privacy & Free/Pro/Enterprise Tiers

### Subphase 14.3.1 — Axum Tower Middleware, Hashed `lf_live_` Keys & Tamper-Evident Audit Logs
- **Minute Details**:
  - Wire `rbac.rs` and `api_gateway.rs` into Axum Tower middleware in `crates/lingua-server` covering all `/api/v1/*` routes.
  - Validate `Authorization: Bearer lf_live_...` using constant-time BLAKE3/Argon2id hash lookup against SQLite, decrement real-time token bucket quotas, and record SHA-256 hash-chained tamper-evident audit logs.

### Subphase 14.3.2 — `zeroize` Memory Scrubbing, OS Keystore Encryption & Offline Entitlement Tiers
- **Minute Details**:
  - Wrap sensitive Zero-Log buffers in `zeroize::Zeroizing`.
  - Implement `EntitlementState` (`Free`, `Pro`, `Enterprise`) verified offline via `Ed25519` signed license tokens in `crates/lingua-core`, exposing tier badges and feature gates in `src/views/EnterpriseView.svelte` and `src/views/SettingsView.svelte`.

---

## Acceptance Criteria
1. `ModelsView.svelte` downloads model packs with resumable HTTP range requests and validates SHA-256 checksums and Ed25519 signatures before enabling neural inference.
2. Saving words in `TranslateView` or `DictionaryView` immediately creates reviewable cards in `LearnView.svelte`.
3. Pronunciation assessment evaluates spoken audio against target texts via real ASR phonemes and CER/chrF metrics.
4. `lingua-server` enforces RBAC policies, API key bearer authentication, and token-bucket quotas with tamper-evident audit logs.
5. Offline entitlement verification (`Free`, `Pro`, `Enterprise`) operates securely with zero network requirements.
