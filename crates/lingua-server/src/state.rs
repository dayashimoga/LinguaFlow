use lingua_core::AppInfo;
use lingua_engine::EngineRegistry;
use lingua_storage::StorageService;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub storage: StorageService,
    pub engine_registry: Arc<EngineRegistry>,
    pub app_info: AppInfo,
}

impl ServerState {
    pub fn new(
        storage: StorageService,
        engine_registry: Arc<EngineRegistry>,
        app_info: AppInfo,
    ) -> Self {
        Self {
            storage,
            engine_registry,
            app_info,
        }
    }
}
