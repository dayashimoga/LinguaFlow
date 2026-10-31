# GLOBAL ENGINEERING CONTRACT: LinguaFlow (Offline-First Cross-Platform Engine)

## 1. Core Architectural Invariants
1. **100% Offline-First Autonomy**:
   - Every core feature—Trilingual Dictionary (`kn-IN`, `ta-IN`, `en-US`), Agglutinative Morphology, Tier-1/Tier-2 Translation, Streaming ASR, VITS/Piper Neural TTS, Indic/English OCR, Structured Document Parsing, and SM-2 Flashcards—MUST work completely offline with `0 bytes` of internet traffic.
2. **Five-Platform Native Parity (Windows, Linux, macOS, Android, iOS)**:
   - Every crate (`lingua-core`, `lingua-storage`, `lingua-engine`, `lingua-server`, `lingua-cli`) MUST compile cleanly for:
     - `x86_64-pc-windows-msvc` / `aarch64-pc-windows-msvc`
     - `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`
     - `aarch64-apple-darwin` / `x86_64-apple-darwin`
     - `aarch64-linux-android`
     - `aarch64-apple-ios` / `aarch64-apple-ios-sim`
   - Forbidden dependencies: Do NOT use `leptess` (C++ Tesseract) or `openssl-sys`. Use pure-Rust crates (`symphonia`, `image`, `rustls`) and quantized ONNX models via `ort` / `tract-onnx`.
   - Platform-divergent features (Desktop borderless overlay window vs. Mobile PiP/floating HUD; Desktop hotkeys vs. Mobile gestures) MUST be gated via `#[cfg(desktop)]` and `#[cfg(mobile)]` behind unified Rust traits.
3. **Zero Hallucination & Honest Capability Gating**:
   - Never fabricate translations (e.g., `"adventure"` MUST translate to Kannada `"ಸಾಹಸ"` and Tamil `"சாகசம்"`, never `"ದೇವೇಗೌಡರು"`).
   - Never emit synthetic confidence scores (no fake `chrF: 100%`).
   - If a neural model pack is not downloaded, return `LinguaError::ModelPackMissing { pack_id, language_pair }` and explicitly label fallback Tier-1 Lexicon outputs in the UI.
4. **Strict Resource, Memory & Latency Budgets**:
   - **Idle RAM Footprint**: `< 30 MB` on Android/iOS; `< 45 MB` on Windows/Linux/macOS.
   - **Active Neural Inference RAM**: `< 380 MB` peak via `mmap` model weights and automatic `90-second` idle model eviction.
   - **Audio Pipeline**: Lock-free ring buffer `< 2 MB` (480k samples @ 16 kHz); automatic polyphase resampling from hardware `44.1/48 kHz` to `16 kHz`; zero heap allocations inside `cpal` real-time audio callbacks.
   - **Media Demuxing**: Sliding 30-second decode window strictly `< 3.2 MB` heap allocation.
5. **Code Quality & Unicode Safety Laws**:
   - **Rust**: Zero `.unwrap()`, `.expect()`, `panic!()`, or `todo!()` in production code. Use `Result<T, LinguaError>` everywhere.
   - **Unicode NFC**: Normalize all inputs to Unicode NFC at crate ingress. Never slice strings by byte index (`&s[a..b]`); iterate by char/grapheme cluster to preserve Kannada virama (`U+0CCD`) and Tamil pulli (`U+0BCD`) conjuncts.
   - **Frontend**: Strictly Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`). Zero legacy Svelte 4 syntax.
6. **Security & Privacy Invariants**:
   - Downloaded model packs require dual `SHA-256` checksum AND `Ed25519` signature verification prior to activation.
   - Zero-Log Privacy mode wraps sensitive audio/text buffers in `zeroize::Zeroizing<T>`.
   - Enterprise API keys (`lf_live_...`) are stored as Argon2id/BLAKE3 hashes and verified in constant time.
