use crate::state::ServerState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Json, Router,
};
use lingua_core::{AppInfo, CapabilityMatrix, DatabaseHealthInfo, ModelPackInfo, UserSettings};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

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
