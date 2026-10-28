use crate::DesktopAppState;
use lingua_core::{
    detect_language, AddFavoriteCmdRequest, AppInfo, CapabilityMatrix, DatabaseHealthInfo,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DictionaryEntry,
    DictionarySearchCmdRequest, DictionarySearchCmdResponse, FavoriteItem, IpcResponse,
    LanguagePair, ModelPackInfo, TranslateCmdRequest, TranslateCmdResponse, TranslationRecord,
    UserSettings,
};
use lingua_engine::TranslationRequest;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn lingua_command_get_app_info(state: State<'_, DesktopAppState>) -> IpcResponse<AppInfo> {
    IpcResponse::ok("tauri-ipc", state.app_info.clone())
}

#[tauri::command]
pub fn lingua_command_get_capabilities(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<CapabilityMatrix> {
    IpcResponse::ok("tauri-ipc", state.engine_registry.get_capabilities())
}

#[tauri::command]
pub fn lingua_command_get_user_settings(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<UserSettings> {
    match state.storage.get_user_settings() {
        Ok(settings) => IpcResponse::ok("tauri-ipc", settings),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_save_user_settings(
    state: State<'_, DesktopAppState>,
    settings: UserSettings,
) -> IpcResponse<UserSettings> {
    match state.storage.save_user_settings(&settings) {
        Ok(_) => IpcResponse::ok("tauri-ipc", settings),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_model_packs(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<ModelPackInfo>> {
    match state.storage.list_model_packs() {
        Ok(packs) => IpcResponse::ok("tauri-ipc", packs),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_check_database_health(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<DatabaseHealthInfo> {
    match state.storage.check_health() {
        Ok(health) => IpcResponse::ok("tauri-ipc", health),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub async fn lingua_command_translate(
    state: State<'_, DesktopAppState>,
    request: TranslateCmdRequest,
) -> Result<IpcResponse<TranslateCmdResponse>, String> {
    let pair = match LanguagePair::new(request.source_language, request.target_language) {
        Ok(p) => p,
        Err(e) => return Ok(IpcResponse::err("tauri-ipc", e)),
    };

    let trans_req = TranslationRequest {
        pair,
        text: request.text,
        allow_cloud: request.allow_cloud,
    };

    match state.engine_registry.translate(trans_req).await {
        Ok(res) => Ok(IpcResponse::ok(
            "tauri-ipc",
            TranslateCmdResponse {
                source_language: res.pair.source,
                target_language: res.pair.target,
                original_text: res.original_text,
                translated_text: res.translated_text,
                engine_id: res.engine_id,
                is_offline: res.is_offline,
                latency_ms: res.latency_ms,
            },
        )),
        Err(e) => Ok(IpcResponse::err("tauri-ipc", e)),
    }
}

#[tauri::command]
pub fn lingua_command_dictionary_search(
    state: State<'_, DesktopAppState>,
    request: DictionarySearchCmdRequest,
) -> IpcResponse<DictionarySearchCmdResponse> {
    let limit = request.limit.unwrap_or(20);
    match state
        .storage
        .search_dictionary(&request.query, request.language, limit)
    {
        Ok(entries) => {
            let total = entries.len();
            IpcResponse::ok(
                "tauri-ipc",
                DictionarySearchCmdResponse {
                    entries,
                    query: request.query,
                    total_matches: total,
                },
            )
        }
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_get_dictionary_word(
    state: State<'_, DesktopAppState>,
    id: String,
) -> IpcResponse<Option<DictionaryEntry>> {
    match state.storage.get_dictionary_entry(&id) {
        Ok(entry) => IpcResponse::ok("tauri-ipc", entry),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_detect_language(
    request: DetectLanguageCmdRequest,
) -> IpcResponse<DetectLanguageCmdResponse> {
    let detected = detect_language(&request.text);
    IpcResponse::ok("tauri-ipc", DetectLanguageCmdResponse { detected })
}

#[tauri::command]
pub fn lingua_command_list_translation_history(
    state: State<'_, DesktopAppState>,
    limit: usize,
) -> IpcResponse<Vec<TranslationRecord>> {
    match state.storage.list_translation_history(limit) {
        Ok(records) => IpcResponse::ok("tauri-ipc", records),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_clear_translation_history(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<()> {
    match state.storage.clear_translation_history() {
        Ok(_) => IpcResponse::ok("tauri-ipc", ()),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_list_favorites(
    state: State<'_, DesktopAppState>,
) -> IpcResponse<Vec<FavoriteItem>> {
    match state.storage.list_favorites() {
        Ok(favs) => IpcResponse::ok("tauri-ipc", favs),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_add_favorite(
    state: State<'_, DesktopAppState>,
    request: AddFavoriteCmdRequest,
) -> IpcResponse<FavoriteItem> {
    let item = FavoriteItem {
        id: Uuid::new_v4().to_string(),
        source_language: request.source_language,
        target_language: request.target_language,
        source_text: request.source_text,
        translated_text: request.translated_text,
        notes: request.notes,
        created_at: chrono::Utc::now(),
    };

    match state.storage.add_favorite(&item) {
        Ok(_) => IpcResponse::ok("tauri-ipc", item),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}

#[tauri::command]
pub fn lingua_command_remove_favorite(
    state: State<'_, DesktopAppState>,
    id: String,
) -> IpcResponse<()> {
    match state.storage.remove_favorite(&id) {
        Ok(_) => IpcResponse::ok("tauri-ipc", ()),
        Err(e) => IpcResponse::err("tauri-ipc", e),
    }
}
