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

    #[tokio::test]
    async fn test_stage_7_learning_and_privacy_endpoints() {
        let storage = StorageService::open_in_memory().unwrap();
        let engine = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
        let app_info = AppInfo::default();
        let state = ServerState::new(storage, engine, app_info);
        let app = create_router(state);

        // 1. Test /api/v1/learning/decks
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/learning/decks")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let deck_resp: lingua_core::ListLearningDecksCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert!(!deck_resp.decks.is_empty());
        let deck_id = deck_resp.decks[0].id.clone();

        // 2. Test /api/v1/learning/deck/{id}/cards
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/learning/deck/{deck_id}/cards"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let card_resp: lingua_core::GetLearningDeckCardsCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert!(!card_resp.cards.is_empty());
        let card_id = card_resp.cards[0].id.clone();

        // 3. Test /api/v1/learning/card/{id}/review
        let review_payload = serde_json::to_string(&serde_json::json!({
            "rating": 4
        }))
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/learning/card/{card_id}/review"))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(review_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let review_resp: lingua_core::ReviewCardCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(review_resp.result.new_repetition, 1);

        // 4. Test /api/v1/learning/progress
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/learning/progress")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let prog_resp: lingua_core::GetLearnerProgressCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert!(prog_resp.progress.total_cards_studied >= 1);

        // 5. Test /api/v1/learning/pronunciation/assess
        let assess_payload = serde_json::to_string(&lingua_core::AssessPronunciationCmdRequest {
            request: lingua_core::PronunciationAssessmentRequest {
                reference_text: "ನಮಸ್ಕಾರ".into(),
                spoken_transcript: "ನಮಸ್ಕಾರ".into(),
                language: LanguageCode::KnIn,
            },
        })
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/learning/pronunciation/assess")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(assess_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let assess_resp: lingua_core::AssessPronunciationCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert!(assess_resp.result.is_passing);

        // 6. Test /api/v1/learning/listening/challenges
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/learning/listening/challenges?language=kn-IN&count=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // 7. Test /api/v1/learning/grammar/topics
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/learning/grammar/topics?language=ta-IN")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // 8. Test /api/v1/privacy/settings
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/privacy/settings")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // 9. Test /api/v1/privacy/export
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/privacy/export")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let export_resp: lingua_core::ExportUserDataCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(export_resp.archive.app_version, "0.1.0");
        assert!(!export_resp.archive.checksum_sha256.is_empty());

        // 10. Test /api/v1/platform/release-status
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/platform/release-status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let rel_resp: lingua_core::GetPlatformReleaseStatusCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(rel_resp.status.len(), 4);
    }
}
