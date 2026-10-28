# ADR-0001: Modular Rust Cargo Workspace and Typed IPC Protocol

## Status
Accepted

## Context
LinguaFlow requires a modular, production-grade foundation that operates on desktop (Windows, macOS, Linux), mobile (Android, iOS), web (WASM/edge), and server/container environments. A monolithic codebase would lead to tangled dependencies, slow compilation times, and leak platform-specific APIs into core linguistic business logic.

## Decision
1. Organize the repository as a Cargo workspace with distinct crates:
   - `crates/lingua-core`: Zero-IO domain types, BCP-47 representations, Unicode normalization, typed IPC schemas, and `LinguaError`.
   - `crates/lingua-storage`: SQLite persistence layer, pooling, and migrations.
   - `crates/lingua-engine`: Pluggable engine traits, capability matrices, and routing abstractions.
   - `crates/lingua-server`: Axum web service providing REST and WebSocket gateway endpoints.
   - `crates/lingua-cli`: Headless CLI for diagnostics, DB management, benchmarks, and automation.
   - `src-tauri`: Desktop and mobile native host bridging Rust core to the frontend.
2. Establish a versioned, typed IPC protocol where all requests and responses are strongly typed Rust structs serialized to JSON.
3. Keep security, validation, database operations, and audio handling strictly in Rust, minimizing frontend dependencies.

## Consequences
- Clean separation of concerns and high modularity.
- Independent testability of core business logic without spinning up a WebView or native window.
- Fast compilation with incremental caching across crates.
