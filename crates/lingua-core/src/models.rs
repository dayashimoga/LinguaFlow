use crate::error::LinguaError;
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

    pub fn stage_2_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::ScheduledForStage(3),
            speech_synthesis: CapabilityState::ScheduledForStage(4),
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::ScheduledForStage(4),
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }

    pub fn stage_3_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::ScheduledForStage(4),
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::ScheduledForStage(4),
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }

    pub fn stage_4_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::Active,
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::Active,
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }

    pub fn stage_5_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::Active,
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::Active,
            media_subtitles: CapabilityState::ScheduledForStage(6),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioDeviceItem {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VadState {
    Silence,
    Speech,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrTranscriptEvent {
    pub session_id: String,
    pub seq_id: u64,
    pub is_final: bool,
    pub text: String,
    pub language: LanguageCode,
    pub confidence: f32,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub stability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeechSessionStatus {
    Active,
    Paused,
    Completed,
}

impl SpeechSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechSession {
    pub id: String,
    pub title: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub status: SpeechSessionStatus,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptionSegment {
    pub id: String,
    pub session_id: String,
    pub seq_id: u64,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub is_final: bool,
    pub confidence: f32,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptionFormat {
    Vtt,
    Srt,
    Json,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "detail")]
pub enum ModelInstallState {
    NotInstalled,
    Downloading {
        progress_pct: f32,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    Installed {
        installed_at: DateTime<Utc>,
        install_path: String,
    },
    Corrupted {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelFileItem {
    pub filename: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelPackManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source_language: LanguageCode,
    pub target_language: LanguageCode,
    pub architecture: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub signature: String,
    pub min_app_version: String,
    pub files: Vec<ModelFileItem>,
    pub vocab_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPackDetailedInfo {
    pub id: String,
    pub name: String,
    pub source: LanguageCode,
    pub target: LanguageCode,
    pub size_bytes: u64,
    pub version: String,
    pub sha256: String,
    pub state: ModelInstallState,
    pub architecture: String,
    pub storage_path: Option<String>,
    pub scheduled_stage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiskSpaceInfo {
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub models_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedTranslation {
    pub id: String,
    pub source_language: LanguageCode,
    pub target_language: LanguageCode,
    pub source_text: String,
    pub translated_text: String,
    pub engine_id: String,
    pub is_offline: bool,
    pub created_at: DateTime<Utc>,
    pub hit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MorphologyEntry {
    pub id: String,
    pub language: LanguageCode,
    pub surface_form: String,
    pub stem_form: String,
    pub part_of_speech: Option<String>,
    pub grammar_notes: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeakerId {
    SpeakerA,
    SpeakerB,
}

impl SpeakerId {
    pub fn other(&self) -> Self {
        match self {
            Self::SpeakerA => Self::SpeakerB,
            Self::SpeakerB => Self::SpeakerA,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SpeakerA => "SpeakerA",
            Self::SpeakerB => "SpeakerB",
        }
    }
}

impl std::fmt::Display for SpeakerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SpeakerId {
    type Err = LinguaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SpeakerA" | "speaker_a" | "A" => Ok(SpeakerId::SpeakerA),
            "SpeakerB" | "speaker_b" | "B" => Ok(SpeakerId::SpeakerB),
            other => Err(LinguaError::Validation(format!(
                "Unknown speaker id: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnState {
    Idle,
    Listening(SpeakerId),
    Processing(SpeakerId),
    Speaking(SpeakerId),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSession {
    pub id: String,
    pub title: String,
    pub speaker_a_lang: LanguageCode,
    pub speaker_b_lang: LanguageCode,
    pub active_speaker: SpeakerId,
    pub is_hands_free: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_turns: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTurnItem {
    pub id: String,
    pub session_id: String,
    pub turn_number: u32,
    pub speaker_id: SpeakerId,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub source_text: String,
    pub translated_text: String,
    pub audio_duration_ms: u64,
    pub is_interrupted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TtsVoiceInfo {
    pub id: String,
    pub name: String,
    pub language: LanguageCode,
    pub gender: String,
    pub is_offline: bool,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsSynthesizeRequest {
    pub text: String,
    pub language: LanguageCode,
    pub voice_id: Option<String>,
    pub speed: f32,
    pub pitch: f32,
}

impl Default for TtsSynthesizeRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            language: LanguageCode::EnUs,
            voice_id: None,
            speed: 1.0,
            pitch: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TtsSynthesizeResponse {
    pub audio_samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
    pub voice_id: String,
    pub language: LanguageCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FormalityMode {
    #[default]
    Neutral,
    Formal,
    Colloquial,
}

impl FormalityMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Formal => "formal",
            Self::Colloquial => "colloquial",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "formal" => Self::Formal,
            "colloquial" | "casual" => Self::Colloquial,
            _ => Self::Neutral,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    #[default]
    Closed,
    Open,
    HalfOpen,
}

impl CircuitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CircuitBreakerStatus {
    pub provider_id: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_ms: Option<u64>,
    pub total_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FallbackStep {
    pub engine_id: String,
    pub attempt: u32,
    pub success: bool,
    pub latency_ms: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelligenceTranslationMetadata {
    pub engine_selected: String,
    pub formality: FormalityMode,
    pub glossary_terms_applied: Vec<String>,
    pub transliterated_input: Option<String>,
    pub fallback_trail: Vec<FallbackStep>,
    pub chrf_estimated: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelligenceTranslationRequest {
    pub text: String,
    pub source_lang: Option<LanguageCode>,
    pub target_lang: LanguageCode,
    pub formality: FormalityMode,
    pub enable_transliteration: bool,
    pub domain_glossary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelligenceTranslationResponse {
    pub source_text: String,
    pub translated_text: String,
    pub detected_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub metadata: IntelligenceTranslationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainGlossaryEntry {
    pub id: String,
    pub domain: String,
    pub source_term: String,
    pub target_term: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub is_exact: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationSample {
    pub id: String,
    pub category: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub source_text: String,
    pub reference_translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    pub sample_id: String,
    pub category: String,
    pub hypothesis: String,
    pub reference: String,
    pub chrf_score: f64,
    pub wer_score: f64,
    pub cer_score: f64,
    pub critical_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationReport {
    pub total_samples: usize,
    pub mean_chrf: f64,
    pub mean_wer: f64,
    pub mean_cer: f64,
    pub samples: Vec<EvaluationResult>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceProfile {
    #[default]
    Balanced,
    PowerSaver,
    HighPerformance,
}

impl ResourceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::PowerSaver => "power_saver",
            Self::HighPerformance => "high_performance",
        }
    }

    pub fn max_cache_entries(&self) -> usize {
        match self {
            Self::PowerSaver => 500,
            Self::Balanced => 2000,
            Self::HighPerformance => 8000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransliterationRequest {
    pub text: String,
    pub target_lang: LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransliterationResponse {
    pub input: String,
    pub transliterated: String,
    pub target_lang: LanguageCode,
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
            stage: 5,
            stage_name: "Translation Intelligence & Optimization".into(),
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

    #[test]
    fn test_capability_matrix_stage_3() {
        let matrix = CapabilityMatrix::stage_3_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(
            matrix.speech_synthesis,
            CapabilityState::ScheduledForStage(4)
        );
    }

    #[test]
    fn test_capability_matrix_stage_4() {
        let matrix = CapabilityMatrix::stage_4_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(matrix.speech_synthesis, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.live_conversation, CapabilityState::Active);
        assert_eq!(
            matrix.media_subtitles,
            CapabilityState::ScheduledForStage(6)
        );
    }

    #[test]
    fn test_capability_matrix_stage_5() {
        let matrix = CapabilityMatrix::stage_5_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(matrix.speech_synthesis, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.live_conversation, CapabilityState::Active);
        assert_eq!(
            matrix.media_subtitles,
            CapabilityState::ScheduledForStage(6)
        );
    }

    #[test]
    fn test_formality_mode_and_resource_profile() {
        assert_eq!(FormalityMode::default(), FormalityMode::Neutral);
        assert_eq!(FormalityMode::parse("formal"), FormalityMode::Formal);
        assert_eq!(FormalityMode::parse("casual"), FormalityMode::Colloquial);
        assert_eq!(FormalityMode::Formal.as_str(), "formal");

        assert_eq!(ResourceProfile::default(), ResourceProfile::Balanced);
        assert_eq!(ResourceProfile::PowerSaver.max_cache_entries(), 500);
        assert_eq!(ResourceProfile::HighPerformance.max_cache_entries(), 8000);
    }

    #[test]
    fn test_speaker_id_toggle_and_parse() {
        let speaker = SpeakerId::SpeakerA;
        assert_eq!(speaker.other(), SpeakerId::SpeakerB);
        assert_eq!(speaker.to_string(), "SpeakerA");

        use std::str::FromStr;
        let parsed = SpeakerId::from_str("SpeakerB").unwrap();
        assert_eq!(parsed, SpeakerId::SpeakerB);
        assert_eq!(parsed.other(), SpeakerId::SpeakerA);
    }
}
