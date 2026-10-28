use lingua_core::LinguaError;
use rusqlite::Connection;
use tracing::info;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

pub fn run_migrations(conn: &mut Connection) -> Result<u32, LinguaError> {
    // Enable WAL mode and foreign keys for high concurrent performance & safety
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to set pragmas: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL,
            description TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to create schema_migrations table: {e}")))?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        apply_migration_001(conn)?;
    }
    if current_version < 2 {
        apply_migration_002(conn)?;
    }

    Ok(CURRENT_SCHEMA_VERSION)
}

fn apply_migration_001(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 001 — Initial Schema");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- User preferences table
        CREATE TABLE IF NOT EXISTS user_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            theme TEXT NOT NULL,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            font_scale REAL NOT NULL,
            engine_preference TEXT NOT NULL,
            offline_only INTEGER NOT NULL,
            audio_settings_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Model packs catalog
        CREATE TABLE IF NOT EXISTS model_packs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            version TEXT NOT NULL,
            sha256 TEXT NOT NULL,
            is_installed INTEGER NOT NULL DEFAULT 0,
            scheduled_stage INTEGER NOT NULL
        );

        -- Translation & dictionary search history
        CREATE TABLE IF NOT EXISTS translation_history (
            id TEXT PRIMARY KEY,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            engine_used TEXT NOT NULL,
            latency_ms INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Bookmarks / Favorites
        CREATE TABLE IF NOT EXISTS favorites (
            id TEXT PRIMARY KEY,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL
        );

        -- Record migration
        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (1, datetime('now'), 'Initial schema: settings, model_packs, history, favorites');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 001: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 001: {e}")))?;

    info!("Migration 001 applied successfully");
    Ok(())
}

fn apply_migration_002(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 002 — FTS5 Dictionary & Vocabulary Index");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- Dictionary entries table
        CREATE TABLE IF NOT EXISTS dictionary_entries (
            id TEXT PRIMARY KEY,
            word TEXT NOT NULL,
            normalized_word TEXT NOT NULL,
            language TEXT NOT NULL,
            part_of_speech TEXT NOT NULL,
            definition TEXT NOT NULL,
            alternate_meanings_json TEXT NOT NULL,
            examples_json TEXT NOT NULL,
            pronunciation_ipa TEXT,
            transliteration TEXT,
            synonyms_json TEXT NOT NULL,
            antonyms_json TEXT NOT NULL,
            provenance TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_dict_lang_norm ON dictionary_entries(language, normalized_word);
        CREATE INDEX IF NOT EXISTS idx_dict_word_prefix ON dictionary_entries(word);

        -- FTS5 Full-Text Virtual Table
        CREATE VIRTUAL TABLE IF NOT EXISTS dictionary_fts USING fts5(
            word,
            normalized_word,
            definition,
            content='dictionary_entries',
            content_rowid='rowid'
        );

        -- Automated sync triggers
        CREATE TRIGGER IF NOT EXISTS dict_ai AFTER INSERT ON dictionary_entries BEGIN
            INSERT INTO dictionary_fts(rowid, word, normalized_word, definition)
            VALUES (new.rowid, new.word, new.normalized_word, new.definition);
        END;

        CREATE TRIGGER IF NOT EXISTS dict_ad AFTER DELETE ON dictionary_entries BEGIN
            INSERT INTO dictionary_fts(dictionary_fts, rowid, word, normalized_word, definition)
            VALUES('delete', old.rowid, old.word, old.normalized_word, old.definition);
        END;

        CREATE TRIGGER IF NOT EXISTS dict_au AFTER UPDATE ON dictionary_entries BEGIN
            INSERT INTO dictionary_fts(dictionary_fts, rowid, word, normalized_word, definition)
            VALUES('delete', old.rowid, old.word, old.normalized_word, old.definition);
            INSERT INTO dictionary_fts(rowid, word, normalized_word, definition)
            VALUES (new.rowid, new.word, new.normalized_word, new.definition);
        END;

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (2, datetime('now'), 'Stage 1: FTS5 Dictionary and Vocabulary Search Tables');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 002: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 002: {e}")))?;

    info!("Migration 002 applied successfully");
    Ok(())
}
