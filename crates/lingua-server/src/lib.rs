pub mod routes;
pub mod state;

pub use routes::create_router;
pub use state::ServerState;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub async fn run_server(
    state: ServerState,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);
    let listener = TcpListener::bind(addr).await?;
    info!("LinguaFlow Gateway Server listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use lingua_core::{
        AppInfo, CapabilityMatrix, DatabaseHealthInfo, DictionarySearchCmdResponse, LanguageCode,
        TranslateCmdRequest, TranslateCmdResponse,
    };
    use lingua_engine::EngineRegistry;
    use lingua_storage::StorageService;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_server_health_and_capabilities_endpoints() {
        let storage = StorageService::open_in_memory().unwrap();
        let engine = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
        let app_info = AppInfo::default();
        let state = ServerState::new(storage, engine, app_info);

        let app = create_router(state);

        // Test /api/v1/health
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let health: DatabaseHealthInfo = serde_json::from_slice(&body).unwrap();
        assert_eq!(health.status, "healthy");

        // Test /api/v1/capabilities
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/capabilities")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let caps: CapabilityMatrix = serde_json::from_slice(&body).unwrap();
        assert_eq!(caps, CapabilityMatrix::stage_1_active());

        // Test /api/v1/translate
        let translate_payload = serde_json::to_string(&TranslateCmdRequest {
            source_language: LanguageCode::KnIn,
            target_language: LanguageCode::EnUs,
            text: "ನಮಸ್ಕಾರ".into(),
            allow_cloud: false,
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/translate")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(translate_payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let tr: TranslateCmdResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(tr.translated_text, "Hello");

        // Test /api/v1/dictionary/search
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/dictionary/search?q=water&limit=5")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let dict_res: DictionarySearchCmdResponse = serde_json::from_slice(&body).unwrap();
        assert!(!dict_res.entries.is_empty());
    }
}
