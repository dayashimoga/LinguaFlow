use crate::dictionary_repo::{
    get_dictionary_entry, get_entry_by_exact_word, import_dictionary_entries, search_dictionary,
    seed_stage_1_dictionary,
};
use crate::history_repo::{
    add_favorite, clear_translation_history, list_favorites, list_translation_history,
    record_translation, remove_favorite,
};
use crate::model_packs_repo::{
    get_detailed_model_pack, get_model_manifest, list_detailed_model_packs, list_model_packs,
    save_model_manifest, seed_initial_model_catalog, update_model_pack_install_state,
};
use crate::offline_repo::{
    lookup_morphology, lookup_translation_cache, record_translation_cache, seed_initial_morphology,
};
use crate::schema::run_migrations;
use crate::settings_repo::{get_user_settings, save_user_settings};
use chrono::{DateTime, Utc};
use lingua_core::{
    CachedTranslation, CaptionSegment, DatabaseHealthInfo, DictionaryEntry, FavoriteItem,
    LanguageCode, LinguaError, ModelInstallState, ModelPackDetailedInfo, ModelPackInfo,
    ModelPackManifest, MorphologyEntry, SpeechSession, SpeechSessionStatus, TranslationRecord,
    UserSettings,
};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct StorageService {
    conn: Arc<Mutex<Connection>>,
    db_path: String,
}

impl StorageService {
    pub fn open_in_memory() -> Result<Self, LinguaError> {
        let mut conn = Connection::open_in_memory().map_err(|e| {
            LinguaError::Database(format!("Failed to open in-memory database: {e}"))
        })?;

        run_migrations(&mut conn)?;
        seed_initial_model_catalog(&conn)?;
        seed_stage_1_dictionary(&conn)?;
        seed_initial_morphology(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: ":memory:".into(),
        })
    }

    pub fn open_at_path<P: AsRef<Path>>(path: P) -> Result<Self, LinguaError> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LinguaError::Database(format!("Failed to create database directory: {e}"))
            })?;
        }

        let mut conn = Connection::open(path_ref).map_err(|e| {
            LinguaError::Database(format!(
                "Failed to open database at {}: {e}",
                path_ref.display()
            ))
        })?;

        run_migrations(&mut conn)?;
        seed_initial_model_catalog(&conn)?;
        seed_stage_1_dictionary(&conn)?;
        seed_initial_morphology(&conn)?;

        info!(
            "Opened LinguaFlow SQLite database at {}",
            path_ref.display()
        );
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path_ref.to_string_lossy().to_string(),
        })
    }

    pub fn open_default() -> Result<Self, LinguaError> {
        let data_dir = default_data_dir()?;
        let db_file = data_dir.join("linguaflow.db");
        Self::open_at_path(db_file)
    }

    pub fn get_user_settings(&self) -> Result<UserSettings, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        get_user_settings(&conn)
    }

    pub fn save_user_settings(&self, settings: &UserSettings) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        save_user_settings(&conn, settings)
    }

    pub fn list_model_packs(&self) -> Result<Vec<ModelPackInfo>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        list_model_packs(&conn)
    }

    // --- Dictionary Methods ---
    pub fn import_dictionary_entries(
        &self,
        entries: &[DictionaryEntry],
    ) -> Result<usize, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        import_dictionary_entries(&conn, entries)
    }

    pub fn search_dictionary(
        &self,
        query: &str,
        lang: Option<LanguageCode>,
        limit: usize,
    ) -> Result<Vec<DictionaryEntry>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        search_dictionary(&conn, query, lang, limit)
    }

    pub fn get_dictionary_entry(&self, id: &str) -> Result<Option<DictionaryEntry>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        get_dictionary_entry(&conn, id)
    }

    pub fn get_entry_by_exact_word(
        &self,
        word: &str,
        lang: LanguageCode,
    ) -> Result<Option<DictionaryEntry>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        get_entry_by_exact_word(&conn, word, lang)
    }

    // --- History & Favorites Methods ---
    pub fn record_translation(&self, record: &TranslationRecord) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        record_translation(&conn, record)
    }

    pub fn list_translation_history(
        &self,
        limit: usize,
    ) -> Result<Vec<TranslationRecord>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        list_translation_history(&conn, limit)
    }

    pub fn clear_translation_history(&self) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        clear_translation_history(&conn)
    }

    pub fn add_favorite(&self, item: &FavoriteItem) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        add_favorite(&conn, item)
    }

    pub fn remove_favorite(&self, id: &str) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        remove_favorite(&conn, id)
    }

    pub fn list_favorites(&self) -> Result<Vec<FavoriteItem>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        list_favorites(&conn)
    }

    // --- Stage 2: Offline Model & Cache Methods ---
    pub fn list_detailed_model_packs(&self) -> Result<Vec<ModelPackDetailedInfo>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        list_detailed_model_packs(&conn)
    }

    pub fn get_detailed_model_pack(
        &self,
        id: &str,
    ) -> Result<Option<ModelPackDetailedInfo>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        get_detailed_model_pack(&conn, id)
    }

    pub fn update_model_pack_install_state(
        &self,
        id: &str,
        state: &ModelInstallState,
        checksum_verified: bool,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        update_model_pack_install_state(&conn, id, state, checksum_verified)
    }

    pub fn update_model_pack_sha256(&self, id: &str, sha256: &str) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::model_packs_repo::update_model_pack_sha256(&conn, id, sha256)
    }

    pub fn save_model_manifest(&self, manifest: &ModelPackManifest) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        save_model_manifest(&conn, manifest)
    }

    pub fn get_model_manifest(&self, id: &str) -> Result<Option<ModelPackManifest>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        get_model_manifest(&conn, id)
    }

    pub fn record_translation_cache(&self, cached: &CachedTranslation) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        record_translation_cache(&conn, cached)
    }

    pub fn lookup_translation_cache(
        &self,
        source_lang: LanguageCode,
        target_lang: LanguageCode,
        source_text: &str,
    ) -> Result<Option<CachedTranslation>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        lookup_translation_cache(&conn, source_lang, target_lang, source_text)
    }

    pub fn lookup_morphology(
        &self,
        lang: LanguageCode,
        surface: &str,
    ) -> Result<Option<MorphologyEntry>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        lookup_morphology(&conn, lang, surface)
    }

    // --- Stage 3: Real-Time Speech & Captions Methods ---
    pub fn create_speech_session(&self, session: &SpeechSession) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::create_speech_session(&conn, session)
    }

    pub fn update_speech_session_status(
        &self,
        session_id: &str,
        status: SpeechSessionStatus,
        duration_seconds: u64,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::update_speech_session_status(
            &conn,
            session_id,
            status,
            duration_seconds,
            ended_at,
        )
    }

    pub fn get_speech_session(
        &self,
        session_id: &str,
    ) -> Result<Option<SpeechSession>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::get_speech_session(&conn, session_id)
    }

    pub fn list_speech_sessions(&self, limit: usize) -> Result<Vec<SpeechSession>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::list_speech_sessions(&conn, limit)
    }

    pub fn record_caption_segment(&self, segment: &CaptionSegment) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::record_caption_segment(&conn, segment)
    }

    pub fn get_caption_segments_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<CaptionSegment>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::get_caption_segments_for_session(&conn, session_id)
    }

    pub fn delete_speech_session(&self, session_id: &str) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;
        crate::speech_repo::delete_speech_session(&conn, session_id)
    }

    pub fn check_health(&self) -> Result<DatabaseHealthInfo, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let schema_ver: u32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let settings_ok = get_user_settings(&conn).is_ok();

        Ok(DatabaseHealthInfo {
            status: "healthy".into(),
            path: self.db_path.clone(),
            schema_version: schema_ver,
            settings_persisted: settings_ok,
            error: None,
        })
    }
}

fn default_data_dir() -> Result<PathBuf, LinguaError> {
    if let Ok(custom) = std::env::var("LINGUAFLOW_DATA_DIR") {
        return Ok(PathBuf::from(custom));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return Ok(PathBuf::from(app_data).join("LinguaFlow"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join("Library/Application Support/LinguaFlow"));
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(data_home).join("linguaflow"));
        } else if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/linguaflow"));
        }
    }

    Ok(PathBuf::from("./data"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::CURRENT_SCHEMA_VERSION;
    use chrono::Utc;
    use lingua_core::{LanguageCode, ThemeMode};

    #[test]
    fn test_in_memory_persistence_and_health() {
        let storage = StorageService::open_in_memory().expect("in-memory DB failed");

        let health = storage.check_health().expect("health check failed");
        assert_eq!(health.status, "healthy");
        assert_eq!(health.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(health.settings_persisted);

        let mut settings = storage
            .get_user_settings()
            .expect("failed to load settings");
        assert_eq!(settings.theme, ThemeMode::System);

        settings.theme = ThemeMode::Dark;
        settings.source_language = LanguageCode::KnIn;
        settings.target_language = LanguageCode::TaIn;
        storage
            .save_user_settings(&settings)
            .expect("failed to save settings");

        let reloaded = storage.get_user_settings().expect("failed to reload");
        assert_eq!(reloaded.theme, ThemeMode::Dark);
        assert_eq!(reloaded.source_language, LanguageCode::KnIn);
        assert_eq!(reloaded.target_language, LanguageCode::TaIn);

        let packs = storage.list_model_packs().expect("failed to list packs");
        assert!(!packs.is_empty());
        assert!(packs.iter().any(|p| p.source == LanguageCode::KnIn));
    }

    #[test]
    fn test_dictionary_search_and_history() {
        let storage = StorageService::open_in_memory().expect("in-memory DB failed");

        // 1. Check seeded dictionary entries
        let kn_results = storage
            .search_dictionary("ನಮಸ್ಕಾರ", Some(LanguageCode::KnIn), 10)
            .expect("search failed");
        assert!(!kn_results.is_empty());
        assert_eq!(kn_results[0].word, "ನಮಸ್ಕಾರ");
        assert!(kn_results[0].pronunciation_ipa.is_some());

        // 2. Exact match lookup
        let exact_ta = storage
            .get_entry_by_exact_word("வணக்கம்", LanguageCode::TaIn)
            .expect("exact lookup failed");
        assert!(exact_ta.is_some());
        assert_eq!(
            exact_ta.unwrap().definition,
            "Hello, traditional Tamil salutation expressing respect and welcoming."
        );

        // 3. FTS5 search across definitions
        let search_salutation = storage
            .search_dictionary("greeting", None, 10)
            .expect("FTS search failed");
        assert!(!search_salutation.is_empty());

        // 4. Record translation & query history
        let record = TranslationRecord {
            id: "rec-1".into(),
            source_language: LanguageCode::EnUs,
            target_language: LanguageCode::KnIn,
            source_text: "water".into(),
            translated_text: "ನೀರು".into(),
            engine_used: "LexiconEngine".into(),
            latency_ms: 2,
            created_at: Utc::now(),
        };
        storage.record_translation(&record).expect("record failed");

        let history = storage.list_translation_history(10).expect("list failed");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].source_text, "water");
        assert_eq!(history[0].translated_text, "ನೀರು");

        // 5. Add and remove favorite
        let fav = FavoriteItem {
            id: "fav-1".into(),
            source_language: LanguageCode::EnUs,
            target_language: LanguageCode::KnIn,
            source_text: "water".into(),
            translated_text: "ನೀರು".into(),
            notes: Some("Essential vocabulary".into()),
            created_at: Utc::now(),
        };
        storage.add_favorite(&fav).expect("add favorite failed");
        let favorites = storage.list_favorites().expect("list favorites failed");
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, "fav-1");

        storage.remove_favorite("fav-1").expect("remove failed");
        let favorites_after = storage.list_favorites().expect("list favorites failed");
        assert!(favorites_after.is_empty());
    }

    #[test]
    fn test_stage_2_offline_models_cache_and_morphology() {
        let storage = StorageService::open_in_memory().expect("in-memory DB failed");

        // 1. Detailed model pack query
        let detailed_packs = storage
            .list_detailed_model_packs()
            .expect("detailed packs list failed");
        assert_eq!(detailed_packs.len(), 5);
        let kn_en_pack = detailed_packs
            .iter()
            .find(|p| p.id == "model-kn-en-indictrans2")
            .expect("pack missing");
        assert_eq!(kn_en_pack.state, ModelInstallState::NotInstalled);
        assert_eq!(kn_en_pack.architecture, "indictrans2-quantized-onnx");

        // 2. Update model pack state to Installed
        let installed_state = ModelInstallState::Installed {
            installed_at: Utc::now(),
            install_path: "/data/models/kn-en".into(),
        };
        storage
            .update_model_pack_install_state("model-kn-en-indictrans2", &installed_state, true)
            .expect("update install state failed");

        let updated_pack = storage
            .get_detailed_model_pack("model-kn-en-indictrans2")
            .expect("fetch pack failed")
            .expect("pack not found");
        match updated_pack.state {
            ModelInstallState::Installed { install_path, .. } => {
                assert_eq!(install_path, "/data/models/kn-en");
            }
            _ => panic!("Expected installed state"),
        }

        // 3. Translation Cache Record and Lookup
        let cached = CachedTranslation {
            id: "cache-1".into(),
            source_language: LanguageCode::KnIn,
            target_language: LanguageCode::EnUs,
            source_text: "ಧನ್ಯವಾದಗಳು".into(),
            translated_text: "Thank you".into(),
            engine_id: "local-model-kn-en".into(),
            is_offline: true,
            created_at: Utc::now(),
            hit_count: 1,
        };
        storage
            .record_translation_cache(&cached)
            .expect("cache record failed");

        let cache_hit = storage
            .lookup_translation_cache(LanguageCode::KnIn, LanguageCode::EnUs, "ಧನ್ಯವಾದಗಳು")
            .expect("cache lookup failed");
        assert!(cache_hit.is_some());
        assert_eq!(cache_hit.unwrap().translated_text, "Thank you");

        // 4. Morphology stem lookup
        let morph_kn = storage
            .lookup_morphology(LanguageCode::KnIn, "ಮಾಡುತ್ತಾನೆ")
            .expect("morphology lookup failed");
        assert!(morph_kn.is_some());
        let morph_entry = morph_kn.unwrap();
        assert_eq!(morph_entry.stem_form, "ಮಾಡು");
        assert_eq!(morph_entry.part_of_speech.as_deref(), Some("Verb"));

        let morph_ta = storage
            .lookup_morphology(LanguageCode::TaIn, "செய்கிறான்")
            .expect("morphology lookup failed");
        assert!(morph_ta.is_some());
        assert_eq!(morph_ta.unwrap().stem_form, "செய்");
    }

    #[test]
    fn test_stage_3_speech_sessions_and_captions() {
        let storage = StorageService::open_in_memory().expect("in-memory DB failed");

        // 1. Create a speech session
        let session = SpeechSession {
            id: "session-test-01".into(),
            title: "Live Kannada Speech Session".into(),
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            status: SpeechSessionStatus::Active,
            created_at: Utc::now(),
            ended_at: None,
            duration_seconds: 0,
        };
        storage
            .create_speech_session(&session)
            .expect("create speech session failed");

        // 2. Query the created session
        let fetched = storage
            .get_speech_session("session-test-01")
            .expect("fetch failed")
            .expect("session missing");
        assert_eq!(fetched.title, "Live Kannada Speech Session");
        assert_eq!(fetched.status, SpeechSessionStatus::Active);

        // 3. Record progressive captions
        let cap1 = CaptionSegment {
            id: "cap-1".into(),
            session_id: "session-test-01".into(),
            seq_id: 1,
            source_text: "ನಮಸ್ಕಾರ".into(),
            translated_text: "Hello".into(),
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            is_final: false,
            confidence: 0.88,
            start_time_ms: 0,
            end_time_ms: 600,
            created_at: Utc::now(),
        };
        storage.record_caption_segment(&cap1).expect("cap1 failed");

        // Update cap1 to final
        let cap1_final = CaptionSegment {
            id: "cap-1".into(),
            session_id: "session-test-01".into(),
            seq_id: 1,
            source_text: "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
            translated_text: "Hello, how are you?".into(),
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            is_final: true,
            confidence: 0.95,
            start_time_ms: 0,
            end_time_ms: 1800,
            created_at: Utc::now(),
        };
        storage
            .record_caption_segment(&cap1_final)
            .expect("cap1_final update failed");

        let captions = storage
            .get_caption_segments_for_session("session-test-01")
            .expect("get captions failed");
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].source_text, "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?");
        assert!(captions[0].is_final);

        // 4. Update session status to completed
        storage
            .update_speech_session_status(
                "session-test-01",
                SpeechSessionStatus::Completed,
                15,
                Some(Utc::now()),
            )
            .expect("update status failed");

        let updated_sess = storage
            .get_speech_session("session-test-01")
            .expect("fetch failed")
            .expect("missing");
        assert_eq!(updated_sess.status, SpeechSessionStatus::Completed);
        assert_eq!(updated_sess.duration_seconds, 15);
    }
}
