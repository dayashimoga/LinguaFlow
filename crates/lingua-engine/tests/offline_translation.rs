use lingua_core::{LanguageCode, LanguagePair, LinguaError, ModelInstallState, ModelPackManifest};
use lingua_engine::{EngineRegistry, ModelManager, TranslationRequest};
use lingua_storage::StorageService;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_offline_neural_translation_with_installed_model() {
    let temp = tempdir().expect("tempdir failed");
    let storage = StorageService::open_in_memory().expect("in-memory DB failed");
    let models_dir = temp.path().join("models");

    let manager = ModelManager::new(storage.clone(), models_dir.clone()).expect("manager failed");

    // 1. Install certified Kannada <-> English model pack
    let weights = b"AUTHENTIC_CERTIFIED_INDICTRANS2_WEIGHTS_KN_EN";
    let mut hasher = Sha256::new();
    hasher.update(weights);
    let valid_sha = hex::encode(hasher.finalize());

    storage
        .update_model_pack_sha256("model-kn-en-indictrans2", &valid_sha)
        .unwrap();

    let manifest = ModelPackManifest {
        id: "model-kn-en-indictrans2".into(),
        name: "Kannada <-> English Neural Translation (IndicTrans2)".into(),
        version: "1.0.0".into(),
        source_language: LanguageCode::KnIn,
        target_language: LanguageCode::EnUs,
        architecture: "indictrans2-quantized-onnx".into(),
        size_bytes: weights.len() as u64,
        sha256: valid_sha,
        signature: "sig-verified-stage-2".into(),
        min_app_version: "0.1.0".into(),
        files: vec![],
        vocab_size: 64000,
    };
    storage.save_model_manifest(&manifest).unwrap();

    manager
        .install_certified_model_pack(
            "model-kn-en-indictrans2",
            weights,
            r#"{"tokens": ["<s>", "</s>", "<2en>", "<2kn>"]}"#,
        )
        .expect("install model pack failed");

    // Initialize EngineRegistry pointing to this storage & models dir
    let registry = EngineRegistry::new_with_storage_and_models(storage.clone(), models_dir);

    // 2. Perform offline translation from English to Kannada (<2kn>) with allow_cloud = false
    let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let resp = registry
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "Hello world".into(),
            allow_cloud: false,
        })
        .await
        .expect("offline translation failed");

    assert_eq!(resp.translated_text, "ನಮಸ್ಕಾರ ಜಗತ್ತು");
    assert!(resp.is_offline);
    assert_eq!(resp.engine_id, "local-neural-indictrans2");

    // 3. Second call should hit local SQLite translation cache (< 5ms)
    let cache_resp = registry
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "Hello world".into(),
            allow_cloud: false,
        })
        .await
        .expect("cache translation failed");

    assert_eq!(cache_resp.translated_text, "ನಮಸ್ಕಾರ ಜಗತ್ತು");
    assert!(cache_resp.is_offline);
    assert!(cache_resp.engine_id.contains("cache"));
    assert!(cache_resp.latency_ms <= 10);
}

#[tokio::test]
async fn test_offline_tamil_to_english_and_kannada_direct() {
    let temp = tempdir().expect("tempdir failed");
    let storage = StorageService::open_in_memory().expect("in-memory DB failed");
    let models_dir = temp.path().join("models");

    let manager = ModelManager::new(storage.clone(), models_dir.clone()).expect("manager failed");

    // Install Tamil-English model
    let ta_weights = b"AUTHENTIC_CERTIFIED_INDICTRANS2_WEIGHTS_TA_EN";
    let mut hasher = Sha256::new();
    hasher.update(ta_weights);
    let ta_sha = hex::encode(hasher.finalize());

    storage
        .update_model_pack_sha256("model-ta-en-indictrans2", &ta_sha)
        .unwrap();

    manager
        .install_certified_model_pack(
            "model-ta-en-indictrans2",
            ta_weights,
            r#"{"tokens": ["<s>", "</s>"]}"#,
        )
        .expect("install model pack failed");

    // Install Kannada-Tamil direct model
    let kn_ta_weights = b"AUTHENTIC_CERTIFIED_INDICTRANS2_WEIGHTS_KN_TA";
    let mut kn_ta_hasher = Sha256::new();
    kn_ta_hasher.update(kn_ta_weights);
    let kn_ta_sha = hex::encode(kn_ta_hasher.finalize());

    storage
        .update_model_pack_sha256("model-kn-ta-direct", &kn_ta_sha)
        .unwrap();

    manager
        .install_certified_model_pack(
            "model-kn-ta-direct",
            kn_ta_weights,
            r#"{"tokens": ["<s>", "</s>"]}"#,
        )
        .expect("install model pack failed");

    let registry = EngineRegistry::new_with_storage_and_models(storage.clone(), models_dir);

    // 1. Tamil to English translation
    let pair_ta_en = LanguagePair::new(LanguageCode::TaIn, LanguageCode::EnUs).unwrap();
    let resp_ta_en = registry
        .translate(TranslationRequest {
            pair: pair_ta_en,
            text: "வணக்கம் உலகம்".into(),
            allow_cloud: false,
        })
        .await
        .expect("ta->en failed");

    assert_eq!(resp_ta_en.translated_text, "Hello world");
    assert!(resp_ta_en.is_offline);

    // 2. Kannada to Tamil direct translation
    let pair_kn_ta = LanguagePair::new(LanguageCode::KnIn, LanguageCode::TaIn).unwrap();
    let resp_kn_ta = registry
        .translate(TranslationRequest {
            pair: pair_kn_ta,
            text: "ಧನ್ಯವಾದಗಳು".into(),
            allow_cloud: false,
        })
        .await
        .expect("kn->ta failed");

    assert_eq!(resp_kn_ta.translated_text, "நன்றி");
    assert!(resp_kn_ta.is_offline);
}

#[tokio::test]
async fn test_offline_fallback_and_missing_model_honesty() {
    let temp = tempdir().expect("tempdir failed");
    let storage = StorageService::open_in_memory().expect("in-memory DB failed");
    let models_dir = temp.path().join("models");

    // No neural models installed
    let registry = EngineRegistry::new_with_storage_and_models(storage.clone(), models_dir);

    // 1. Seeded lexicon terms should still translate offline via verified lexicon fallback!
    let pair_kn_en = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
    let lexicon_res = registry
        .translate(TranslationRequest {
            pair: pair_kn_en,
            text: "ನಮಸ್ಕಾರ".into(),
            allow_cloud: false,
        })
        .await
        .expect("lexicon fallback failed");

    assert_eq!(lexicon_res.translated_text, "Hello");
    assert!(lexicon_res.is_offline);
    assert_eq!(lexicon_res.engine_id, "verified-lexicon-engine");

    // 2. Unseeded sentence with allow_cloud = false MUST fail with explicit OfflineModelMissing
    let missing_res = registry
        .translate(TranslationRequest {
            pair: pair_kn_en,
            text: "ಇಂದು ಸಂಜೆ ಸಭೆ ಇದೆ".into(),
            allow_cloud: false,
        })
        .await;

    assert!(missing_res.is_err());
    match missing_res.err().unwrap() {
        LinguaError::OfflineModelMissing(msg) => {
            assert!(msg.contains("model-kn-en-indictrans2"));
            assert!(msg.contains("Model Manager"));
        }
        other => panic!("Expected OfflineModelMissing, got {:?}", other),
    }
}

#[tokio::test]
async fn test_corrupted_model_detection_and_rollback() {
    let temp = tempdir().expect("tempdir failed");
    let storage = StorageService::open_in_memory().expect("in-memory DB failed");
    let models_dir = temp.path().join("models");

    let manager = ModelManager::new(storage.clone(), models_dir).expect("manager init failed");

    // Install model
    let weights = b"VALID_MODEL_PAYLOAD_FOR_INTEGRITY_CHECK";
    let mut hasher = Sha256::new();
    hasher.update(weights);
    let valid_sha = hex::encode(hasher.finalize());

    storage
        .update_model_pack_sha256("model-kn-en-indictrans2", &valid_sha)
        .unwrap();

    let install_dir = manager
        .install_certified_model_pack("model-kn-en-indictrans2", weights, "{}")
        .expect("clean install failed");

    // Tamper with binary
    fs::write(install_dir.join("model.bin"), b"TAMPERED_MALICIOUS_BYTES").unwrap();

    let is_valid = manager
        .verify_model_pack("model-kn-en-indictrans2")
        .expect("verify ran");
    assert!(!is_valid);

    let pack = storage
        .get_detailed_model_pack("model-kn-en-indictrans2")
        .unwrap()
        .unwrap();
    match pack.state {
        ModelInstallState::Corrupted { reason } => {
            assert!(reason.contains("Integrity verification failed"));
        }
        other => panic!("Expected Corrupted state, got {:?}", other),
    }
}

#[tokio::test]
async fn test_cancellation_token_in_download() {
    let temp = tempdir().expect("tempdir failed");
    let storage = StorageService::open_in_memory().expect("in-memory DB failed");
    let models_dir = temp.path().join("models");

    let manager = ModelManager::new(storage.clone(), models_dir).expect("manager init failed");
    let cancel_token = Arc::new(AtomicBool::new(false));
    let token_clone = cancel_token.clone();

    let err = manager
        .download_and_install_simulated(
            "model-kn-en-indictrans2",
            Some(move |_down, _tot, _pct| {
                token_clone.store(true, Ordering::SeqCst);
            }),
            Some(cancel_token),
        )
        .unwrap_err();

    match err {
        LinguaError::DownloadCancelled(msg) => {
            assert!(msg.contains("cancelled by user"));
        }
        other => panic!("Expected DownloadCancelled, got {:?}", other),
    }

    let pack = storage
        .get_detailed_model_pack("model-kn-en-indictrans2")
        .unwrap()
        .unwrap();
    assert_eq!(pack.state, ModelInstallState::NotInstalled);
}
