# LinguaFlow — End-User Guide (Production Stage 8)

## 1. Overview
LinguaFlow is an offline-capable, cross-platform communication and language learning platform designed for multilingual users. It delivers instant, authentic communication between **Kannada (`kn-IN`)**, **Tamil (`ta-IN`)**, and **English (`en-US`)** with zero nonfunctional placeholders or simulated responses.

---

## 2. Navigating the Application

### 2.1 Navigation Bar & Keyboard Shortcuts
You can navigate using the top header on desktop, bottom navigation bar on mobile, or global shortcuts:
- `Alt + 1`: **Home View** (System health, engine status, capability matrix).
- `Alt + 2`: **Translate View** (Bidirectional translation, debounced lookup, language swap `Ctrl+S`).
- `Alt + 3`: **Dictionary View** (Full-text FTS5 search, parts-of-speech, transliteration, audio pronunciation).
- `Alt + 4`: **Conversation View** (Push-to-talk dual-speaker voice dialogue with 180° flippable face-to-face tabletop UI).
- `Alt + 5`: **Media Studio** (Subtitle studio, Camera & OCR scanner, Document translator, Live caption overlay).
- `Alt + 6`: **Learning Studio** (SM-2 Spaced Repetition flashcards, Pronunciation coach, Listening challenges, Grammar guide).
- `Alt + 7`: **Enterprise Studio** (Multi-tenant workspaces, RBAC, API keys, usage quotas, billing invoices, audit trails).
- `Alt + 8`: **Model Packs View** (Offline neural package manager, SHA-256 integrity audit).
- `Alt + 9`: **Settings View** (Themes, font scale, audio devices, data privacy export and purge).

---

## 3. Core Translation & Intelligence Features

### 3.1 Live Translation & Intelligence Pipeline
- **Debounced Translation**: As you type in Translate view, translation executes with automatic language detection and caching.
- **Formality Modes**: Toggle between **Formal** (honorific Dravidian verb endings, e.g. ಬನ್ನಿ / வாருங்கள்) and **Colloquial** (informal everyday speech, e.g. ಬಾ / வா).
- **Custom Glossaries**: Add domain-specific terminology rules (e.g., Medical, Legal, Technical) to enforce exact phrasing in translations.
- **Romanized Indic Transliteration**: Type Indic words using standard English Latin script (e.g., `namaskara` -> `ನಮಸ್ಕಾರ`, `vanakkam` -> `வணக்கம்`).
- **Offline & Online Failover**: The engine seamlessly shifts between cloud HTTP inference and local offline neural models with automatic circuit breakers.

### 3.2 Speech & Conversation Modes
- **Speech Captions**: Live bilingual audio transcription with energy-based Voice Activity Detection (VAD).
- **Speech Synthesis (TTS)**: Multi-formant acoustic synthesis for Kannada, Tamil, and English with customizable speaking rate and pitch.
- **Dual-Person Tabletop Mode**: 180° flip layout allowing two people across a table to speak their native languages naturally with push-to-talk and acoustic echo suppression.

### 3.3 Media, OCR & Documents
- **Subtitle Studio**: Import video or audio files (MP4, WebM, WAV, MP3, OGG), auto-generate bilingual subtitles, and export to SRT, VTT, TXT, or JSON.
- **Camera & OCR Scanner**: Scan text from photos or documents with in-place translated text overlays.
- **Document Translator**: Translate Markdown (`.md`), plain text (`.txt`), PDF streams, and CSV tables with strict format preservation.
- **Live Caption Overlay**: Desktop floating, resizable, always-on-top caption bar for meetings, videos, and calls.

---

## 4. Learning Studio & Pronunciation Coach

### 4.1 SM-2 Spaced Repetition Vocabulary Practice
- **Authentic SuperMemo-2 (SM-2) Algorithm**: Calculates dynamic review intervals and ease factor adjustments based on your recall accuracy:
  - `0 - Blackout`: Complete memory failure; resets interval to 1 day.
  - `1 - Incorrect`: Incorrect response; reset.
  - `2 - Hard`: Recalled with serious difficulty.
  - `3 - Good`: Correct response with some hesitation.
  - `4 - Easy`: Correct response with hesitation.
  - `5 - Perfect`: Immediate, effortless recall; maximizes interval expansion.
- **Interactive 3D Flashcards**: Tap the card or press `Space` to flip between target phrase, transliteration, grammar notes, and audio playback.
- **Pre-Seeded Decks**: Includes curated beginner and intermediate decks for **Kannada Everyday Vocabulary** and **Tamil Daily Conversational**.

### 4.2 Native Pronunciation Assessment
- **Microphone Voice Evaluation**: Read the target phrase aloud to test your accent and pronunciation.
- **Objective Multi-Metric Scoring**:
  - **Accuracy Score (0–100%)**: Composite phonetic alignment index.
  - **Character Error Rate (CER)**: Exact character edit distance against native reference.
  - **chrF Precision Score**: Character n-gram F-score evaluating morphological root correctness.
- **Phonemic Feedback**: Highlights specific Dravidian phonemes requiring attention (retroflex `ಳ`/`ள`, aspirated consonants `ಖ`/`ಘ`, nasal vowels).

### 4.3 Listening Challenges
- **Acoustic Discrimination**: Listen to authentic synthesized audio and identify the correct phrase or translation among phonetically similar distractors.
- **Grammar Explanations**: Review detailed morphological breakdowns explaining why distractors are incorrect.

### 4.4 Dravidian Grammar Guide
- **Structured Knowledge Base**:
  - **Kannada Vibhakti**: 7 grammatical cases (Prathama, Dwitiya, Trithiya, Chaturthi, Panchami, Shashti, Saptami) with suffixes and contextual examples.
  - **Tamil Vēṟṟumai**: 8 grammatical cases (Mudhal, Irandam, Mundram, Nankam, Aindham, Ararm, Ezham, Ettam) with inflections and sandhi rules.
- **Searchable Topics**: Instant filtering across verb paradigms, pronouns, tenses, and case inflections.

---

## 5. Data Privacy & Local Governance

### 5.1 Zero-Log Transcript Guarantee
- LinguaFlow guarantees that user utterances, speech recordings, and translation transcripts are **never** logged to disk or sent to telemetry collectors.
- All internal logging routes through redaction filters stripping credentials and transcripts.

### 5.2 Authenticated User Data Export
- Navigate to **Settings > Data Privacy & Local Governance**.
- Click **Export User Data Archive** or execute `lingua privacy export`.
- Generates a complete JSON archive of your settings, favorites, translation history, conversation sessions, media sessions, and learning review logs with an immutable **SHA-256 checksum** verifying archive integrity.

### 5.3 "Right to be Forgotten" (Atomic Purge)
- If you wish to erase your activity permanently, click **Purge All User Data** in Settings or execute `lingua privacy purge`.
- LinguaFlow executes an atomic SQLite transaction deleting all user records, vacuuming the database file, and resetting tables to default factory state.

---

## 6. Commercial & Enterprise Platform

### 6.1 Anonymous Offline Guarantee
- LinguaFlow functions completely offline with zero account requirements, zero cloud tokens, and zero network calls via pre-seeded local anonymous mode (`acc-anonymous-local`).
- Upgrading to `Pro` or `Enterprise` tier enables cross-device synchronization and extended quotas while preserving 100% offline capabilities.

### 6.2 Workspaces & Role-Based Access Control (RBAC)
- **Workspaces**: Create isolated organization workspaces with dedicated shared glossaries and audit logging.
- **Roles**:
  - **Owner**: Full workspace authority, billing, deletion.
  - **Admin**: Member management, glossary publishing, audit inspection.
  - **Linguist**: Glossary editing and terminology approvals.
  - **Member**: Standard read and translation access.

### 6.3 Shared Dravidian Glossaries
- Define brand and technical terms for Kannada and Tamil with guaranteed substitution across all workspace users.

### 6.4 Developer API Keys & Usage Quotas
- Generate cryptographic API keys (`lf_live_*`) with token bucket rate limiting.
- **Fail-Safe Billing Contract**: If any billable operation encounters an execution error, reserved character and audio quotas are immediately refunded.

