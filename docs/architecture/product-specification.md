# LinguaFlow — Product Specification & Architecture (Phase 0.1)

## 1. Executive Summary & Product Vision
LinguaFlow is an open, high-performance, offline-capable multilingual communication platform designed to bridge language divides across South Asian and global languages. LinguaFlow launches with primary focus on **Kannada (`kn-IN`)**, **Tamil (`ta-IN`)**, and **English (`en-US`)**, architected from inception to scale to other Indic and international languages.

The platform bridges desktop, mobile, and web through a unified Rust core with native platform integration via Tauri 2 and a modern, high-framerate Svelte 5 frontend.

---

## 2. Target User Personas

### Persona A: Dr. Ananya (Rural Healthcare Physician - Karnataka)
- **Context**: Interacts with Kannada-speaking rural patients and Tamil-speaking migrant workers in community health clinics. Connectivity is intermittent or absent.
- **Needs**: Dependable offline speech translation, medically clear vocabulary lookup, hands-free voice conversation.
- **Constraints**: Entry-level Android device, zero external cloud leakage of patient dialogues (HIPAA / DISHA compliance).

### Persona B: Murugan (Interstate Logistics Coordinator - Chennai & Bengaluru)
- **Context**: Operates cross-border supply chains connecting Tamil Nadu and Karnataka. Frequently uses desktop workstations and Android phones on the go.
- **Needs**: Instant dictionary search, technical term translation, document translation for shipping invoices and bills of lading.
- **Constraints**: Needs rapid keyboard shortcuts on desktop, instant copy/paste, low memory consumption.

### Persona C: Vikram (University Student & Language Enthusiast)
- **Context**: Learning Kannada and Tamil literature, reading dual-language articles on laptop and phone.
- **Needs**: Accurate phonetic pronunciation, grammatical breakdown, spaced-repetition vocabulary builder, live subtitles for recorded audio lectures.
- **Constraints**: Uses modern laptop (Windows/macOS/Linux) and smartphone; demands aesthetic UI with perfect Indic Unicode typography.

---

## 3. Scope Definition & Staged Evolution

| Stage | Milestones & Scope | Stage Status |
| :--- | :--- | :--- |
| **Stage 0** | **Foundation & Architecture**: Workspace initialization, Rust core, SQLite storage, Svelte UI, Podman tooling, multi-platform CI/CD, capability baseline. | **Active Target** |
| **Stage 1** | **Dictionary & Text Translation**: Authentic bidirectional translation, SQLite FTS5 dictionary, single-screen UX, linguistic correctness tests. | Pending Stage 0 Gate |
| **Stage 2** | **Offline Translation**: Local ONNX inference, model pack manager, hybrid routing, network-disabled verification. | Pending Stage 1 Gate |
| **Stage 3** | **Real-Time Speech & Captions**: 16kHz audio capture, streaming ASR, continuous translated captions, session resilience. | Pending Stage 2 Gate |
| **Stage 4** | **Real-Time Voice Conversation**: Push-to-talk, full-duplex conversation state machine, platform & neural TTS, AEC. | Pending Stage 3 Gate |
| **Stage 5** | **Translation Intelligence & Optimization**: Multi-engine registry, context & colloquial modes, transliteration, quality framework. | Pending Stage 4 Gate |
| **Stage 6** | **Media, Captions & Integrations**: Video/audio file subtitling, desktop live overlay, camera OCR, document translation. | Pending Stage 5 Gate |
| **Stage 7** | **Learning & Production Certification**: Spaced repetition, UX accessibility audit, security hardening, release packaging. | Pending Stage 6 Gate |
| **Stage 8** | **Commercial & Enterprise Platform**: Multi-tenant workspaces, usage metering, developer APIs, scalable edge gateways. | Pending Stage 7 Gate |

---

## 4. Multi-Platform Support Matrix

| Target Platform | Runtime Architecture | Distribution Format | Signing Status (Stage 0) | Limitations & Requirements |
| :--- | :--- | :--- | :--- | :--- |
| **Windows** | x86_64, aarch64 | NSIS Installer, Portable `.exe` | Unsigned test binary (Authenticode signing ready in CI) | Windows 10+ (WebView2 pre-installed); 100% offline embedded assets via native custom protocol (`tauri://localhost`); no localhost HTTP dev server required |
| **Linux** | x86_64, aarch64 | AppImage, `.deb` | GPG signed release metadata ready | Requires WebKitGTK 4.1, libayatana-appindicator, ALSA; 100% offline embedded assets |
| **macOS** | Universal (x86_64 + Apple Silicon) | `.app`, `.dmg` | Ad-hoc unsigned local build; Apple Developer ID in CI | macOS 11.0+ (Big Sur or newer); 100% offline embedded assets |
| **Android** | aarch64, armv7, x86_64 | `.apk`, `.aab` | Debug unsigned keystore in development; Release keystore in CI | Android 8.0+ (API Level 26+), Audio record permissions |
| **iOS** | arm64 | Xcode Archive, Simulator `.app` | Unsigned archive in CI; Apple Development Certificate required for physical provisioning | iOS 14.0+, Microphone privacy string in Info.plist |
| **Web** | WebAssembly / Modern Browsers | Static PWA bundle | Web origin HTTPS / TLS | Web Audio API & MediaDevices permissions required |
