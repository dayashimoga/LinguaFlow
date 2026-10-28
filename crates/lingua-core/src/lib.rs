pub mod error;
pub mod ipc;
pub mod language;
pub mod models;
pub mod unicode;

pub use error::LinguaError;
pub use ipc::{
    DatabaseHealthInfo, GetAppInfoResponse, GetCapabilitiesResponse, GetUserSettingsResponse,
    IpcErrorPayload, IpcRequest, IpcResponse, ListModelPacksResponse, SaveUserSettingsRequest,
};
pub use language::{LanguageCode, LanguagePair};
pub use models::{
    AppInfo, AudioSettings, CapabilityMatrix, CapabilityState, EnginePreference, ModelPackInfo,
    ThemeMode, UserSettings,
};
pub use unicode::{detect_script, normalize_text_nfc, sanitize_translation_input, DetectedScript};
