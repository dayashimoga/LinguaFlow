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
        assert_eq!(caps, CapabilityMatrix::stage_4_active());

        // Test /api/v1/models
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/models")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let models: Vec<lingua_core::ModelPackDetailedInfo> =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(models.len(), 5);

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
            .clone()
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

        // Test /api/v1/speech/devices
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/speech/devices")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/speech/session/start
        let start_req = serde_json::to_string(&lingua_core::StartSpeechSessionCmdRequest {
            source_language: LanguageCode::KnIn,
            target_language: LanguageCode::EnUs,
            title: Some("Test Stage 3 Live Captions".into()),
            device_id: None,
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/speech/session/start")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(start_req))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let start_res: lingua_core::StartSpeechSessionCmdResponse =
            serde_json::from_slice(&body).unwrap();
        let session_id = start_res.session.id;

        // Test /api/v1/speech/session/pause
        let pause_req = serde_json::to_string(&lingua_core::PauseSpeechSessionCmdRequest {
            session_id: session_id.clone(),
        })
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/speech/session/pause")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(pause_req))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/speech/session/resume
        let resume_req = serde_json::to_string(&lingua_core::ResumeSpeechSessionCmdRequest {
            session_id: session_id.clone(),
        })
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/speech/session/resume")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(resume_req))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/speech/session/stop
        let stop_req = serde_json::to_string(&lingua_core::StopSpeechSessionCmdRequest {
            session_id: session_id.clone(),
        })
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/speech/session/stop")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(stop_req))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/speech/session/{id}/captions
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/speech/session/{session_id}/captions"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/speech/session/{id}/export?format=vtt
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!(
                        "/api/v1/speech/session/{session_id}/export?format=vtt"
                    ))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // Test /api/v1/tts/voices
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/tts/voices")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let voice_resp: lingua_core::GetTtsVoicesCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(voice_resp.voices.len(), 6);

        // Test /api/v1/conversation/start
        let start_conv_payload = serde_json::to_string(&lingua_core::StartConversationCmdRequest {
            speaker_a_lang: LanguageCode::KnIn,
            speaker_b_lang: LanguageCode::EnUs,
            title: Some("Consultation".into()),
            is_hands_free: false,
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/conversation/start")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(start_conv_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let conv_resp: lingua_core::StartConversationCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(conv_resp.session.speaker_a_lang, LanguageCode::KnIn);
        assert_eq!(conv_resp.session.speaker_b_lang, LanguageCode::EnUs);
    }
}
