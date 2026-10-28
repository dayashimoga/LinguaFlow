pub mod error;
pub mod ipc;
pub mod language;
pub mod models;
pub mod unicode;

pub use error::LinguaError;
pub use ipc::{
    AddFavoriteCmdRequest, DatabaseHealthInfo, DetectLanguageCmdRequest, DetectLanguageCmdResponse,
    DictionarySearchCmdRequest, DictionarySearchCmdResponse, GetAppInfoResponse,
    GetCapabilitiesResponse, GetUserSettingsResponse, IpcErrorPayload, IpcRequest, IpcResponse,
    ListModelPacksResponse, RemoveFavoriteCmdRequest, SaveUserSettingsRequest, TranslateCmdRequest,
    TranslateCmdResponse,
};
pub use language::{LanguageCode, LanguagePair};
pub use models::{
    AppInfo, AudioSettings, CapabilityMatrix, CapabilityState, DictionaryEntry, DictionaryExample,
    EnginePreference, FavoriteItem, ModelPackInfo, ThemeMode, TranslationRecord, UserSettings,
};
pub use unicode::{
    detect_language, detect_script, normalize_text_nfc, sanitize_translation_input, DetectedScript,
};
