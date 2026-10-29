pub mod commands;

use lingua_core::AppInfo;
use lingua_engine::{
    ConversationOrchestrator, EngineRegistry, InferenceWorkerPool, SpeechSessionCoordinator,
};
use lingua_storage::StorageService;
use std::sync::Arc;
use tracing::info;

pub struct DesktopAppState {
    pub storage: StorageService,
    pub engine_registry: Arc<EngineRegistry>,
    pub speech_coordinator: Arc<SpeechSessionCoordinator>,
    pub conversation_orchestrator: Arc<ConversationOrchestrator>,
    pub worker_pool: Arc<InferenceWorkerPool>,
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
    let conversation_orchestrator = Arc::new(ConversationOrchestrator::new(
        Arc::clone(&engine_registry),
        engine_registry.tts_engine(),
        Some(storage.clone()),
    ));
    let worker_pool = Arc::new(InferenceWorkerPool::new(16, 50));
    let app_info = AppInfo::default();

    let app_state = DesktopAppState {
        storage,
        engine_registry,
        speech_coordinator,
        conversation_orchestrator,
        worker_pool,
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
            commands::lingua_command_get_audio_output_devices,
            commands::lingua_command_start_speech_session,
            commands::lingua_command_pause_speech_session,
            commands::lingua_command_resume_speech_session,
            commands::lingua_command_stop_speech_session,
            commands::lingua_command_get_session_captions,
            commands::lingua_command_export_captions,
            commands::lingua_command_feed_audio_chunk,
            commands::lingua_command_start_conversation,
            commands::lingua_command_submit_turn,
            commands::lingua_command_switch_speaker,
            commands::lingua_command_interrupt_conversation,
            commands::lingua_command_stop_conversation,
            commands::lingua_command_get_conversation_turns,
            commands::lingua_command_list_tts_voices,
            commands::lingua_command_synthesize_speech,
            // Stage 5 commands
            commands::lingua_command_translate_intelligence,
            commands::lingua_command_transliterate,
            commands::lingua_command_get_circuit_breakers,
            commands::lingua_command_reset_circuit_breaker,
            commands::lingua_command_list_glossary_terms,
            commands::lingua_command_create_glossary_term,
            commands::lingua_command_delete_glossary_term,
            commands::lingua_command_run_evaluation,
            commands::lingua_command_get_latest_evaluation_report,
            commands::lingua_command_set_resource_profile,
            // Stage 6 commands
            commands::lingua_command_start_media_transcription,
            commands::lingua_command_list_media_segments,
            commands::lingua_command_update_media_segment,
            commands::lingua_command_export_subtitles,
            commands::lingua_command_perform_ocr,
            commands::lingua_command_translate_document,
            commands::lingua_command_get_overlay_settings,
            commands::lingua_command_save_overlay_settings,
            commands::lingua_command_get_overlay_permission,
            commands::lingua_command_get_global_shortcuts,
            commands::lingua_command_save_global_shortcuts,
            // Stage 7 commands
            commands::lingua_command_list_learning_decks,
            commands::lingua_command_get_deck_cards,
            commands::lingua_command_review_card,
            commands::lingua_command_get_learner_progress,
            commands::lingua_command_assess_pronunciation,
            commands::lingua_command_get_listening_challenges,
            commands::lingua_command_get_grammar_topics,
            commands::lingua_command_get_privacy_settings,
            commands::lingua_command_save_privacy_settings,
            commands::lingua_command_export_user_data,
            commands::lingua_command_purge_user_data,
            commands::lingua_command_get_platform_release_status,
            // Stage 8 commands
            commands::lingua_command_get_enterprise_account,
            commands::lingua_command_update_account_tier,
            commands::lingua_command_register_device,
            commands::lingua_command_list_devices,
            commands::lingua_command_create_api_key,
            commands::lingua_command_list_api_keys,
            commands::lingua_command_revoke_api_key,
            commands::lingua_command_get_usage_summary,
            commands::lingua_command_list_invoices,
            commands::lingua_command_create_organization,
            commands::lingua_command_list_org_members,
            commands::lingua_command_add_org_member,
            commands::lingua_command_list_org_glossaries,
            commands::lingua_command_create_org_glossary,
            commands::lingua_command_add_org_glossary_term,
            commands::lingua_command_list_audit_events,
            commands::lingua_command_get_worker_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LinguaFlow application");
}
