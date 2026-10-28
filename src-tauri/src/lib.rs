pub mod commands;

use lingua_core::AppInfo;
use lingua_engine::{EngineRegistry, SpeechSessionCoordinator};
use lingua_storage::StorageService;
use std::sync::Arc;
use tracing::info;

pub struct DesktopAppState {
    pub storage: StorageService,
    pub engine_registry: Arc<EngineRegistry>,
    pub speech_coordinator: Arc<SpeechSessionCoordinator>,
    pub app_info: AppInfo,
}

pub fn run() {
    let storage = StorageService::open_default().expect("Failed to initialize SQLite storage");
    let engine_registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let audio_capture = engine_registry.audio_capture();
    let speech_coordinator = Arc::new(SpeechSessionCoordinator::new(
        Arc::clone(&engine_registry),
        Some(storage.clone()),
        audio_capture,
    ));
    let app_info = AppInfo::default();

    let app_state = DesktopAppState {
        storage,
        engine_registry,
        speech_coordinator,
        app_info,
    };

    info!("Starting LinguaFlow Desktop / Mobile Host Application");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::lingua_command_get_app_info,
            commands::lingua_command_get_capabilities,
            commands::lingua_command_get_user_settings,
            commands::lingua_command_save_user_settings,
            commands::lingua_command_list_model_packs,
            commands::lingua_command_check_database_health,
            commands::lingua_command_translate,
            commands::lingua_command_dictionary_search,
            commands::lingua_command_get_dictionary_word,
            commands::lingua_command_detect_language,
            commands::lingua_command_list_translation_history,
            commands::lingua_command_clear_translation_history,
            commands::lingua_command_list_favorites,
            commands::lingua_command_add_favorite,
            commands::lingua_command_remove_favorite,
            commands::lingua_command_list_detailed_model_packs,
            commands::lingua_command_get_detailed_model_pack,
            commands::lingua_command_download_model_pack,
            commands::lingua_command_verify_model_pack,
            commands::lingua_command_delete_model_pack,
            commands::lingua_command_get_disk_space,
            commands::lingua_command_get_audio_devices,
            commands::lingua_command_start_speech_session,
            commands::lingua_command_pause_speech_session,
            commands::lingua_command_resume_speech_session,
            commands::lingua_command_stop_speech_session,
            commands::lingua_command_get_session_captions,
            commands::lingua_command_export_captions,
            commands::lingua_command_feed_audio_chunk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LinguaFlow application");
}
