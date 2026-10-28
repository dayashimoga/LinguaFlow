//! Integration test suite for Stage 5: Translation Intelligence and Optimization.
//! Verifies circuit breakers, observable fallback trails, transliteration,
//! entity protection, domain glossaries, evaluation metrics (chrF, WER, CER),
//! and bounded LRU caching with resource profiles.

use chrono::Utc;
use lingua_core::{
    CircuitState, DomainGlossaryEntry, EvaluationSample, FormalityMode,
    IntelligenceTranslationRequest, LanguageCode, ResourceProfile,
};
use lingua_engine::{
    BoundedLruCache, CircuitBreaker, ContextNlpProcessor, EngineRegistry, QualityEvaluator,
    TransliterationEngine,
};
use lingua_storage::StorageService;

#[tokio::test]
async fn test_circuit_breaker_tripping_and_fallback_diversion() {
    // 1. Initialize Circuit Breaker with threshold 3 and 15s reset timeout
    let cb = CircuitBreaker::new("cloud-deep-translator", 3, 15);
    assert_eq!(cb.status().state, CircuitState::Closed);
    assert!(cb.can_execute());

    // 2. Simulate 2 consecutive failures
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.status().state, CircuitState::Closed);
    assert_eq!(cb.status().failure_count, 2);
    assert!(cb.can_execute());

    // 3. 3rd failure trips the breaker to Open
    cb.record_failure();
    assert_eq!(cb.status().state, CircuitState::Open);
    assert_eq!(cb.status().failure_count, 3);
    assert!(!cb.can_execute());

    // 4. In Open state, requests fail immediately without blocking or waiting
    let start = std::time::Instant::now();
    let can_run = cb.can_execute();
    let elapsed = start.elapsed();
    assert!(!can_run);
    assert!(
        elapsed.as_millis() < 5,
        "Open breaker must fail fast in < 5ms"
    );

    // 5. Explicit reset brings it back to Closed
    cb.reset();
    assert_eq!(cb.status().state, CircuitState::Closed);
    assert_eq!(cb.status().failure_count, 0);
    assert!(cb.can_execute());
}

#[tokio::test]
async fn test_transliteration_kannada_and_tamil() {
    let engine = TransliterationEngine::new(500);

    // Test Kannada Romanized mappings
    let kn_res1 = engine
        .transliterate("namaskara", LanguageCode::KnIn)
        .unwrap();
    assert_eq!(kn_res1.transliterated, "ನಮಸ್ಕಾರ");

    let kn_res2 = engine
        .transliterate("hegiddira", LanguageCode::KnIn)
        .unwrap();
    assert_eq!(kn_res2.transliterated, "ಹೇಗಿದ್ದೀರಾ");

    let kn_res3 = engine
        .transliterate("dhanyavada", LanguageCode::KnIn)
        .unwrap();
    assert_eq!(kn_res3.transliterated, "ಧನ್ಯವಾದ");

    // Test Tamil Romanized mappings
    let ta_res1 = engine
        .transliterate("vanakkam", LanguageCode::TaIn)
        .unwrap();
    assert_eq!(ta_res1.transliterated, "வணக்கம்");

    let ta_res2 = engine.transliterate("nandri", LanguageCode::TaIn).unwrap();
    assert_eq!(ta_res2.transliterated, "நன்றி");

    let ta_res3 = engine.transliterate("chennai", LanguageCode::TaIn).unwrap();
    assert_eq!(ta_res3.transliterated, "சென்னை");

    // Test syllabic transliteration fallback for unseen words
    let syllabic = engine
        .transliterate("bengaluru", LanguageCode::KnIn)
        .unwrap();
    assert!(!syllabic.transliterated.is_empty());
    // Syllabic output must produce Kannada Unicode block (U+0C80 to U+0CFF)
    assert!(syllabic
        .transliterated
        .chars()
        .any(|c| ('\u{0C80}'..='\u{0CFF}').contains(&c)));
}

#[tokio::test]
async fn test_context_nlp_formality_and_glossaries() {
    // 1. Kannada formality transformations
    let kn_informal = "ಬಾ";
    let kn_formal = ContextNlpProcessor::adapt_formality(
        kn_informal,
        LanguageCode::KnIn,
        FormalityMode::Formal,
    );
    assert_eq!(kn_formal, "ಬನ್ನಿ");

    let kn_colloquial =
        ContextNlpProcessor::adapt_formality("ಬನ್ನಿ", LanguageCode::KnIn, FormalityMode::Colloquial);
    assert_eq!(kn_colloquial, "ಬಾ");

    // 2. Tamil formality transformations
    let ta_informal = "வா";
    let ta_formal = ContextNlpProcessor::adapt_formality(
        ta_informal,
        LanguageCode::TaIn,
        FormalityMode::Formal,
    );
    assert_eq!(ta_formal, "வாருங்கள்");

    let ta_colloquial = ContextNlpProcessor::adapt_formality(
        "வாருங்கள்",
        LanguageCode::TaIn,
        FormalityMode::Colloquial,
    );
    assert_eq!(ta_colloquial, "வா");

    // 3. Domain glossary replacement
    let glossary = vec![DomainGlossaryEntry {
        id: "entry-1".into(),
        domain: "tech".into(),
        source_lang: LanguageCode::EnUs,
        target_lang: LanguageCode::KnIn,
        source_term: "Artificial Intelligence".into(),
        target_term: "ಕೃತಕ ಬುದ್ಧಿಮತ್ತೆ".into(),
        is_exact: true,
        created_at: Utc::now(),
    }];

    let text = "We study Artificial Intelligence and its impacts.";
    let (replaced, terms) = ContextNlpProcessor::apply_domain_glossary(
        text,
        LanguageCode::EnUs,
        LanguageCode::KnIn,
        &glossary,
    );
    assert!(replaced.contains("ಕೃತಕ ಬುದ್ಧಿಮತ್ತೆ"));
    assert_eq!(terms, vec!["Artificial Intelligence"]);
}

#[tokio::test]
async fn test_proper_noun_and_entity_protection() {
    let input = "Meet with {Siddaramaiah} and {Sundar Pichai} at Bengaluru.";
    let (masked, entities) = ContextNlpProcessor::protect_named_entities(input);

    assert_eq!(entities.len(), 3);
    assert_eq!(entities.get("__LF_PROTECT_0__").unwrap(), "{Siddaramaiah}");
    assert_eq!(entities.get("__LF_PROTECT_1__").unwrap(), "{Sundar Pichai}");
    assert_eq!(entities.get("__LF_PROTECT_2__").unwrap(), "Bengaluru");

    assert!(masked.contains("__LF_PROTECT_0__"));
    assert!(masked.contains("__LF_PROTECT_1__"));
    assert!(masked.contains("__LF_PROTECT_2__"));

    // Simulate translation preserving masks and test roundtrip
    let translated = "ಭೇಟಿಯಾಗಿ __LF_PROTECT_0__ ಮತ್ತು __LF_PROTECT_1__ ಅವರೊಂದಿಗೆ __LF_PROTECT_2__ ನಲ್ಲಿ.";
    let restored = ContextNlpProcessor::restore_named_entities(translated, &entities);

    assert_eq!(
        restored,
        "ಭೇಟಿಯಾಗಿ {Siddaramaiah} ಮತ್ತು {Sundar Pichai} ಅವರೊಂದಿಗೆ Bengaluru ನಲ್ಲಿ."
    );
}

#[tokio::test]
async fn test_evaluation_metrics_chrf_wer_cer() {
    // 1. Identical string chrF = 100.0%
    let chrf_exact = QualityEvaluator::compute_chrf("ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೆ", "ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೆ");
    assert!((chrf_exact - 100.0).abs() < 1e-4);

    // 2. High overlap translation
    let chrf_close = QualityEvaluator::compute_chrf("ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೇ", "ನಮಸ್ಕಾರ ಸ್ನೇಹಿತರೆ");
    assert!(
        chrf_close > 80.0,
        "Expected chrF > 80.0, got {}",
        chrf_close
    );

    // 3. WER calculation
    let wer = QualityEvaluator::compute_wer("hello world", "hello beautiful world");
    // 1 insertion / 3 reference words = 33.33%
    assert!((wer - 33.33).abs() < 1.0);

    // 4. CER calculation
    let cer = QualityEvaluator::compute_cer("abc", "abd");
    // 1 substitution / 3 reference chars = 33.33%
    assert!((cer - 33.33).abs() < 1.0);

    // 5. Curated benchmark dataset batch evaluation
    let samples = QualityEvaluator::curated_evaluation_dataset();
    assert_eq!(samples.len(), 8);

    let pairs: Vec<(EvaluationSample, String)> = samples
        .iter()
        .map(|s| (s.clone(), s.reference_translation.clone())) // Perfect hypothesis
        .collect();

    let report = QualityEvaluator::evaluate_batch(&pairs);
    assert_eq!(report.total_samples, 8);
    assert!((report.mean_chrf - 100.0).abs() < 1e-4);
    assert!((report.mean_wer - 0.0).abs() < 1e-4);
    assert!((report.mean_cer - 0.0).abs() < 1e-4);
}

#[tokio::test]
async fn test_bounded_lru_cache_and_resource_profiles() {
    let cache: BoundedLruCache<String, String> = BoundedLruCache::new(2);

    cache.insert("key1".into(), "val1".into());
    cache.insert("key2".into(), "val2".into());
    assert_eq!(cache.len(), 2);

    // Evicts key1 when key3 is inserted
    cache.insert("key3".into(), "val3".into());
    assert_eq!(cache.len(), 2);
    assert!(cache.get(&"key1".to_string()).is_none());
    assert_eq!(cache.get(&"key2".to_string()), Some("val2".into()));
    assert_eq!(cache.get(&"key3".to_string()), Some("val3".into()));

    let stats = cache.stats();
    assert_eq!(stats.evictions, 1);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);

    // Resource profile capacity adjustment
    cache.set_capacity(ResourceProfile::PowerSaver.max_cache_entries());
    assert_eq!(cache.capacity(), 500);

    cache.set_capacity(ResourceProfile::Balanced.max_cache_entries());
    assert_eq!(cache.capacity(), 2000);

    cache.set_capacity(ResourceProfile::HighPerformance.max_cache_entries());
    assert_eq!(cache.capacity(), 8000);
}

#[tokio::test]
async fn test_full_intelligence_translation_pipeline() {
    let storage = StorageService::open_in_memory().unwrap();
    let registry = EngineRegistry::new_with_storage(storage.clone());

    // 1. Insert domain glossary term
    let glossary_entry = DomainGlossaryEntry {
        id: "g1".into(),
        domain: "general".into(),
        source_lang: LanguageCode::KnIn,
        target_lang: LanguageCode::EnUs,
        source_term: "ನಮಸ್ಕಾರ".into(),
        target_term: "Greetings".into(),
        is_exact: true,
        created_at: Utc::now(),
    };
    storage.insert_glossary_entry(&glossary_entry).unwrap();

    // 2. Intelligent translation with Romanized Kanglish input + formality mode
    let req = IntelligenceTranslationRequest {
        text: "namaskara".into(),
        source_lang: None,
        target_lang: LanguageCode::EnUs,
        formality: FormalityMode::Formal,
        enable_transliteration: true,
        domain_glossary: Some("general".into()),
    };

    let res = registry.translate_with_intelligence(req).await.unwrap();
    assert_eq!(res.source_text, "namaskara");
    assert!(!res.translated_text.is_empty());
    assert_eq!(res.metadata.formality, FormalityMode::Formal);
    assert!(res.metadata.transliterated_input.is_some());
    assert_eq!(res.metadata.transliterated_input.as_deref(), Some("ನಮಸ್ಕಾರ"));
    assert!(res.metadata.chrf_estimated.is_some());
    assert!(res.metadata.chrf_estimated.unwrap() >= 0.0);

    // 3. Second call should hit the LRU cache
    let req2 = IntelligenceTranslationRequest {
        text: "ನಮಸ್ಕಾರ".into(),
        source_lang: Some(LanguageCode::KnIn),
        target_lang: LanguageCode::EnUs,
        formality: FormalityMode::Neutral,
        enable_transliteration: false,
        domain_glossary: None,
    };
    let res2 = registry.translate_with_intelligence(req2).await.unwrap();
    assert_eq!(res2.metadata.engine_selected, "in_memory_lru_cache");
    assert!(res2
        .metadata
        .fallback_trail
        .iter()
        .any(|s| s.engine_id == "in_memory_lru_cache" && s.success));
}
