# ADR-0002: Local Persistence Strategy with SQLite and Schema Migrations

## Status
Accepted

## Context
LinguaFlow requires robust, offline-first persistence for user settings, search history, translation bookmarks/favorites, and model catalog metadata. The persistence engine must support FTS5 full-text indexing (for Stage 1 dictionary search), operate reliably without a server daemon, run across desktop and mobile, and safely handle schema updates over time.

## Decision
1. Adopt SQLite via `rusqlite` (with `bundled` feature and `fts5` support enabled).
2. Implement an embedded, sequential schema migration runner with a dedicated `schema_migrations` table.
3. Manage database access via a thread-safe connection pool (`r2d2` or mutex-protected connection manager).
4. Store the SQLite file in platform-standard application data directories (e.g., `%APPDATA%/LinguaFlow` on Windows, `~/.local/share/linguaflow` on Linux, `~/Library/Application Support/LinguaFlow` on macOS, and app internal storage on Android/iOS).
5. For web-only deployments without native SQLite access, provide a mock-free in-memory/IndexedDB fallback adapter satisfying the same repository trait.

## Consequences
- Guaranteed ACID persistence with robust transaction semantics.
- Fast sub-millisecond local queries for user preferences and history.
- Immediate readiness for Stage 1 full-text search indexing with SQLite FTS5.
- Bounded local disk usage via explicit vacuuming and history record pruning.
