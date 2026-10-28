pub mod error;
pub mod ipc;
pub mod language;
pub mod models;
pub mod unicode;

pub use error::LinguaError;
pub use ipc::{
    AddFavoriteCmdRequest, CancelDownloadCmdRequest, DatabaseHealthInfo, DeleteModelCmdRequest,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DictionarySearchCmdRequest,
    DictionarySearchCmdResponse, DownloadModelCmdRequest, ExportCaptionsCmdRequest,
    ExportCaptionsCmdResponse, FeedAudioChunkCmdRequest, FeedAudioChunkCmdResponse,
    GetAppInfoResponse, GetAudioDevicesCmdResponse, GetCapabilitiesResponse,
    GetConversationTurnsCmdRequest, GetConversationTurnsCmdResponse, GetDiskSpaceCmdResponse,
    GetSessionCaptionsCmdRequest, GetSessionCaptionsCmdResponse, GetTtsVoicesCmdResponse,
    GetUserSettingsResponse, InterruptConversationCmdRequest, InterruptConversationCmdResponse,
    IpcErrorPayload, IpcRequest, IpcResponse, ListDetailedModelPacksResponse,
    ListModelPacksResponse, PauseSpeechSessionCmdRequest, RemoveFavoriteCmdRequest,
    ResumeSpeechSessionCmdRequest, SaveUserSettingsRequest, StartConversationCmdRequest,
    StartConversationCmdResponse, StartSpeechSessionCmdRequest, StartSpeechSessionCmdResponse,
    StopConversationCmdRequest, StopConversationCmdResponse, StopSpeechSessionCmdRequest,
    StopSpeechSessionCmdResponse, SubmitTurnCmdRequest, SubmitTurnCmdResponse,
    SwitchSpeakerCmdRequest, SwitchSpeakerCmdResponse, SynthesizeSpeechCmdRequest,
    SynthesizeSpeechCmdResponse, TranslateCmdRequest, TranslateCmdResponse, VerifyModelCmdRequest,
    VerifyModelCmdResponse,
};
pub use language::{LanguageCode, LanguagePair};
pub use models::{
    AppInfo, AsrTranscriptEvent, AudioDeviceItem, AudioSettings, CachedTranslation,
    CapabilityMatrix, CapabilityState, CaptionFormat, CaptionSegment, ConversationSession,
    ConversationTurnItem, DictionaryEntry, DictionaryExample, DiskSpaceInfo, EnginePreference,
    FavoriteItem, ModelFileItem, ModelInstallState, ModelPackDetailedInfo, ModelPackInfo,
    ModelPackManifest, MorphologyEntry, SpeakerId, SpeechSession, SpeechSessionStatus, ThemeMode,
    TranslationRecord, TtsSynthesizeRequest, TtsSynthesizeResponse, TtsVoiceInfo, TurnState,
    UserSettings, VadState,
};
pub use unicode::{
    detect_language, detect_script, normalize_text_nfc, sanitize_translation_input, DetectedScript,
};
