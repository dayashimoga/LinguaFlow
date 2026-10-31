use lingua_core::LinguaError;
use rusqlite::Connection;
use tracing::info;

pub const CURRENT_SCHEMA_VERSION: u32 = 12;

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
    if current_version < 3 {
        apply_migration_003(conn)?;
    }
    if current_version < 4 {
        apply_migration_004(conn)?;
    }
    if current_version < 5 {
        apply_migration_005(conn)?;
    }
    if current_version < 6 {
        apply_migration_006(conn)?;
    }
    if current_version < 7 {
        apply_migration_007(conn)?;
    }
    if current_version < 8 {
        apply_migration_008(conn)?;
    }
    if current_version < 9 {
        apply_migration_009(conn)?;
    }
    if current_version < 10 {
        apply_migration_010(conn)?;
    }
    if current_version < 11 {
        apply_migration_011(conn)?;
    }
    if current_version < 12 {
        apply_migration_012(conn)?;
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

fn apply_migration_003(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 003 — Offline Models, Cache & Morphology");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    // Safely add columns to model_packs if they don't already exist
    let existing_cols: Vec<String> = {
        let mut stmt = tx
            .prepare("PRAGMA table_info(model_packs)")
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        rows.filter_map(Result::ok).collect()
    };

    if !existing_cols.contains(&"install_path".to_string()) {
        tx.execute("ALTER TABLE model_packs ADD COLUMN install_path TEXT", [])
            .map_err(|e| {
                LinguaError::Database(format!("Failed to add install_path column: {e}"))
            })?;
    }
    if !existing_cols.contains(&"installed_at".to_string()) {
        tx.execute("ALTER TABLE model_packs ADD COLUMN installed_at TEXT", [])
            .map_err(|e| {
                LinguaError::Database(format!("Failed to add installed_at column: {e}"))
            })?;
    }
    if !existing_cols.contains(&"manifest_json".to_string()) {
        tx.execute("ALTER TABLE model_packs ADD COLUMN manifest_json TEXT", [])
            .map_err(|e| {
                LinguaError::Database(format!("Failed to add manifest_json column: {e}"))
            })?;
    }
    if !existing_cols.contains(&"status".to_string()) {
        tx.execute(
            "ALTER TABLE model_packs ADD COLUMN status TEXT NOT NULL DEFAULT 'not_installed'",
            [],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to add status column: {e}")))?;
    }
    if !existing_cols.contains(&"checksum_verified".to_string()) {
        tx.execute(
            "ALTER TABLE model_packs ADD COLUMN checksum_verified INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| {
            LinguaError::Database(format!("Failed to add checksum_verified column: {e}"))
        })?;
    }
    if !existing_cols.contains(&"architecture".to_string()) {
        tx.execute(
            "ALTER TABLE model_packs ADD COLUMN architecture TEXT NOT NULL DEFAULT 'indictrans2-quantized-onnx'",
            [],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to add architecture column: {e}")))?;
    }

    tx.execute_batch(
        "
        -- Model manifests signed metadata table
        CREATE TABLE IF NOT EXISTS model_manifests (
            id TEXT PRIMARY KEY,
            version TEXT NOT NULL,
            manifest_json TEXT NOT NULL,
            signature TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Offline translation cache table
        CREATE TABLE IF NOT EXISTS cached_translations (
            id TEXT PRIMARY KEY,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            source_text_hash TEXT NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            engine_id TEXT NOT NULL,
            is_offline INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            hit_count INTEGER NOT NULL DEFAULT 1
        );

        CREATE INDEX IF NOT EXISTS idx_cached_trans_lookup
        ON cached_translations(source_language, target_language, source_text_hash);

        -- Morphological resources & stem lookups
        CREATE TABLE IF NOT EXISTS morphological_stems (
            id TEXT PRIMARY KEY,
            language TEXT NOT NULL,
            surface_form TEXT NOT NULL,
            normalized_surface TEXT NOT NULL,
            stem_form TEXT NOT NULL,
            part_of_speech TEXT,
            grammar_notes TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_morph_lookup
        ON morphological_stems(language, normalized_surface);

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (3, datetime('now'), 'Stage 2: Offline model metadata, translation cache, and morphology stems');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 003: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 003: {e}")))?;

    info!("Migration 003 applied successfully");
    Ok(())
}

fn apply_migration_004(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 004 — Real-Time Speech Sessions and Captions");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS speech_sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            ended_at TEXT,
            duration_seconds INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS caption_segments (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            seq_id INTEGER NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            is_final INTEGER NOT NULL,
            confidence REAL NOT NULL,
            start_time_ms INTEGER NOT NULL,
            end_time_ms INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES speech_sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_caption_session_seq
        ON caption_segments(session_id, seq_id);

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (4, datetime('now'), 'Stage 3: Real-Time Speech sessions and caption segments');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 004: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 004: {e}")))?;

    info!("Migration 004 applied successfully");
    Ok(())
}

fn apply_migration_005(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 005 — Real-Time Voice Conversation Sessions and Turns");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conversation_sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            speaker_a_lang TEXT NOT NULL,
            speaker_b_lang TEXT NOT NULL,
            active_speaker TEXT NOT NULL,
            is_hands_free INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            ended_at TEXT,
            total_turns INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS conversation_turns (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            turn_number INTEGER NOT NULL,
            speaker_id TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            audio_duration_ms INTEGER NOT NULL DEFAULT 0,
            is_interrupted INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES conversation_sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_conv_turns_session_turn
        ON conversation_turns(session_id, turn_number);

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (5, datetime('now'), 'Stage 4: Real-Time Voice Conversation sessions and turns');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 005: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 005: {e}")))?;

    info!("Migration 005 applied successfully");
    Ok(())
}

fn apply_migration_006(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 006 — Translation Intelligence, Glossaries & Circuit Breakers");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS domain_glossaries (
            id TEXT PRIMARY KEY,
            domain TEXT NOT NULL,
            source_term TEXT NOT NULL,
            target_term TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            is_exact INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_glossary_domain_lang
        ON domain_glossaries(domain, source_lang, target_lang);

        CREATE INDEX IF NOT EXISTS idx_glossary_source_term
        ON domain_glossaries(source_term);

        CREATE TABLE IF NOT EXISTS evaluation_reports (
            id TEXT PRIMARY KEY,
            report_json TEXT NOT NULL,
            mean_chrf REAL NOT NULL,
            mean_wer REAL NOT NULL,
            mean_cer REAL NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS circuit_breaker_metrics (
            provider_id TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            failure_count INTEGER NOT NULL DEFAULT 0,
            success_count INTEGER NOT NULL DEFAULT 0,
            total_requests INTEGER NOT NULL DEFAULT 0,
            last_failure_ms INTEGER,
            updated_at TEXT NOT NULL
        );

        -- Seed initial technical and polite greeting domain glossary terms
        INSERT OR IGNORE INTO domain_glossaries (id, domain, source_term, target_term, source_lang, target_lang, is_exact, created_at)
        VALUES
            ('glossary-tech-kn-1', 'technology', 'computer', 'ಗಣಕಯಂತ್ರ', 'en-US', 'kn-IN', 1, datetime('now')),
            ('glossary-tech-kn-2', 'technology', 'software', 'ತಂತ್ರಾಂಶ', 'en-US', 'kn-IN', 1, datetime('now')),
            ('glossary-tech-kn-3', 'technology', 'database', 'ದತ್ತಸಂಚಯ', 'en-US', 'kn-IN', 1, datetime('now')),
            ('glossary-tech-ta-1', 'technology', 'computer', 'கணினி', 'en-US', 'ta-IN', 1, datetime('now')),
            ('glossary-tech-ta-2', 'technology', 'software', 'மென்பொருள்', 'en-US', 'ta-IN', 1, datetime('now')),
            ('glossary-tech-ta-3', 'technology', 'database', 'தரவுத்தளம்', 'en-US', 'ta-IN', 1, datetime('now')),
            ('glossary-med-kn-1', 'medical', 'hospital', 'ಆಸ್ಪತ್ರೆ', 'en-US', 'kn-IN', 1, datetime('now')),
            ('glossary-med-ta-1', 'medical', 'hospital', 'மருத்துவமனை', 'en-US', 'ta-IN', 1, datetime('now'));

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (6, datetime('now'), 'Stage 5: Translation Intelligence, Glossaries, Evaluation Reports and Circuit Breakers');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 006: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 006: {e}")))?;

    info!("Migration 006 applied successfully");
    Ok(())
}

fn apply_migration_007(conn: &mut Connection) -> Result<(), LinguaError> {
    info!(
        "Applying LinguaFlow Database Migration 007 — Media, Subtitles, Overlay & Document Storage"
    );

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- Media transcription sessions
        CREATE TABLE IF NOT EXISTS media_sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            format TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Media subtitle segments
        CREATE TABLE IF NOT EXISTS media_segments (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            seq_id INTEGER NOT NULL,
            start_ms INTEGER NOT NULL,
            end_ms INTEGER NOT NULL,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES media_sessions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_media_segments_session_seq
        ON media_segments(session_id, seq_id);

        -- Live caption overlay settings
        CREATE TABLE IF NOT EXISTS overlay_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            enabled INTEGER NOT NULL DEFAULT 1,
            always_on_top INTEGER NOT NULL DEFAULT 1,
            x INTEGER NOT NULL DEFAULT 100,
            y INTEGER NOT NULL DEFAULT 100,
            width INTEGER NOT NULL DEFAULT 700,
            height INTEGER NOT NULL DEFAULT 140,
            font_size INTEGER NOT NULL DEFAULT 22,
            font_family TEXT NOT NULL,
            text_color TEXT NOT NULL,
            background_color TEXT NOT NULL,
            opacity REAL NOT NULL DEFAULT 0.88,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            show_source INTEGER NOT NULL DEFAULT 1,
            show_translated INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL
        );

        -- Document translation jobs
        CREATE TABLE IF NOT EXISTS document_jobs (
            id TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            format TEXT NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            total_blocks INTEGER NOT NULL,
            completed_blocks INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Insert default overlay settings row
        INSERT OR IGNORE INTO overlay_settings (
            id, enabled, always_on_top, x, y, width, height, font_size,
            font_family, text_color, background_color, opacity,
            source_lang, target_lang, show_source, show_translated, updated_at
        ) VALUES (
            1, 1, 1, 100, 100, 700, 140, 22,
            'Inter, Noto Sans Kannada, Noto Sans Tamil, sans-serif', '#ffffff', '#0a0d14', 0.88,
            'en-US', 'kn-IN', 1, 1, datetime('now')
        );

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (7, datetime('now'), 'Stage 6: Media Sessions, Subtitle Segments, Overlay Settings, and Document Jobs');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 007: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 007: {e}")))?;

    info!("Migration 007 applied successfully");
    Ok(())
}

fn apply_migration_008(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 008 — Language Learning & Privacy Settings");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- Learning decks
        CREATE TABLE IF NOT EXISTS learning_decks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Flashcards with SM-2 state tracking
        CREATE TABLE IF NOT EXISTS flashcards (
            id TEXT PRIMARY KEY,
            deck_id TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            source_term TEXT NOT NULL,
            target_term TEXT NOT NULL,
            romanization TEXT,
            ipa TEXT,
            part_of_speech TEXT NOT NULL,
            example_source TEXT NOT NULL,
            example_target TEXT NOT NULL,
            grammar_notes TEXT,
            repetition INTEGER NOT NULL DEFAULT 0,
            interval_days REAL NOT NULL DEFAULT 1.0,
            ease_factor REAL NOT NULL DEFAULT 2.5,
            next_review_at TEXT NOT NULL,
            total_reviews INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (deck_id) REFERENCES learning_decks(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_flashcards_deck_next_review
        ON flashcards(deck_id, next_review_at);

        CREATE INDEX IF NOT EXISTS idx_flashcards_next_review
        ON flashcards(next_review_at);

        -- SM-2 Review history logs
        CREATE TABLE IF NOT EXISTS learning_review_logs (
            id TEXT PRIMARY KEY,
            card_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            repetition INTEGER NOT NULL,
            interval_days REAL NOT NULL,
            ease_factor REAL NOT NULL,
            reviewed_at TEXT NOT NULL,
            FOREIGN KEY (card_id) REFERENCES flashcards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_review_logs_card_id
        ON learning_review_logs(card_id);

        -- Privacy consent and local data retention settings
        CREATE TABLE IF NOT EXISTS privacy_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            analytics_enabled INTEGER NOT NULL DEFAULT 0,
            local_transcripts_retention_days INTEGER NOT NULL DEFAULT 30,
            microphone_consent INTEGER NOT NULL DEFAULT 1,
            camera_consent INTEGER NOT NULL DEFAULT 1,
            cloud_processing_consent INTEGER NOT NULL DEFAULT 0,
            policy_version_accepted TEXT NOT NULL DEFAULT '2026.10-v1',
            updated_at TEXT NOT NULL
        );

        -- Default privacy settings
        INSERT OR IGNORE INTO privacy_settings (
            id, analytics_enabled, local_transcripts_retention_days,
            microphone_consent, camera_consent, cloud_processing_consent,
            policy_version_accepted, updated_at
        ) VALUES (
            1, 0, 30, 1, 1, 0, '2026.10-v1', datetime('now')
        );

        -- Seed initial learning decks
        INSERT OR IGNORE INTO learning_decks (id, title, description, source_lang, target_lang, created_at)
        VALUES 
        ('deck-kn-core', 'Kannada Essential Vocabulary & Grammar', 'Core expressions, case markers (Vibhakti), and daily vocabulary', 'kn-IN', 'en-US', datetime('now')),
        ('deck-ta-core', 'Tamil Essential Vocabulary & Grammar', 'Core expressions, case markers (Vēṟṟumai), and daily vocabulary', 'ta-IN', 'en-US', datetime('now'));

        -- Seed Kannada flashcards
        INSERT OR IGNORE INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES
        ('card-kn-01', 'deck-kn-core', 'kn-IN', 'en-US', 'ನಮಸ್ಕಾರ', 'Hello / Greetings',
         'Namaskāra', '/nʌmʌskɑːrʌ/', 'interjection', 'ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?', 'Hello, how are you?',
         'Indeclinable (ಅವ್ಯಯ - Avyaya); standard polite greeting for all times of day.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-kn-02', 'deck-kn-core', 'kn-IN', 'en-US', 'ಧನ್ಯವಾದ', 'Thank you',
         'Dhanyavāda', '/d̪ʱʌnjʌʋɑːd̪ʌ/', 'noun', 'ನಿಮ್ಮ ಸಹಾಯಕ್ಕೆ ಧನ್ಯವಾದಗಳು.', 'Thank you for your assistance.',
         'Root Sanskrit loanword; often pluralized as ಧನ್ಯವಾದಗಳು (Dhanyavādagaḷu).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-kn-03', 'deck-kn-core', 'kn-IN', 'en-US', 'ಮನೆ', 'House / Home',
         'Mane', '/mʌne/', 'noun', 'ನಮ್ಮ ಮನೆ ಬೆಂಗಳೂರಿನಲ್ಲಿದೆ.', 'Our house is in Bengaluru.',
         'Accusative (ದ್ವಿತೀಯಾ): ಮನೆಯನ್ನು; Genitive (ಷಷ್ಠೀ): ಮನೆಯ; Locative (ಸಪ್ತಮೀ): ಮನೆಯಲ್ಲಿ.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-kn-04', 'deck-kn-core', 'kn-IN', 'en-US', 'ನೀರು', 'Water',
         'Nīru', '/niːru/', 'noun', 'ದಯವಿಟ್ಟು ಕುಡಿಯಲು ನೀರು ಕೊಡಿ.', 'Please give water to drink.',
         'Neuter non-rational noun; Dative (ಚತುರ್ಥೀ): ನೀರಿಗೆ (nīrige).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-kn-05', 'deck-kn-core', 'kn-IN', 'en-US', 'ಪುಸ್ತಕ', 'Book',
         'Pustaka', '/pust̪ʌkʌ/', 'noun', 'ನಾನು ಹೊಸ ಪುಸ್ತಕವನ್ನು ಓದುತ್ತಿದ್ದೇನೆ.', 'I am reading a new book.',
         'Plural suffix: -ಗಳು (Pustakagaḷu); Instrumental (ತೃತೀಯಾ): ಪುಸ್ತಕದಿಂದ (from the book).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now'));

        -- Seed Tamil flashcards
        INSERT OR IGNORE INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES
        ('card-ta-01', 'deck-ta-core', 'ta-IN', 'en-US', 'வணக்கம்', 'Hello / Greetings',
         'Vaṇakkam', '/ʋɐɳɐkːɐm/', 'interjection', 'வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?', 'Hello, how are you?',
         'Universal traditional Tamil greeting; literally means reverence or bowing.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ta-02', 'deck-ta-core', 'ta-IN', 'en-US', 'நன்றி', 'Thank you',
         'Naṉṟi', '/n̪ɐndri/', 'noun', 'உங்கள் உதவிக்கு மிக்க நன்றி.', 'Thank you very much for your help.',
         'Traditional Dravidian noun expressing gratitude; combined with மிக்க (mikka - much).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ta-03', 'deck-ta-core', 'ta-IN', 'en-US', 'வீடு', 'House / Home',
         'Vīṭu', '/ʋiːɖɯ/', 'noun', 'எங்கள் வீடு சென்னையில் உள்ளது.', 'Our house is in Chennai.',
         'Accusative (இரண்டாம் வேற்றுமை): வீட்டை; Dative (நான்காம்): வீட்டிற்கு; Locative (ஏழாம்): வீட்டில்.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ta-04', 'deck-ta-core', 'ta-IN', 'en-US', 'தண்ணீர்', 'Water',
         'Taṇṇīr', '/t̪ɐɳːiːr/', 'noun', 'குடிப்பதற்கு தண்ணீர் கொடுங்கள்.', 'Please give water to drink.',
         'Compound of தண் (cold/pure) + நீர் (water); Dative: தண்ணீருக்கு.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ta-05', 'deck-ta-core', 'ta-IN', 'en-US', 'புத்தகம்', 'Book',
         'Puttakam', '/put̪ːɐɡɐm/', 'noun', 'நான் ஒரு நல்ல புத்தகத்தை படிக்கிறேன்.', 'I am reading a good book.',
         'Plural suffix: -கள் (Puttakangkaḷ); Instrumental (மூன்றாம் வேற்றுமை): புத்தகத்தால்.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now'));

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (8, datetime('now'), 'Stage 7: Spaced Repetition Learning Decks, Flashcards, Review Logs, and Privacy Settings');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 008: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 008: {e}")))?;

    info!("Migration 008 applied successfully");
    Ok(())
}

fn apply_migration_009(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 009 — Commercial & Enterprise Platform");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- User accounts & entitlements (Phase 8.1)
        CREATE TABLE IF NOT EXISTS user_accounts (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'free',
            is_anonymous INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL
        );

        -- Device registrations (Phase 8.1)
        CREATE TABLE IF NOT EXISTS device_registrations (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
            device_name TEXT NOT NULL,
            device_type TEXT NOT NULL,
            public_key_fingerprint TEXT NOT NULL,
            registered_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL
        );

        -- Organizations & workspaces (Phase 8.3)
        CREATE TABLE IF NOT EXISTS organizations (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            max_seats INTEGER NOT NULL DEFAULT 10,
            sso_enabled INTEGER NOT NULL DEFAULT 0,
            data_retention_days INTEGER NOT NULL DEFAULT 90,
            enforce_zero_transcript_logging INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        -- Organization team members & RBAC (Phase 8.3)
        CREATE TABLE IF NOT EXISTS org_members (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
            email TEXT NOT NULL,
            display_name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            joined_at TEXT NOT NULL,
            UNIQUE(org_id, user_id)
        );

        -- Developer API keys & HMAC credentials (Phase 8.4)
        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
            org_id TEXT REFERENCES organizations(id) ON DELETE CASCADE,
            key_hash TEXT NOT NULL UNIQUE,
            key_prefix TEXT NOT NULL,
            name TEXT NOT NULL,
            rate_limit_per_min INTEGER NOT NULL DEFAULT 60,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            expires_at TEXT
        );

        -- Shared organization glossaries & term enforcement (Phase 8.3)
        CREATE TABLE IF NOT EXISTS org_glossaries (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'approved',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS org_glossary_terms (
            id TEXT PRIMARY KEY,
            glossary_id TEXT NOT NULL REFERENCES org_glossaries(id) ON DELETE CASCADE,
            source_term TEXT NOT NULL,
            target_term TEXT NOT NULL,
            notes TEXT,
            case_sensitive INTEGER NOT NULL DEFAULT 0,
            UNIQUE(glossary_id, source_term)
        );

        -- Usage metering & cost tracking (Phase 8.2)
        CREATE TABLE IF NOT EXISTS usage_records (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
            org_id TEXT REFERENCES organizations(id) ON DELETE CASCADE,
            usage_type TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            billable_cents INTEGER NOT NULL DEFAULT 0,
            was_successful INTEGER NOT NULL DEFAULT 1,
            timestamp TEXT NOT NULL
        );

        -- Billing invoices & payment records (Phase 8.2)
        CREATE TABLE IF NOT EXISTS billing_invoices (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
            org_id TEXT REFERENCES organizations(id) ON DELETE CASCADE,
            invoice_number TEXT NOT NULL UNIQUE,
            billing_period TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            status TEXT NOT NULL DEFAULT 'paid',
            line_items_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- Auditable administrative actions & SOC2 log (Phase 8.3)
        CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY,
            org_id TEXT REFERENCES organizations(id) ON DELETE CASCADE,
            actor_id TEXT NOT NULL,
            actor_email TEXT NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            details TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            timestamp TEXT NOT NULL
        );

        -- Performance Indexes
        CREATE INDEX IF NOT EXISTS idx_usage_records_account ON usage_records(account_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_events_org ON audit_events(org_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_org_members_org ON org_members(org_id);
        CREATE INDEX IF NOT EXISTS idx_org_glossary_terms ON org_glossary_terms(glossary_id, source_term);
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);

        -- Pre-seed default anonymous local user account (Anonymous Offline Mode Guarantee - REQ-8.1-AUTH)
        INSERT OR IGNORE INTO user_accounts (id, email, display_name, tier, is_anonymous, status, created_at)
        VALUES ('acc-anonymous-local', 'anonymous@local.linguaflow', 'Local Anonymous User', 'free', 1, 'active', datetime('now'));

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (9, datetime('now'), 'Stage 8: Commercial & Enterprise Platform (Accounts, Devices, Orgs, Glossaries, Usage, Billing, Audit)');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 009: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 009: {e}")))?;

    info!("Migration 009 applied successfully");
    Ok(())
}

fn apply_migration_010(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 010 — Rich Lexicon Schema & Indic FTS5 unicode61 tokenizer");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- Rich Lexicon Entries table (Stage 9: Linguistic Data Integrity)
        CREATE TABLE IF NOT EXISTS rich_lexicon_entries (
            id TEXT PRIMARY KEY,
            headword TEXT NOT NULL,
            normalized_stem TEXT NOT NULL,
            language_code TEXT NOT NULL,
            ipa_pronunciation TEXT,
            native_phonetic_respelling TEXT,
            part_of_speech TEXT NOT NULL,
            gender_inflection_notes TEXT,
            register TEXT NOT NULL DEFAULT 'general',
            senses_json TEXT NOT NULL,
            bilingual_examples_json TEXT NOT NULL,
            synonyms_json TEXT NOT NULL,
            antonyms_json TEXT NOT NULL,
            derived_forms_json TEXT NOT NULL,
            target_equivalents_json TEXT NOT NULL,
            frequency_rank INTEGER DEFAULT 1000,
            provenance TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_rich_lex_lang_stem ON rich_lexicon_entries(language_code, normalized_stem);
        CREATE INDEX IF NOT EXISTS idx_rich_lex_headword ON rich_lexicon_entries(headword);
        CREATE INDEX IF NOT EXISTS idx_rich_lex_pos ON rich_lexicon_entries(part_of_speech);
        CREATE INDEX IF NOT EXISTS idx_rich_lex_register ON rich_lexicon_entries(register);

        -- FTS5 Virtual Table with unicode61 tokenizer preserving Indic virama (tokenchars=್்)
        CREATE VIRTUAL TABLE IF NOT EXISTS rich_dictionary_fts USING fts5(
            headword,
            normalized_stem,
            native_phonetic_respelling,
            senses_text,
            tokenize = 'unicode61 tokenchars ''್்'''
        );

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (10, datetime('now'), 'Stage 9: Rich Lexicon Schema, Indic FTS5 unicode61 tokenizer & Trilingual Synsets');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 010: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 010: {e}")))?;

    info!("Migration 010 applied successfully");
    Ok(())
}

fn apply_migration_011(conn: &mut Connection) -> Result<(), LinguaError> {
    info!("Applying LinguaFlow Database Migration 011 — Developer HUD & Settings Polish");

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        ALTER TABLE user_settings ADD COLUMN show_developer_hud INTEGER NOT NULL DEFAULT 0;

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (11, datetime('now'), 'Stage 13: Developer HUD Toggle & UI Settings Polish');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 011: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 011: {e}")))?;

    info!("Migration 011 applied successfully");
    Ok(())
}

fn apply_migration_012(conn: &mut Connection) -> Result<(), LinguaError> {
    info!(
        "Applying LinguaFlow Database Migration 012 — Persona Learning Decks & User Bookmark Deck"
    );

    let tx = conn
        .transaction()
        .map_err(|e| LinguaError::Database(format!("Failed to begin transaction: {e}")))?;

    tx.execute_batch(
        "
        -- Seed Persona Decks (Phase 14.2.1)
        INSERT OR IGNORE INTO learning_decks (id, title, description, source_lang, target_lang, created_at)
        VALUES 
        ('deck-persona-ananya', 'Healthcare & Medical Triage (Dr. Ananya)', 'Clinical anamnesis, vital symptoms, emergency triage, dosage instructions in Kannada, Tamil, and English', 'kn-IN', 'en-US', datetime('now')),
        ('deck-persona-murugan', 'Interstate Logistics & Freight (Murugan)', 'Waybill manifests, toll clearance, cargo loading, route navigation, axle weight permits across Karnataka and Tamil Nadu', 'ta-IN', 'en-US', datetime('now')),
        ('deck-persona-vikram', 'Kannada & Tamil Literature & Grammar (Vikram)', 'Classical aesthetics, Tolkappiyam & Kavirajamarga rules, Sandhi junctions, Sangam & Halegannada poetry', 'kn-IN', 'ta-IN', datetime('now')),
        ('deck-user-bookmarks', 'Bookmarked Vocabulary (Personal Favorites)', 'Personalized flashcards dynamically synchronized from saved Dictionary and Translation favorites', 'kn-IN', 'en-US', datetime('now'));

        -- Seed Dr. Ananya Healthcare flashcards
        INSERT OR IGNORE INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES
        ('card-ananya-01', 'deck-persona-ananya', 'kn-IN', 'en-US', 'ಜ್ವರ', 'Fever / Pyrexia',
         'Jvara', '/dʒʋʌrʌ/', 'noun', 'ರೋಗಿಗೆ ತೀವ್ರ ಜ್ವರ ಮತ್ತು ತಲೆನೋವು ಇದೆ.', 'The patient has severe fever and headache.',
         'Clinical triage indicator; neuter noun; Dative: ಜ್ವರಕ್ಕೆ (jvarakke).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ananya-02', 'deck-persona-ananya', 'kn-IN', 'en-US', 'ಎದೆ ನೋವು', 'Chest pain / Angina',
         'Ede nōvu', '/ede noːʋu/', 'noun phrase', 'ಎದೆ ನೋವು ಇದ್ದರೆ ತಕ್ಷಣ ಇಸಿಜಿ ಪರೀಕ್ಷೆ ಮಾಡಿ.', 'If chest pain is present, perform an ECG test immediately.',
         'Red flag triage condition; Locative: ಎದೆಯಲ್ಲಿ (edeyalli).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ananya-03', 'deck-persona-ananya', 'kn-IN', 'en-US', 'ಉಸಿರಾಟದ ತೊಂದರೆ', 'Shortness of breath / Dyspnea',
         'Usirāṭada tondare', '/usirɑːʈʌd̪ʌ t̪ond̪ʌre/', 'noun phrase', 'ಉಸಿರಾಟದ ತೊಂದರೆ ಇರುವ ರೋಗಿಗೆ ಆಮ್ಲಜನಕ ನೀಡಿ.', 'Administer oxygen to the patient with respiratory distress.',
         'Compound of ಉಸಿರಾಟದ (respiratory genitive) + ತೊಂದರೆ (difficulty).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ananya-04', 'deck-persona-ananya', 'kn-IN', 'en-US', 'ರಕ್ತದೊತ್ತಡ', 'Blood pressure / Hypertension',
         'Raktadottada', '/rʌkt̪ʌd̪ot̪ːʌɖʌ/', 'noun', 'ರಕ್ತದೊತ್ತಡ ಸಾಮಾನ್ಯ ಮಟ್ಟಕ್ಕಿಂತ ಹೆಚ್ಚಾಗಿದೆ.', 'Blood pressure is elevated above normal levels.',
         'Medical compound: ರಕ್ತ (blood) + ಒತ್ತಡ (pressure); Accusative: ರಕ್ತದೊತ್ತಡವನ್ನು.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-ananya-05', 'deck-persona-ananya', 'kn-IN', 'en-US', 'ಮಾತ್ರೆ', 'Tablet / Medication pill',
         'Mātre', '/mɑːt̪re/', 'noun', 'ಊಟದ ನಂತರ ಈ ಮಾತ್ರೆಯನ್ನು ದಿನಕ್ಕೆ ಎರಡು ಬಾರಿ ತೆಗೆದುಕೊಳ್ಳಿ.', 'Take this tablet twice daily after meals.',
         'Dosage instruction; Plural: ಮಾತ್ರೆಗಳು (mātregaḷu); Instrumental: ಮಾತ್ರೆಯಿಂದ.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now'));

        -- Seed Murugan Logistics flashcards
        INSERT OR IGNORE INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES
        ('card-murugan-01', 'deck-persona-murugan', 'ta-IN', 'en-US', 'சரக்கு', 'Cargo / Freight',
         'Carakku', '/sɐrɐkːɯ/', 'noun', 'சரக்கு வாகனம் நாளை காலை பெங்களூரு சென்றடையும்.', 'The freight vehicle will reach Bengaluru tomorrow morning.',
         'Dravidian logistics root; Accusative: சரக்கை (carakkai); Dative: சரக்கிற்கு.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-murugan-02', 'deck-persona-murugan', 'ta-IN', 'en-US', 'சுங்கச்சாவடி', 'Toll Plaza / Checkpoint',
         'Cuṅkaccāvaṭi', '/suŋɡɐtː͡ʃɑːʋɐɖi/', 'noun', 'அடுத்த சுங்கச்சாவடியில் மின்னணு கட்டணம் செலுத்த வேண்டும்.', 'Electronic toll must be paid at the next toll plaza.',
         'Compound of சுங்கம் (toll/customs) + சாவடி (post/station).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-murugan-03', 'deck-persona-murugan', 'ta-IN', 'en-US', 'எடை மேடை', 'Weighbridge / Axle scale',
         'Eṭai mēṭai', '/eɖɐi meːɖɐi/', 'noun phrase', 'லாரியின் மொத்த எடையை எடை மேடையில் சரிபார்க்கவும்.', 'Verify the total gross weight of the truck on the weighbridge.',
         'Freight compliance term; Locative: எடை மேடையில் (eṭai mēṭaiyil).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-murugan-04', 'deck-persona-murugan', 'ta-IN', 'en-US', 'வழித்தட அனுமதி', 'Route permit / Transit clearance',
         'Vaḻittaṭa aṉumati', '/ʋɐɻit̪ːɐɖɐ ɐn̪umɐd̪i/', 'noun phrase', 'மாநிலங்களுக்கு இடையேயான வழித்தட அனுமதி தயார் நிலையில் உள்ளது.', 'The interstate route transit permit is ready.',
         'Compound of வழித்தட (route genitive) + அனுமதி (permission/clearance).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-murugan-05', 'deck-persona-murugan', 'ta-IN', 'en-US', 'விநியோக ரசீது', 'Delivery receipt / Proof of Delivery (POD)',
         'Viṉiyōka racītu', '/ʋin̪ijoːɡɐ rɐsiːd̪ɯ/', 'noun phrase', 'சரக்கை இறக்கிய பின் விநியோக ரசீதில் கையெழுத்து பெறவும்.', 'Obtain signature on the delivery receipt after unloading cargo.',
         'Commercial freight documentation; Instrumental: ரசீதால்; Locative: ரசீதில்.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now'));

        -- Seed Vikram Literature flashcards
        INSERT OR IGNORE INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES
        ('card-vikram-01', 'deck-persona-vikram', 'kn-IN', 'ta-IN', 'ಸಂಸ್ಕೃತಿ', 'பண்பாடு (Culture / Heritage)',
         'Saṁskr̥ti', '/sʌmskɾut̪i/', 'noun', 'ಕನ್ನಡ ಮತ್ತು ತಮಿಳು ಶ್ರೀಮಂತ ಸಂಸ್ಕೃತಿಯನ್ನು ಹೊಂದಿವೆ.', 'கன்னடமும் தமிழும் செழுமையான பண்பாட்டைக் கொண்டுள்ளன.',
         'High-register cultural cognate; Accusative: ಸಂಸ್ಕೃತಿಯನ್ನು / பண்பாட்டை.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-vikram-02', 'deck-persona-vikram', 'kn-IN', 'ta-IN', 'ಕಾವ್ಯ', 'காவியம் (Epic Poetry)',
         'Kāvya', '/kɑːʋjʌ/', 'noun', 'ಪಂಪನ ವಿಕ್ರಮಾರ್ಜುನ ವಿಜಯವು ಪ್ರಸಿದ್ಧ ಕನ್ನಡ ಕಾವ್ಯವಾಗಿದೆ.', 'பம்பனின் விக்ரமார்ஜுன விஜயம் புகழ்பெற்ற கன்னடக் காவியமாகும்.',
         'Classical poetic genre; Sanskrit loan preserved across Halegannada and Classical Tamil.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-vikram-03', 'deck-persona-vikram', 'kn-IN', 'ta-IN', 'ಸಂಧಿ', 'புணர்ச்சி (Sandhi / Euphonic Junction)',
         'Sandhi', '/sʌnd̪ʱi/', 'noun', 'ಕೇಶಿರಾಜನ ಶಬ್ದಮಣಿದರ್ಪಣದಲ್ಲಿ ಸಂಧಿ ನಿಯಮಗಳನ್ನು ವಿವರಿಸಲಾಗಿದೆ.', 'தொல்காப்பியத்தில் புணர்ச்சி விதிகள் விரிவாக விளக்கப்பட்டுள்ளன.',
         'Phonological junction rule (Lopa, Agama, Adesha in Kannada; Meyppadu, Udanpadumey in Tamil).', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-vikram-04', 'deck-persona-vikram', 'kn-IN', 'ta-IN', 'ಛಂದಸ್ಸು', 'யாப்பிலக்கணம் (Prosody / Metrical Grammar)',
         'Chandassu', '/t͡ʃʰʌnd̪ʌsːu/', 'noun', 'ಕಂದ ಪದ್ಯ ಮತ್ತು ರಗಳೆ ಕನ್ನಡ ಛಂದಸ್ಸಿನ ವೈಶಿಷ್ಟ್ಯಗಳಾಗಿವೆ.', 'வெண்பா மற்றும் ஆசிரியப்பா தமிழ் யாப்பிலக்கணத்தின் சிறப்புகளாகும்.',
         'Metrical framework; Halegannada Shatpadi/Tripadi vs Tamil Asiriyappa/Venba.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now')),
        ('card-vikram-05', 'deck-persona-vikram', 'kn-IN', 'ta-IN', 'ರಸಧ್ವನಿ', 'மெய்ப்பாடு (Aesthetic Sentiment / Rasa)',
         'Rasadhvani', '/rʌsʌd̪ʱʋʌni/', 'noun', 'ಉತ್ತಮ ಸಾಹಿತ್ಯದಲ್ಲಿ ರಸಧ್ವನಿಯು ಹೃದಯಸ್ಪರ್ಶಿಯಾಗಿರುತ್ತದೆ.', 'சிறந்த இலக்கியத்தில் மெய்ப்பாடு உள்ளத்தை உருக்குவதாக அமைகிறது.',
         'Aesthetic poetics; Anandavardhana Dhvanyaloka vs Tolkappiyam Meyppattiyal.', 0, 1.0, 2.5, datetime('now'), 0, datetime('now'));

        INSERT INTO schema_migrations (version, applied_at, description)
        VALUES (12, datetime('now'), 'Stage 14: Persona Learning Decks (Ananya, Murugan, Vikram) and User Bookmark Deck');
        ",
    )
    .map_err(|e| LinguaError::Database(format!("Failed to execute migration 012: {e}")))?;

    tx.commit()
        .map_err(|e| LinguaError::Database(format!("Failed to commit migration 012: {e}")))?;

    info!("Migration 012 applied successfully");
    Ok(())
}
