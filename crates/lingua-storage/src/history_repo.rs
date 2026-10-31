use chrono::{DateTime, Utc};
use lingua_core::{FavoriteItem, LanguageCode, LinguaError, TranslationRecord};
use rusqlite::{params, Connection};
use std::str::FromStr;

pub fn record_translation(
    conn: &Connection,
    record: &TranslationRecord,
) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO translation_history (
            id, source_language, target_language, source_text, translated_text,
            engine_used, latency_ms, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.source_language.as_bcp47(),
            record.target_language.as_bcp47(),
            record.source_text,
            record.translated_text,
            record.engine_used,
            record.latency_ms as i64,
            record.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to record translation: {e}")))?;

    Ok(())
}

pub fn list_translation_history(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<TranslationRecord>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, source_language, target_language, source_text, translated_text,
                    engine_used, latency_ms, created_at
             FROM translation_history
             ORDER BY created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![limit], |row| {
            let id: String = row.get(0)?;
            let src_str: String = row.get(1)?;
            let tgt_str: String = row.get(2)?;
            let source_text: String = row.get(3)?;
            let translated_text: String = row.get(4)?;
            let engine_used: String = row.get(5)?;
            let latency_ms: i64 = row.get(6)?;
            let created_str: String = row.get(7)?;

            let source_language = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target_language = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(TranslationRecord {
                id,
                source_language,
                target_language,
                source_text,
                translated_text,
                engine_used,
                latency_ms: latency_ms as u64,
                created_at,
            })
        })
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| LinguaError::Database(e.to_string()))?);
    }
    Ok(list)
}

pub fn clear_translation_history(conn: &Connection) -> Result<(), LinguaError> {
    conn.execute("DELETE FROM translation_history", [])
        .map_err(|e| LinguaError::Database(e.to_string()))?;
    Ok(())
}

pub fn add_favorite(conn: &Connection, item: &FavoriteItem) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO favorites (id, source_language, target_language, source_text, translated_text, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            notes = excluded.notes",
        params![
            item.id,
            item.source_language.as_bcp47(),
            item.target_language.as_bcp47(),
            item.source_text,
            item.translated_text,
            item.notes,
            item.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to add favorite: {e}")))?;

    // Automatically sync favorite to Bookmark-to-SM2 flashcard deck (Phase 14.2.1)
    let card_id = format!("fav-card-{}", item.id);
    let note_desc = item
        .notes
        .as_deref()
        .unwrap_or("Bookmarked from dictionary/translate");
    let _ = conn.execute(
        "INSERT INTO flashcards (
            id, deck_id, source_lang, target_lang, source_term, target_term,
            romanization, ipa, part_of_speech, example_source, example_target,
            grammar_notes, repetition, interval_days, ease_factor, next_review_at, total_reviews, created_at
        ) VALUES (
            ?1, 'deck-user-bookmarks', ?2, ?3, ?4, ?5,
            NULL, NULL, 'vocabulary', ?4, ?5,
            ?6, 0, 1.0, 2.5, ?7, 0, ?7
        ) ON CONFLICT(id) DO UPDATE SET
            source_term = excluded.source_term,
            target_term = excluded.target_term,
            grammar_notes = excluded.grammar_notes",
        params![
            card_id,
            item.source_language.as_bcp47(),
            item.target_language.as_bcp47(),
            item.source_text,
            item.translated_text,
            note_desc,
            item.created_at.to_rfc3339(),
        ],
    );

    Ok(())
}

pub fn remove_favorite(conn: &Connection, id: &str) -> Result<(), LinguaError> {
    conn.execute("DELETE FROM favorites WHERE id = ?1", params![id])
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    // Also remove from Bookmark-to-SM2 flashcards
    let card_id = format!("fav-card-{}", id);
    let _ = conn.execute("DELETE FROM flashcards WHERE id = ?1", params![card_id]);

    Ok(())
}

pub fn list_favorites(conn: &Connection) -> Result<Vec<FavoriteItem>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, source_language, target_language, source_text, translated_text, notes, created_at
             FROM favorites
             ORDER BY created_at DESC",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let src_str: String = row.get(1)?;
            let tgt_str: String = row.get(2)?;
            let source_text: String = row.get(3)?;
            let translated_text: String = row.get(4)?;
            let notes: Option<String> = row.get(5)?;
            let created_str: String = row.get(6)?;

            let source_language = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target_language = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(FavoriteItem {
                id,
                source_language,
                target_language,
                source_text,
                translated_text,
                notes,
                created_at,
            })
        })
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| LinguaError::Database(e.to_string()))?);
    }
    Ok(list)
}
