pub mod registry;
pub mod traits;

pub use registry::EngineRegistry;
pub use traits::{
    SpeechToTextEngine, TextToSpeechEngine, TranslationEngine, TranslationRequest,
    TranslationResponse,
};
