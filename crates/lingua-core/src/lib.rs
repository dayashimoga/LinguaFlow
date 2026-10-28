pub mod error;
pub mod ipc;
pub mod language;
pub mod models;
pub mod unicode;

pub use error::LinguaError;
pub use ipc::{
    AddFavoriteCmdRequest, CancelDownloadCmdRequest, CreateGlossaryTermCmdRequest,
    DatabaseHealthInfo, DeleteGlossaryTermCmdRequest, DeleteModelCmdRequest,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DictionarySearchCmdRequest,
    DictionarySearchCmdResponse, DownloadModelCmdRequest, ExportCaptionsCmdRequest,
    ExportCaptionsCmdResponse, ExportSubtitlesCmdRequest, ExportSubtitlesCmdResponse,
    FeedAudioChunkCmdRequest, FeedAudioChunkCmdResponse, GetAppInfoResponse,
    GetAudioDevicesCmdResponse, GetCapabilitiesResponse, GetCircuitBreakersCmdResponse,
    GetConversationTurnsCmdRequest, GetConversationTurnsCmdResponse, GetDiskSpaceCmdResponse,
    GetGlobalShortcutsCmdResponse, GetOverlaySettingsCmdResponse, GetSessionCaptionsCmdRequest,
    GetSessionCaptionsCmdResponse, GetTtsVoicesCmdResponse, GetUserSettingsResponse,
    InterruptConversationCmdRequest, InterruptConversationCmdResponse, IpcErrorPayload, IpcRequest,
    IpcResponse, ListDetailedModelPacksResponse, ListGlossaryTermsCmdRequest,
    ListGlossaryTermsCmdResponse, ListMediaSegmentsCmdRequest, ListMediaSegmentsCmdResponse,
    ListModelPacksResponse, PauseSpeechSessionCmdRequest, PerformOcrCmdRequest,
    PerformOcrCmdResponse, RemoveFavoriteCmdRequest, ResetCircuitBreakerCmdRequest,
    ResumeSpeechSessionCmdRequest, RunEvaluationCmdResponse, SaveGlobalShortcutsCmdRequest,
    SaveOverlaySettingsCmdRequest, SaveUserSettingsRequest, SetResourceProfileCmdRequest,
    StartConversationCmdRequest, StartConversationCmdResponse, StartMediaTranscriptionCmdRequest,
    StartMediaTranscriptionCmdResponse, StartSpeechSessionCmdRequest,
    StartSpeechSessionCmdResponse, StopConversationCmdRequest, StopConversationCmdResponse,
    StopSpeechSessionCmdRequest, StopSpeechSessionCmdResponse, SubmitTurnCmdRequest,
    SubmitTurnCmdResponse, SwitchSpeakerCmdRequest, SwitchSpeakerCmdResponse,
    SynthesizeSpeechCmdRequest, SynthesizeSpeechCmdResponse, TranslateCmdRequest,
    TranslateCmdResponse, TranslateDocumentCmdRequest, TranslateDocumentCmdResponse,
    TranslateIntelligenceCmdRequest, TranslateIntelligenceCmdResponse, TransliterateCmdRequest,
    TransliterateCmdResponse, UpdateMediaSegmentCmdRequest, VerifyModelCmdRequest,
    VerifyModelCmdResponse,
};
pub use language::{LanguageCode, LanguagePair};
pub use models::{
    AppInfo, AsrTranscriptEvent, AudioDeviceItem, AudioSettings, CachedTranslation,
    CapabilityMatrix, CapabilityState, CaptionFormat, CaptionSegment, CircuitBreakerStatus,
    CircuitState, ConversationSession, ConversationTurnItem, DictionaryEntry, DictionaryExample,
    DiskSpaceInfo, DocumentBlock, DocumentFormat, DocumentJobStatus, DocumentTranslationJob,
    DomainGlossaryEntry, EnginePreference, EvaluationReport, EvaluationResult, EvaluationSample,
    FallbackStep, FavoriteItem, FormalityMode, GlobalShortcutConfig,
    IntelligenceTranslationMetadata, IntelligenceTranslationRequest,
    IntelligenceTranslationResponse, MediaFormat, MediaProcessingProgress, MediaProgressStatus,
    MediaSession, MediaSubtitleExportFormat, MediaSubtitleSegment, ModelFileItem,
    ModelInstallState, ModelPackDetailedInfo, ModelPackInfo, ModelPackManifest, MorphologyEntry,
    OcrBoundingBox, OcrResult, OcrTextBlock, OverlaySettings, ResourceProfile, SpeakerId,
    SpeechSession, SpeechSessionStatus, SystemAudioPermission, ThemeMode, TranslationRecord,
    TransliterationRequest, TransliterationResponse, TtsSynthesizeRequest, TtsSynthesizeResponse,
    TtsVoiceInfo, TurnState, UserSettings, VadState,
};
pub use unicode::{
    detect_language, detect_script, normalize_text_nfc, sanitize_translation_input, DetectedScript,
};
