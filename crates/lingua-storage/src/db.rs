use crate::model_packs_repo::{list_model_packs, seed_initial_model_catalog};
use crate::schema::run_migrations;
use crate::settings_repo::{get_user_settings, save_user_settings};
use lingua_core::{DatabaseHealthInfo, LinguaError, ModelPackInfo, UserSettings};
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
}
