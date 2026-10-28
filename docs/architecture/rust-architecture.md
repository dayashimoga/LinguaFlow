# LinguaFlow — Modular Rust Architecture & IPC Contracts

## 1. Workspace Topology

The LinguaFlow backend is designed as an idiomatic, modular Cargo workspace where each crate has strict boundaries, minimal dependencies, and well-defined responsibilities.

```
LinguaFlow/
├── crates/
│   ├── lingua-core/        # Shared domain models, BCP-47 codes, Unicode validation,
│   │                       # structured errors, typed IPC contracts, configuration.
│   ├── lingua-storage/     # SQLite persistence layer, connection pooling,
│   │                       # schema migrations, user settings, search history, model inventory.
│   ├── lingua-engine/      # Pluggable translation and speech inference abstractions,
│   │                       # capability detection matrix, offline/online router foundation.
│   ├── lingua-server/      # Axum REST & WebSocket gateway service for server/edge deployments.
│   └── lingua-cli/         # Headless CLI for diagnostics, DB management, benchmarks, and CI.
├── src-tauri/              # Tauri 2 host integrating desktop and mobile native layers.
├── src/                    # Shared Svelte 5 / TypeScript frontend application.
└── edge/                   # Cloudflare Workers edge configuration.
```

---

## 2. Strong Typing & IPC Protocol

Frontend-to-Rust communication utilizes a strongly typed protocol. Every command and event adheres to a versioned schema defined in `lingua-core::ipc`.

### Command IPC Envelope
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest<T> {
    pub request_id: String,
    pub timestamp_ms: u64,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse<T> {
    pub request_id: String,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<IpcErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}
```

### Core IPC Commands
- `lingua_get_app_info`: Returns application metadata, version, commit, and build profile.
- `lingua_get_capabilities`: Queries the current system capability matrix across language pairs and engines.
- `lingua_get_user_settings`: Retrieves persisted user preferences (theme, font scale, language pair, audio defaults).
- `lingua_save_user_settings`: Persists updated preferences to SQLite with instant reactivity.
- `lingua_list_model_packs`: Queries available, downloaded, and downloadable offline model packages.
- `lingua_check_database_health`: Performs integrity checks on the SQLite database and connection pool.

---

## 3. Structured Error Architecture (`LinguaError`)

All workspace crates return structured errors using `thiserror`. Panics are strictly disallowed across all business logic.

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum LinguaError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid language pair: source {0} cannot equal target {1}")]
    IdenticalLanguagePair(String, String),

    #[error("Language '{0}' is unsupported for capability '{1}'")]
    UnsupportedLanguage(String, String),

    #[error("Engine capability '{0}' is unavailable: {1}")]
    CapabilityUnavailable(String, String),

    #[error("Network connection required for online operation")]
    NetworkUnavailable,

    #[error("Storage budget exceeded: requested {requested_bytes}B, available {available_bytes}B")]
    StorageBudgetExceeded { requested_bytes: u64, available_bytes: u64 },

    #[error("Audio device failure: {0}")]
    AudioDeviceError(String),

    #[error("Model verification failed for '{0}': checksum mismatch")]
    ModelIntegrityFailure(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 4. Dependency Injection & Service Container

Services are composed using Rust thread-safe reference counters (`Arc<AppState>`).

```rust
pub struct AppState {
    pub storage: Arc<StorageService>,
    pub engine_registry: Arc<EngineRegistry>,
    pub app_info: AppInfo,
}
```

State is injected cleanly into:
1. Tauri runtime state via `.manage(app_state)`.
2. Axum gateway router via `.with_state(app_state)`.
3. CLI commands via direct instance creation.
