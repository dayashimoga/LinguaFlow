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
        let state = ServerState::new(storage.clone(), engine, app_info);

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
        assert_eq!(caps, CapabilityMatrix::stage_8_active());

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
        assert_eq!(models.len(), 9);

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

        // Install required speech model for kn-IN ASR
        storage
            .update_model_pack_install_state(
                "asr-kn-streaming-sherpa",
                &lingua_core::ModelInstallState::Installed {
                    installed_at: chrono::Utc::now(),
                    install_path: "/test/models/asr-kn".into(),
                },
                true,
            )
            .unwrap();

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
        let state = ServerState::new(storage.clone(), engine, app_info);
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

    #[tokio::test]
    async fn test_stage_8_enterprise_endpoints() {
        let storage = StorageService::open_in_memory().unwrap();
        let engine = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
        let app_info = AppInfo::default();
        let state = ServerState::new(storage.clone(), engine, app_info);
        let app = create_router(state);

        // 1. GET /api/v1/enterprise/account
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/account")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let account_resp: lingua_core::GetAccountInfoCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(account_resp.account.id, "acc-anonymous-local");
        assert!(account_resp.account.is_anonymous);
        let user_id = account_resp.account.id.clone();

        // 2. POST /api/v1/enterprise/devices
        let reg_payload = serde_json::to_string(&lingua_core::RegisterDeviceCmdRequest {
            account_id: user_id.clone(),
            device_name: "MacBook Pro M3 Max".into(),
            device_type: lingua_core::DeviceType::DesktopMac,
            public_key_fingerprint: "SHA256:7b92abc".into(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/enterprise/devices")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(reg_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let reg_resp: lingua_core::RegisterDeviceCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(reg_resp.device.device_name, "MacBook Pro M3 Max");

        // 3. GET /api/v1/enterprise/devices
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/devices")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let devices_resp: lingua_core::ListDevicesCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(devices_resp.devices.len(), 1);

        // 4. POST /api/v1/enterprise/keys
        let key_payload = serde_json::to_string(&lingua_core::CreateApiKeyCmdRequest {
            user_id: user_id.clone(),
            org_id: None,
            name: "CI/CD Pipeline Key".into(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/enterprise/keys")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(key_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let key_resp: lingua_core::CreateApiKeyCmdResponse = serde_json::from_slice(&body).unwrap();
        assert!(key_resp.raw_key.starts_with("lf_live_"));

        // 5. GET /api/v1/enterprise/keys
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/keys")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let keys_resp: lingua_core::ListApiKeysCmdResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(keys_resp.api_keys.len(), 1);

        // 6. GET /api/v1/enterprise/usage
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/usage")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let usage_resp: lingua_core::GetUsageSummaryCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(usage_resp.summary.tier, lingua_core::AccountTier::Free);

        // 7. GET /api/v1/enterprise/invoices
        let test_invoice = lingua_core::BillingInvoiceRecord {
            id: "inv-test-1".into(),
            account_id: user_id.clone(),
            org_id: None,
            invoice_number: "INV-2026-0001".into(),
            billing_period: "2026-10".into(),
            amount_cents: 0,
            currency: "USD".into(),
            status: lingua_core::InvoiceStatus::Paid,
            line_items: vec![],
            created_at: chrono::Utc::now(),
        };
        storage.create_invoice(&test_invoice).unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/invoices")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let invoices_resp: lingua_core::ListInvoicesCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(invoices_resp.invoices.len(), 1);

        // 8. POST /api/v1/enterprise/orgs
        let org_payload = serde_json::to_string(&lingua_core::CreateOrgCmdRequest {
            name: "Karnataka State Digital Governance".into(),
            slug: "karnataka-gov".into(),
            owner_id: user_id.clone(),
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/enterprise/orgs")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(org_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let org_resp: lingua_core::OrgCmdResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(org_resp.organization.slug, "karnataka-gov");
        let org_id = org_resp.organization.id;

        // 9. GET /api/v1/enterprise/orgs/{id}/members
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/enterprise/orgs/{org_id}/members"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let members_resp: lingua_core::ListOrgMembersCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(members_resp.members.len(), 1);

        // 10. POST /api/v1/enterprise/orgs/{id}/glossaries
        let glos_payload = serde_json::to_string(&lingua_core::CreateOrgGlossaryCmdRequest {
            org_id: org_id.clone(),
            name: "Official Governance Dravidian Lexicon".into(),
            description: "Standard terms for digital services".into(),
            source_lang: lingua_core::LanguageCode::EnUs,
            target_lang: lingua_core::LanguageCode::KnIn,
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/enterprise/orgs/{org_id}/glossaries"))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(glos_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        // 11. GET /api/v1/enterprise/orgs/{id}/glossaries
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/enterprise/orgs/{org_id}/glossaries"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let glos_resp: lingua_core::ListOrgGlossariesCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(glos_resp.glossaries.len(), 1);

        // 12. GET /api/v1/enterprise/audit?org_id={org_id}
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/enterprise/audit?org_id={org_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let audit_resp: lingua_core::ListAuditEventsCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert!(!audit_resp.events.is_empty());

        // 13. GET /api/v1/enterprise/workers/metrics
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/enterprise/workers/metrics")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let metrics_resp: lingua_core::GetWorkerMetricsCmdResponse =
            serde_json::from_slice(&body).unwrap();
        assert_eq!(metrics_resp.metrics.active_workers, 0);
        assert_eq!(metrics_resp.metrics.queue_depth, 0);
    }
}
