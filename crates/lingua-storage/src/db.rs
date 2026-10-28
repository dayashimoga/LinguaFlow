use crate::dictionary_repo::{
    get_dictionary_entry, get_entry_by_exact_word, import_dictionary_entries, search_dictionary,
    seed_stage_1_dictionary,
};
use crate::history_repo::{
    add_favorite, clear_translation_history, list_favorites, list_translation_history,
    record_translation, remove_favorite,
};
use crate::model_packs_repo::{list_model_packs, seed_initial_model_catalog};
use crate::schema::run_migrations;
use crate::settings_repo::{get_user_settings, save_user_settings};
use lingua_core::{
    DatabaseHealthInfo, DictionaryEntry, FavoriteItem, LanguageCode, LinguaError, ModelPackInfo,
    TranslationRecord, UserSettings,
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
}
