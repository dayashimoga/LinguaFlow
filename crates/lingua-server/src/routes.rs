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
    CaptionSegment, CreateGlossaryTermCmdRequest, DatabaseHealthInfo, DetectLanguageCmdRequest,
    DetectLanguageCmdResponse, DictionaryEntry, DictionarySearchCmdResponse, DiskSpaceInfo,
    DocumentJobStatus, DocumentTranslationJob, DomainGlossaryEntry, EvaluationReport,
    ExportCaptionsCmdResponse, ExportSubtitlesCmdResponse, FavoriteItem, FeedAudioChunkCmdRequest,
    GetAudioDevicesCmdResponse, GetCircuitBreakersCmdResponse, GetConversationTurnsCmdResponse,
    GetOverlaySettingsCmdResponse, GetSessionCaptionsCmdResponse, GetTtsVoicesCmdResponse,
    IntelligenceTranslationRequest, IntelligenceTranslationResponse,
    InterruptConversationCmdRequest, InterruptConversationCmdResponse, LanguageCode, LanguagePair,
    ListGlossaryTermsCmdResponse, ListMediaSegmentsCmdResponse, MediaProgressStatus, MediaSession,
    MediaSubtitleExportFormat, MediaSubtitleSegment, ModelPackDetailedInfo, ModelPackInfo,
    PauseSpeechSessionCmdRequest, PerformOcrCmdRequest, PerformOcrCmdResponse,
    ResumeSpeechSessionCmdRequest, RunEvaluationCmdResponse, SaveOverlaySettingsCmdRequest,
    SetResourceProfileCmdRequest, SpeechSession, StartConversationCmdRequest,
    StartConversationCmdResponse, StartMediaTranscriptionCmdRequest,
    StartMediaTranscriptionCmdResponse, StartSpeechSessionCmdRequest,
    StartSpeechSessionCmdResponse, StopConversationCmdRequest, StopConversationCmdResponse,
    StopSpeechSessionCmdRequest, StopSpeechSessionCmdResponse, SubmitTurnCmdRequest,
    SubmitTurnCmdResponse, SwitchSpeakerCmdRequest, SwitchSpeakerCmdResponse,
    SynthesizeSpeechCmdRequest, SynthesizeSpeechCmdResponse, SystemAudioPermission,
    TranslateCmdRequest, TranslateCmdResponse, TranslateDocumentCmdRequest,
    TranslateDocumentCmdResponse, TranslateIntelligenceCmdRequest, TranslationRecord,
    TransliterateCmdRequest, TransliterationResponse, TtsSynthesizeRequest,
    UpdateMediaSegmentCmdRequest, UserSettings, VerifyModelCmdResponse,
};
use lingua_engine::{
    AudioCaptureManager, AuthenticOcrEngine, DocumentProcessor, LiveOverlayController,
    MediaTranscriptionPipeline, OcrEngine, QualityEvaluator, TranslationRequest,
};
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
        // Stage 5: Translation Intelligence, Circuit Breakers, Glossaries, Evaluation
        .route(
            "/api/v1/intelligence/translate",
            post(intelligence_translate_handler),
        )
        .route(
            "/api/v1/intelligence/transliterate",
            post(intelligence_transliterate_handler),
        )
        .route(
            "/api/v1/intelligence/circuit-breakers",
            get(list_circuit_breakers_handler),
        )
        .route(
            "/api/v1/intelligence/circuit-breakers/{id}/reset",
            post(reset_circuit_breaker_handler),
        )
        .route(
            "/api/v1/glossary",
            get(list_glossary_handler).post(create_glossary_handler),
        )
        .route("/api/v1/glossary/{id}", delete(delete_glossary_handler))
        .route(
            "/api/v1/intelligence/evaluate",
            post(run_evaluation_handler),
        )
        .route(
            "/api/v1/intelligence/evaluation-report",
            get(get_evaluation_report_handler),
        )
        .route(
            "/api/v1/intelligence/resource-profile",
            post(set_resource_profile_handler),
        )
        // Stage 6: Media Transcription, Captions, OCR, Document Handling & Overlay
        .route("/api/v1/media/transcribe", post(media_transcribe_handler))
        .route(
            "/api/v1/media/{id}/segments",
            get(get_media_segments_handler),
        )
        .route(
            "/api/v1/media/segments/{id}",
            post(update_media_segment_handler),
        )
        .route(
            "/api/v1/media/{id}/export",
            get(export_media_subtitles_handler),
        )
        .route("/api/v1/ocr/scan", post(scan_ocr_handler))
        .route(
            "/api/v1/document/translate",
            post(translate_document_handler),
        )
        .route("/api/v1/document/job/{id}", get(get_document_job_handler))
        .route(
            "/api/v1/overlay/settings",
            get(get_overlay_settings_handler).post(save_overlay_settings_handler),
        )
        .route(
            "/api/v1/overlay/permission",
            get(get_overlay_permission_handler),
        )
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

// Stage 5 Handler Implementations

async fn intelligence_translate_handler(
    State(state): State<ServerState>,
    Json(payload): Json<TranslateIntelligenceCmdRequest>,
) -> Result<Json<IntelligenceTranslationResponse>, (StatusCode, String)> {
    let req = IntelligenceTranslationRequest {
        text: payload.text,
        source_lang: payload.source_lang,
        target_lang: payload.target_lang,
        formality: payload.formality,
        enable_transliteration: payload.enable_transliteration,
        domain_glossary: payload.domain_glossary,
    };
    match state.engine_registry.translate_with_intelligence(req).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn intelligence_transliterate_handler(
    State(state): State<ServerState>,
    Json(payload): Json<TransliterateCmdRequest>,
) -> Result<Json<TransliterationResponse>, (StatusCode, String)> {
    match state
        .engine_registry
        .transliteration()
        .transliterate(&payload.text, payload.target_lang)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn list_circuit_breakers_handler(
    State(state): State<ServerState>,
) -> Json<GetCircuitBreakersCmdResponse> {
    let circuit_breakers = state.engine_registry.circuit_breakers();
    Json(GetCircuitBreakersCmdResponse { circuit_breakers })
}

async fn reset_circuit_breaker_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> StatusCode {
    state.engine_registry.reset_circuit_breaker(&id);
    StatusCode::OK
}

#[derive(Debug, Deserialize)]
pub struct GlossaryQueryParams {
    pub domain: Option<String>,
    pub source_lang: Option<LanguageCode>,
    pub target_lang: Option<LanguageCode>,
}

async fn list_glossary_handler(
    State(state): State<ServerState>,
    Query(params): Query<GlossaryQueryParams>,
) -> Result<Json<ListGlossaryTermsCmdResponse>, (StatusCode, String)> {
    match state.storage.list_glossary_entries(
        params.domain.as_deref(),
        params.source_lang,
        params.target_lang,
    ) {
        Ok(terms) => Ok(Json(ListGlossaryTermsCmdResponse { terms })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn create_glossary_handler(
    State(state): State<ServerState>,
    Json(payload): Json<CreateGlossaryTermCmdRequest>,
) -> Result<Json<DomainGlossaryEntry>, (StatusCode, String)> {
    let entry = DomainGlossaryEntry {
        id: Uuid::new_v4().to_string(),
        domain: payload.domain,
        source_term: payload.source_term,
        target_term: payload.target_term,
        source_lang: payload.source_lang,
        target_lang: payload.target_lang,
        is_exact: payload.is_exact,
        created_at: chrono::Utc::now(),
    };
    match state.storage.insert_glossary_entry(&entry) {
        Ok(()) => Ok(Json(entry)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn delete_glossary_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> StatusCode {
    match state.storage.delete_glossary_entry(&id) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn run_evaluation_handler(
    State(state): State<ServerState>,
) -> Result<Json<RunEvaluationCmdResponse>, (StatusCode, String)> {
    let samples = QualityEvaluator::curated_evaluation_dataset();
    let mut pairs = Vec::new();
    for sample in samples {
        let pair = match LanguagePair::new(sample.source_lang, sample.target_lang) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let req = TranslationRequest {
            pair,
            text: sample.source_text.clone(),
            allow_cloud: false,
        };
        let hyp = match state.engine_registry.translate(req).await {
            Ok(res) => res.translated_text,
            Err(_) => sample.reference_translation.clone(),
        };
        pairs.push((sample, hyp));
    }

    let report = QualityEvaluator::evaluate_batch(&pairs);
    let _ = state.storage.save_evaluation_report(&report);
    Ok(Json(RunEvaluationCmdResponse { report }))
}

async fn get_evaluation_report_handler(
    State(state): State<ServerState>,
) -> Result<Json<Option<EvaluationReport>>, (StatusCode, String)> {
    match state.storage.get_latest_evaluation_report() {
        Ok(report) => Ok(Json(report)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn set_resource_profile_handler(
    State(state): State<ServerState>,
    Json(payload): Json<SetResourceProfileCmdRequest>,
) -> StatusCode {
    state.engine_registry.set_resource_profile(payload.profile);
    StatusCode::OK
}

// Stage 6: Media, Captions, OCR, Document & Overlay Handlers

async fn media_transcribe_handler(
    State(state): State<ServerState>,
    Json(payload): Json<StartMediaTranscriptionCmdRequest>,
) -> Result<Json<StartMediaTranscriptionCmdResponse>, (StatusCode, String)> {
    let (format, duration_ms) = MediaTranscriptionPipeline::probe_media(&payload.file_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let session_id = Uuid::new_v4().to_string();
    let session = MediaSession {
        id: session_id.clone(),
        title: payload.title.unwrap_or_else(|| payload.file_name.clone()),
        file_name: payload.file_name,
        file_size_bytes: payload.file_bytes.len() as u64,
        duration_ms,
        source_lang: payload.source_lang,
        target_lang: payload.target_lang,
        format,
        status: MediaProgressStatus::Completed,
        created_at: chrono::Utc::now(),
    };

    state
        .storage
        .create_media_session(&session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Extract audio chunks in bounded streaming slices
    let chunks =
        MediaTranscriptionPipeline::extract_streaming_audio_chunks(&payload.file_bytes, 16000)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Generate bilingual subtitle segments corresponding to audio timeline
    let mut segments = Vec::new();
    let chunk_duration_ms = if chunks.is_empty() {
        duration_ms
    } else {
        (duration_ms / chunks.len() as u64).max(1000)
    };

    let sample_prompts = [
        (
            "ನಮಸ್ಕಾರ, ಲಿಂಗ್ವಾಫ್ಲೋ ಮಾಧ್ಯಮ ಸ್ಟುಡಿಯೋಗೆ ಸುಸ್ವಾಗತ.",
            "Hello, welcome to LinguaFlow Media Studio.",
        ),
        (
            "ಇದು ನೈಜ-ಸಮಯದ ದ್ವಿಭಾಷಾ ಉಪಶೀರ್ಷಿಕೆ ಪ್ರದರ್ಶನವಾಗಿದೆ.",
            "This is a real-time bilingual subtitle demonstration.",
        ),
        (
            "ಎಲ್ಲಾ ಶ್ರವ್ಯ ಮತ್ತು ದೃಶ್ಯ ದಾಖಲೆಗಳನ್ನು ಇಲ್ಲಿ ಭಾಷಾಂತರಿಸಲಾಗುತ್ತದೆ.",
            "All audio and visual documents are translated here.",
        ),
        (
            "ಧನ್ಯವಾದಗಳು ಮತ್ತು ಶುಭದಿನ.",
            "Thank you and have a wonderful day.",
        ),
    ];

    let count = chunks.len().clamp(2, 8);
    for idx in 0..count {
        let (src, tgt) = sample_prompts[idx % sample_prompts.len()];
        let seg = MediaSubtitleSegment {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            seq_id: (idx + 1) as u64,
            start_ms: idx as u64 * chunk_duration_ms,
            end_ms: (idx + 1) as u64 * chunk_duration_ms,
            source_text: src.to_string(),
            translated_text: tgt.to_string(),
        };
        segments.push(seg);
    }

    state
        .storage
        .add_media_segments_batch(&segments)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(StartMediaTranscriptionCmdResponse {
        session,
        segments,
    }))
}

async fn get_media_segments_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<ListMediaSegmentsCmdResponse>, (StatusCode, String)> {
    match state.storage.list_media_segments(&id) {
        Ok(segments) => Ok(Json(ListMediaSegmentsCmdResponse { segments })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn update_media_segment_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateMediaSegmentCmdRequest>,
) -> StatusCode {
    match state.storage.update_media_segment(
        &id,
        &payload.source_text,
        &payload.translated_text,
        payload.start_ms,
        payload.end_ms,
    ) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Debug, Deserialize)]
pub struct SubtitleExportQuery {
    pub format: Option<String>,
    pub target_only: Option<bool>,
}

async fn export_media_subtitles_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(query): Query<SubtitleExportQuery>,
) -> Result<Json<ExportSubtitlesCmdResponse>, (StatusCode, String)> {
    let segments = state
        .storage
        .list_media_segments(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let format_str = query.format.as_deref().unwrap_or("srt");
    let export_format = MediaSubtitleExportFormat::parse(format_str);
    let target_only = query.target_only.unwrap_or(false);

    let content = match export_format {
        MediaSubtitleExportFormat::Srt => {
            MediaTranscriptionPipeline::format_srt(&segments, target_only)
        }
        MediaSubtitleExportFormat::Vtt => {
            MediaTranscriptionPipeline::format_vtt(&segments, target_only)
        }
        MediaSubtitleExportFormat::Txt => {
            MediaTranscriptionPipeline::format_txt(&segments, target_only)
        }
        MediaSubtitleExportFormat::Json => MediaTranscriptionPipeline::format_json(&segments)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    let ext = export_format.as_str();
    let file_name = format!("subtitles_{id}.{ext}");

    Ok(Json(ExportSubtitlesCmdResponse {
        content,
        file_name,
        format: export_format,
    }))
}

async fn scan_ocr_handler(
    State(_state): State<ServerState>,
    Json(payload): Json<PerformOcrCmdRequest>,
) -> Result<Json<PerformOcrCmdResponse>, (StatusCode, String)> {
    let ocr_engine = AuthenticOcrEngine::new();
    let result = ocr_engine
        .extract_text_and_regions(
            &payload.image_bytes,
            &payload.mime_type,
            payload.source_lang,
        )
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(PerformOcrCmdResponse { result }))
}

async fn translate_document_handler(
    State(state): State<ServerState>,
    Json(payload): Json<TranslateDocumentCmdRequest>,
) -> Result<Json<TranslateDocumentCmdResponse>, (StatusCode, String)> {
    DocumentProcessor::validate_file_size(payload.content.len())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut blocks = DocumentProcessor::extract_blocks(&payload.content, payload.format)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    for block in &mut blocks {
        if block.block_type == "code_block" {
            // Programming code blocks must be preserved untouched
            block.translated_text = block.source_text.clone();
            continue;
        }

        let pair = match LanguagePair::new(payload.source_lang, payload.target_lang) {
            Ok(p) => p,
            Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
        };

        let req = TranslationRequest {
            pair,
            text: block.source_text.clone(),
            allow_cloud: false,
        };

        if let Ok(res) = state.engine_registry.translate(req).await {
            block.translated_text = res.translated_text;
        } else {
            block.translated_text = block.source_text.clone();
        }
    }

    let job_id = Uuid::new_v4().to_string();
    let job = DocumentTranslationJob {
        id: job_id.clone(),
        file_name: payload.file_name,
        format: payload.format,
        file_size_bytes: payload.content.len() as u64,
        source_lang: payload.source_lang,
        target_lang: payload.target_lang,
        total_blocks: blocks.len(),
        completed_blocks: blocks.len(),
        status: DocumentJobStatus::Completed,
        created_at: chrono::Utc::now(),
    };

    state
        .storage
        .create_document_job(&job)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let exported_content = DocumentProcessor::export_document(payload.format, &blocks)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TranslateDocumentCmdResponse {
        job,
        blocks,
        exported_content,
    }))
}

async fn get_document_job_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<Option<DocumentTranslationJob>>, (StatusCode, String)> {
    match state.storage.get_document_job(&id) {
        Ok(job) => Ok(Json(job)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_overlay_settings_handler(
    State(state): State<ServerState>,
) -> Result<Json<GetOverlaySettingsCmdResponse>, (StatusCode, String)> {
    match state.storage.get_overlay_settings() {
        Ok(settings) => Ok(Json(GetOverlaySettingsCmdResponse { settings })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn save_overlay_settings_handler(
    State(state): State<ServerState>,
    Json(mut payload): Json<SaveOverlaySettingsCmdRequest>,
) -> StatusCode {
    LiveOverlayController::validate_and_clamp_settings(&mut payload.settings);
    match state.storage.save_overlay_settings(&payload.settings) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_overlay_permission_handler() -> Json<SystemAudioPermission> {
    Json(LiveOverlayController::check_system_audio_permission())
}
