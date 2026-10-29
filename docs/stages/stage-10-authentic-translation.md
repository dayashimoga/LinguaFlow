# LinguaFlow — Stage 10: Authentic Translation Engine, Transliteration Fix & Telemetry Honesty

Execute the Global Engineering Contract. Eradicate the `"adventure"` $\rightarrow$ `"ದೇವೇಗೌಡರು"` hallucination, fix the broken `Script: ಡವೆನಟುರೆ` transliteration bug, eliminate contradictory engine badges, and remove fake `chrF: 100.0%` live scores.

---

## Phase 10.1 — Eradication of Mock/Broken Providers & Authentic Engine Routing

### Subphase 10.1.1 — Audit & Purge Corrupted Translation Lookups
- Audit `crates/lingua-engine/src/lexicon_engine.rs`, `local_model_engine.rs`, `registry.rs`, and `src-tauri/src/commands.rs` to identify and purge the defective fallback mapping or mock HTTP adapter (`http-rest-provider (2 hop)`) that produced `"ದೇವೇಗೌಡರು"` for `"adventure"`.
- Enforce strict engine routing: if an online provider is selected (*Cloud Online*) and no valid API key or endpoint is configured or reachable, return a structured `LinguaError::ProviderUnavailable` or explicitly fall back to the verified local engine with a clear visual notification—never scrape unreliable unauthenticated endpoints or return hash-indexed dummy strings.

### Subphase 10.1.2 — Tiered Offline Translation Pipeline (Lexicon + Phrasebook + Neural ONNX)
- Implement a transparent 3-tier deterministic offline pipeline in `lingua-engine`:
  - **Tier 1 (Exact & Lemmatized Lexicon Engine)**: For single words and known compound phrases (1–3 tokens), query the Stage 9 SQLite FTS5 lexicon ($< 2$ ms) and return the primary high-frequency target equivalent plus alternate senses.
  - **Tier 2 (Local Quantized Neural NMT Engine — ort / CTranslate2)**: For multi-word phrases, sentences, and paragraphs, execute real local neural inference using quantized (int8) ONNX / CTranslate2 models (IndicTrans2-Distilled 200M or NLLB-200-Distilled-600M INT8) with authentic SentencePiece (`spm`) tokenization for `en-US`, `kn-IN`, and `ta-IN`.
  - **Tier 3 (Authorized Cloud Provider Adapter)**: Optional user-configured REST adapters (Bhashini ULCA API, Google Cloud Translation v3, Azure AI Translator, or self-hosted `lingua-server` gateway) using authenticated TLS requests.

### Subphase 10.1.3 — Compound & Multi-Word Compositional Fallback
- If a user has not yet downloaded the multi-hundred-megabyte neural NMT pack in `ModelsView.svelte` and translates a multi-word sentence offline, perform morphological chunking + SOV (Subject-Object-Verb) syntactic reordering + lexicon glossing, and explicitly label the output badge as:
  ```
  ⚠️ OFFLINE GLOSS (DOWNLOAD NMT MODEL FOR FULL FLUENCY)
  ```

---

## Phase 10.2 — Bi-Directional Phonetic Transliteration & Script Rendering Fix

### Subphase 10.2.1 — Separate Input IME Transliteration from Output Phonetic Reading Aid
- Fix the critical bug where `"adventure"` was phonetically mangled into Kannada characters as `Script: ಡವೆನಟುರೆ`.
- Establish clear directional semantics in `crates/lingua-engine/src/transliteration.rs`:
  - **Mode A — Source Input Transliteration (Roman-to-Indic IME)**: Active only when the Source Language is `kn-IN` or `ta-IN` and *Transliterate ON* is enabled. Typing `"namaskara"` in the source box converts to **ನಮಸ್ಕಾರ**; typing `"vanakkam"` converts to **வணக்கம்**. When Source Language is `en-US`, typing `"adventure"` must never be transliterated into Kannada or Tamil script.
  - **Mode B — Target Output Phonetic Reading Aid (Indic-to-Roman ISO-15919 / ITRANS)**: In the Translated Text pane, display the phonetic pronunciation of the translated output so learners can read it aloud (e.g., when output is **ಸಾಹಸ**, display `🗣️ Phonetic: sāhasa`; when output is **சாகசம்**, display `🗣️ Phonetic: cākacam`).

### Subphase 10.2.2 — Syllabic Abugida Grapheme Engine (`kn-IN` & `ta-IN`)
- Implement full vowel-consonant *gunintam* / *kagunita* (Kannada) and *uyirmei* (Tamil) syllabic state machines supporting halant/virama (`್` / `்`), anusvara (`ಂ` / `ம்`), visarga (`ಃ` / `ஃ`), geminated consonants (`ಕ್ಕ`, `ತ್ತ`, `ப்ப`, `த்த`), and retroflex vs. dental distinctions (`ಟ`/`ತ`, `ಡ`/`ದ`, `ಣ`/`ನ`, `ಳ`/`ಲ`, `ழ`/`ள`/`ல`).

---

## Phase 10.3 — Honest Telemetry Badges & Removal of Fake Live chrF Scores

### Subphase 10.3.1 — Single Truthful Engine Attribution Badge
- Update `TranslateView.svelte` so the top badge, metadata chip, and footer always reflect the exact single engine that generated the result:
  - `⚡ OFFLINE LEXICON (SQLite FTS5)` — when served from the local dictionary.
  - `🧠 OFFLINE NEURAL (ONNX INT8)` — when served from the local neural model.
  - `☁️ CLOUD PROVIDER (<Provider Name>)` — when served over HTTP/REST when Cloud Online is active.
  - `💾 LOCAL LRU CACHE (<Original Engine>)` — when served from the SHA-256 memory/SQLite cache.
- Ensure the displayed latency (ms or µs) matches the actual monotonic wall-clock time of the IPC call.

### Subphase 10.3.2 — Replace Self-Referential chrF: 100.0% with True Confidence / Dictionary Senses
- Remove `🎯 chrF: 100.0%` from live translation outputs where no ground-truth reference exists.
- Restrict chrF, WER, and CER exclusively to `IntelligenceView.svelte`'s Benchmark & Evaluation Suite (where outputs are scored against curated human reference datasets).
- In `TranslateView.svelte`, replace the chrF chip with Engine Confidence / Sense Match (e.g., `✓ Verified Lexicon Match (3 senses)` or `🧠 Neural Confidence: 94.2%`).

### Subphase 10.3.3 — Fix Dictionary Drill-Down Synchronization
- Update Dictionary drill-down in `TranslateView.svelte` to list every translated token and alternate synonym (e.g., `[ಸಾಹಸ (Adventure)]`, `[ಧೈರ್ಯದ ಕೆಲಸ (Daring feat)]`, `[ರೋಮಾಂಚನ (Thrill)]`).
- Clicking any chip must open its full definition card or navigate directly to `DictionaryView.svelte` pre-populated with that word.

---

## Acceptance Criteria
1. Translating `"adventure"` (`en-US` $\rightarrow$ `kn-IN`) returns **ಸಾಹಸ** and never **ದೇವೇಗೌಡರು**.
2. Translating `"adventure"` (`en-US` $\rightarrow$ `ta-IN`) returns **சாகசம்**.
3. Typing English source text with transliteration enabled does not mangle English into Indic script (no `ಡವೆನಟುರೆ`).
4. Output phonetic aids correctly display ISO-15919 pronunciations (`sāhasa`, `cākacam`).
5. Live translation outputs display truthful engine badges and zero self-referential `chrF: 100.0%` scores.
