# LinguaFlow — Stage 15: End-to-End Golden Certification, Honest Dual-Tier Benchmarking & 5-Platform Release

Execute the Global Engineering Contract (`.antigravity/contract.md`). Replace stubbed microbenchmarks with separate **Deterministic System Microbenchmarks** and **End-to-End Neural/Linguistic Quality Benchmarks**, and certify all platform deliverables across Windows, Linux, macOS, Android, and iOS.

---

## Phase 15.1: Curated 500-Pair Golden Regression Suite & Visual E2E Verification

### Subphase 15.1.1 — 500-Pair Trilingual Golden Suite (`tests/golden_linguistic_suite.rs`)
- **Minute Details**:
  - Build an automated integration suite verifying 500 curated clinical, logistics, conversational, and literary pairs across all 6 directional combinations:
    - English $\leftrightarrow$ Kannada (`en-US` $\leftrightarrow$ `kn-IN`)
    - English $\leftrightarrow$ Tamil (`en-US` $\leftrightarrow$ `ta-IN`)
    - Kannada $\leftrightarrow$ Tamil (`kn-IN` $\leftrightarrow$ `ta-IN`)
  - **Explicit Regression Gate**:
    - Assert that translating `"adventure"` (`en-US` $\rightarrow$ `kn-IN`) returns **ಸಾಹಸ** (and never `"ದೇವೇಗೌಡರು"`).
    - Assert that translating `"adventure"` (`en-US` $\rightarrow$ `ta-IN`) returns **சாகசம்**.
    - Assert that searching `"adventure"` in the SQLite FTS5 dictionary returns $\ge 1$ complete lexical entry with IPA, examples, and part-of-speech tags.

### Subphase 15.1.2 — Headless Playwright / WebDriver E2E Suite
- **Minute Details**:
  - Verify all 12 Svelte views render with zero console errors, non-truncated language selectors, working keyboard shortcuts, and zero vertical clipping on Indic *ottaksharas*.

---

## Phase 15.2: Honest Dual-Tier Benchmarking (`lingua bench` & `BENCHMARKS.md`)

### Subphase 15.2.1 — Infrastructure vs. Neural Execution Benchmark Split
- **Minute Details**:
  - Refactor `crates/lingua-cli` benchmark runner and `docs/BENCHMARKS.md` into two transparent tiers:
    - **Tier 1 — Deterministic Infrastructure & Algorithmic Overhead ($\mu$s)**: IPC serialization, SQLite WAL reads, FTS5 BM25 lookup, ring buffer soak, VAD frame energy/periodicity, RBAC & rate-limit checks, SM-2 math.
    - **Tier 2 — End-to-End Neural Model Execution (ms & Peak RSS MB)**: Quantized ONNX NMT inference, streaming ASR chunk decoding, VITS/Piper neural TTS synthesis, and ONNX Indic OCR extraction, along with idle-to-active RAM telemetry.

---

## Phase 15.3: 5-Platform Native Release Packaging (Windows, Linux, macOS, Android, iOS)

### Subphase 15.3.1 — Native Mobile Lifecycle & Permissions (`gen/android` & `gen/apple`)
- **Minute Details**:
  - Configure Tauri v2 mobile targets for Android (`RECORD_AUDIO`, `CAMERA`, `arm64-v8a` / universal APK & AAB) and iOS (`NSMicrophoneUsageDescription`, `NSCameraUsageDescription`, `AVAudioSession` interruption recovery, `.ipa` Xcode archive).
  - Add `cpal` audio stream pause/resume hooks when app backgrounding or phone calls occur.

### Subphase 15.3.2 — Multi-Platform Installer Matrix, CycloneDX SBOM & SHA-256 Manifests
- **Minute Details**:
  - Configure release builds and CI/CD pipelines for Windows (`.msi`, `.exe` NSIS), Linux (`.AppImage`, `.deb`), macOS (`.dmg` Universal binary), Android (`.apk`, `.aab`), iOS (`.ipa`), and Web PWA.
  - Generate CycloneDX SBOM (`sbom.cdx.json`) and signed `SHA256SUMS.txt` integrity manifest.

---

## Acceptance Criteria
1. Automated 500-pair golden linguistic suite passes with 100% assertions met.
2. Playwright/WebDriver headless tests verify zero visual regressions across all views.
3. `docs/BENCHMARKS.md` clearly bifurcates microsecond-scale pipeline overhead from millisecond-scale neural model execution times.
4. All CI matrix builds (Windows, Linux, macOS, Android, iOS, Web PWA) compile cleanly with matching SHA-256 manifests.
