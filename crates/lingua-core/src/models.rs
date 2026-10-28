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
    pub language_learning: CapabilityState,
    pub enterprise_platform: CapabilityState,
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
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
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
        }
    }

    pub fn stage_6_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::Active,
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::Active,
            media_subtitles: CapabilityState::Active,
            language_learning: CapabilityState::ScheduledForStage(7),
            enterprise_platform: CapabilityState::ScheduledForStage(8),
        }
    }

    pub fn stage_7_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::Active,
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::Active,
            media_subtitles: CapabilityState::Active,
            language_learning: CapabilityState::Active,
            enterprise_platform: CapabilityState::ScheduledForStage(8),
        }
    }

    pub fn stage_8_active() -> Self {
        Self {
            dictionary: CapabilityState::Active,
            text_translation: CapabilityState::Active,
            streaming_asr: CapabilityState::Active,
            speech_synthesis: CapabilityState::Active,
            offline_models: CapabilityState::Active,
            live_conversation: CapabilityState::Active,
            media_subtitles: CapabilityState::Active,
            language_learning: CapabilityState::Active,
            enterprise_platform: CapabilityState::Active,
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
            stage: 8,
            stage_name: "Commercial & Enterprise Platform".into(),
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

// Stage 6 Models: Media, Live Overlay, OCR, Document Translation, Integrations

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MediaFormat {
    #[default]
    Wav,
    Mp3,
    Mp4,
    Webm,
    Ogg,
}

impl MediaFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Ogg => "ogg",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "mp4" => Self::Mp4,
            "webm" => Self::Webm,
            "ogg" => Self::Ogg,
            _ => Self::Wav,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MediaProgressStatus {
    #[default]
    Queued,
    Processing,
    Completed,
    Cancelled,
    Failed,
}

impl MediaProgressStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            "failed" => Self::Failed,
            _ => Self::Queued,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaSubtitleSegment {
    pub id: String,
    pub session_id: String,
    pub seq_id: u64,
    pub start_ms: u64,
    pub end_ms: u64,
    pub source_text: String,
    pub translated_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaSession {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub duration_ms: u64,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub format: MediaFormat,
    pub status: MediaProgressStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MediaSubtitleExportFormat {
    #[default]
    Srt,
    Vtt,
    Txt,
    Json,
}

impl MediaSubtitleExportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Srt => "srt",
            Self::Vtt => "vtt",
            Self::Txt => "txt",
            Self::Json => "json",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "vtt" => Self::Vtt,
            "txt" => Self::Txt,
            "json" => Self::Json,
            _ => Self::Srt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaProcessingProgress {
    pub session_id: String,
    pub percent: f32,
    pub current_time_ms: u64,
    pub total_time_ms: u64,
    pub status: MediaProgressStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverlaySettings {
    pub enabled: bool,
    pub always_on_top: bool,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub font_size: u32,
    pub font_family: String,
    pub text_color: String,
    pub background_color: String,
    pub opacity: f32,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub show_source: bool,
    pub show_translated: bool,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            always_on_top: true,
            x: 100,
            y: 100,
            width: 700,
            height: 140,
            font_size: 22,
            font_family: "Inter, Noto Sans Kannada, Noto Sans Tamil, sans-serif".into(),
            text_color: "#ffffff".into(),
            background_color: "#0a0d14".into(),
            opacity: 0.88,
            source_lang: LanguageCode::EnUs,
            target_lang: LanguageCode::KnIn,
            show_source: true,
            show_translated: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SystemAudioPermission {
    #[default]
    NotDetermined,
    Granted,
    Denied,
    Restricted,
}

impl SystemAudioPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotDetermined => "not_determined",
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::Restricted => "restricted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OcrBoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OcrTextBlock {
    pub id: String,
    pub text: String,
    pub confidence: f32,
    pub bounding_box: OcrBoundingBox,
    pub line_number: u32,
    pub translated_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OcrResult {
    pub blocks: Vec<OcrTextBlock>,
    pub detected_language: LanguageCode,
    pub full_text: String,
    pub translated_full_text: Option<String>,
    pub processing_time_ms: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    #[default]
    Txt,
    Markdown,
    Pdf,
    Csv,
    Html,
}

impl DocumentFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Markdown => "markdown",
            Self::Pdf => "pdf",
            Self::Csv => "csv",
            Self::Html => "html",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Self::Markdown,
            "pdf" => Self::Pdf,
            "csv" | "tsv" => Self::Csv,
            "html" | "htm" => Self::Html,
            _ => Self::Txt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentBlock {
    pub index: usize,
    pub block_type: String,
    pub source_text: String,
    pub translated_text: String,
    pub page_number: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentJobStatus {
    #[default]
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl DocumentJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Queued,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentTranslationJob {
    pub id: String,
    pub file_name: String,
    pub format: DocumentFormat,
    pub file_size_bytes: u64,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub total_blocks: usize,
    pub completed_blocks: usize,
    pub status: DocumentJobStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalShortcutConfig {
    pub quick_translate: String,
    pub overlay_toggle: String,
    pub enabled: bool,
}

impl Default for GlobalShortcutConfig {
    fn default() -> Self {
        Self {
            quick_translate: "CommandOrControl+Shift+L".into(),
            overlay_toggle: "CommandOrControl+Shift+C".into(),
            enabled: true,
        }
    }
}

// =========================================================================
// Stage 7 Models: Language Learning, Production Security & Release Certification
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningCard {
    pub id: String,
    pub deck_id: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub source_term: String,
    pub target_term: String,
    pub romanization: Option<String>,
    pub ipa: Option<String>,
    pub part_of_speech: String,
    pub example_source: String,
    pub example_target: String,
    pub grammar_notes: Option<String>,
    pub repetition: u32,
    pub interval_days: f32,
    pub ease_factor: f32,
    pub next_review_at: DateTime<Utc>,
    pub total_reviews: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningDeck {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub total_cards: usize,
    pub due_cards: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewRating {
    Blackout = 0,
    Incorrect = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
    Perfect = 5,
}

impl ReviewRating {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::Blackout,
            1 => Self::Incorrect,
            2 => Self::Hard,
            3 => Self::Good,
            4 => Self::Easy,
            _ => Self::Perfect,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewResult {
    pub card_id: String,
    pub rating: u8,
    pub new_repetition: u32,
    pub new_interval_days: f32,
    pub new_ease_factor: f32,
    pub next_review_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearnerProgress {
    pub total_cards_studied: u32,
    pub cards_due_today: u32,
    pub cards_mastered: u32,
    pub current_streak_days: u32,
    pub retention_rate: f32,
    pub study_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PronunciationAssessmentRequest {
    pub reference_text: String,
    pub spoken_transcript: String,
    pub language: LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PronunciationAssessmentResult {
    pub accuracy_score: f32,
    pub wer: f32,
    pub cer: f32,
    pub chrf: f32,
    pub phoneme_feedback: Vec<String>,
    pub is_passing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListeningChallenge {
    pub id: String,
    pub prompt_audio_text: String,
    pub language: LanguageCode,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrammarGuideTopic {
    pub id: String,
    pub language: LanguageCode,
    pub title: String,
    pub category: String,
    pub content_markdown: String,
    pub examples: Vec<DictionaryExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataPrivacySettings {
    pub analytics_enabled: bool,
    pub local_transcripts_retention_days: u32,
    pub microphone_consent: bool,
    pub camera_consent: bool,
    pub cloud_processing_consent: bool,
    pub policy_version_accepted: String,
}

impl Default for DataPrivacySettings {
    fn default() -> Self {
        Self {
            analytics_enabled: false,
            local_transcripts_retention_days: 30,
            microphone_consent: true,
            camera_consent: true,
            cloud_processing_consent: false,
            policy_version_accepted: "2026.10-v1".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportUserDataArchive {
    pub export_id: String,
    pub exported_at: DateTime<Utc>,
    pub app_version: String,
    pub translation_history_count: usize,
    pub learning_cards_count: usize,
    pub checksum_sha256: String,
    pub raw_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurgeUserDataResult {
    pub purged_history_rows: usize,
    pub purged_sessions_rows: usize,
    pub purged_cards_rows: usize,
    pub purged_media_rows: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformReleaseStatus {
    pub platform: String,
    pub binary_name: String,
    pub is_signed: bool,
    pub signature_details: String,
    pub limitations: Vec<String>,
}

// =========================================================================
// Stage 8 Models: Commercial & Enterprise Platform
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountTier {
    #[default]
    Free,
    Pro,
    Enterprise,
}

impl AccountTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }

    pub fn max_monthly_words(&self) -> u64 {
        match self {
            Self::Free => 10_000,
            Self::Pro => 500_000,
            Self::Enterprise => 100_000_000,
        }
    }

    pub fn max_monthly_audio_seconds(&self) -> u64 {
        match self {
            Self::Free => 600,
            Self::Pro => 18_000,
            Self::Enterprise => 360_000,
        }
    }

    pub fn max_devices(&self) -> usize {
        match self {
            Self::Free => 1,
            Self::Pro => 5,
            Self::Enterprise => 100,
        }
    }

    pub fn rate_limit_per_minute(&self) -> u32 {
        match self {
            Self::Free => 60,
            Self::Pro => 300,
            Self::Enterprise => 1200,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    #[default]
    Active,
    Suspended,
    PendingVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAccount {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub tier: AccountTier,
    pub is_anonymous: bool,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    #[default]
    DesktopWindows,
    DesktopMac,
    DesktopLinux,
    MobileAndroid,
    MobileIos,
    WebBrowser,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceRegistration {
    pub id: String,
    pub user_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub public_key_fingerprint: String,
    pub registered_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKeyRecord {
    pub id: String,
    pub user_id: String,
    pub org_id: Option<String>,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub rate_limit_per_min: u32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Owner,
    Admin,
    LinguistEditor,
    #[default]
    Member,
    Viewer,
}

impl OrgRole {
    pub fn can_manage_team(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_edit_glossary(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::LinguistEditor)
    }

    pub fn can_manage_api_keys(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub max_seats: u32,
    pub sso_enabled: bool,
    pub data_retention_days: u32,
    pub enforce_zero_transcript_logging: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrgMember {
    pub id: String,
    pub org_id: String,
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub role: OrgRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OrgGlossaryStatus {
    #[default]
    Draft,
    Approved,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrgGlossary {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub status: OrgGlossaryStatus,
    pub terms_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrgGlossaryTerm {
    pub id: String,
    pub glossary_id: String,
    pub source_term: String,
    pub target_term: String,
    pub notes: Option<String>,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageType {
    CloudTranslationWords,
    CloudSpeechSeconds,
    CloudOcrPages,
    ApiRequests,
}

impl UsageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CloudTranslationWords => "cloud_translation_words",
            Self::CloudSpeechSeconds => "cloud_speech_seconds",
            Self::CloudOcrPages => "cloud_ocr_pages",
            Self::ApiRequests => "api_requests",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageRecord {
    pub id: String,
    pub account_id: String,
    pub org_id: Option<String>,
    pub usage_type: UsageType,
    pub quantity: u64,
    pub billable_cents: u64,
    pub was_successful: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageSummary {
    pub account_id: String,
    pub tier: AccountTier,
    pub words_used: u64,
    pub words_limit: u64,
    pub speech_seconds_used: u64,
    pub speech_seconds_limit: u64,
    pub ocr_pages_used: u64,
    pub ocr_pages_limit: u64,
    pub current_period_cents: u64,
    pub projected_month_end_cents: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    #[default]
    Pending,
    Paid,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: u64,
    pub unit_price_cents: u64,
    pub total_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingInvoiceRecord {
    pub id: String,
    pub account_id: String,
    pub org_id: Option<String>,
    pub invoice_number: String,
    pub billing_period: String,
    pub amount_cents: u64,
    pub currency: String,
    pub status: InvoiceStatus,
    pub line_items: Vec<InvoiceLineItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: String,
    pub org_id: Option<String>,
    pub actor_id: String,
    pub actor_email: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details: String,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerMetrics {
    pub active_workers: u32,
    pub queue_depth: u32,
    pub max_queue_capacity: u32,
    pub processed_tasks: u64,
    pub failed_tasks: u64,
    pub avg_latency_ms: f64,
    pub is_accepting_tasks: bool,
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

    #[test]
    fn test_capability_matrix_stage_6() {
        let matrix = CapabilityMatrix::stage_6_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(matrix.speech_synthesis, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.live_conversation, CapabilityState::Active);
        assert_eq!(matrix.media_subtitles, CapabilityState::Active);
    }

    #[test]
    fn test_stage_6_models() {
        assert_eq!(MediaFormat::parse("MP4"), MediaFormat::Mp4);
        assert_eq!(MediaFormat::Wav.as_str(), "wav");

        assert_eq!(
            MediaSubtitleExportFormat::parse("vtt"),
            MediaSubtitleExportFormat::Vtt
        );
        assert_eq!(MediaSubtitleExportFormat::Srt.as_str(), "srt");

        assert_eq!(DocumentFormat::parse("pdf"), DocumentFormat::Pdf);
        assert_eq!(DocumentFormat::parse("md"), DocumentFormat::Markdown);
        assert_eq!(DocumentFormat::Txt.as_str(), "txt");

        let overlay = OverlaySettings::default();
        assert!(overlay.enabled);
        assert_eq!(overlay.font_size, 22);

        let shortcuts = GlobalShortcutConfig::default();
        assert!(shortcuts.enabled);
        assert_eq!(shortcuts.quick_translate, "CommandOrControl+Shift+L");
    }

    #[test]
    fn test_capability_matrix_stage_7() {
        let matrix = CapabilityMatrix::stage_7_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(matrix.speech_synthesis, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.live_conversation, CapabilityState::Active);
        assert_eq!(matrix.media_subtitles, CapabilityState::Active);
        assert_eq!(matrix.language_learning, CapabilityState::Active);
    }

    #[test]
    fn test_stage_7_models_and_defaults() {
        let privacy = DataPrivacySettings::default();
        assert!(!privacy.analytics_enabled);
        assert!(privacy.microphone_consent);
        assert_eq!(privacy.policy_version_accepted, "2026.10-v1");

        assert_eq!(ReviewRating::from_u8(0), ReviewRating::Blackout);
        assert_eq!(ReviewRating::from_u8(3), ReviewRating::Good);
        assert_eq!(ReviewRating::from_u8(5), ReviewRating::Perfect);

        let app_info = AppInfo::default();
        assert_eq!(app_info.stage, 8);
        assert_eq!(app_info.stage_name, "Commercial & Enterprise Platform");
    }

    #[test]
    fn test_stage_8_models_and_capabilities() {
        let matrix = CapabilityMatrix::stage_8_active();
        assert_eq!(matrix.dictionary, CapabilityState::Active);
        assert_eq!(matrix.text_translation, CapabilityState::Active);
        assert_eq!(matrix.streaming_asr, CapabilityState::Active);
        assert_eq!(matrix.speech_synthesis, CapabilityState::Active);
        assert_eq!(matrix.offline_models, CapabilityState::Active);
        assert_eq!(matrix.live_conversation, CapabilityState::Active);
        assert_eq!(matrix.media_subtitles, CapabilityState::Active);
        assert_eq!(matrix.language_learning, CapabilityState::Active);
        assert_eq!(matrix.enterprise_platform, CapabilityState::Active);

        assert_eq!(AccountTier::Free.as_str(), "free");
        assert_eq!(AccountTier::Pro.as_str(), "pro");
        assert_eq!(AccountTier::Enterprise.as_str(), "enterprise");
        assert_eq!(AccountTier::Free.max_monthly_words(), 10_000);
        assert_eq!(AccountTier::Enterprise.max_devices(), 100);

        assert!(OrgRole::Owner.can_manage_team());
        assert!(OrgRole::Admin.can_manage_team());
        assert!(!OrgRole::Member.can_manage_team());
        assert!(OrgRole::LinguistEditor.can_edit_glossary());

        assert_eq!(
            UsageType::CloudTranslationWords.as_str(),
            "cloud_translation_words"
        );
    }
}
