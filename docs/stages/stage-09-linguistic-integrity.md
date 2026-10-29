# LinguaFlow — Stage 9: Linguistic Data Integrity, Comprehensive Lexicon Seeding & Dictionary UX Overhaul

Execute the Global Engineering Contract. Eliminate the empty dictionary failure (e.g., `Found 0 matching entries for "adventure"`) and replace toy seed data with production-grade, open-licensed trilingual datasets covering **English (`en-US`)**, **Kannada (`kn-IN`)**, and **Tamil (`ta-IN`)**.

---

## Phase 9.1 — Open-Licensed Trilingual Lexicon Acquisition & ETL Pipeline

### Subphase 9.1.1 — Dataset Sourcing & License Verification
- Integrate verified, permissively licensed lexical datasets:
  - **Kannada ↔ English**: Alar (V. Krishna's Kannada-English dictionary, ODC-By / CC-BY-4.0) and Kaikki.org / Wiktextract machine-readable Wiktionary dumps (`kn`, `en`).
  - **Tamil ↔ English**: University of Madras Tamil Lexicon (Public Domain / CC-BY), Project Madurai / Open-Tamil lexical lists, and Wiktextract (`ta`, `en`).
  - **Kannada ↔ Tamil Bridge**: Construct direct cross-linked synset and pivot tables (`kn-IN` $\leftrightarrow$ `ta-IN`) using shared English sense IDs, Sanskrit/Dravidian cognate mappings, and curated bidirectional equivalents.
- Record exact dataset versions, upstream URLs, SHA-256 cryptographic hashes, and license terms in `docs/LEXICON_PROVENANCE.md` and `docs/compliance/LICENSES.md`.

### Subphase 9.1.2 — Automated Rust ETL & Normalization Compiler (`lingua-cli dict-import`)
- Build a deterministic build-time and CLI ingestion tool (`crates/lingua-cli/src/dict_import.rs`) that parses raw JSONL, CSV, and XML lexical sources.
- Enforce strict Unicode NFC normalization (`crates/lingua-core/src/unicode.rs`), strip malformed HTML/wikitext markup, validate BCP-47 script boundaries (`U+0C80`–`U+0CFF` for Kannada, `U+0B80`–`U+0BFF` for Tamil, `U+0020`–`U+007F` for English), and deduplicate headwords.
- Ensure minimum baseline coverage:
  - $\ge$ 50,000 English headwords (explicitly verifying common A1–C2 vocabulary including `"adventure"` $\rightarrow$ Kannada: **ಸಾಹಸ** (*sāhasa*), **ರೋಮಾಂಚಕ ಅನುಭವ**; Tamil: **சாகசம்** (*cākacam*), **துணிகரச் செயல்**).
  - $\ge$ 60,000 Kannada headwords.
  - $\ge$ 60,000 Tamil headwords.

### Subphase 9.1.3 — Rich Lexical Schema Expansion (Migration `010_rich_lexicon_schema.sql`)
- Extend `lingua-storage` SQLite dictionary tables to store structured multi-sense entries:
  - `headword`, `language_code`, `normalized_stem`, `ipa_pronunciation`, `native_phonetic_respelling` (e.g., `"sāhasa"` in ISO-15919 and Kannada/Tamil script phonetics).
  - `part_of_speech` (noun, verb, adjective, adverb, postposition, idiom), `gender_inflection_notes`, `register` (formal, colloquial, literary, medical, logistics).
  - Relational child tables for `senses` (ordered by frequency rank), `bilingual_examples` (source sentence + target translation), `synonyms`, `antonyms`, and `derived_forms`.

---

## Phase 9.2 — High-Precision SQLite FTS5 & Morphological Search Engine

### Subphase 9.2.1 — Custom FTS5 Tokenizer & Ranking Configuration
- Configure SQLite FTS5 virtual tables (`dictionary_fts`) with `unicode61` tokenizer preserving Indic vowel signs (matras / pulli / virama: `್` in Kannada, `்` in Tamil) so prefix searches do not split grapheme clusters.
- Implement custom BM25 column weighting (`bm25(dictionary_fts, 10.0, 5.0, 2.0, 1.0)`) prioritizing exact headword matches first, prefix matches second, transliterated roman keys third, and definition body keywords fourth.

### Subphase 9.2.2 — Agglutinative Indic Morphological Stemmer & Lemmatizer
- Upgrade `crates/lingua-engine/src/lexicon_engine.rs` to handle agglutinative suffixes in Kannada and Tamil before FTS5 fallback:
  - **Kannada Suffixes**: Strip plural/case markers (`-ಗಳು`, `-ಅನ್ನು`, `-ಇಂದ`, `-ಗೆ`, `-ಅಲ್ಲಿ`, `-ಉತ್ತದೆ`) to resolve inflected words (e.g., ಸಾಹಸಗಳನ್ನು $\rightarrow$ root ಸಾಹಸ).
  - **Tamil Suffixes**: Strip case/plural markers (`-கள்`, `-ஐ`, `-ஆல்`, `-க்கு`, `-இல்`, `-உடன்`) and sandhi doubled consonants (`க்`, `ச்`, `த்`, `ப்`) to resolve root forms (e.g., சாகசங்களை $\rightarrow$ root சாகசம்).
  - **English Lemmatization**: Resolve plurals, past tenses, and participles (*adventures*, *adventuring*, *adventurous* $\rightarrow$ *adventure*).

### Subphase 9.2.3 — Fuzzy & Phonetic Search Fallback (Trigram + Levenshtein)
- Add a trigram index and bounded Levenshtein distance (distance $\le$ 2) fallback when exact/prefix FTS5 returns 0 results, providing *"Did you mean: ...?"* suggestions for misspelled English or romanized Indic queries (e.g., `"advnture"` $\rightarrow$ `"adventure"`, `"namaskra"` $\rightarrow$ `"ನಮಸ್ಕಾರ"`).

---

## Phase 9.3 — Dictionary View (`DictionaryView.svelte`) UX & Drill-Down Overhaul

### Subphase 9.3.1 — Instant Search-as-You-Type & Filter Controls
- Bind search input in `src/views/DictionaryView.svelte` to a 120 ms debounced reactive query so typing `"adventure"` immediately populates results without requiring a manual click on the Search button.
- Enhance language selector (*All Languages*, *English $\rightarrow$ Kannada*, *English $\rightarrow$ Tamil*, *Kannada $\rightarrow$ English*, *Tamil $\rightarrow$ English*, *Kannada $\leftrightarrow$ Tamil*) and add Part-of-Speech filter pills (*All*, *Nouns*, *Verbs*, *Adjectives*, *Medical*, *Logistics*).

### Subphase 9.3.2 — Rich Entry Cards & Interactive Word Drill-Down
- Render each dictionary result with:
  - Primary Headword + Language Badge + Copy & Favorite (☆/★) toggle.
  - IPA Phonetics (`/ədˈvɛntʃər/`) + Romanized Indic Transliteration (*sāhasa* / *cākacam*) + Play Audio (🔊) TTS button.
  - Numbered sense definitions grouped by Part of Speech (*n.*, *v.*), showing both Kannada (**ಸಾಹಸ**) and Tamil (**சாகசம்**) equivalents side-by-side when *All Languages* is selected.
  - Clickable synonym/antonym chips that trigger immediate drill-down navigation and synchronize with `TranslateView.svelte`'s dictionary drill-down bar.

---

## Acceptance Criteria
1. Typing `"adventure"` in `DictionaryView.svelte` yields instant, verified definitions for both Kannada (**ಸಾಹಸ**) and Tamil (**சாகசம்**) with IPA phonetics, parts of speech, and bilingual examples in $\le 10$ ms.
2. Ingested datasets pass Unicode normalization and script boundary checks with zero malformed characters across $\ge 50,000$ English, $\ge 60,000$ Kannada, and $\ge 60,000$ Tamil headwords.
3. Morphological lemmatizer resolves inflected forms (`ಸಾಹಸಗಳನ್ನು`, `சாகசங்களை`, `adventures`) to base stems.
4. All unit, integration, and UI tests pass with zero regressions.
