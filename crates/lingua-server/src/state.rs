use lingua_core::AppInfo;
use lingua_engine::{
    ApiGateway, ConversationOrchestrator, EngineRegistry, InferenceWorkerPool,
    SpeechSessionCoordinator,
};
use lingua_storage::StorageService;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub storage: StorageService,
    pub engine_registry: Arc<EngineRegistry>,
    pub speech_coordinator: Arc<SpeechSessionCoordinator>,
    pub conversation_orchestrator: Arc<ConversationOrchestrator>,
    pub worker_pool: Arc<InferenceWorkerPool>,
    pub api_gateway: Arc<ApiGateway>,
    pub app_info: AppInfo,
}

impl ServerState {
    pub fn new(
        storage: StorageService,
        engine_registry: Arc<EngineRegistry>,
        app_info: AppInfo,
    ) -> Self {
        let audio_capture = engine_registry.audio_capture();
        let speech_coordinator = Arc::new(SpeechSessionCoordinator::new(
            Arc::clone(&engine_registry),
            Some(storage.clone()),
            audio_capture,
        ));
        let conversation_orchestrator = Arc::new(ConversationOrchestrator::new(
            Arc::clone(&engine_registry),
            engine_registry.tts_engine(),
            Some(storage.clone()),
        ));
        let worker_pool = Arc::new(InferenceWorkerPool::new(16, 500));
        let api_gateway = Arc::new(ApiGateway::new());
        Self {
            storage,
            engine_registry,
            speech_coordinator,
            conversation_orchestrator,
            worker_pool,
            api_gateway,
            app_info,
        }
    }

    pub fn new_with_coordinator(
        storage: StorageService,
        engine_registry: Arc<EngineRegistry>,
        speech_coordinator: Arc<SpeechSessionCoordinator>,
        app_info: AppInfo,
    ) -> Self {
        let conversation_orchestrator = Arc::new(ConversationOrchestrator::new(
            Arc::clone(&engine_registry),
            engine_registry.tts_engine(),
            Some(storage.clone()),
        ));
        let worker_pool = Arc::new(InferenceWorkerPool::new(16, 500));
        let api_gateway = Arc::new(ApiGateway::new());
        Self {
            storage,
            engine_registry,
            speech_coordinator,
            conversation_orchestrator,
            worker_pool,
            api_gateway,
            app_info,
        }
    }
}
