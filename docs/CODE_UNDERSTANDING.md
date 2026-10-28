# LinguaFlow — Code Understanding & Architecture Guide

## 1. System Architecture & Dependency Graph

LinguaFlow decouples presentation, persistence, server networking, and inference orchestration into distinct workspace crates:

```
                  ┌──────────────────────┐
                  │      src-tauri       │ (Desktop & Mobile Host)
                  └──────────┬───────────┘
                             │
       ┌─────────────────────┼─────────────────────┐
       │                     │                     │
       ▼                     ▼                     ▼
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│  lingua-cli  │      │lingua-server │      │lingua-storage│ (SQLite WAL)
└──────┬───────┘      └──────┬───────┘      └──────┬───────┘
       │                     │                     │
       └─────────────────────┼─────────────────────┘
                             │
                             ▼
                      ┌──────────────┐
                      │lingua-engine │ (Capability Matrix & Adapters)
                      └──────┬───────┘
                             │
                             ▼
                      ┌──────────────┐
                      │ lingua-core  │ (Domain Types, BCP-47, Errors, IPC)
                      └──────────────┘
```

---

## 2. Key Subsystems

### 2.1 `lingua-core`
- **Zero I/O Domain Layer**: Defines core primitives without pulling in heavy network or OS dependencies.
- `language.rs`: Strongly typed `LanguageCode` enum (`KnIn`, `TaIn`, `EnUs`) with BCP-47 strings, display names, and `LanguagePair` validation.
- `unicode.rs`: Unicode normalization (NFC) and Indic script detection routines.
- `error.rs`: `LinguaError` with structured machine-readable error codes and recovery flags.
- `ipc.rs`: Typed message envelope `IpcRequest<T>` and `IpcResponse<T>`.

### 2.2 `lingua-storage`
- **Local Persistence**: Manages SQLite with connection pooling and WAL mode.
- `schema.rs`: Versioned migration runner tracking applied schema versions in `schema_migrations`.
- `settings_repo.rs`: Reads and persists `UserSettings` to disk.
- `model_packs_repo.rs`: Maintains the offline neural model catalogue.

### 2.3 `lingua-engine`
- **Inference Coordinator**: Pluggable `TranslationEngine`, `SpeechToTextEngine`, and `TextToSpeechEngine` traits.
- `registry.rs`: Tracks available engines and dynamically builds the system `CapabilityMatrix`.
- Global Contract Enforcement: Rejects unlinked translation requests with `LinguaError::CapabilityUnavailable` instead of faking outputs.

### 2.4 `lingua-server`
- **Axum Edge Gateway**: Exposes REST endpoints (`/api/v1/health`, `/api/v1/capabilities`, `/api/v1/settings`) and WebSocket duplex streaming (`/api/v1/ws`).

### 2.5 `src-tauri`
- **Native Host**: Bridges the Svelte frontend to the Rust core via `#[tauri::command]` handlers that wrap `IpcResponse<T>`.

### 2.6 Frontend (`src/`)
- Svelte 5 / TypeScript application structured as modular views (`HomeView`, `TranslateView`, `DictionaryView`, `ConversationView`, `DownloadsView`, `SettingsView`).
- `src/lib/ipc.ts`: Dynamic bridge that auto-detects Tauri native IPC vs browser REST gateway.
- `src/app.css`: Modern glassmorphic theme system with dark/light modes and Kannada/Tamil typography.
