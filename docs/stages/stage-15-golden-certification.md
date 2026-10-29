# LinguaFlow — Stage 15: End-to-End Golden Certification, Honest Benchmarking & Release Packaging

Execute the Global Engineering Contract. Replace stubbed microbenchmarks with separate **Deterministic System Microbenchmarks** and **End-to-End Neural/Linguistic Quality Benchmarks**, and certify all platform deliverables across all target platforms.

---

## Phase 15.1 — Golden Linguistic & Functional Regression Suite

### Subphase 15.1.1 — Curated 500-Pair Golden Test Suite (`tests/golden_linguistic_suite.rs`)
- Create an automated integration test suite verifying 500 curated words, medical sentences, logistics invoices, and conversational idioms across all 6 directional pairs:
  - English $\leftrightarrow$ Kannada (`en-US` $\leftrightarrow$ `kn-IN`)
  - English $\leftrightarrow$ Tamil (`en-US` $\leftrightarrow$ `ta-IN`)
  - Kannada $\leftrightarrow$ Tamil (`kn-IN` $\leftrightarrow$ `ta-IN`)
- **Explicit Regression Gate**:
  - Assert that translating `"adventure"` (`en-US` $\rightarrow$ `kn-IN`) returns **ಸಾಹಸ** (and never `"ದೇವೇಗೌಡರು"`).
  - Assert that translating `"adventure"` (`en-US` $\rightarrow$ `ta-IN`) returns **சாகசம்**.
  - Assert that searching `"adventure"` in the SQLite FTS5 dictionary returns $\ge 1$ complete lexical entry with IPA, examples, and part-of-speech tags.

### Subphase 15.1.2 — Automated Playwright / WebDriver Visual & E2E Verification
- Run headless UI tests across all views asserting:
  - Zero truncated dropdowns (verifying `English (English)` selector regression fix).
  - Zero contradictory engine badges (`OFFLINE LEXICON` and `http-rest-provider` must never appear simultaneously).
  - Zero `chrF: 100.0%` badges on unreferenced live translations.
  - Correct target phonetic transliteration (*sāhasa* instead of *ಡವೆನಟುರೆ*).

---

## Phase 15.2 — Honest Dual-Tier Benchmarking & Documentation Refresh

### Subphase 15.2.1 — Separate In-Memory Overhead from Full Model Inference in `BENCHMARKS.md`
- Update `lingua-cli bench` and `docs/BENCHMARKS.md` to report two distinct tables:
  1. **Infrastructure & Memory Pipeline Overhead ($\mu$s scale)**: Ring buffer operations, VAD pre-filtering, FTS5 index lookup, LRU cache hits, RBAC evaluation.
  2. **End-to-End Real Inference Latency (ms scale)**: Cold/warm SQLite 50k-entry FTS5 query, ONNX INT8 neural sentence translation (`en` $\leftrightarrow$ `kn`, `en` $\leftrightarrow$ `ta`), sherpa-onnx streaming ASR chunk inference, neural VITS TTS waveform synthesis, and Tesseract/ONNX OCR page extraction.

### Subphase 15.2.2 — Final Artifact Generation & Traceability Closure
- Run the full quality verification suite:
  ```powershell
  npm run check
  npm run build
  cargo fmt --check
  cargo clippy --workspace -- -D warnings
  cargo test --workspace
  ```
- Capture updated production screenshots verifying all visual fixes.
- Update `CHANGELOG.md`, `TODO.md`, `docs/SBOM.json`, `docs/BENCHMARKS.md`, and `docs/PROJECT_WALKTHROUGH.md`.

---

## Acceptance Criteria
1. Automated 500-pair golden linguistic suite passes with 100% assertions met.
2. Playwright/WebDriver headless tests verify zero visual regressions across all views.
3. `docs/BENCHMARKS.md` clearly bifurcates microsecond-scale pipeline overhead from millisecond-scale neural model execution times.
4. All CI matrix builds (Windows, Linux, macOS, Android, iOS, Web PWA) compile cleanly with matching SHA-256 manifests.

