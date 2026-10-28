use chrono::Utc;
use lingua_core::{
    AccountTier, ApiKeyRecord, AuditEvent, LanguageCode, LinguaError, OrgGlossaryTerm, OrgRole,
    Organization, UsageType,
};
use lingua_engine::{ApiGateway, InferenceWorkerPool, RbacController, UsageQuotaManager};
use lingua_storage::StorageService;

#[tokio::test]
async fn test_anonymous_offline_mode_guarantee() {
    let storage = StorageService::open_in_memory().expect("in-memory db");
    let anon = storage.get_or_create_anonymous_account().expect("get anon");

    // Anonymous Offline Mode Guarantee (REQ-8.1-AUTH):
    // Anonymous users must have a valid Free tier account with local offline capabilities
    assert!(anon.is_anonymous);
    assert_eq!(anon.tier, AccountTier::Free);

    // Dictionary and local operations must remain fully accessible
    let results = storage
        .search_dictionary("ನಮಸ್ಕಾರ", Some(LanguageCode::KnIn), 5)
        .expect("search dictionary");
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_quota_enforcement_and_fail_safe_refund() {
    let storage = StorageService::open_in_memory().expect("in-memory db");
    let anon = storage.get_or_create_anonymous_account().expect("get anon");
    let manager = UsageQuotaManager::new(storage.clone());

    // 1. Quota check passes within free tier limit (10,000 words)
    let tier = manager
        .check_quota(&anon.id, UsageType::CloudTranslationWords, 250)
        .expect("quota check");
    assert_eq!(tier, AccountTier::Free);

    // 2. Quota check fails when requesting more than limit
    let err = manager.check_quota(&anon.id, UsageType::CloudTranslationWords, 20_000);
    assert!(err.is_err());
    match err {
        Err(LinguaError::QuotaExceeded {
            usage_type, limit, ..
        }) => {
            assert_eq!(usage_type, "cloud_translation_words");
            assert_eq!(limit, 10_000);
        }
        _ => panic!("Expected QuotaExceeded error"),
    }

    // 3. Fail-safe billing rule: failed billable operation does NOT charge quota
    let initial_summary = storage.get_usage_summary(&anon.id).expect("summary");
    assert_eq!(initial_summary.words_used, 0);

    let failed_op: Result<(), LinguaError> = manager
        .execute_billable_op(
            &anon.id,
            None,
            UsageType::CloudTranslationWords,
            500,
            25,
            || async { Err(LinguaError::NetworkUnavailable) },
        )
        .await;

    assert!(failed_op.is_err());
    let summary_after_fail = storage.get_usage_summary(&anon.id).expect("summary");
    assert_eq!(summary_after_fail.words_used, 0); // Must be zero!

    // 4. Successful operation correctly commits usage
    let success_op: Result<String, LinguaError> = manager
        .execute_billable_op(
            &anon.id,
            None,
            UsageType::CloudTranslationWords,
            350,
            15,
            || async { Ok("ಧನ್ಯವಾದಗಳು".to_string()) },
        )
        .await;

    assert_eq!(success_op.unwrap(), "ಧನ್ಯವಾದಗಳು");
    let summary_after_success = storage.get_usage_summary(&anon.id).expect("summary");
    assert_eq!(summary_after_success.words_used, 350);
    assert_eq!(summary_after_success.current_period_cents, 15);
}

#[test]
fn test_rbac_tenancy_isolation_and_glossary_replacement() {
    // 1. RBAC permissions check
    assert!(RbacController::authorize_action(OrgRole::Owner, "invite_member").is_ok());
    assert!(RbacController::authorize_action(OrgRole::Admin, "create_glossary").is_ok());
    assert!(RbacController::authorize_action(OrgRole::LinguistEditor, "add_term").is_ok());
    assert!(RbacController::authorize_action(OrgRole::Member, "invite_member").is_err());
    assert!(RbacController::authorize_action(OrgRole::Viewer, "add_term").is_err());

    // 2. Tenancy isolation enforcement
    assert!(RbacController::enforce_tenant_isolation(Some("org-alpha"), Some("org-alpha")).is_ok());
    let isolation_err =
        RbacController::enforce_tenant_isolation(Some("org-alpha"), Some("org-beta"));
    assert!(isolation_err.is_err());
    match isolation_err {
        Err(LinguaError::Forbidden(msg)) => {
            assert!(msg.contains("Cross-tenant isolation violation"));
        }
        _ => panic!("Expected Forbidden error"),
    }

    // 3. Shared glossary replacement
    let terms = vec![
        OrgGlossaryTerm {
            id: "t1".into(),
            glossary_id: "g1".into(),
            source_term: "Artificial Intelligence".into(),
            target_term: "ಕೃತಕ ಬುದ್ಧಿಮತ್ತೆ".into(),
            notes: Some("Standard Kannada AI nomenclature".into()),
            case_sensitive: false,
        },
        OrgGlossaryTerm {
            id: "t2".into(),
            glossary_id: "g1".into(),
            source_term: "Machine Learning".into(),
            target_term: "ಯಂತ್ರ ಕಲಿಕೆ".into(),
            notes: None,
            case_sensitive: false,
        },
    ];

    let (replaced, applied) = RbacController::apply_org_glossaries(
        "We are studying Artificial Intelligence and Machine Learning algorithms.",
        &terms,
    );

    assert_eq!(
        replaced,
        "We are studying ಕೃತಕ ಬುದ್ಧಿಮತ್ತೆ and ಯಂತ್ರ ಕಲಿಕೆ algorithms."
    );
    assert_eq!(applied.len(), 2);
}

#[test]
fn test_api_key_authentication_and_rate_limiting() {
    let gateway = ApiGateway::new();

    // 1. Generate API key and compute cryptographic hash
    let (raw_key, prefix, key_hash) = ApiGateway::generate_raw_api_key("live");
    assert!(raw_key.starts_with("lf_live_"));
    assert_eq!(prefix, "lf_live");
    assert_eq!(ApiGateway::hash_api_key(&raw_key), key_hash);

    let key_record = ApiKeyRecord {
        id: "key-rec-01".into(),
        user_id: "user-test".into(),
        org_id: Some("org-alpha".into()),
        key_hash,
        key_prefix: prefix,
        name: "Production Gateway Key".into(),
        rate_limit_per_min: 10,
        is_active: true,
        created_at: Utc::now(),
        expires_at: None,
    };

    // 2. Rate limit allows up to 10 requests per minute
    for _ in 0..10 {
        assert!(gateway.check_rate_limit(&key_record).is_ok());
    }

    // 3. 11th request triggers RateLimitExceeded
    let exceeded = gateway.check_rate_limit(&key_record);
    assert!(exceeded.is_err());
    match exceeded {
        Err(LinguaError::RateLimitExceeded {
            limit_per_minute, ..
        }) => {
            assert_eq!(limit_per_minute, 10);
        }
        _ => panic!("Expected RateLimitExceeded"),
    }
}

#[tokio::test]
async fn test_inference_worker_pool_backpressure_and_failover() {
    // Pool with 2 max workers and queue capacity of 2
    let pool = InferenceWorkerPool::new(2, 2);

    // 1. Dispatch normal tasks
    let res = pool
        .dispatch_task(|| async { Ok("kannada translated speech chunk".to_string()) })
        .await;
    assert_eq!(res.unwrap(), "kannada translated speech chunk");

    let metrics = pool.get_metrics();
    assert_eq!(metrics.processed_tasks, 1);
    assert_eq!(metrics.failed_tasks, 0);
    assert!(metrics.is_accepting_tasks);

    // 2. Simulated regional cloud worker outage
    pool.set_healthy(false);
    let outage_call: Result<(), LinguaError> = pool.dispatch_task(|| async { Ok(()) }).await;
    assert!(outage_call.is_err());
    match outage_call {
        Err(LinguaError::CapabilityUnavailable(cap, _)) => {
            assert_eq!(cap, "WorkerPool");
        }
        _ => panic!("Expected CapabilityUnavailable error"),
    }

    // 3. Restore worker health
    pool.set_healthy(true);
    let recovered_call = pool
        .dispatch_task(|| async { Ok("recovered".to_string()) })
        .await;
    assert_eq!(recovered_call.unwrap(), "recovered");
}

#[test]
fn test_audit_event_soc2_compliance_trail() {
    let storage = StorageService::open_in_memory().expect("in-memory db");
    let anon = storage.get_or_create_anonymous_account().expect("get anon");

    // Create organization
    let org = Organization {
        id: "org-soc2".into(),
        name: "Enterprise Compliance Workspace".into(),
        slug: "compliance-ws".into(),
        max_seats: 100,
        sso_enabled: true,
        data_retention_days: 365,
        enforce_zero_transcript_logging: true,
        created_at: Utc::now(),
    };
    storage
        .create_organization(&org, &anon.id)
        .expect("create org");

    // Record administrative action
    let audit_event = AuditEvent {
        id: "evt-01".into(),
        org_id: Some(org.id.clone()),
        actor_id: anon.id.clone(),
        actor_email: anon.email.clone(),
        action: "policy.retention_updated".into(),
        resource_type: "organization_policy".into(),
        resource_id: org.id.clone(),
        details: "Data retention policy set to 365 days; zero-transcript logging enforced".into(),
        ip_address: "192.168.1.100".into(),
        timestamp: Utc::now(),
    };
    storage
        .record_audit_event(&audit_event)
        .expect("record audit");

    let events = storage
        .list_audit_events(Some(&org.id), 10)
        .expect("list events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "policy.retention_updated");
    assert_eq!(events[0].resource_type, "organization_policy");
}
