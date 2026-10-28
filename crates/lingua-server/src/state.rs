use lingua_core::AppInfo;
use lingua_engine::{EngineRegistry, SpeechSessionCoordinator};
use lingua_storage::StorageService;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub storage: StorageService,
    pub engine_registry: Arc<EngineRegistry>,
    pub speech_coordinator: Arc<SpeechSessionCoordinator>,
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
        Self {
            storage,
            engine_registry,
            speech_coordinator,
            app_info,
        }
    }

    pub fn new_with_coordinator(
        storage: StorageService,
        engine_registry: Arc<EngineRegistry>,
        speech_coordinator: Arc<SpeechSessionCoordinator>,
        app_info: AppInfo,
    ) -> Self {
        Self {
            storage,
            engine_registry,
            speech_coordinator,
            app_info,
        }
    }
}
