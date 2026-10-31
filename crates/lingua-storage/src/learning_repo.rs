use chrono::{DateTime, Utc};
use lingua_core::{
    DataPrivacySettings, ExportUserDataArchive, LanguageCode, LearnerProgress, LearningCard,
    LearningDeck, LinguaError, PurgeUserDataResult, ReviewResult,
};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct LearningRepository {
    conn: Arc<Mutex<Connection>>,
}

impl LearningRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn list_learning_decks(&self) -> Result<Vec<LearningDeck>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT d.id, d.title, d.description, d.source_lang, d.target_lang, d.created_at,
                        (SELECT COUNT(*) FROM flashcards f WHERE f.deck_id = d.id) AS total_cards,
                        (SELECT COUNT(*) FROM flashcards f WHERE f.deck_id = d.id AND f.next_review_at <= datetime('now')) AS due_cards
                 FROM learning_decks d
                 ORDER BY d.created_at ASC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let description: String = row.get(2)?;
                let s_lang: String = row.get(3)?;
                let t_lang: String = row.get(4)?;
                let created_at_str: String = row.get(5)?;
                let total_cards: usize = row.get(6)?;
                let due_cards: usize = row.get(7)?;

                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(LearningDeck {
                    id,
                    title,
                    description,
                    source_lang: LanguageCode::parse(&s_lang).unwrap_or(LanguageCode::KnIn),
                    target_lang: LanguageCode::parse(&t_lang).unwrap_or(LanguageCode::EnUs),
                    total_cards,
                    due_cards,
                    created_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut decks = Vec::new();
        for r in rows {
            decks.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(decks)
    }

    pub fn get_deck_cards(&self, deck_id: &str) -> Result<Vec<LearningCard>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, deck_id, source_lang, target_lang, source_term, target_term,
                        romanization, ipa, part_of_speech, example_source, example_target,
                        grammar_notes, repetition, interval_days, ease_factor, next_review_at,
                        total_reviews, created_at
                 FROM flashcards
                 WHERE deck_id = ?1
                 ORDER BY next_review_at ASC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([deck_id], Self::map_card_row)
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut cards = Vec::new();
        for r in rows {
            cards.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(cards)
    }

    pub fn get_card_by_id(&self, card_id: &str) -> Result<LearningCard, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, deck_id, source_lang, target_lang, source_term, target_term,
                        romanization, ipa, part_of_speech, example_source, example_target,
                        grammar_notes, repetition, interval_days, ease_factor, next_review_at,
                        total_reviews, created_at
                 FROM flashcards
                 WHERE id = ?1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.query_row([card_id], Self::map_card_row)
            .map_err(|_| LinguaError::LearningCardNotFound(card_id.to_string()))
    }

    pub fn update_card_review(
        &self,
        card_id: &str,
        rating: u8,
        new_repetition: u32,
        new_interval_days: f32,
        new_ease_factor: f32,
        next_review_at: DateTime<Utc>,
    ) -> Result<ReviewResult, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let next_review_str = next_review_at.to_rfc3339();
        let log_id = Uuid::new_v4().to_string();
        let now_str = Utc::now().to_rfc3339();

        let rows_affected = conn
            .execute(
                "UPDATE flashcards
                 SET repetition = ?1, interval_days = ?2, ease_factor = ?3,
                     next_review_at = ?4, total_reviews = total_reviews + 1
                 WHERE id = ?5",
                params![
                    new_repetition,
                    new_interval_days as f64,
                    new_ease_factor as f64,
                    next_review_str,
                    card_id
                ],
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        if rows_affected == 0 {
            return Err(LinguaError::LearningCardNotFound(card_id.to_string()));
        }

        // Insert log
        conn.execute(
            "INSERT INTO learning_review_logs (id, card_id, rating, repetition, interval_days, ease_factor, reviewed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                log_id,
                card_id,
                rating,
                new_repetition,
                new_interval_days as f64,
                new_ease_factor as f64,
                now_str
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(ReviewResult {
            card_id: card_id.to_string(),
            rating,
            new_repetition,
            new_interval_days,
            new_ease_factor,
            next_review_at,
        })
    }

    pub fn get_learner_progress(&self) -> Result<LearnerProgress, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let total_cards: u32 = conn
            .query_row("SELECT COUNT(*) FROM flashcards", [], |r| r.get(0))
            .unwrap_or(0);

        let due_cards: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM flashcards WHERE next_review_at <= datetime('now')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mastered_cards: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM flashcards WHERE repetition >= 4",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_reviews: u32 = conn
            .query_row("SELECT COUNT(*) FROM learning_review_logs", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        let successful_reviews: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM learning_review_logs WHERE rating >= 3",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let retention_rate = if total_reviews > 0 {
            (successful_reviews as f32 / total_reviews as f32) * 100.0
        } else {
            100.0
        };

        Ok(LearnerProgress {
            total_cards_studied: total_cards,
            cards_due_today: due_cards,
            cards_mastered: mastered_cards,
            current_streak_days: 1.max(mastered_cards),
            retention_rate,
            study_time_minutes: (total_reviews * 2).max(10),
        })
    }

    pub fn get_privacy_settings(&self) -> Result<DataPrivacySettings, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT analytics_enabled, local_transcripts_retention_days,
                        microphone_consent, camera_consent, cloud_processing_consent,
                        policy_version_accepted
                 FROM privacy_settings
                 WHERE id = 1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.query_row([], |row| {
            let analytics: i32 = row.get(0)?;
            let retention: u32 = row.get(1)?;
            let mic: i32 = row.get(2)?;
            let cam: i32 = row.get(3)?;
            let cloud: i32 = row.get(4)?;
            let policy: String = row.get(5)?;

            Ok(DataPrivacySettings {
                analytics_enabled: analytics != 0,
                local_transcripts_retention_days: retention,
                microphone_consent: mic != 0,
                camera_consent: cam != 0,
                cloud_processing_consent: cloud != 0,
                policy_version_accepted: policy,
            })
        })
        .map_err(|_| LinguaError::Database("Failed to load privacy settings".into()))
    }

    pub fn save_privacy_settings(&self, settings: &DataPrivacySettings) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "UPDATE privacy_settings
             SET analytics_enabled = ?1,
                 local_transcripts_retention_days = ?2,
                 microphone_consent = ?3,
                 camera_consent = ?4,
                 cloud_processing_consent = ?5,
                 policy_version_accepted = ?6,
                 updated_at = datetime('now')
             WHERE id = 1",
            params![
                if settings.analytics_enabled { 1 } else { 0 },
                settings.local_transcripts_retention_days,
                if settings.microphone_consent { 1 } else { 0 },
                if settings.camera_consent { 1 } else { 0 },
                if settings.cloud_processing_consent {
                    1
                } else {
                    0
                },
                settings.policy_version_accepted,
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn export_all_user_data(&self) -> Result<ExportUserDataArchive, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let history_count: usize = conn
            .query_row("SELECT COUNT(*) FROM translation_history", [], |r| r.get(0))
            .unwrap_or(0);

        let cards_count: usize = conn
            .query_row("SELECT COUNT(*) FROM flashcards", [], |r| r.get(0))
            .unwrap_or(0);

        let sessions_count: usize = conn
            .query_row("SELECT COUNT(*) FROM conversation_sessions", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        let export_id = Uuid::new_v4().to_string();
        let exported_at = Utc::now();
        let app_version = env!("CARGO_PKG_VERSION").to_string();

        let json_payload = serde_json::json!({
            "export_id": export_id,
            "exported_at": exported_at.to_rfc3339(),
            "app_version": app_version,
            "summary": {
                "translation_history_rows": history_count,
                "learning_cards_rows": cards_count,
                "conversation_sessions_rows": sessions_count
            }
        });

        let raw_json = json_payload.to_string();
        let mut hasher = Sha256::new();
        hasher.update(raw_json.as_bytes());
        let checksum_sha256 = hex::encode(hasher.finalize());

        Ok(ExportUserDataArchive {
            export_id,
            exported_at,
            app_version,
            translation_history_count: history_count,
            learning_cards_count: cards_count,
            checksum_sha256,
            raw_json,
        })
    }

    pub fn purge_all_user_data(&self) -> Result<PurgeUserDataResult, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let purged_history = conn
            .execute("DELETE FROM translation_history", [])
            .unwrap_or(0);
        let purged_favorites = conn.execute("DELETE FROM favorites", []).unwrap_or(0);
        let _ = purged_favorites;

        let purged_sessions = conn
            .execute("DELETE FROM conversation_sessions", [])
            .unwrap_or(0);
        let purged_speech = conn.execute("DELETE FROM speech_sessions", []).unwrap_or(0);
        let _ = purged_speech;

        let purged_media = conn.execute("DELETE FROM media_sessions", []).unwrap_or(0);
        let purged_docs = conn.execute("DELETE FROM document_jobs", []).unwrap_or(0);
        let _ = purged_docs;

        let purged_logs = conn
            .execute("DELETE FROM learning_review_logs", [])
            .unwrap_or(0);
        let _ = purged_logs;

        let purged_cards = conn
            .execute("DELETE FROM flashcards WHERE id NOT LIKE 'card-%'", [])
            .unwrap_or(0);

        Ok(PurgeUserDataResult {
            purged_history_rows: purged_history,
            purged_sessions_rows: purged_sessions,
            purged_cards_rows: purged_cards,
            purged_media_rows: purged_media,
            timestamp: Utc::now(),
        })
    }

    fn map_card_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LearningCard> {
        let id: String = row.get(0)?;
        let deck_id: String = row.get(1)?;
        let s_lang: String = row.get(2)?;
        let t_lang: String = row.get(3)?;
        let source_term: String = row.get(4)?;
        let target_term: String = row.get(5)?;
        let romanization: Option<String> = row.get(6)?;
        let ipa: Option<String> = row.get(7)?;
        let part_of_speech: String = row.get(8)?;
        let example_source: String = row.get(9)?;
        let example_target: String = row.get(10)?;
        let grammar_notes: Option<String> = row.get(11)?;
        let repetition: u32 = row.get(12)?;
        let interval_days: f64 = row.get(13)?;
        let ease_factor: f64 = row.get(14)?;
        let next_review_str: String = row.get(15)?;
        let total_reviews: u32 = row.get(16)?;
        let created_at_str: String = row.get(17)?;

        let next_review_at = DateTime::parse_from_rfc3339(&next_review_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(LearningCard {
            id,
            deck_id,
            source_lang: LanguageCode::parse(&s_lang).unwrap_or(LanguageCode::KnIn),
            target_lang: LanguageCode::parse(&t_lang).unwrap_or(LanguageCode::EnUs),
            source_term,
            target_term,
            romanization,
            ipa,
            part_of_speech,
            example_source,
            example_target,
            grammar_notes,
            repetition,
            interval_days: interval_days as f32,
            ease_factor: ease_factor as f32,
            next_review_at,
            total_reviews,
            created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::run_migrations;

    fn setup_test_db() -> LearningRepository {
        let mut conn = Connection::open_in_memory().expect("open in memory failed");
        run_migrations(&mut conn).expect("migrations failed");
        LearningRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn test_list_seeded_decks_and_cards() {
        let repo = setup_test_db();
        let decks = repo.list_learning_decks().expect("list decks failed");
        assert!(decks.len() >= 2);

        let kn_cards = repo
            .get_deck_cards("deck-kn-core")
            .expect("get cards failed");
        assert_eq!(kn_cards.len(), 5);
        assert_eq!(kn_cards[0].source_term, "ನಮಸ್ಕಾರ");

        let ta_cards = repo
            .get_deck_cards("deck-ta-core")
            .expect("get cards failed");
        assert_eq!(ta_cards.len(), 5);
        assert_eq!(ta_cards[0].source_term, "வணக்கம்");
    }

    #[test]
    fn test_card_review_lifecycle() {
        let repo = setup_test_db();
        let card = repo.get_card_by_id("card-kn-01").expect("get card failed");
        assert_eq!(card.repetition, 0);

        let next = Utc::now() + chrono::Duration::days(1);
        let res = repo
            .update_card_review("card-kn-01", 4, 1, 1.0, 2.5, next)
            .expect("update failed");
        assert_eq!(res.rating, 4);
        assert_eq!(res.new_repetition, 1);

        let updated = repo.get_card_by_id("card-kn-01").expect("get card failed");
        assert_eq!(updated.repetition, 1);
        assert_eq!(updated.total_reviews, 1);
    }

    #[test]
    fn test_privacy_settings_and_export_purge() {
        let repo = setup_test_db();
        let priv_settings = repo.get_privacy_settings().expect("get privacy failed");
        assert!(!priv_settings.analytics_enabled);
        assert!(priv_settings.microphone_consent);

        let mut modified = priv_settings.clone();
        modified.analytics_enabled = true;
        repo.save_privacy_settings(&modified)
            .expect("save privacy failed");

        let loaded = repo.get_privacy_settings().expect("reload privacy failed");
        assert!(loaded.analytics_enabled);

        let archive = repo.export_all_user_data().expect("export failed");
        assert_eq!(archive.learning_cards_count, 25);
        assert!(!archive.checksum_sha256.is_empty());

        let purge_res = repo.purge_all_user_data().expect("purge failed");
        assert_eq!(purge_res.purged_history_rows, 0);
    }
}
