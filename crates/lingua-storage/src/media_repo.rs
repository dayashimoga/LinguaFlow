use chrono::{DateTime, Utc};
use lingua_core::{
    DocumentFormat, DocumentJobStatus, DocumentTranslationJob, LanguageCode, LinguaError,
    MediaFormat, MediaProgressStatus, MediaSession, MediaSubtitleSegment, OverlaySettings,
};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MediaRepository {
    conn: Arc<Mutex<Connection>>,
}

impl MediaRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // --- Media Sessions ---

    pub fn create_media_session(&self, session: &MediaSession) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT INTO media_sessions (
                id, title, file_name, file_size_bytes, duration_ms,
                source_lang, target_lang, format, status, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.id,
                session.title,
                session.file_name,
                session.file_size_bytes,
                session.duration_ms,
                session.source_lang.as_bcp47(),
                session.target_lang.as_bcp47(),
                session.format.as_str(),
                session.status.as_str(),
                session.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to insert media session: {e}")))?;

        Ok(())
    }

    pub fn get_media_session(&self, id: &str) -> Result<Option<MediaSession>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, file_name, file_size_bytes, duration_ms,
                        source_lang, target_lang, format, status, created_at
                 FROM media_sessions WHERE id = ?",
            )
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let result = stmt.query_row(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, u64>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        });

        match result {
            Ok((
                id,
                title,
                file_name,
                file_size_bytes,
                duration_ms,
                sl_str,
                tl_str,
                fmt_str,
                status_str,
                created_at_str,
            )) => {
                let source_lang = LanguageCode::parse(&sl_str)
                    .map_err(|e| LinguaError::Validation(e.to_string()))?;
                let target_lang = LanguageCode::parse(&tl_str)
                    .map_err(|e| LinguaError::Validation(e.to_string()))?;
                let format = MediaFormat::parse(&fmt_str);
                let status = MediaProgressStatus::parse(&status_str);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(MediaSession {
                    id,
                    title,
                    file_name,
                    file_size_bytes,
                    duration_ms,
                    source_lang,
                    target_lang,
                    format,
                    status,
                    created_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LinguaError::Database(format!(
                "Failed to query media session: {e}"
            ))),
        }
    }

    pub fn list_media_sessions(&self) -> Result<Vec<MediaSession>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, file_name, file_size_bytes, duration_ms,
                        source_lang, target_lang, format, status, created_at
                 FROM media_sessions ORDER BY created_at DESC",
            )
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let file_name: String = row.get(2)?;
                let file_size_bytes: u64 = row.get(3)?;
                let duration_ms: u64 = row.get(4)?;
                let sl_str: String = row.get(5)?;
                let tl_str: String = row.get(6)?;
                let fmt_str: String = row.get(7)?;
                let status_str: String = row.get(8)?;
                let created_at_str: String = row.get(9)?;

                let source_lang = LanguageCode::parse(&sl_str).unwrap_or(LanguageCode::EnUs);
                let target_lang = LanguageCode::parse(&tl_str).unwrap_or(LanguageCode::KnIn);
                let format = MediaFormat::parse(&fmt_str);
                let status = MediaProgressStatus::parse(&status_str);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(MediaSession {
                    id,
                    title,
                    file_name,
                    file_size_bytes,
                    duration_ms,
                    source_lang,
                    target_lang,
                    format,
                    status,
                    created_at,
                })
            })
            .map_err(|e| LinguaError::Database(format!("Failed to query media sessions: {e}")))?;

        let mut sessions = Vec::new();
        for s in rows {
            sessions.push(s.map_err(|e| LinguaError::Database(format!("Row error: {e}")))?)
        }
        Ok(sessions)
    }

    pub fn update_media_session_status(
        &self,
        id: &str,
        status: MediaProgressStatus,
        duration_ms: u64,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "UPDATE media_sessions SET status = ?, duration_ms = ? WHERE id = ?",
            params![status.as_str(), duration_ms, id],
        )
        .map_err(|e| {
            LinguaError::Database(format!("Failed to update media session status: {e}"))
        })?;

        Ok(())
    }

    // --- Subtitle Segments ---

    pub fn add_media_segment(&self, seg: &MediaSubtitleSegment) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT INTO media_segments (id, session_id, seq_id, start_ms, end_ms, source_text, translated_text)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                seg.id,
                seg.session_id,
                seg.seq_id,
                seg.start_ms,
                seg.end_ms,
                seg.source_text,
                seg.translated_text
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to insert media segment: {e}")))?;

        Ok(())
    }

    pub fn add_media_segments_batch(
        &self,
        segments: &[MediaSubtitleSegment],
    ) -> Result<(), LinguaError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let tx = conn
            .transaction()
            .map_err(|e| LinguaError::Database(format!("Failed to begin tx: {e}")))?;

        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO media_segments (id, session_id, seq_id, start_ms, end_ms, source_text, translated_text)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .map_err(|e| LinguaError::Database(format!("Failed to prepare insert: {e}")))?;

            for seg in segments {
                stmt.execute(params![
                    seg.id,
                    seg.session_id,
                    seg.seq_id,
                    seg.start_ms,
                    seg.end_ms,
                    seg.source_text,
                    seg.translated_text
                ])
                .map_err(|e| LinguaError::Database(format!("Failed to execute insert: {e}")))?;
            }
        }

        tx.commit()
            .map_err(|e| LinguaError::Database(format!("Failed to commit batch: {e}")))?;
        Ok(())
    }

    pub fn list_media_segments(
        &self,
        session_id: &str,
    ) -> Result<Vec<MediaSubtitleSegment>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, seq_id, start_ms, end_ms, source_text, translated_text
                 FROM media_segments WHERE session_id = ? ORDER BY seq_id ASC",
            )
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map(params![session_id], |row| {
                Ok(MediaSubtitleSegment {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    seq_id: row.get(2)?,
                    start_ms: row.get(3)?,
                    end_ms: row.get(4)?,
                    source_text: row.get(5)?,
                    translated_text: row.get(6)?,
                })
            })
            .map_err(|e| LinguaError::Database(format!("Failed to query segments: {e}")))?;

        let mut segments = Vec::new();
        for r in rows {
            segments.push(r.map_err(|e| LinguaError::Database(format!("Row error: {e}")))?)
        }
        Ok(segments)
    }

    pub fn update_media_segment(
        &self,
        id: &str,
        source_text: &str,
        translated_text: &str,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "UPDATE media_segments SET source_text = ?, translated_text = ?, start_ms = ?, end_ms = ? WHERE id = ?",
            params![source_text, translated_text, start_ms, end_ms, id],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to update segment: {e}")))?;

        Ok(())
    }

    // --- Overlay Settings ---

    pub fn get_overlay_settings(&self) -> Result<OverlaySettings, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT enabled, always_on_top, x, y, width, height, font_size,
                        font_family, text_color, background_color, opacity,
                        source_lang, target_lang, show_source, show_translated
                 FROM overlay_settings WHERE id = 1",
            )
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let result = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i32>(0)? != 0,
                row.get::<_, i32>(1)? != 0,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, u32>(4)?,
                row.get::<_, u32>(5)?,
                row.get::<_, u32>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, f64>(10)? as f32,
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, i32>(13)? != 0,
                row.get::<_, i32>(14)? != 0,
            ))
        });

        match result {
            Ok((
                enabled,
                always_on_top,
                x,
                y,
                width,
                height,
                font_size,
                font_family,
                text_color,
                background_color,
                opacity,
                sl_str,
                tl_str,
                show_source,
                show_translated,
            )) => {
                let source_lang = LanguageCode::parse(&sl_str).unwrap_or(LanguageCode::EnUs);
                let target_lang = LanguageCode::parse(&tl_str).unwrap_or(LanguageCode::KnIn);

                Ok(OverlaySettings {
                    enabled,
                    always_on_top,
                    x,
                    y,
                    width,
                    height,
                    font_size,
                    font_family,
                    text_color,
                    background_color,
                    opacity,
                    source_lang,
                    target_lang,
                    show_source,
                    show_translated,
                })
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(OverlaySettings::default()),
            Err(e) => Err(LinguaError::Database(format!(
                "Failed to query overlay settings: {e}"
            ))),
        }
    }

    pub fn save_overlay_settings(&self, s: &OverlaySettings) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT INTO overlay_settings (
                id, enabled, always_on_top, x, y, width, height, font_size,
                font_family, text_color, background_color, opacity,
                source_lang, target_lang, show_source, show_translated, updated_at
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                enabled = excluded.enabled,
                always_on_top = excluded.always_on_top,
                x = excluded.x,
                y = excluded.y,
                width = excluded.width,
                height = excluded.height,
                font_size = excluded.font_size,
                font_family = excluded.font_family,
                text_color = excluded.text_color,
                background_color = excluded.background_color,
                opacity = excluded.opacity,
                source_lang = excluded.source_lang,
                target_lang = excluded.target_lang,
                show_source = excluded.show_source,
                show_translated = excluded.show_translated,
                updated_at = datetime('now')",
            params![
                s.enabled as i32,
                s.always_on_top as i32,
                s.x,
                s.y,
                s.width,
                s.height,
                s.font_size,
                s.font_family,
                s.text_color,
                s.background_color,
                s.opacity as f64,
                s.source_lang.as_bcp47(),
                s.target_lang.as_bcp47(),
                s.show_source as i32,
                s.show_translated as i32,
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to save overlay settings: {e}")))?;

        Ok(())
    }

    // --- Document Translation Jobs ---

    pub fn create_document_job(&self, job: &DocumentTranslationJob) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT INTO document_jobs (
                id, file_name, format, file_size_bytes, source_lang,
                target_lang, total_blocks, completed_blocks, status, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                job.id,
                job.file_name,
                job.format.as_str(),
                job.file_size_bytes,
                job.source_lang.as_bcp47(),
                job.target_lang.as_bcp47(),
                job.total_blocks as i64,
                job.completed_blocks as i64,
                job.status.as_str(),
                job.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to insert document job: {e}")))?;

        Ok(())
    }

    pub fn get_document_job(
        &self,
        id: &str,
    ) -> Result<Option<DocumentTranslationJob>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, file_name, format, file_size_bytes, source_lang,
                        target_lang, total_blocks, completed_blocks, status, created_at
                 FROM document_jobs WHERE id = ?",
            )
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let result = stmt.query_row(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        });

        match result {
            Ok((
                id,
                file_name,
                fmt_str,
                file_size_bytes,
                sl_str,
                tl_str,
                total_blocks,
                completed_blocks,
                status_str,
                created_at_str,
            )) => {
                let format = DocumentFormat::parse(&fmt_str);
                let source_lang = LanguageCode::parse(&sl_str)
                    .map_err(|e| LinguaError::Validation(e.to_string()))?;
                let target_lang = LanguageCode::parse(&tl_str)
                    .map_err(|e| LinguaError::Validation(e.to_string()))?;
                let status = DocumentJobStatus::parse(&status_str);
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Some(DocumentTranslationJob {
                    id,
                    file_name,
                    format,
                    file_size_bytes,
                    source_lang,
                    target_lang,
                    total_blocks: total_blocks as usize,
                    completed_blocks: completed_blocks as usize,
                    status,
                    created_at,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(LinguaError::Database(format!(
                "Failed to query document job: {e}"
            ))),
        }
    }

    pub fn update_document_job_progress(
        &self,
        id: &str,
        completed_blocks: usize,
        status: DocumentJobStatus,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "UPDATE document_jobs SET completed_blocks = ?, status = ? WHERE id = ?",
            params![completed_blocks as i64, status.as_str(), id],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to update document job: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::run_migrations;

    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_media_session_and_segments_crud() {
        let db = setup_test_db();
        let repo = MediaRepository::new(db);

        let session = MediaSession {
            id: "media-test-1".into(),
            title: "Test Kannada Interview".into(),
            file_name: "interview.wav".into(),
            file_size_bytes: 1048576,
            duration_ms: 12000,
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            format: MediaFormat::Wav,
            status: MediaProgressStatus::Completed,
            created_at: Utc::now(),
        };

        repo.create_media_session(&session).unwrap();

        let fetched = repo.get_media_session("media-test-1").unwrap().unwrap();
        assert_eq!(fetched.title, "Test Kannada Interview");
        assert_eq!(fetched.format, MediaFormat::Wav);

        let seg1 = MediaSubtitleSegment {
            id: "seg-1".into(),
            session_id: "media-test-1".into(),
            seq_id: 1,
            start_ms: 0,
            end_ms: 2500,
            source_text: "ನಮಸ್ಕಾರ, ಹೇಗಿದ್ದೀರಿ?".into(),
            translated_text: "Hello, how are you?".into(),
        };
        let seg2 = MediaSubtitleSegment {
            id: "seg-2".into(),
            session_id: "media-test-1".into(),
            seq_id: 2,
            start_ms: 2600,
            end_ms: 5000,
            source_text: "ನಾನು ಆರಾಮಾಗಿದ್ದೇನೆ.".into(),
            translated_text: "I am doing well.".into(),
        };

        repo.add_media_segments_batch(&[seg1, seg2]).unwrap();

        let segments = repo.list_media_segments("media-test-1").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].seq_id, 1);
        assert_eq!(segments[1].seq_id, 2);

        // Edit segment
        repo.update_media_segment("seg-1", "ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೆ!", "Hello friends!", 0, 2400)
            .unwrap();
        let updated = repo.list_media_segments("media-test-1").unwrap();
        assert_eq!(updated[0].source_text, "ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೆ!");
        assert_eq!(updated[0].translated_text, "Hello friends!");
    }

    #[test]
    fn test_overlay_settings_persistence() {
        let db = setup_test_db();
        let repo = MediaRepository::new(db);

        let settings = repo.get_overlay_settings().unwrap();
        assert!(settings.enabled);
        assert_eq!(settings.font_size, 22);

        let modified = OverlaySettings {
            font_size: 28,
            opacity: 0.95,
            always_on_top: true,
            ..settings
        };
        repo.save_overlay_settings(&modified).unwrap();

        let reloaded = repo.get_overlay_settings().unwrap();
        assert_eq!(reloaded.font_size, 28);
        assert!((reloaded.opacity - 0.95).abs() < 1e-4);
    }

    #[test]
    fn test_document_jobs_crud() {
        let db = setup_test_db();
        let repo = MediaRepository::new(db);

        let job = DocumentTranslationJob {
            id: "doc-job-1".into(),
            file_name: "report.pdf".into(),
            format: DocumentFormat::Pdf,
            file_size_bytes: 204800,
            source_lang: LanguageCode::EnUs,
            target_lang: LanguageCode::KnIn,
            total_blocks: 15,
            completed_blocks: 0,
            status: DocumentJobStatus::Processing,
            created_at: Utc::now(),
        };

        repo.create_document_job(&job).unwrap();

        let fetched = repo.get_document_job("doc-job-1").unwrap().unwrap();
        assert_eq!(fetched.total_blocks, 15);
        assert_eq!(fetched.completed_blocks, 0);

        repo.update_document_job_progress("doc-job-1", 15, DocumentJobStatus::Completed)
            .unwrap();

        let updated = repo.get_document_job("doc-job-1").unwrap().unwrap();
        assert_eq!(updated.completed_blocks, 15);
        assert_eq!(updated.status, DocumentJobStatus::Completed);
    }
}
