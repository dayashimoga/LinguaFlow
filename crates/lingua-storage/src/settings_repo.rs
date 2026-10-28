use chrono::{DateTime, Utc};
use lingua_core::{
    AudioSettings, EnginePreference, LanguageCode, LinguaError, ThemeMode, UserSettings,
};
use rusqlite::{params, Connection};
use std::str::FromStr;

pub fn get_user_settings(conn: &Connection) -> Result<UserSettings, LinguaError> {
    let mut stmt = conn
        .prepare(
            "SELECT theme, source_language, target_language, font_scale,
                    engine_preference, offline_only, audio_settings_json, updated_at
             FROM user_settings WHERE id = 1",
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

    let result = stmt.query_row([], |row| {
        let theme_str: String = row.get(0)?;
        let src_str: String = row.get(1)?;
        let tgt_str: String = row.get(2)?;
        let font_scale: f64 = row.get(3)?;
        let pref_str: String = row.get(4)?;
        let offline_int: i64 = row.get(5)?;
        let audio_json: String = row.get(6)?;
        let updated_str: String = row.get(7)?;

        Ok((
            theme_str,
            src_str,
            tgt_str,
            font_scale,
            pref_str,
            offline_int,
            audio_json,
            updated_str,
        ))
    });

    match result {
        Ok((
            theme_str,
            src_str,
            tgt_str,
            font_scale,
            pref_str,
            offline_int,
            audio_json,
            updated_str,
        )) => {
            let theme = match theme_str.as_str() {
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                _ => ThemeMode::System,
            };

            let source_language = LanguageCode::from_str(&src_str).unwrap_or(LanguageCode::EnUs);
            let target_language = LanguageCode::from_str(&tgt_str).unwrap_or(LanguageCode::KnIn);

            let engine_preference = match pref_str.as_str() {
                "offline_only" => EnginePreference::OfflineOnly,
                "online_only" => EnginePreference::OnlineOnly,
                _ => EnginePreference::AutoHybrid,
            };

            let audio: AudioSettings = serde_json::from_str(&audio_json).unwrap_or_default();
            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(UserSettings {
                theme,
                source_language,
                target_language,
                font_scale: font_scale as f32,
                engine_preference,
                offline_only: offline_int != 0,
                audio,
                updated_at,
            })
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // First run, populate with default settings
            let default_settings = UserSettings::default();
            save_user_settings(conn, &default_settings)?;
            Ok(default_settings)
        }
        Err(e) => Err(LinguaError::Database(e.to_string())),
    }
}

pub fn save_user_settings(conn: &Connection, settings: &UserSettings) -> Result<(), LinguaError> {
    let theme_str = match settings.theme {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
        ThemeMode::System => "system",
    };

    let pref_str = match settings.engine_preference {
        EnginePreference::OfflineOnly => "offline_only",
        EnginePreference::OnlineOnly => "online_only",
        EnginePreference::AutoHybrid => "auto_hybrid",
    };

    let audio_json = serde_json::to_string(&settings.audio)
        .map_err(|e| LinguaError::Database(format!("Failed to serialize audio settings: {e}")))?;

    let updated_str = settings.updated_at.to_rfc3339();

    conn.execute(
        "INSERT INTO user_settings (id, theme, source_language, target_language, font_scale,
                                   engine_preference, offline_only, audio_settings_json, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            theme = excluded.theme,
            source_language = excluded.source_language,
            target_language = excluded.target_language,
            font_scale = excluded.font_scale,
            engine_preference = excluded.engine_preference,
            offline_only = excluded.offline_only,
            audio_settings_json = excluded.audio_settings_json,
            updated_at = excluded.updated_at",
        params![
            theme_str,
            settings.source_language.as_bcp47(),
            settings.target_language.as_bcp47(),
            settings.font_scale as f64,
            pref_str,
            settings.offline_only as i64,
            audio_json,
            updated_str,
        ],
    )
    .map_err(|e| LinguaError::Database(format!("Failed to save user settings: {e}")))?;

    Ok(())
}
