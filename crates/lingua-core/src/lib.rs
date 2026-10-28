pub mod error;
pub mod ipc;
pub mod language;
pub mod models;
pub mod unicode;

pub use error::LinguaError;
pub use ipc::{
    AddFavoriteCmdRequest, CancelDownloadCmdRequest, DatabaseHealthInfo, DeleteModelCmdRequest,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DictionarySearchCmdRequest,
    DictionarySearchCmdResponse, DownloadModelCmdRequest, GetAppInfoResponse,
    GetCapabilitiesResponse, GetDiskSpaceCmdResponse, GetUserSettingsResponse, IpcErrorPayload,
    IpcRequest, IpcResponse, ListDetailedModelPacksResponse, ListModelPacksResponse,
    RemoveFavoriteCmdRequest, SaveUserSettingsRequest, TranslateCmdRequest, TranslateCmdResponse,
    VerifyModelCmdRequest, VerifyModelCmdResponse,
};
pub use language::{LanguageCode, LanguagePair};
pub use models::{
    AppInfo, AudioSettings, CachedTranslation, CapabilityMatrix, CapabilityState, DictionaryEntry,
    DictionaryExample, DiskSpaceInfo, EnginePreference, FavoriteItem, ModelFileItem,
    ModelInstallState, ModelPackDetailedInfo, ModelPackInfo, ModelPackManifest, MorphologyEntry,
    ThemeMode, TranslationRecord, UserSettings,
};
pub use unicode::{
    detect_language, detect_script, normalize_text_nfc, sanitize_translation_input, DetectedScript,
};
