use crate::error::LinguaError;
use crate::models::{AppInfo, CapabilityMatrix, ModelPackInfo, UserSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcRequest<T> {
    pub request_id: String,
    pub timestamp_ms: u64,
    pub payload: T,
}

impl<T> IpcRequest<T> {
    pub fn new(payload: T) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcResponse<T> {
    pub request_id: String,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<IpcErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

impl From<LinguaError> for IpcErrorPayload {
    fn from(err: LinguaError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.to_string(),
            details: None,
            recoverable: err.is_recoverable(),
        }
    }
}

impl<T> IpcResponse<T> {
    pub fn ok(request_id: impl Into<String>, data: T) -> Self {
        Self {
            request_id: request_id.into(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(request_id: impl Into<String>, err: LinguaError) -> Self {
        Self {
            request_id: request_id.into(),
            success: false,
            data: None,
            error: Some(IpcErrorPayload::from(err)),
        }
    }
}

// Concrete Command Payloads

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAppInfoResponse {
    pub app_info: AppInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetCapabilitiesResponse {
    pub capabilities: CapabilityMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserSettingsResponse {
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveUserSettingsRequest {
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListModelPacksResponse {
    pub model_packs: Vec<ModelPackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseHealthInfo {
    pub status: String,
    pub path: String,
    pub schema_version: u32,
    pub settings_persisted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslateCmdRequest {
    pub source_language: crate::language::LanguageCode,
    pub target_language: crate::language::LanguageCode,
    pub text: String,
    pub allow_cloud: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslateCmdResponse {
    pub source_language: crate::language::LanguageCode,
    pub target_language: crate::language::LanguageCode,
    pub original_text: String,
    pub translated_text: String,
    pub engine_id: String,
    pub is_offline: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionarySearchCmdRequest {
    pub query: String,
    pub language: Option<crate::language::LanguageCode>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionarySearchCmdResponse {
    pub entries: Vec<crate::models::DictionaryEntry>,
    pub query: String,
    pub total_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectLanguageCmdRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectLanguageCmdResponse {
    pub detected: Option<crate::language::LanguageCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddFavoriteCmdRequest {
    pub source_language: crate::language::LanguageCode,
    pub target_language: crate::language::LanguageCode,
    pub source_text: String,
    pub translated_text: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoveFavoriteCmdRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadModelCmdRequest {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancelDownloadCmdRequest {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteModelCmdRequest {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyModelCmdRequest {
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyModelCmdResponse {
    pub model_id: String,
    pub is_valid: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetDiskSpaceCmdResponse {
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub models_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListDetailedModelPacksResponse {
    pub model_packs: Vec<crate::models::ModelPackDetailedInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartSpeechSessionCmdRequest {
    pub source_language: crate::language::LanguageCode,
    pub target_language: crate::language::LanguageCode,
    pub title: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartSpeechSessionCmdResponse {
    pub session: crate::models::SpeechSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PauseSpeechSessionCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResumeSpeechSessionCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StopSpeechSessionCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StopSpeechSessionCmdResponse {
    pub session: crate::models::SpeechSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAudioDevicesCmdResponse {
    pub devices: Vec<crate::models::AudioDeviceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetSessionCaptionsCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetSessionCaptionsCmdResponse {
    pub captions: Vec<crate::models::CaptionSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportCaptionsCmdRequest {
    pub session_id: String,
    pub format: crate::models::CaptionFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportCaptionsCmdResponse {
    pub content: String,
    pub filename: String,
    pub format: crate::models::CaptionFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedAudioChunkCmdRequest {
    pub session_id: String,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedAudioChunkCmdResponse {
    pub session_id: String,
    pub new_events: Vec<crate::models::AsrTranscriptEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartConversationCmdRequest {
    pub speaker_a_lang: crate::language::LanguageCode,
    pub speaker_b_lang: crate::language::LanguageCode,
    pub title: Option<String>,
    pub is_hands_free: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartConversationCmdResponse {
    pub session: crate::models::ConversationSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitTurnCmdRequest {
    pub session_id: String,
    pub speaker_id: crate::models::SpeakerId,
    pub source_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitTurnCmdResponse {
    pub turn: crate::models::ConversationTurnItem,
    pub audio_samples: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwitchSpeakerCmdRequest {
    pub session_id: String,
    pub next_speaker: Option<crate::models::SpeakerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwitchSpeakerCmdResponse {
    pub session: crate::models::ConversationSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterruptConversationCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterruptConversationCmdResponse {
    pub session_id: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StopConversationCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StopConversationCmdResponse {
    pub session: crate::models::ConversationSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetConversationTurnsCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetConversationTurnsCmdResponse {
    pub turns: Vec<crate::models::ConversationTurnItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetTtsVoicesCmdResponse {
    pub voices: Vec<crate::models::TtsVoiceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SynthesizeSpeechCmdRequest {
    pub text: String,
    pub language: crate::language::LanguageCode,
    pub voice_id: Option<String>,
    pub speed: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SynthesizeSpeechCmdResponse {
    pub response: crate::models::TtsSynthesizeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslateIntelligenceCmdRequest {
    pub text: String,
    pub source_lang: Option<crate::language::LanguageCode>,
    pub target_lang: crate::language::LanguageCode,
    pub formality: crate::models::FormalityMode,
    pub enable_transliteration: bool,
    pub domain_glossary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslateIntelligenceCmdResponse {
    pub response: crate::models::IntelligenceTranslationResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransliterateCmdRequest {
    pub text: String,
    pub target_lang: crate::language::LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransliterateCmdResponse {
    pub response: crate::models::TransliterationResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetCircuitBreakersCmdResponse {
    pub circuit_breakers: Vec<crate::models::CircuitBreakerStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResetCircuitBreakerCmdRequest {
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListGlossaryTermsCmdRequest {
    pub domain: Option<String>,
    pub source_lang: Option<crate::language::LanguageCode>,
    pub target_lang: Option<crate::language::LanguageCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListGlossaryTermsCmdResponse {
    pub terms: Vec<crate::models::DomainGlossaryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGlossaryTermCmdRequest {
    pub domain: String,
    pub source_term: String,
    pub target_term: String,
    pub source_lang: crate::language::LanguageCode,
    pub target_lang: crate::language::LanguageCode,
    pub is_exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteGlossaryTermCmdRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunEvaluationCmdResponse {
    pub report: crate::models::EvaluationReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetResourceProfileCmdRequest {
    pub profile: crate::models::ResourceProfile,
}

// Stage 6 IPC Payloads: Media, Captions, OCR, Document & Platform Integrations

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartMediaTranscriptionCmdRequest {
    pub file_name: String,
    pub file_bytes: Vec<u8>,
    pub source_lang: crate::language::LanguageCode,
    pub target_lang: crate::language::LanguageCode,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartMediaTranscriptionCmdResponse {
    pub session: crate::models::MediaSession,
    pub segments: Vec<crate::models::MediaSubtitleSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListMediaSegmentsCmdRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListMediaSegmentsCmdResponse {
    pub segments: Vec<crate::models::MediaSubtitleSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateMediaSegmentCmdRequest {
    pub segment_id: String,
    pub source_text: String,
    pub translated_text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportSubtitlesCmdRequest {
    pub session_id: String,
    pub format: crate::models::MediaSubtitleExportFormat,
    pub target_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportSubtitlesCmdResponse {
    pub content: String,
    pub file_name: String,
    pub format: crate::models::MediaSubtitleExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetOverlaySettingsCmdResponse {
    pub settings: crate::models::OverlaySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveOverlaySettingsCmdRequest {
    pub settings: crate::models::OverlaySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformOcrCmdRequest {
    pub image_bytes: Vec<u8>,
    pub mime_type: String,
    pub source_lang: Option<crate::language::LanguageCode>,
    pub target_lang: Option<crate::language::LanguageCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformOcrCmdResponse {
    pub result: crate::models::OcrResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslateDocumentCmdRequest {
    pub file_name: String,
    pub content: Vec<u8>,
    pub format: crate::models::DocumentFormat,
    pub source_lang: crate::language::LanguageCode,
    pub target_lang: crate::language::LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslateDocumentCmdResponse {
    pub job: crate::models::DocumentTranslationJob,
    pub blocks: Vec<crate::models::DocumentBlock>,
    pub exported_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGlobalShortcutsCmdResponse {
    pub config: crate::models::GlobalShortcutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveGlobalShortcutsCmdRequest {
    pub config: crate::models::GlobalShortcutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListLearningDecksCmdResponse {
    pub decks: Vec<crate::models::LearningDeck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetLearningDeckCardsCmdRequest {
    pub deck_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetLearningDeckCardsCmdResponse {
    pub cards: Vec<crate::models::LearningCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewCardCmdRequest {
    pub card_id: String,
    pub rating: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewCardCmdResponse {
    pub result: crate::models::ReviewResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetLearnerProgressCmdResponse {
    pub progress: crate::models::LearnerProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssessPronunciationCmdRequest {
    pub request: crate::models::PronunciationAssessmentRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssessPronunciationCmdResponse {
    pub result: crate::models::PronunciationAssessmentResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetListeningChallengesCmdRequest {
    pub language: crate::language::LanguageCode,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetListeningChallengesCmdResponse {
    pub challenges: Vec<crate::models::ListeningChallenge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGrammarTopicsCmdRequest {
    pub language: crate::language::LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetGrammarTopicsCmdResponse {
    pub topics: Vec<crate::models::GrammarGuideTopic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetDataPrivacySettingsCmdResponse {
    pub settings: crate::models::DataPrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveDataPrivacySettingsCmdRequest {
    pub settings: crate::models::DataPrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportUserDataCmdResponse {
    pub archive: crate::models::ExportUserDataArchive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurgeUserDataCmdRequest {
    pub confirm_phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurgeUserDataCmdResponse {
    pub result: crate::models::PurgeUserDataResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPlatformReleaseStatusCmdResponse {
    pub status: Vec<crate::models::PlatformReleaseStatus>,
}

// =========================================================================
// Stage 8 IPC Messages: Commercial & Enterprise Platform
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAccountInfoCmdRequest {
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAccountInfoCmdResponse {
    pub account: crate::models::UserAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterDeviceCmdRequest {
    pub account_id: String,
    pub device_name: String,
    pub device_type: crate::models::DeviceType,
    pub public_key_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterDeviceCmdResponse {
    pub device: crate::models::DeviceRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListDevicesCmdResponse {
    pub devices: Vec<crate::models::DeviceRegistration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateApiKeyCmdRequest {
    pub user_id: String,
    pub org_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateApiKeyCmdResponse {
    pub api_key_record: crate::models::ApiKeyRecord,
    pub raw_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListApiKeysCmdResponse {
    pub api_keys: Vec<crate::models::ApiKeyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUsageSummaryCmdResponse {
    pub summary: crate::models::UsageSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListInvoicesCmdResponse {
    pub invoices: Vec<crate::models::BillingInvoiceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateOrgCmdRequest {
    pub name: String,
    pub slug: String,
    pub owner_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrgCmdResponse {
    pub organization: crate::models::Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListOrgMembersCmdResponse {
    pub members: Vec<crate::models::OrgMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListOrgGlossariesCmdResponse {
    pub glossaries: Vec<crate::models::OrgGlossary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateOrgGlossaryCmdRequest {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub source_lang: crate::language::LanguageCode,
    pub target_lang: crate::language::LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListAuditEventsCmdResponse {
    pub events: Vec<crate::models::AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetWorkerMetricsCmdResponse {
    pub metrics: crate::models::WorkerMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_envelope_ok_and_err() {
        let ok_resp: IpcResponse<String> = IpcResponse::ok("req-123", "success data".to_string());
        assert!(ok_resp.success);
        assert_eq!(ok_resp.data.as_deref(), Some("success data"));
        assert!(ok_resp.error.is_none());

        let err_resp: IpcResponse<()> = IpcResponse::err(
            "req-456",
            LinguaError::IdenticalLanguagePair("kn-IN".into(), "kn-IN".into()),
        );
        assert!(!err_resp.success);
        assert!(err_resp.data.is_none());
        let err_payload = err_resp.error.unwrap();
        assert_eq!(err_payload.code, "INVALID_LANGUAGE_PAIR");
        assert!(err_payload.recoverable);
    }

    #[test]
    fn test_ipc_request_serialization() {
        let req = IpcRequest::new(GetAppInfoResponse {
            app_info: AppInfo::default(),
        });
        let json = serde_json::to_string(&req).expect("Failed to serialize");
        let deserialized: IpcRequest<GetAppInfoResponse> =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(req.request_id, deserialized.request_id);
    }
}
