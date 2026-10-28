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
    detect_language, AddFavoriteCmdRequest, AppInfo, CapabilityMatrix, DatabaseHealthInfo,
    DetectLanguageCmdRequest, DetectLanguageCmdResponse, DictionaryEntry,
    DictionarySearchCmdResponse, FavoriteItem, LanguageCode, LanguagePair, ModelPackInfo,
    TranslateCmdRequest, TranslateCmdResponse, TranslationRecord, UserSettings,
};
use lingua_engine::TranslationRequest;
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
