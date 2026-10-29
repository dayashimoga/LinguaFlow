// LinguaFlow — Desktop Offline Startup & Embedded Assets Verification Suite
// Adheres strictly to the Global Engineering Contract (AGENTS.md) and P0 defect verification.

use lingua_core::{LanguageCode, LanguagePair, VadState};
use lingua_engine::audio_buffer::AudioRingBuffer;
use lingua_engine::registry::EngineRegistry;
use lingua_engine::vad::VoiceActivityDetector;
use lingua_engine::{ModelManager, TranslationRequest};
use lingua_storage::StorageService;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_tauri_config_offline_integrity() {
    // 1. Verify src-tauri/tauri.conf.json does not leak a development server URL
    let candidates = [
        Path::new("../../src-tauri/tauri.conf.json"),
        Path::new("src-tauri/tauri.conf.json"),
    ];
    let content = candidates
        .iter()
        .find(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .expect("Cannot find src-tauri/tauri.conf.json");

    let json: serde_json::Value =
        serde_json::from_str(&content).expect("tauri.conf.json is not valid JSON");

    let build = json
        .get("build")
        .expect("tauri.conf.json must have 'build' object");
    assert!(
        build.get("devUrl").is_none(),
        "Production tauri.conf.json MUST NOT configure 'devUrl' (must load embedded frontendDist offline)"
    );

    let frontend_dist = build
        .get("frontendDist")
        .and_then(|v| v.as_str())
        .expect("frontendDist must be configured in tauri.conf.json");
    assert_eq!(
        frontend_dist, "../dist",
        "frontendDist must point to '../dist' production asset directory"
    );

    // 2. Verify src-tauri/Cargo.toml enables custom-protocol by default
    let cargo_candidates = [
        Path::new("../../src-tauri/Cargo.toml"),
        Path::new("src-tauri/Cargo.toml"),
    ];
    let cargo_content = cargo_candidates
        .iter()
        .find(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .expect("Cannot find src-tauri/Cargo.toml");

    assert!(
        cargo_content.contains("default = [\"custom-protocol\"]"),
        "src-tauri/Cargo.toml MUST include custom-protocol in default features to ensure assets are bundled in release mode"
    );
}

#[test]
fn test_offline_dictionary_and_fts5_without_network() {
    let storage =
        StorageService::open_in_memory().expect("Failed to initialize in-memory SQLite storage");

    // Search Kannada greeting
    let kn_results = storage
        .search_dictionary("ನಮಸ್ಕಾರ", Some(LanguageCode::KnIn), 10)
        .expect("Dictionary search for Kannada greeting failed");
    assert!(
        !kn_results.is_empty(),
        "Kannada seeded dictionary must contain 'ನಮಸ್ಕಾರ'"
    );
    assert_eq!(kn_results[0].word, "ನಮಸ್ಕಾರ");
    assert_eq!(kn_results[0].language, LanguageCode::KnIn);

    // Search Tamil greeting
    let ta_results = storage
        .search_dictionary("வணக்கம்", Some(LanguageCode::TaIn), 10)
        .expect("Dictionary search for Tamil greeting failed");
    assert!(
        !ta_results.is_empty(),
        "Tamil seeded dictionary must contain 'வணக்கம்'"
    );
    assert_eq!(ta_results[0].word, "வணக்கம்");
    assert_eq!(ta_results[0].language, LanguageCode::TaIn);

    // Search English greeting
    let en_results = storage
        .search_dictionary("hello", Some(LanguageCode::EnUs), 10)
        .expect("Dictionary search for English greeting failed");
    assert!(
        !en_results.is_empty(),
        "English seeded dictionary must contain 'hello'"
    );
    assert_eq!(en_results[0].word.to_lowercase(), "hello");
    assert_eq!(en_results[0].language, LanguageCode::EnUs);
}

#[tokio::test]
async fn test_offline_translation_lexicon_without_network() {
    let storage =
        StorageService::open_in_memory().expect("Failed to initialize in-memory SQLite storage");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));

    // 1. English to Kannada offline
    let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let res_en_kn = registry
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "hello".to_string(),
            allow_cloud: false, // Strict offline guarantee
        })
        .await
        .expect("Offline translation English -> Kannada failed");
    assert!(res_en_kn.is_offline, "is_offline flag must be true");
    assert_eq!(res_en_kn.translated_text, "ನಮಸ್ಕಾರ");
    assert_eq!(res_en_kn.engine_id, "verified-lexicon-engine");

    // 2. Kannada to English offline
    let pair_kn_en = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
    let res_kn_en = registry
        .translate(TranslationRequest {
            pair: pair_kn_en,
            text: "ನಮಸ್ಕಾರ".to_string(),
            allow_cloud: false,
        })
        .await
        .expect("Offline translation Kannada -> English failed");
    assert!(res_kn_en.is_offline, "is_offline flag must be true");
    assert_eq!(res_kn_en.translated_text, "Hello");
    assert_eq!(res_kn_en.engine_id, "verified-lexicon-engine");

    // 3. Tamil to English offline
    let pair_ta_en = LanguagePair::new(LanguageCode::TaIn, LanguageCode::EnUs).unwrap();
    let res_ta_en = registry
        .translate(TranslationRequest {
            pair: pair_ta_en,
            text: "வணக்கம்".to_string(),
            allow_cloud: false,
        })
        .await
        .expect("Offline translation Tamil -> English failed");
    assert!(res_ta_en.is_offline, "is_offline flag must be true");
    assert_eq!(res_ta_en.translated_text, "Hello");
    assert_eq!(res_ta_en.engine_id, "verified-lexicon-engine");
}

#[tokio::test]
async fn test_model_manager_offline_catalog_and_integrity() {
    let temp = tempdir().expect("tempdir failed");
    let storage =
        StorageService::open_in_memory().expect("Failed to initialize in-memory SQLite storage");
    let model_manager = ModelManager::new(storage.clone(), temp.path().join("models"))
        .expect("Failed to initialize ModelManager");

    // Verify catalog enumeration
    let packs = storage
        .list_model_packs()
        .expect("Failed to list model packs");
    assert_eq!(
        packs.len(),
        5,
        "Expected 5 production registered model packs"
    );

    for pack in &packs {
        assert!(pack.size_bytes > 0, "Model pack size must be non-zero");
        assert!(
            !pack.sha256.is_empty(),
            "Model pack checksum must be defined"
        );
    }

    // Verify disk space calculation returns a valid non-zero report
    let space = model_manager
        .get_disk_space()
        .expect("Failed to get disk space");
    assert!(
        space.available_bytes > 0,
        "Free disk space must be greater than zero"
    );
}

#[test]
fn test_audio_buffer_and_vad_without_network() {
    let mut ring_buffer = AudioRingBuffer::new(5000); // 5000 samples
    let silence = vec![0.0f32; 1000];
    ring_buffer.push(&silence);
    assert_eq!(ring_buffer.len(), 1000);

    let mut vad = VoiceActivityDetector::new(-45.0, 3, 5);
    let silence_frame = vec![0.001f32; 512];
    let res = vad.process_frame(&silence_frame);
    assert_eq!(res.state, VadState::Silence);
    assert!(!res.is_speech_onset);

    // Active tone
    let speech_frame: Vec<f32> = (0..512).map(|i| 0.35 * (i as f32 * 0.1).sin()).collect();
    let mut detected = false;
    for _ in 0..10 {
        let r = vad.process_frame(&speech_frame);
        if r.state == VadState::Speech || r.is_speech_onset {
            detected = true;
        }
    }
    assert!(detected, "Active tone must be detected as speech by VAD");
}

#[test]
fn test_desktop_binary_embedded_assets_if_built() {
    let candidates = [
        Path::new("../../release-artifacts/LinguaFlow.exe"),
        Path::new("../../target/release/LinguaFlow.exe"),
        Path::new("../../target/release/lingua-tauri.exe"),
        Path::new("release-artifacts/LinguaFlow.exe"),
        Path::new("target/release/LinguaFlow.exe"),
        Path::new("target/release/lingua-tauri.exe"),
    ];

    for path in &candidates {
        if path.exists() {
            let bytes = fs::read(path).expect("Failed to read desktop binary");
            let text = String::from_utf8_lossy(&bytes);
            assert!(
                !text.contains("http://localhost:5173"),
                "Production binary at {:?} MUST NOT contain 'http://localhost:5173'",
                path
            );
            return;
        }
    }
}
