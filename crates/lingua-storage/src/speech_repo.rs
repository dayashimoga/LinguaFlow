use chrono::{DateTime, Utc};
use lingua_core::{CaptionSegment, LanguageCode, LinguaError, SpeechSession, SpeechSessionStatus};
use rusqlite::{params, Connection};

pub fn create_speech_session(
    conn: &Connection,
    session: &SpeechSession,
) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO speech_sessions (id, title, source_lang, target_lang, status, created_at, ended_at, duration_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session.id,
            session.title,
            session.source_lang.as_str(),
            session.target_lang.as_str(),
            session.status.as_str(),
            session.created_at.to_rfc3339(),
            session.ended_at.map(|dt| dt.to_rfc3339()),
            session.duration_seconds,
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to create speech session: {e}")))?;
    Ok(())
}

pub fn update_speech_session_status(
    conn: &Connection,
    session_id: &str,
    status: SpeechSessionStatus,
    duration_seconds: u64,
    ended_at: Option<DateTime<Utc>>,
) -> Result<(), LinguaError> {
    conn.execute(
        "UPDATE speech_sessions
         SET status = ?1, duration_seconds = ?2, ended_at = ?3
         WHERE id = ?4",
        params![
            status.as_str(),
            duration_seconds,
            ended_at.map(|dt| dt.to_rfc3339()),
            session_id,
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to update speech session: {e}")))?;
    Ok(())
}

pub fn get_speech_session(
    conn: &Connection,
    session_id: &str,
) -> Result<Option<SpeechSession>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, source_lang, target_lang, status, created_at, ended_at, duration_seconds
             FROM speech_sessions WHERE id = ?1",
        )
        .map_err(|e| LinguaError::Database(format!("Failed to prepare get_speech_session query: {e}")))?;

    let mut rows = stmt
        .query(params![session_id])
        .map_err(|e| LinguaError::Database(format!("Failed to query speech session: {e}")))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| LinguaError::Database(e.to_string()))?
    {
        let id: String = row
            .get(0)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let title: String = row
            .get(1)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let source_str: String = row
            .get(2)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let target_str: String = row
            .get(3)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let status_str: String = row
            .get(4)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let created_str: String = row
            .get(5)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let ended_str: Option<String> = row
            .get(6)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let duration_seconds: u64 = row
            .get(7)
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let source_lang = LanguageCode::parse(&source_str)?;
        let target_lang = LanguageCode::parse(&target_str)?;
        let status = SpeechSessionStatus::parse(&status_str);
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| LinguaError::Database(format!("Failed to parse created_at: {e}")))?;
        let ended_at = ended_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Ok(Some(SpeechSession {
            id,
            title,
            source_lang,
            target_lang,
            status,
            created_at,
            ended_at,
            duration_seconds,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_speech_sessions(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SpeechSession>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, source_lang, target_lang, status, created_at, ended_at, duration_seconds
             FROM speech_sessions ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| LinguaError::Database(format!("Failed to prepare list_speech_sessions query: {e}")))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, u64>(7)?,
            ))
        })
        .map_err(|e| LinguaError::Database(format!("Failed to query speech sessions: {e}")))?;

    let mut sessions = Vec::new();
    for row in rows {
        let (
            id,
            title,
            source_str,
            target_str,
            status_str,
            created_str,
            ended_str,
            duration_seconds,
        ) = row.map_err(|e| LinguaError::Database(e.to_string()))?;

        let source_lang = LanguageCode::parse(&source_str)?;
        let target_lang = LanguageCode::parse(&target_str)?;
        let status = SpeechSessionStatus::parse(&status_str);
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| LinguaError::Database(format!("Failed to parse created_at: {e}")))?;
        let ended_at = ended_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        sessions.push(SpeechSession {
            id,
            title,
            source_lang,
            target_lang,
            status,
            created_at,
            ended_at,
            duration_seconds,
        });
    }

    Ok(sessions)
}

pub fn record_caption_segment(
    conn: &Connection,
    segment: &CaptionSegment,
) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO caption_segments (
            id, session_id, seq_id, source_text, translated_text, source_lang, target_lang,
            is_final, confidence, start_time_ms, end_time_ms, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(id) DO UPDATE SET
            source_text = excluded.source_text,
            translated_text = excluded.translated_text,
            is_final = excluded.is_final,
            confidence = excluded.confidence,
            end_time_ms = excluded.end_time_ms",
        params![
            segment.id,
            segment.session_id,
            segment.seq_id,
            segment.source_text,
            segment.translated_text,
            segment.source_lang.as_str(),
            segment.target_lang.as_str(),
            if segment.is_final { 1 } else { 0 },
            segment.confidence,
            segment.start_time_ms,
            segment.end_time_ms,
            segment.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to record caption segment: {e}")))?;
    Ok(())
}

pub fn get_caption_segments_for_session(
    conn: &Connection,
    session_id: &str,
) -> Result<Vec<CaptionSegment>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, seq_id, source_text, translated_text, source_lang, target_lang,
                    is_final, confidence, start_time_ms, end_time_ms, created_at
             FROM caption_segments
             WHERE session_id = ?1
             ORDER BY seq_id ASC",
        )
        .map_err(|e| LinguaError::Database(format!("Failed to prepare get_captions query: {e}")))?;

    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, f32>(8)?,
                row.get::<_, u64>(9)?,
                row.get::<_, u64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })
        .map_err(|e| LinguaError::Database(format!("Failed to query caption segments: {e}")))?;

    let mut segments = Vec::new();
    for row in rows {
        let (
            id,
            session_id,
            seq_id,
            source_text,
            translated_text,
            source_str,
            target_str,
            is_final_int,
            confidence,
            start_time_ms,
            end_time_ms,
            created_str,
        ) = row.map_err(|e| LinguaError::Database(e.to_string()))?;

        let source_lang = LanguageCode::parse(&source_str)?;
        let target_lang = LanguageCode::parse(&target_str)?;
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                LinguaError::Database(format!("Failed to parse caption created_at: {e}"))
            })?;

        segments.push(CaptionSegment {
            id,
            session_id,
            seq_id,
            source_text,
            translated_text,
            source_lang,
            target_lang,
            is_final: is_final_int == 1,
            confidence,
            start_time_ms,
            end_time_ms,
            created_at,
        });
    }

    Ok(segments)
}

pub fn delete_speech_session(conn: &Connection, session_id: &str) -> Result<(), LinguaError> {
    conn.execute(
        "DELETE FROM speech_sessions WHERE id = ?1",
        params![session_id],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to delete speech session: {e}")))?;
    Ok(())
}
