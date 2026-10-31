---
name: indic-linguistics-ocr-guard
description: Guarantees Kannada (kn-IN) and Tamil (ta-IN) Unicode NFC integrity, agglutinative morphology, virama/pulli conjunct rendering, and trilingual OCR accuracy.
---
# Indic Linguistics, OCR & Typography Guard Skill

1. **Script & Grapheme Rules**:
   - Kannada (`U+0C80..U+0CFF`, Virama `U+0CCD`) and Tamil (`U+0B80..U+0BFF`, Pulli `U+0BCD`) strings MUST pass through `lingua_core::unicode::normalize_nfc()` before tokenization, FTS5 querying, or OCR output emission.
2. **Typography & Anti-Clipping Rules**:
   - Every CSS selector displaying Indic script must apply `font-family: 'Noto Sans Kannada', 'Noto Sans Tamil', 'Inter', system-ui, sans-serif; line-height: 1.65;` so Kannada subscript conjuncts (*ottaksharas* like `ಕ್ಷ`, `ಜ್ಞ`, `ಷ್ಟ್ರ`) are never clipped vertically.
3. **Golden Translation Integrity**:
   - Verify that Tier-1 FTS5 lexicon (`< 0.4 ms`), agglutinative stemmer, Mode A IME transliteration, and Mode B ISO-15919 phonetic aids never regress on the 500-pair golden suite.

