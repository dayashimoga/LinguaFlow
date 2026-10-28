pub mod audio_buffer;
pub mod audio_capture;
pub mod caption_coordinator;
pub mod http_provider;
pub mod lexicon_engine;
pub mod local_model_engine;
pub mod model_manager;
pub mod registry;
pub mod speech_session_manager;
pub mod streaming_asr;
pub mod traits;
pub mod vad;

pub use audio_buffer::{compute_rms_db, resample_to_16k, AudioRingBuffer};
pub use audio_capture::{AudioCaptureManager, AudioChunkEvent};
pub use caption_coordinator::CaptionCoordinator;
pub use http_provider::HttpTranslationProvider;
pub use lexicon_engine::LexiconTranslationEngine;
pub use local_model_engine::LocalNeuralTranslationEngine;
pub use model_manager::ModelManager;
pub use registry::EngineRegistry;
pub use speech_session_manager::SpeechSessionCoordinator;
pub use streaming_asr::{AsrStream, SherpaStreamingAsrEngine, StreamingAsrEngine};
pub use traits::{
    SpeechToTextEngine, TextToSpeechEngine, TranslationEngine, TranslationRequest,
    TranslationResponse,
};
pub use vad::{VadProcessResult, VoiceActivityDetector};
