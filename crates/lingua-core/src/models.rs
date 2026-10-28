use crate::language::{LanguageCode, LanguagePair};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnginePreference {
    #[default]
    AutoHybrid,
    OfflineOnly,
    OnlineOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSettings {
    pub input_device_id: Option<String>,
    pub output_device_id: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub vad_sensitivity: f32,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            input_device_id: None,
            output_device_id: None,
            sample_rate: 16000,
            channels: 1,
            vad_sensitivity: 0.5,
            echo_cancellation: true,
            noise_suppression: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    pub theme: ThemeMode,
    pub source_language: LanguageCode,
    pub target_language: LanguageCode,
    pub font_scale: f32,
    pub engine_preference: EnginePreference,
    pub offline_only: bool,
    pub audio: AudioSettings,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            source_language: LanguageCode::EnUs,
            target_language: LanguageCode::KnIn,
            font_scale: 1.0,
            engine_preference: EnginePreference::AutoHybrid,
            offline_only: false,
            audio: AudioSettings::default(),
            updated_at: Utc::now(),
        }
    }
}

impl UserSettings {
    pub fn language_pair(&self) -> Result<LanguagePair, crate::error::LinguaError> {
        LanguagePair::new(self.source_language, self.target_language)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "detail")]
pub enum CapabilityState {
    Active,
    Ready,
    NotInstalled,
    UnsupportedHardware,
    UnreachableProvider,
    ScheduledForStage(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityMatrix {
    pub dictionary: CapabilityState,
    pub text_translation: CapabilityState,
    pub streaming_asr: CapabilityState,
    pub speech_synthesis: CapabilityState,
    pub offline_models: CapabilityState,
    pub live_conversation: CapabilityState,
    pub media_subtitles: CapabilityState,
}

impl CapabilityMatrix {
    pub fn stage_0_baseline() -> Self {
        Self {
            dictionary: CapabilityState::ScheduledForStage(1),
            text_translation: CapabilityState::ScheduledForStage(1),
            streaming_asr: CapabilityState::ScheduledForStage(3),
            speech_synthesis: CapabilityState::ScheduledForStage(4),
            offline_models: CapabilityState::ScheduledForStage(2),
            live_conversation: CapabilityState::ScheduledForStage(4),
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }

    pub fn stage_1_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::ScheduledForStage(3),
            speech_synthesis: CapabilityState::ScheduledForStage(4),
            offline_models: CapabilityState::ScheduledForStage(2),
            live_conversation: CapabilityState::ScheduledForStage(4),
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryExample {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub id: String,
    pub word: String,
    pub normalized_word: String,
    pub language: LanguageCode,
    pub part_of_speech: String,
    pub definition: String,
    pub alternate_meanings: Vec<String>,
    pub examples: Vec<DictionaryExample>,
    pub pronunciation_ipa: Option<String>,
    pub transliteration: Option<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationRecord {
    pub id: String,
    pub source_language: LanguageCode,
    pub target_language: LanguageCode,
    pub source_text: String,
    pub translated_text: String,
    pub engine_used: String,
    pub latency_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FavoriteItem {
    pub id: String,
    pub source_language: LanguageCode,
    pub target_language: LanguageCode,
    pub source_text: String,
    pub translated_text: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelPackInfo {
    pub id: String,
    pub name: String,
    pub source: LanguageCode,
    pub target: LanguageCode,
    pub size_bytes: u64,
    pub version: String,
    pub sha256: String,
    pub is_installed: bool,
    pub scheduled_stage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub stage: u8,
    pub stage_name: String,
    pub platform: String,
    pub arch: String,
    pub build_profile: String,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "LinguaFlow".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            stage: 1,
            stage_name: "Dictionary & Text Translation".into(),
            platform: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            build_profile: if cfg!(debug_assertions) {
                "debug".into()
            } else {
                "release".into()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_settings_defaults() {
        let settings = UserSettings::default();
        assert_eq!(settings.theme, ThemeMode::System);
        assert_eq!(settings.source_language, LanguageCode::EnUs);
        assert_eq!(settings.target_language, LanguageCode::KnIn);
        assert_eq!(settings.font_scale, 1.0);
        let pair = settings.language_pair().unwrap();
        assert_eq!(pair.pair_key(), "en-US_kn-IN");
    }

    #[test]
    fn test_capability_matrix_stage_0() {
        let matrix = CapabilityMatrix::stage_0_baseline();
        assert_eq!(matrix.dictionary, CapabilityState::ScheduledForStage(1));
        assert_eq!(matrix.streaming_asr, CapabilityState::ScheduledForStage(3));
        assert_eq!(
            matrix.speech_synthesis,
            CapabilityState::ScheduledForStage(4)
        );
    }
}
