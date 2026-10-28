use crate::state::ServerState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::Response,
    routing::{delete, get, post},
    Json, Router,
};
use lingua_core::{
    detect_language, AddFavoriteCmdRequest, AppInfo, CapabilityMatrix, CaptionFormat,
    CaptionSegment, DatabaseHealthInfo, DetectLanguageCmdRequest, DetectLanguageCmdResponse,
    DictionaryEntry, DictionarySearchCmdResponse, DiskSpaceInfo, ExportCaptionsCmdResponse,
    FavoriteItem, FeedAudioChunkCmdRequest, GetAudioDevicesCmdResponse,
    GetConversationTurnsCmdResponse, GetSessionCaptionsCmdResponse, GetTtsVoicesCmdResponse,
    InterruptConversationCmdRequest, InterruptConversationCmdResponse, LanguageCode, LanguagePair,
    ModelPackDetailedInfo, ModelPackInfo, PauseSpeechSessionCmdRequest,
    ResumeSpeechSessionCmdRequest, SpeechSession, StartConversationCmdRequest,
    StartConversationCmdResponse, StartSpeechSessionCmdRequest, StartSpeechSessionCmdResponse,
    StopConversationCmdRequest, StopConversationCmdResponse, StopSpeechSessionCmdRequest,
    StopSpeechSessionCmdResponse, SubmitTurnCmdRequest, SubmitTurnCmdResponse,
    SwitchSpeakerCmdRequest, SwitchSpeakerCmdResponse, SynthesizeSpeechCmdRequest,
    SynthesizeSpeechCmdResponse, TranslateCmdRequest, TranslateCmdResponse, TranslationRecord,
    TtsSynthesizeRequest, UserSettings, VerifyModelCmdResponse,
};
use lingua_engine::{AudioCaptureManager, TranslationRequest};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;

pub fn create_router(state: ServerState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/app-info", get(app_info_handler))
        .route("/api/v1/capabilities", get(capabilities_handler))
        .route(
            "/api/v1/settings",
            get(get_settings_handler).post(save_settings_handler),
        )
        .route("/api/v1/model-packs", get(model_packs_handler))
        .route("/api/v1/models", get(list_models_handler))
        .route(
            "/api/v1/models/{id}",
            get(get_model_handler).delete(delete_model_handler),
        )
        .route("/api/v1/models/download/{id}", post(download_model_handler))
        .route("/api/v1/models/verify/{id}", post(verify_model_handler))
        .route("/api/v1/models/disk-space", get(get_disk_space_handler))
        .route("/api/v1/translate", post(translate_handler))
        .route("/api/v1/dictionary/search", get(dictionary_search_handler))
        .route("/api/v1/dictionary/word/{id}", get(dictionary_word_handler))
        .route("/api/v1/detect-language", post(detect_language_handler))
        .route(
            "/api/v1/history",
            get(list_history_handler).delete(clear_history_handler),
        )
        .route(
            "/api/v1/favorites",
            get(list_favorites_handler).post(add_favorite_handler),
        )
        .route("/api/v1/favorites/{id}", delete(remove_favorite_handler))
        .route("/api/v1/speech/devices", get(get_speech_devices_handler))
        .route(
            "/api/v1/speech/session/start",
            post(start_speech_session_handler),
        )
        .route(
            "/api/v1/speech/session/pause",
            post(pause_speech_session_handler),
        )
        .route(
            "/api/v1/speech/session/resume",
            post(resume_speech_session_handler),
        )
        .route(
            "/api/v1/speech/session/stop",
            post(stop_speech_session_handler),
        )
        .route(
            "/api/v1/speech/session/{id}/captions",
            get(get_speech_captions_handler),
        )
        .route(
            "/api/v1/speech/session/{id}/export",
            get(export_speech_captions_handler),
        )
        .route("/api/v1/speech/feed-chunk", post(feed_speech_chunk_handler))
        .route("/api/v1/speech/ws", get(speech_ws_handler))
        .route(
            "/api/v1/conversation/start",
            post(start_conversation_handler),
        )
        .route(
            "/api/v1/conversation/turn",
            post(submit_conversation_turn_handler),
        )
        .route(
            "/api/v1/conversation/switch-speaker",
            post(switch_speaker_handler),
        )
        .route(
            "/api/v1/conversation/interrupt",
            post(interrupt_conversation_handler),
        )
        .route("/api/v1/conversation/stop", post(stop_conversation_handler))
        .route(
            "/api/v1/conversation/{id}/turns",
            get(get_conversation_turns_handler),
        )
        .route("/api/v1/tts/voices", get(list_tts_voices_handler))
        .route("/api/v1/tts/synthesize", post(synthesize_tts_handler))
        .route("/api/v1/conversation/ws", get(conversation_ws_handler))
        .route("/api/v1/ws", get(ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_handler(
    State(state): State<ServerState>,
) -> Result<Json<DatabaseHealthInfo>, (StatusCode, String)> {
    match state.storage.check_health() {
        Ok(info) => Ok(Json(info)),
        Err(e) => Err((StatusCode::SERVICE_UNAVAILABLE, e.to_string())),
    }
}

async fn app_info_handler(State(state): State<ServerState>) -> Json<AppInfo> {
    Json(state.app_info.clone())
}

async fn capabilities_handler(State(state): State<ServerState>) -> Json<CapabilityMatrix> {
    Json(state.engine_registry.get_capabilities())
}

async fn get_settings_handler(
    State(state): State<ServerState>,
) -> Result<Json<UserSettings>, (StatusCode, String)> {
    match state.storage.get_user_settings() {
        Ok(settings) => Ok(Json(settings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn save_settings_handler(
    State(state): State<ServerState>,
    Json(settings): Json<UserSettings>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.storage.save_user_settings(&settings) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn model_packs_handler(
    State(state): State<ServerState>,
) -> Result<Json<Vec<ModelPackInfo>>, (StatusCode, String)> {
    match state.storage.list_model_packs() {
        Ok(packs) => Ok(Json(packs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn list_models_handler(
    State(state): State<ServerState>,
) -> Result<Json<Vec<ModelPackDetailedInfo>>, StatusCode> {
    state
        .storage
        .list_detailed_model_packs()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_model_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<Option<ModelPackDetailedInfo>>, StatusCode> {
    state
        .storage
        .get_detailed_model_pack(&id)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn download_model_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<String>, (StatusCode, String)> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.download_and_install_simulated::<fn(u64, u64, f32)>(&id, None, None) {
            Ok(path) => Ok(Json(path.to_string_lossy().to_string())),
            Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Model manager not initialized".into(),
        ))
    }
}

async fn verify_model_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<VerifyModelCmdResponse>, (StatusCode, String)> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.verify_model_pack(&id) {
            Ok(is_valid) => Ok(Json(VerifyModelCmdResponse {
                model_id: id,
                is_valid,
                details: if is_valid {
                    "Checksum verified".into()
                } else {
                    "Verification failed".into()
                },
            })),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Model manager not initialized".into(),
        ))
    }
}

async fn delete_model_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.evict_model(&id) {
            Ok(_) => Ok(StatusCode::NO_CONTENT),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Model manager not initialized".into(),
        ))
    }
}

async fn get_disk_space_handler(
    State(state): State<ServerState>,
) -> Result<Json<DiskSpaceInfo>, (StatusCode, String)> {
    if let Some(mm) = state.engine_registry.model_manager() {
        match mm.get_disk_space() {
            Ok(space) => Ok(Json(space)),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Model manager not initialized".into(),
        ))
    }
}

async fn translate_handler(
    State(state): State<ServerState>,
    Json(req): Json<TranslateCmdRequest>,
) -> Result<Json<TranslateCmdResponse>, (StatusCode, String)> {
    let pair = LanguagePair::new(req.source_language, req.target_language)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let translation_req = TranslationRequest {
        pair,
        text: req.text,
        allow_cloud: req.allow_cloud,
    };

    match state.engine_registry.translate(translation_req).await {
        Ok(res) => Ok(Json(TranslateCmdResponse {
            source_language: res.pair.source,
            target_language: res.pair.target,
            original_text: res.original_text,
            translated_text: res.translated_text,
            engine_id: res.engine_id,
            is_offline: res.is_offline,
            latency_ms: res.latency_ms,
        })),
        Err(e) => {
            let status = match e {
                lingua_core::LinguaError::Validation(_)
                | lingua_core::LinguaError::IdenticalLanguagePair(_, _) => StatusCode::BAD_REQUEST,
                lingua_core::LinguaError::NetworkUnavailable => StatusCode::SERVICE_UNAVAILABLE,
                lingua_core::LinguaError::CapabilityUnavailable(_, _) => {
                    StatusCode::NOT_IMPLEMENTED
                }
                lingua_core::LinguaError::OfflineModelMissing(_) => {
                    StatusCode::UNPROCESSABLE_ENTITY
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, e.to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
struct DictSearchQuery {
    q: Option<String>,
    lang: Option<String>,
    limit: Option<usize>,
}

async fn dictionary_search_handler(
    State(state): State<ServerState>,
    Query(query): Query<DictSearchQuery>,
) -> Result<Json<DictionarySearchCmdResponse>, (StatusCode, String)> {
    let query_str = query.q.unwrap_or_default();
    let lang = query.lang.and_then(|l| l.parse::<LanguageCode>().ok());
    let limit = query.limit.unwrap_or(20);

    match state.storage.search_dictionary(&query_str, lang, limit) {
        Ok(entries) => {
            let total = entries.len();
            Ok(Json(DictionarySearchCmdResponse {
                entries,
                query: query_str,
                total_matches: total,
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn dictionary_word_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<Option<DictionaryEntry>>, (StatusCode, String)> {
    match state.storage.get_dictionary_entry(&id) {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn detect_language_handler(
    Json(req): Json<DetectLanguageCmdRequest>,
) -> Json<DetectLanguageCmdResponse> {
    let detected = detect_language(&req.text);
    Json(DetectLanguageCmdResponse { detected })
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    limit: Option<usize>,
}

async fn list_history_handler(
    State(state): State<ServerState>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<TranslationRecord>>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(50);
    match state.storage.list_translation_history(limit) {
        Ok(history) => Ok(Json(history)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn clear_history_handler(
    State(state): State<ServerState>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.storage.clear_translation_history() {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn list_favorites_handler(
    State(state): State<ServerState>,
) -> Result<Json<Vec<FavoriteItem>>, (StatusCode, String)> {
    match state.storage.list_favorites() {
        Ok(favs) => Ok(Json(favs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn add_favorite_handler(
    State(state): State<ServerState>,
    Json(req): Json<AddFavoriteCmdRequest>,
) -> Result<Json<FavoriteItem>, (StatusCode, String)> {
    let item = FavoriteItem {
        id: Uuid::new_v4().to_string(),
        source_language: req.source_language,
        target_language: req.target_language,
        source_text: req.source_text,
        translated_text: req.translated_text,
        notes: req.notes,
        created_at: chrono::Utc::now(),
    };

    match state.storage.add_favorite(&item) {
        Ok(_) => Ok(Json(item)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn remove_favorite_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.storage.remove_favorite(&id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: ServerState) {
    info!("WebSocket client connected to LinguaFlow edge gateway");

    // Send initial capability and health handshake
    let health = state.storage.check_health().unwrap_or(DatabaseHealthInfo {
        status: "degraded".into(),
        path: "".into(),
        schema_version: 0,
        settings_persisted: false,
        error: Some("Health query failed".into()),
    });

    if let Ok(json) = serde_json::to_string(&health) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                error!("WebSocket receive error: {e}");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                if text == "ping" {
                    let _ = socket.send(Message::Text("pong".into())).await;
                } else if text == "get_capabilities" {
                    let caps = state.engine_registry.get_capabilities();
                    if let Ok(json) = serde_json::to_string(&caps) {
                        let _ = socket.send(Message::Text(json.into())).await;
                    }
                }
            }
            Message::Ping(payload) => {
                let _ = socket.send(Message::Pong(payload)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    info!("WebSocket client disconnected");
}

async fn get_speech_devices_handler(
) -> Result<Json<GetAudioDevicesCmdResponse>, (StatusCode, String)> {
    match AudioCaptureManager::list_input_devices() {
        Ok(devices) => Ok(Json(GetAudioDevicesCmdResponse { devices })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn start_speech_session_handler(
    State(state): State<ServerState>,
    Json(payload): Json<StartSpeechSessionCmdRequest>,
) -> Result<Json<StartSpeechSessionCmdResponse>, (StatusCode, String)> {
    match state
        .speech_coordinator
        .start_session(
            payload.source_language,
            payload.target_language,
            payload.title,
            payload.device_id,
        )
        .await
    {
        Ok(session) => Ok(Json(StartSpeechSessionCmdResponse { session })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn pause_speech_session_handler(
    State(state): State<ServerState>,
    Json(payload): Json<PauseSpeechSessionCmdRequest>,
) -> Result<Json<SpeechSession>, (StatusCode, String)> {
    match state
        .speech_coordinator
        .pause_session(&payload.session_id)
        .await
    {
        Ok(session) => Ok(Json(session)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

async fn resume_speech_session_handler(
    State(state): State<ServerState>,
    Json(payload): Json<ResumeSpeechSessionCmdRequest>,
) -> Result<Json<SpeechSession>, (StatusCode, String)> {
    match state
        .speech_coordinator
        .resume_session(&payload.session_id)
        .await
    {
        Ok(session) => Ok(Json(session)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

async fn stop_speech_session_handler(
    State(state): State<ServerState>,
    Json(payload): Json<StopSpeechSessionCmdRequest>,
) -> Result<Json<StopSpeechSessionCmdResponse>, (StatusCode, String)> {
    match state
        .speech_coordinator
        .stop_session(&payload.session_id)
        .await
    {
        Ok(session) => Ok(Json(StopSpeechSessionCmdResponse { session })),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

async fn get_speech_captions_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<GetSessionCaptionsCmdResponse>, (StatusCode, String)> {
    match state.speech_coordinator.get_session_captions(&id).await {
        Ok(captions) => Ok(Json(GetSessionCaptionsCmdResponse { captions })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn export_speech_captions_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<Json<ExportCaptionsCmdResponse>, (StatusCode, String)> {
    let format = match query.format.as_deref() {
        Some("srt") => CaptionFormat::Srt,
        Some("json") => CaptionFormat::Json,
        _ => CaptionFormat::Vtt,
    };
    match state
        .speech_coordinator
        .export_session_captions(&id, format)
        .await
    {
        Ok((content, filename)) => Ok(Json(ExportCaptionsCmdResponse {
            content,
            filename,
            format,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn feed_speech_chunk_handler(
    State(state): State<ServerState>,
    Json(payload): Json<FeedAudioChunkCmdRequest>,
) -> Result<Json<Vec<CaptionSegment>>, (StatusCode, String)> {
    match state
        .speech_coordinator
        .feed_audio_chunk(&payload.session_id, &payload.samples)
        .await
    {
        Ok(segments) => Ok(Json(segments)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn speech_ws_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> Response {
    ws.on_upgrade(|socket| handle_speech_socket(socket, state))
}

async fn handle_speech_socket(mut socket: WebSocket, state: ServerState) {
    info!("WebSocket client connected to LinguaFlow speech captions stream");
    let mut caption_rx = state.speech_coordinator.subscribe_captions();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => {
                        error!("Speech WebSocket receive error: {e}");
                        break;
                    }
                    None => break,
                };

                match msg {
                    Message::Text(text) => {
                        if text == "ping" {
                            let _ = socket.send(Message::Text("pong".into())).await;
                        } else if let Ok(feed_req) = serde_json::from_str::<FeedAudioChunkCmdRequest>(&text) {
                            let _ = state.speech_coordinator.feed_audio_chunk(&feed_req.session_id, &feed_req.samples).await;
                        }
                    }
                    Message::Ping(payload) => {
                        let _ = socket.send(Message::Pong(payload)).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            Ok(caption) = caption_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&caption) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }
        }
    }
    info!("Speech WebSocket client disconnected");
}

async fn start_conversation_handler(
    State(state): State<ServerState>,
    Json(payload): Json<StartConversationCmdRequest>,
) -> Result<Json<StartConversationCmdResponse>, (StatusCode, String)> {
    match state
        .conversation_orchestrator
        .start_session(
            payload.speaker_a_lang,
            payload.speaker_b_lang,
            payload.title,
            payload.is_hands_free,
        )
        .await
    {
        Ok(session) => Ok(Json(StartConversationCmdResponse { session })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn submit_conversation_turn_handler(
    State(state): State<ServerState>,
    Json(payload): Json<SubmitTurnCmdRequest>,
) -> Result<Json<SubmitTurnCmdResponse>, (StatusCode, String)> {
    match state
        .conversation_orchestrator
        .submit_turn(
            &payload.session_id,
            payload.speaker_id,
            &payload.source_text,
        )
        .await
    {
        Ok((turn, audio_samples, sample_rate)) => Ok(Json(SubmitTurnCmdResponse {
            turn,
            audio_samples,
            sample_rate,
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn switch_speaker_handler(
    State(state): State<ServerState>,
    Json(payload): Json<SwitchSpeakerCmdRequest>,
) -> Result<Json<SwitchSpeakerCmdResponse>, (StatusCode, String)> {
    match state
        .conversation_orchestrator
        .switch_speaker(&payload.session_id, payload.next_speaker)
        .await
    {
        Ok(session) => Ok(Json(SwitchSpeakerCmdResponse { session })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn interrupt_conversation_handler(
    State(state): State<ServerState>,
    Json(payload): Json<InterruptConversationCmdRequest>,
) -> Result<Json<InterruptConversationCmdResponse>, (StatusCode, String)> {
    match state
        .conversation_orchestrator
        .interrupt(&payload.session_id)
        .await
    {
        Ok(success) => Ok(Json(InterruptConversationCmdResponse {
            session_id: payload.session_id,
            success,
        })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn stop_conversation_handler(
    State(state): State<ServerState>,
    Json(payload): Json<StopConversationCmdRequest>,
) -> Result<Json<StopConversationCmdResponse>, (StatusCode, String)> {
    match state
        .conversation_orchestrator
        .stop_session(&payload.session_id)
        .await
    {
        Ok(session) => Ok(Json(StopConversationCmdResponse { session })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn get_conversation_turns_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<GetConversationTurnsCmdResponse>, (StatusCode, String)> {
    match state.conversation_orchestrator.get_turns(&id).await {
        Ok(turns) => Ok(Json(GetConversationTurnsCmdResponse { turns })),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

async fn list_tts_voices_handler(
    State(state): State<ServerState>,
) -> Json<GetTtsVoicesCmdResponse> {
    let voices = state.conversation_orchestrator.tts_engine().list_voices();
    Json(GetTtsVoicesCmdResponse { voices })
}

async fn synthesize_tts_handler(
    State(state): State<ServerState>,
    Json(payload): Json<SynthesizeSpeechCmdRequest>,
) -> Result<Json<SynthesizeSpeechCmdResponse>, (StatusCode, String)> {
    let req = TtsSynthesizeRequest {
        text: payload.text,
        language: payload.language,
        voice_id: payload.voice_id,
        speed: payload.speed,
        pitch: payload.pitch,
    };

    match state
        .conversation_orchestrator
        .tts_engine()
        .synthesize(req)
        .await
    {
        Ok(response) => Ok(Json(SynthesizeSpeechCmdResponse { response })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn conversation_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_conversation_socket(socket, state))
}

async fn handle_conversation_socket(mut socket: WebSocket, state: ServerState) {
    let mut rx = state.conversation_orchestrator.subscribe();
    while let Ok(event) = rx.recv().await {
        let json_str = match &event {
            lingua_engine::ConversationEvent::TurnStateChanged { session_id, state } => {
                serde_json::json!({
                    "type": "turn_state",
                    "session_id": session_id,
                    "state": state
                })
            }
            lingua_engine::ConversationEvent::TurnCompleted { session_id, turn } => {
                serde_json::json!({
                    "type": "turn_completed",
                    "session_id": session_id,
                    "turn": turn
                })
            }
            lingua_engine::ConversationEvent::TurnInterrupted {
                session_id,
                turn_id,
            } => {
                serde_json::json!({
                    "type": "turn_interrupted",
                    "session_id": session_id,
                    "turn_id": turn_id
                })
            }
            lingua_engine::ConversationEvent::SessionEnded { session_id } => {
                serde_json::json!({
                    "type": "session_ended",
                    "session_id": session_id
                })
            }
        };

        if socket
            .send(Message::Text(json_str.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }
}
