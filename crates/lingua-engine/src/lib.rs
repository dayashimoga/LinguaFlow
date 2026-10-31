pub mod api_gateway;
pub mod audio_buffer;
pub mod audio_capture;
pub mod caption_coordinator;
pub mod context_nlp;
pub mod conversation_orchestrator;
pub mod document;
pub mod evaluation;
pub mod http_provider;
pub mod learning;
pub mod lexicon_engine;
pub mod local_model_engine;
pub mod lru_cache;
pub mod media_transcription;
pub mod metering;
pub mod model_manager;
pub mod model_pool;
pub mod morphology;
pub mod ocr;
pub mod platform_integrations;
pub mod rbac;
pub mod registry;
pub mod security_audit;
pub mod speech_session_manager;
pub mod streaming_asr;
pub mod traits;
pub mod transliteration;
pub mod tts_engine;
pub mod vad;
pub mod worker_pool;

pub use api_gateway::ApiGateway;
pub use audio_buffer::{compute_rms_db, resample_to_16k, AudioRingBuffer};
pub use audio_capture::{AudioCaptureManager, AudioChunkEvent};
pub use caption_coordinator::CaptionCoordinator;
pub use context_nlp::ContextNlpProcessor;
pub use conversation_orchestrator::{ConversationEvent, ConversationOrchestrator};
pub use document::DocumentProcessor;
pub use evaluation::QualityEvaluator;
pub use http_provider::HttpTranslationProvider;
pub use learning::{
    GrammarKnowledgeBase, ListeningChallengeGenerator, PronunciationEvaluator,
    SpacedRepetitionSystem,
};
pub use lexicon_engine::LexiconTranslationEngine;
pub use local_model_engine::LocalNeuralTranslationEngine;
pub use lru_cache::BoundedLruCache;
pub use media_transcription::MediaTranscriptionPipeline;
pub use metering::UsageQuotaManager;
pub use model_manager::ModelManager;
pub use model_pool::{ModelLifecyclePool, ModelSessionEntry};
pub use morphology::{find_did_you_mean_suggestions, stem_word, StemResult};
pub use ocr::{AuthenticOcrEngine, OcrEngine};
pub use platform_integrations::{LiveOverlayController, PlatformIntegrationsCoordinator};
pub use rbac::RbacController;
pub use registry::{CircuitBreaker, EngineRegistry};
pub use security_audit::{SbomComponent, SbomReport, SecurityAuditor};
pub use speech_session_manager::SpeechSessionCoordinator;
pub use streaming_asr::{AsrStream, SherpaStreamingAsrEngine, StreamingAsrEngine};
pub use traits::{
    SpeechToTextEngine, TextToSpeechEngine, TranslationEngine, TranslationRequest,
    TranslationResponse,
};
pub use transliteration::TransliterationEngine;
pub use tts_engine::AuthenticTtsEngine;
pub use vad::{VadProcessResult, VoiceActivityDetector};
pub use worker_pool::InferenceWorkerPool;
