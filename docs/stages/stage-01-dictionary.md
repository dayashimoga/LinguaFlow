# LinguaFlow — Stage 1: Dictionary and Text Translation

Execute the Global Engineering Contract and preserve Stage 0 functionality.

**Phase 1.1 — Translation engine**
Implement production Rust translation contracts, Unicode normalization, BCP-47 language handling, explicit language selection, automatic detection, supported-pair validation and real provider adapters. Integrate a verified Kannada/Tamil/English translation provider or executable model. Require authentic outputs; fail explicitly when credentials or models are unavailable.

**Phase 1.2 — Dictionary**
Implement SQLite/FTS5 dictionary storage, prefix/fuzzy searches, definitions, alternate meanings, grammatical information, examples, pronunciation metadata, synonyms/antonyms and transliteration where licensed data exists. Build an import pipeline for properly licensed dictionary datasets with provenance, validation and indexing.

**Phase 1.3 — UX**
Create an exceptional one-screen translate/dictionary experience: language swap, translate-as-you-type with adaptive debounce, result streaming when supported, copy, favorites, history, word drill-down, paste, keyboard shortcuts, touch actions and helpful loading/error states. Avoid unnecessary clicks or navigation.

**Phase 1.4 — Correctness**
Test Kannada↔English, Tamil↔English and Kannada↔Tamil, both directions, with curated real examples. Include Unicode, punctuation, inflections, long sentences, mixed scripts, empty input, special characters, invalid languages, unavailable providers and disconnected networks. Separate translation accuracy evaluations from deterministic software tests.

**Phase 1.5 — Performance and release**
Benchmark cold/warm queries, search index speed, cache behavior, memory growth, responsive rendering and repeated requests. Add automated UI E2E flows and real-provider integration tests using authorized test credentials. Enforce no regression.

Regenerate all desktop/mobile/web artifacts through GitHub Actions. Publish working screenshots, performance results, linguistic findings, coverage and updated append-only records.

Acceptance: A user can install the app and complete real dictionary searches and text translations, with trustworthy results and no hidden dependence on fake engines.