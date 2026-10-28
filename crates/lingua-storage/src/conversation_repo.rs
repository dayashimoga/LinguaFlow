use chrono::{DateTime, Utc};
use lingua_core::{
    ConversationSession, ConversationTurnItem, LanguageCode, LinguaError, SpeakerId,
};
use rusqlite::{params, Connection};
use std::str::FromStr;

pub fn create_conversation_session(
    conn: &Connection,
    session: &ConversationSession,
) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO conversation_sessions (id, title, speaker_a_lang, speaker_b_lang, active_speaker, is_hands_free, status, created_at, ended_at, total_turns)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            session.id,
            session.title,
            session.speaker_a_lang.as_str(),
            session.speaker_b_lang.as_str(),
            session.active_speaker.as_str(),
            session.is_hands_free as i32,
            session.status,
            session.created_at.to_rfc3339(),
            session.ended_at.map(|dt| dt.to_rfc3339()),
            session.total_turns,
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to create conversation session: {e}")))?;
    Ok(())
}

pub fn get_conversation_session(
    conn: &Connection,
    session_id: &str,
) -> Result<Option<ConversationSession>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, speaker_a_lang, speaker_b_lang, active_speaker, is_hands_free, status, created_at, ended_at, total_turns
             FROM conversation_sessions WHERE id = ?1",
        )
        .map_err(|e| LinguaError::Database(format!("Failed to prepare get_conversation_session query: {e}")))?;

    let mut rows = stmt
        .query(params![session_id])
        .map_err(|e| LinguaError::Database(format!("Failed to query conversation session: {e}")))?;

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
        let a_str: String = row
            .get(2)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let b_str: String = row
            .get(3)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let spk_str: String = row
            .get(4)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let is_hands_free_val: i32 = row
            .get(5)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let status: String = row
            .get(6)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let created_str: String = row
            .get(7)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let ended_str: Option<String> = row
            .get(8)
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        let total_turns: u32 = row
            .get(9)
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let speaker_a_lang = LanguageCode::parse(&a_str)?;
        let speaker_b_lang = LanguageCode::parse(&b_str)?;
        let active_speaker = SpeakerId::from_str(&spk_str)?;
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map_err(|e| LinguaError::Database(format!("Invalid created_at: {e}")))?
            .with_timezone(&Utc);
        let ended_at = match ended_str {
            Some(s) => Some(
                DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| LinguaError::Database(format!("Invalid ended_at: {e}")))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        Ok(Some(ConversationSession {
            id,
            title,
            speaker_a_lang,
            speaker_b_lang,
            active_speaker,
            is_hands_free: is_hands_free_val != 0,
            status,
            created_at,
            ended_at,
            total_turns,
        }))
    } else {
        Ok(None)
    }
}

pub fn update_conversation_session(
    conn: &Connection,
    session_id: &str,
    active_speaker: SpeakerId,
    status: &str,
    total_turns: u32,
    ended_at: Option<DateTime<Utc>>,
) -> Result<(), LinguaError> {
    conn.execute(
        "UPDATE conversation_sessions
         SET active_speaker = ?1, status = ?2, total_turns = ?3, ended_at = ?4
         WHERE id = ?5",
        params![
            active_speaker.as_str(),
            status,
            total_turns,
            ended_at.map(|dt| dt.to_rfc3339()),
            session_id,
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to update conversation session: {e}")))?;
    Ok(())
}

pub fn insert_conversation_turn(
    conn: &Connection,
    turn: &ConversationTurnItem,
) -> Result<(), LinguaError> {
    conn.execute(
        "INSERT INTO conversation_turns (id, session_id, turn_number, speaker_id, source_lang, target_lang, source_text, translated_text, audio_duration_ms, is_interrupted, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            turn.id,
            turn.session_id,
            turn.turn_number,
            turn.speaker_id.as_str(),
            turn.source_lang.as_str(),
            turn.target_lang.as_str(),
            turn.source_text,
            turn.translated_text,
            turn.audio_duration_ms,
            turn.is_interrupted as i32,
            turn.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to insert conversation turn: {e}")))?;
    Ok(())
}

pub fn get_conversation_turns(
    conn: &Connection,
    session_id: &str,
) -> Result<Vec<ConversationTurnItem>, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, turn_number, speaker_id, source_lang, target_lang, source_text, translated_text, audio_duration_ms, is_interrupted, created_at
             FROM conversation_turns
             WHERE session_id = ?1
             ORDER BY turn_number ASC",
        )
        .map_err(|e| LinguaError::Database(format!("Failed to prepare get_conversation_turns query: {e}")))?;

    let rows = stmt
        .query_map(params![session_id], |row| {
            let id: String = row.get(0)?;
            let session_id: String = row.get(1)?;
            let turn_number: u32 = row.get(2)?;
            let spk_str: String = row.get(3)?;
            let src_str: String = row.get(4)?;
            let tgt_str: String = row.get(5)?;
            let source_text: String = row.get(6)?;
            let translated_text: String = row.get(7)?;
            let audio_duration_ms: u64 = row.get(8)?;
            let is_interrupted_val: i32 = row.get(9)?;
            let created_str: String = row.get(10)?;

            Ok((
                id,
                session_id,
                turn_number,
                spk_str,
                src_str,
                tgt_str,
                source_text,
                translated_text,
                audio_duration_ms,
                is_interrupted_val,
                created_str,
            ))
        })
        .map_err(|e| LinguaError::Database(format!("Failed to query conversation turns: {e}")))?;

    let mut turns = Vec::new();
    for row_res in rows {
        let (
            id,
            session_id,
            turn_number,
            spk_str,
            src_str,
            tgt_str,
            source_text,
            translated_text,
            audio_duration_ms,
            is_interrupted_val,
            created_str,
        ) = row_res.map_err(|e| LinguaError::Database(e.to_string()))?;

        let speaker_id = SpeakerId::from_str(&spk_str)?;
        let source_lang = LanguageCode::parse(&src_str)?;
        let target_lang = LanguageCode::parse(&tgt_str)?;
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map_err(|e| LinguaError::Database(format!("Invalid created_at: {e}")))?
            .with_timezone(&Utc);

        turns.push(ConversationTurnItem {
            id,
            session_id,
            turn_number,
            speaker_id,
            source_lang,
            target_lang,
            source_text,
            translated_text,
            audio_duration_ms,
            is_interrupted: is_interrupted_val != 0,
            created_at,
        });
    }

    Ok(turns)
}

pub fn mark_turn_interrupted(conn: &Connection, turn_id: &str) -> Result<(), LinguaError> {
    conn.execute(
        "UPDATE conversation_turns SET is_interrupted = 1 WHERE id = ?1",
        params![turn_id],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to mark turn interrupted: {e}")))?;
    Ok(())
}
