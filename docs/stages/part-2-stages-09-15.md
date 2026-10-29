# LinguaFlow — Part 2: Next-Generation Remediation & Enhancement Stages (Stages 9 – 15)

Part 2 establishes an end-to-end engineering roadmap structured as **Stages $\rightarrow$ Phases $\rightarrow$ Subphases** to systematically eliminate every stub, resolve visual and linguistic defects, and elevate LinguaFlow to genuine enterprise production readiness.

---

## Stage Index & Navigation

| Stage | Document | Title | Scope Summary |
| :---: | :--- | :--- | :--- |
| **Stage 9** | [**`stage-09-linguistic-integrity.md`**](stage-09-linguistic-integrity.md) | **Linguistic Data Integrity, Comprehensive Lexicon Seeding & Dictionary UX Overhaul** | Sourcing open-licensed corpora (Alar, Wiktextract, Madras Tamil Lexicon), Rust ETL compiler (`dict-import`), rich multi-sense schema migration 010, agglutinative Indic morphology, and instant debounced search UI. |
| **Stage 10** | [**`stage-10-authentic-translation.md`**](stage-10-authentic-translation.md) | **Authentic Translation Engine, Transliteration Fix & Telemetry Honesty** | Purging mock provider hallucinations, 3-tier deterministic offline translation, directional transliteration IME fix, Indic abugida grapheme engine, and honest telemetry badges (removing fake chrF 100%). |
| **Stage 11** | [**`stage-11-neural-speech.md`**](stage-11-neural-speech.md) | **Real Audio Capture, Neural Speech Pipeline (ASR & TTS) & Conversation Engine** | `cpal` non-blocking audio capture, Silero VAD v5 ONNX, real offline streaming ASR (`sherpa-onnx`/`whisper`), neural VITS/Piper/MMS-TTS voice synthesis, and in-app audio controls. |
| **Stage 12** | [**`stage-12-media-ocr-docs.md`**](stage-12-media-ocr-docs.md) | **Authentic Media Transcription, Unicode Indic OCR, Document Parsing & Native Overlay** | Pure-Rust `symphonia` media demuxing, timestamped SRT/VTT subtitle editor, authentic Unicode Indic OCR (PaddleOCR/Tesseract), structured doc parser (PDF/DOCX/CSV/MD), and borderless desktop overlay. |
| **Stage 13** | [**`stage-13-ui-ux-polish.md`**](stage-13-ui-ux-polish.md) | **UI/UX Polish, Responsive Layout Fixes & Navigation Architecture** | Eliminating selector truncations, removing internal development stage headers, consolidated navigation drawer, responsive bottom nav, keyboard shortcuts, and embedded Noto Indic webfonts. |
| **Stage 14** | [**`stage-14-model-lifecycle.md`**](stage-14-model-lifecycle.md) | **Model Pack Lifecycle, Dynamic Learning Decks & Enterprise API Hardening** | Resumable HTTP Range model downloads with SHA-256 verification, dictionary-driven SM-2 flashcard decks, microphone pronunciation practice, and end-to-end Axum RBAC middleware. |
| **Stage 15** | [**`stage-15-golden-certification.md`**](stage-15-golden-certification.md) | **End-to-End Golden Certification, Honest Benchmarking & Release Packaging** | Automated 500-pair golden linguistic regression suite, headless Playwright E2E verification, bifurcated dual-tier benchmarks in `BENCHMARKS.md`, and multi-platform signed distribution packages. |

---

## Architectural Objectives

1. **Eradicate Stubs & Mock Artifacts**: Replace mathematical sine-wave buzzers and dummy pass-throughs with real neural models (`sherpa-onnx`, VITS, IndicTrans2, Silero VAD).
2. **Linguistic Authenticity**: Ensure common vocabulary (e.g., `"adventure"`) reliably maps to accurate Kannada (**ಸಾಹಸ**) and Tamil (**சாகசம்**) equivalents across both dictionary search and sentence translation.
3. **Telemetry Truthfulness**: Enforce single-engine attribution badges, display actual wall-clock monotonic latencies, and eliminate self-referential `chrF: 100.0%` scores on live translation outputs.
4. **Visual & Accessibility Polish**: Resolve control bar clipping, provide fluid responsive navigation, and support proper Unicode rendering for complex Dravidian conjuncts.

