use lingua_core::{FormalityMode, IntelligenceTranslationRequest, LanguageCode, LanguagePair};
use lingua_engine::{EngineRegistry, TranslationRequest};
use lingua_storage::StorageService;
use std::sync::Arc;

#[tokio::test]
async fn test_adventure_translation_en_to_kn_returns_sahasa_and_never_devegouda() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = EngineRegistry::new_with_storage(storage);

    // 1. Direct Tier 1 Translation
    let req = TranslationRequest {
        text: "adventure".into(),
        pair: LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).expect("valid pair"),
        allow_cloud: true, // Even if cloud is allowed, Tier 1 offline verified lexicon takes precedence
    };

    let res = registry.translate(req).await.expect("translation succeeds");
    assert_eq!(
        res.translated_text, "ಸಾಹಸ",
        "Translating 'adventure' to kn-IN must return 'ಸಾಹಸ'"
    );
    assert_ne!(
        res.translated_text, "ದೇವೇಗೌಡರು",
        "Translating 'adventure' must never return 'ದೇವೇಗೌಡರು'"
    );
    assert!(res.is_offline);

    // 2. Intelligence Pipeline Translation
    let intel_req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::KnIn,
        domain_glossary: None,
        enable_transliteration: true, // Should not mangle English input
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let intel_res = registry
        .translate_with_intelligence(intel_req)
        .await
        .expect("intelligence translation succeeds");

    assert_eq!(
        intel_res.translated_text, "ಸಾಹಸ",
        "Intelligence pipeline must return 'ಸಾಹಸ'"
    );
    assert_ne!(intel_res.translated_text, "ದೇವೇಗೌಡರು");
    assert_eq!(
        intel_res.metadata.engine_selected, "offline_fts5_lexicon",
        "Must be attributed to offline FTS5 lexicon"
    );
    assert!(
        intel_res.metadata.confidence_score.unwrap_or(0.0) >= 0.95,
        "Verified lexicon confidence must be >= 0.95"
    );
    assert!(
        !intel_res.metadata.dictionary_senses.is_empty(),
        "Must return alternate dictionary senses"
    );
}

#[tokio::test]
async fn test_adventure_translation_en_to_ta_returns_cakacam() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = EngineRegistry::new_with_storage(storage);

    let req = TranslationRequest {
        text: "adventure".into(),
        pair: LanguagePair::new(LanguageCode::EnUs, LanguageCode::TaIn).expect("valid pair"),
        allow_cloud: false,
    };

    let res = registry.translate(req).await.expect("translation succeeds");
    assert_eq!(
        res.translated_text, "சாகசம்",
        "Translating 'adventure' to ta-IN must return 'சாகசம்'"
    );
    assert!(res.is_offline);

    let intel_req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::TaIn,
        domain_glossary: None,
        enable_transliteration: false,
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let intel_res = registry
        .translate_with_intelligence(intel_req)
        .await
        .expect("intelligence translation succeeds");

    assert_eq!(intel_res.translated_text, "சாகசம்");
    assert_eq!(intel_res.metadata.engine_selected, "offline_fts5_lexicon");
}

#[tokio::test]
async fn test_transliteration_ime_does_not_mangle_english_adventure_into_indic() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = EngineRegistry::new_with_storage(storage);

    // English source input with transliteration IME enabled
    let intel_req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::KnIn,
        domain_glossary: None,
        enable_transliteration: true, // User left transliteration IME button active
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let intel_res = registry
        .translate_with_intelligence(intel_req)
        .await
        .expect("intelligence translation succeeds");

    // Transliterated input should NOT be "ಡವೆನಟುರೆ"
    assert_eq!(
        intel_res.metadata.transliterated_input, None,
        "English source input must never be transliterated into Indic characters"
    );
    assert_eq!(intel_res.translated_text, "ಸಾಹಸ");
}

#[tokio::test]
async fn test_mode_b_output_phonetic_reading_aid_iso15919() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage));

    // 1. Kannada Output Phonetic Aid
    let kn_req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::KnIn,
        domain_glossary: None,
        enable_transliteration: false,
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let kn_res = registry
        .translate_with_intelligence(kn_req)
        .await
        .expect("translation succeeds");
    assert_eq!(
        kn_res.metadata.target_phonetic,
        Some("sāhasa".to_string()),
        "Mode B reading aid for 'ಸಾಹಸ' must be 'sāhasa'"
    );

    // 2. Tamil Output Phonetic Aid
    let ta_req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::TaIn,
        domain_glossary: None,
        enable_transliteration: false,
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let ta_res = registry
        .translate_with_intelligence(ta_req)
        .await
        .expect("translation succeeds");
    assert_eq!(
        ta_res.metadata.target_phonetic,
        Some("cākacam".to_string()),
        "Mode B reading aid for 'சாகசம்' must be 'cākacam'"
    );
}

#[tokio::test]
async fn test_honest_telemetry_no_fake_chrf_on_live_translation() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = EngineRegistry::new_with_storage(storage);

    let req = IntelligenceTranslationRequest {
        text: "adventure".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::KnIn,
        domain_glossary: None,
        enable_transliteration: false,
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let res = registry
        .translate_with_intelligence(req)
        .await
        .expect("translation succeeds");

    // Live translation has no ground-truth reference, so chrF MUST be None (no fake 100.0%)
    assert_eq!(
        res.metadata.chrf_estimated, None,
        "Live translation must not display self-referential fake chrF score"
    );
    assert_eq!(res.metadata.engine_selected, "offline_fts5_lexicon");
    assert!(res.metadata.confidence_score.is_some());
    assert!(!res.metadata.fallback_trail.is_empty());
}

#[tokio::test]
async fn test_offline_compositional_gloss_on_uninstalled_nmt() {
    let storage = StorageService::open_in_memory().expect("in-memory sqlite storage");
    let registry = EngineRegistry::new_with_storage(storage);

    // A multi-word sentence without an installed local NMT model
    let req = IntelligenceTranslationRequest {
        text: "adventure courage knowledge".into(),
        source_lang: Some(LanguageCode::EnUs),
        target_lang: LanguageCode::KnIn,
        domain_glossary: None,
        enable_transliteration: false,
        formality: FormalityMode::Neutral,
        allow_cloud: false,
    };

    let res = registry
        .translate_with_intelligence(req)
        .await
        .expect("compositional gloss succeeds");

    assert!(
        res.metadata.is_compositional_gloss,
        "Must flag as compositional gloss when local neural model is absent"
    );
    assert_eq!(
        res.metadata.engine_selected, "offline_compositional_gloss",
        "Must be attributed to offline compositional gloss"
    );
    assert_eq!(
        res.metadata.confidence_score,
        Some(0.65),
        "Compositional gloss confidence score is 0.65"
    );
    // Output should contain glossed words for each token
    assert!(
        res.translated_text.contains("ಸಾಹಸ"),
        "Gloss should include 'ಸಾಹಸ' for 'adventure'"
    );
}
