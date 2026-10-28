# LinguaFlow — Stage 2: Offline Translation

Execute the Global Engineering Contract; retain fully working Stage 1 features.

**Phase 2.1 — Local inference**
Evaluate actual lightweight translation models for Kannada, Tamil and English on Android ARM64, Windows x64, macOS ARM64 and Linux. Prototype approved ONNX/other optimized runtimes and verify tokenizer/preprocessing compatibility. Benchmark quality, memory and CPU versus provider translation. Integrate only models that genuinely execute and pass validation.

**Phase 2.2 — Model manager**
Implement downloadable language/model packs with versioned signed manifests, SHA-256 verification, integrity checks, resumable downloads, disk-space checks, progress/cancel/retry, rollback, model eviction and optional storage location selection. Never bundle huge models silently.

**Phase 2.3 — Offline engine selection**
Create Rust routing for offline-only, automatic hybrid and online modes. Route by available model, language pair, battery/thermal/memory limits, quality preference and user privacy settings. Support deterministic fallback without silently sending text to cloud services.

**Phase 2.4 — Offline dictionary**
Cache licensed definitions, common words, morphological resources and relevant examples in indexed SQLite; support offline search, favorites and recent history. Implement schema migrations and encrypted sensitive local data where appropriate.

**Phase 2.5 — Validation**
Run actual translations with networking disabled, downloaded models installed and clean app restarts. Test corrupted models, download interruption, insufficient storage, unavailable language models, app upgrades, memory pressure and low-resource hardware. Measure cold/warm inference latency, model footprint, RAM, CPU and battery.

Deliver working offline capabilities for explicitly certified language pairs. Mark unsupported capabilities accurately instead of simulating offline support. Rebuild/upload all platform artifacts and publish updated model capability/benchmark matrices.