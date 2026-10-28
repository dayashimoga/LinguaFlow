# LinguaFlow — End-User Guide (Stage 0 Foundation)

## 1. Overview
LinguaFlow is an offline-capable, cross-platform communication app designed for multilingual users. It allows instant communication between **Kannada (`kn-IN`)**, **Tamil (`ta-IN`)**, and **English (`en-US`)**.

---

## 2. Navigating the Application

### 2.1 Navigation Bar & Keyboard Shortcuts
You can navigate using the top header on desktop, bottom navigation bar on mobile, or global shortcuts:
- `Alt + 1`: **Home View** (Overview, health status, capability matrix).
- `Alt + 2`: **Translate View** (Bidirectional text translation and language swap).
- `Alt + 3`: **Dictionary View** (Full-text dictionary and grammatical breakdown).
- `Alt + 4`: **Conversation View** (Push-to-talk dual-speaker voice dialogue).
- `Alt + 5`: **Model Packs View** (Offline package manager and storage audit).
- `Alt + 6`: **Settings View** (Themes, font scale, audio devices, and SQLite database diagnostics).

---

## 3. Working Features in Stage 0

### 3.1 Theme & Appearance
- Switch between **Light**, **Dark**, and **System Default** themes using the top-right button or Settings.
- Adjust **Font Scale Factor** from 0.8x to 1.5x with real-time preview of Kannada and Tamil font shaping.
- All preferences are persisted into SQLite and restored on restart.

### 3.2 Language Pair Selection & Swapping
- Choose any non-identical language pair from the supported triad:
  - Kannada ➔ English
  - English ➔ Kannada
  - Tamil ➔ English
  - English ➔ Tamil
  - Kannada ➔ Tamil
  - Tamil ➔ Kannada
- Tap the **⇄** swap button to quickly reverse translation direction.

### 3.3 Honest Capability Transparency
- In accordance with our Global Engineering Contract, Stage 0 does not fake or simulate AI translation.
- When an AI feature (translation, streaming ASR, neural voice) is selected, its scheduled implementation stage (Stage 1 to 4) is clearly displayed.
- Model pack metadata and cryptographic SHA-256 hashes can be reviewed in the Model Packs view.
