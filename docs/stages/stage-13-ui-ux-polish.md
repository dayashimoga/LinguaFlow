# LinguaFlow — Stage 13: UI/UX Polish, Responsive Layout Fixes & Navigation Architecture

Execute the Global Engineering Contract. Fix all visual defects visible in the UI screenshots and elevate the frontend to a clean, accessible, modern production design.

---

## Phase 13.1 — Visual Defect Remediation from Screenshots

### Subphase 13.1.1 — Fix Truncated Language Selectors & Control Bar Alignment
- In `TranslateView.svelte`, fix the CSS width and padding on `<select>` elements so labels such as `English (English)` and `ಕನ್ನಡ (Kannada)` never truncate.
- Render clean full labels:
  - `English (en-US)`
  - `ಕನ್ನಡ — Kannada (kn-IN)`
  - `தமிழ் — Tamil (ta-IN)`
  - `Auto-Detect Script`
- Add a one-click Paste (`📋 Paste`) button next to Clear and Copy in the Source Text pane.
- Add Listen (`🔊`) buttons with playing states to both panes.

### Subphase 13.1.2 — Remove Internal Stage Labels & Fix Window Title Bar Telemetry
- In `App.svelte`, replace internal development titles (`STAGE 8 COMMERCIAL & ENTERPRISE PLATFORM`) with clean product branding:
  ```
  LinguaFlow — Offline-First Indic & Global Communication
  ```
- Fix the top-right native window overlay bar (`FPS N/A | GPU 0% | CPU 17% | LAT N/A`):
  - Populate with real metrics (actual IPC latency in milliseconds and `requestAnimationFrame` FPS), or
  - Hide the developer HUD by default and make it toggleable via `SettingsView.svelte` (*"Show Developer Performance HUD"*).

### Subphase 13.1.3 — Navigation Bar Consolidation & Keyboard Accessibility
- Reorganize the crowded 11-item horizontal navigation bar in `App.svelte` into a clean, grouped layout:
  - **Core Workspace**: Home, Translate, Dictionary, Live Captions, Conversation, Media & OCR, Learn.
  - **System & Admin (Dropdown / Drawer)**: Intelligence, Model Packs, Enterprise, Settings.
- Implement responsive bottom navigation for mobile and tablet viewports.
- Bind global and view-specific keyboard shortcuts:
  - `Ctrl + Enter`: Trigger translation.
  - `Ctrl + Shift + S`: Swap source and target languages.
  - `Ctrl + K`: Focus dictionary search input.
  - `Esc`: Clear inputs or cancel in-flight operations.

---

## Phase 13.2 — Indic Typography & Theme Consistency

### Subphase 13.2.1 — Embedded Indic Webfonts & Line-Height Scaling
- Bundle offline variable fonts (**Noto Sans Kannada**, **Noto Sans Tamil**, and **Inter**) inside `src/assets/fonts/`.
- Ensure complex Kannada *ottaksharas* (conjuncts like ಕ್ಷ, ಜ್ಞ, ಷ್ಟ್ರ) and Tamil ligatures render crisply with `line-height: 1.65` and zero vertical glyph clipping across Windows, Linux, macOS, and Android.

---

## Acceptance Criteria
1. Language selectors in `TranslateView.svelte` display full names with zero text truncation across mobile, tablet, and desktop viewports.
2. Internal stage development headers are replaced with clean product branding.
3. Developer HUD is hidden by default and toggleable via user settings.
4. Top navigation is consolidated into core views plus an administrative drawer, with responsive mobile bottom bar adaptation.
5. All Kannada and Tamil complex conjuncts render without clipping or overlapping glyphs using embedded Noto fonts.

