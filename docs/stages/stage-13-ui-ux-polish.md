# LinguaFlow — Stage 13: UI/UX Polish, Responsive Desktop/Mobile Shell & Embedded Indic Typography

Execute the Global Engineering Contract (`.antigravity/contract.md`). Fix all visual defects visible in the UI screenshots and elevate the frontend to a clean, accessible, modern production design.

---

## Phase 13.1: Visual Defect Remediation & Clean Product Branding

### Subphase 13.1.1 — Non-Truncated Language Selectors & One-Click Clipboard Paste
- **Minute Details**:
  - In `src/views/TranslateView.svelte`, replace abbreviated language selector labels with full non-truncated labels: `English (en-US)`, `ಕನ್ನಡ — Kannada (kn-IN)`, `தமிழ் — Tamil (ta-IN)`, and `Auto-Detect Script`.
  - Add a `📋 Paste` button (wired to Tauri clipboard read with browser fallback) and `✕ Clear` button in the Source Text card header.

### Subphase 13.1.2 — Production Branding & Developer HUD Toggle
- **Minute Details**:
  - Replace all internal engineering stage banners (`STAGE 8 COMMERCIAL & ENTERPRISE PLATFORM`, etc.) across all 12 views with clean product branding: `LinguaFlow — Offline-First Indic & Global Communication`.
  - Hide the developer microsecond performance HUD by default and add a persistent Developer Performance HUD toggle (`show_developer_hud`) in `src/views/SettingsView.svelte`.

---

## Phase 13.2: Consolidated Navigation Architecture, Mobile Safe-Area Shell & Global Shortcuts

### Subphase 13.2.1 — Core Workspace + System Drawer & Responsive Mobile Bottom Bar
- **Minute Details**:
  - Reorganize the 11-item navigation in `src/App.svelte` into:
    - **Core Workspace**: Home, Translate, Dictionary, Live Captions, Conversation, Media & OCR, Learn.
    - **System Drawer**: Intelligence, Model Packs, Enterprise, Settings.
  - On mobile viewports ($< 768$px for Android & iOS), render a responsive bottom navigation bar with `env(safe-area-inset-bottom)` and a slide-up sheet for secondary views.

### Subphase 13.2.2 — Global Keyboard Shortcut Engine
- **Minute Details**:
  - Implement global keyboard shortcuts in `src/App.svelte`:
    - `Ctrl/Cmd + Enter`: Execute Translation.
    - `Ctrl/Cmd + Shift + S`: Swap Source/Target Languages.
    - `Ctrl/Cmd + K`: Focus Dictionary Search.
    - `Esc`: Clear active input / Stop audio playback.

---

## Phase 13.3: Embedded Offline Indic Webfonts & Conjunct Rendering Guarantee

### Subphase 13.3.1 — Bundled Noto Sans Kannada, Noto Sans Tamil & Inter
- **Minute Details**:
  - Bundle offline `.woff2` fonts (NotoSansKannada, NotoSansTamil, Inter) inside `src/assets/fonts/` and register `@font-face` declarations with `font-display: block`.
  - Enforce `line-height: 1.65` and proper vertical padding on all inputs, cards, dictionary headwords, and subtitle overlays so Kannada *ottaksharas* (ಕ್ಷ, ಜ್ಞ, ಷ್ಟ್ರ) and Tamil ligatures never clip.

---

## Acceptance Criteria
1. Language selectors in `TranslateView.svelte` display full names with zero text truncation across mobile, tablet, and desktop viewports.
2. Internal stage development headers are replaced with clean product branding.
3. Developer HUD is hidden by default and toggleable via user settings.
4. Top navigation is consolidated into core views plus an administrative drawer, with responsive mobile bottom bar adaptation.
5. All Kannada and Tamil complex conjuncts render without clipping or overlapping glyphs using embedded Noto fonts.
