use lingua_core::{detect_language, LanguageCode, LanguagePair, LinguaError};
use lingua_engine::{EngineRegistry, TranslationRequest};
use lingua_storage::StorageService;
use std::sync::Arc;

#[tokio::test]
async fn test_bidirectional_all_pairs_lexicon() {
    let storage = StorageService::open_in_memory().unwrap();
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));

    // 1. Kannada <-> English
    let kn_en = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: kn_en,
            text: "ನಮಸ್ಕಾರ".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "Hello");
    assert!(res.is_offline);

    let en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: en_kn,
            text: "Water".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "ನೀರು");

    // 2. Tamil <-> English
    let ta_en = LanguagePair::new(LanguageCode::TaIn, LanguageCode::EnUs).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: ta_en,
            text: "வணக்கம்".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "Hello");

    let en_ta = LanguagePair::new(LanguageCode::EnUs, LanguageCode::TaIn).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: en_ta,
            text: "Peace".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "அமைதி");

    // 3. Kannada <-> Tamil
    let kn_ta = LanguagePair::new(LanguageCode::KnIn, LanguageCode::TaIn).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: kn_ta,
            text: "ಸ್ನೇಹ".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "அன்பு");

    let ta_kn = LanguagePair::new(LanguageCode::TaIn, LanguageCode::KnIn).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: ta_kn,
            text: "தண்ணீர்".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "ನೀರು");
}

#[tokio::test]
async fn test_unicode_normalization_and_whitespace_resilience() {
    let storage = StorageService::open_in_memory().unwrap();
    let registry = EngineRegistry::new_with_storage(storage);

    let pair = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();

    // Whitespace padding and tab
    let res = registry
        .translate(TranslationRequest {
            pair,
            text: "   \t ನಮಸ್ಕಾರ \n  ".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "Hello");

    // Case insensitivity for Latin input
    let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let res = registry
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "hELlo".into(),
            allow_cloud: false,
        })
        .await
        .unwrap();
    assert_eq!(res.translated_text, "ನಮಸ್ಕಾರ");
}

#[tokio::test]
async fn test_linguistic_edge_cases_and_error_handling() {
    let storage = StorageService::open_in_memory().unwrap();
    let registry = EngineRegistry::new_with_storage(storage);

    let pair = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();

    // 1. Empty string validation
    let err_empty = registry
        .translate(TranslationRequest {
            pair,
            text: "".into(),
            allow_cloud: false,
        })
        .await
        .unwrap_err();
    match err_empty {
        LinguaError::Validation(msg) => assert!(msg.contains("empty text")),
        other => panic!("Expected Validation error, got {:?}", other),
    }

    // 2. Whitespace-only validation
    let err_spaces = registry
        .translate(TranslationRequest {
            pair,
            text: "    \t   ".into(),
            allow_cloud: false,
        })
        .await
        .unwrap_err();
    match err_spaces {
        LinguaError::Validation(msg) => assert!(msg.contains("empty text")),
        other => panic!("Expected Validation error, got {:?}", other),
    }

    // 3. Offline model missing for unknown term
    let err_missing = registry
        .translate(TranslationRequest {
            pair,
            text: "ಅಪರಿಚಿತ ವಾಕ್ಯ".into(),
            allow_cloud: false,
        })
        .await
        .unwrap_err();
    match err_missing {
        LinguaError::OfflineModelMissing(msg) => {
            assert!(msg.contains("offline dictionary"));
        }
        other => panic!("Expected OfflineModelMissing, got {:?}", other),
    }

    // 4. Identical language pair rejection
    let identical = LanguagePair::new(LanguageCode::KnIn, LanguageCode::KnIn);
    assert!(identical.is_err());
    match identical {
        Err(LinguaError::IdenticalLanguagePair(src, tgt)) => {
            assert_eq!(src, "kn-IN");
            assert_eq!(tgt, "kn-IN");
        }
        other => panic!("Expected IdenticalLanguagePair, got {:?}", other),
    }
}

#[test]
fn test_script_detection_accuracy() {
    assert_eq!(
        detect_language("ನಮಸ್ಕಾರ, ಎಲ್ಲರಿಗೂ ಸುಸ್ವಾಗತ."),
        Some(LanguageCode::KnIn)
    );
    assert_eq!(
        detect_language("வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?"),
        Some(LanguageCode::TaIn)
    );
    assert_eq!(
        detect_language("Welcome to LinguaFlow platform."),
        Some(LanguageCode::EnUs)
    );
    assert_eq!(detect_language("12345 !@#$%"), None);
}

#[test]
fn test_fts5_cross_language_dictionary_search() {
    let storage = StorageService::open_in_memory().unwrap();

    // 1. Kannada prefix search
    let kn_results = storage
        .search_dictionary("ನಮ", Some(LanguageCode::KnIn), 10)
        .unwrap();
    assert!(!kn_results.is_empty());
    assert_eq!(kn_results[0].word, "ನಮಸ್ಕಾರ");
    assert!(kn_results[0].pronunciation_ipa.is_some());
    assert!(!kn_results[0].examples.is_empty());

    // 2. Tamil prefix search
    let ta_results = storage
        .search_dictionary("வண", Some(LanguageCode::TaIn), 10)
        .unwrap();
    assert!(!ta_results.is_empty());
    assert_eq!(ta_results[0].word, "வணக்கம்");

    // 3. Full-text search across definitions: "water" should match English entry and foreign entries mentioning water
    let fts_water = storage.search_dictionary("water", None, 10).unwrap();
    assert!(fts_water.len() >= 3);
    assert!(fts_water.iter().any(|e| e.language == LanguageCode::EnUs));
    assert!(fts_water.iter().any(|e| e.language == LanguageCode::KnIn));
    assert!(fts_water.iter().any(|e| e.language == LanguageCode::TaIn));

    // 4. Exact word lookup
    let exact = storage
        .get_entry_by_exact_word("ಶಾಂತಿ", LanguageCode::KnIn)
        .unwrap()
        .unwrap();
    assert_eq!(exact.word, "ಶಾಂತಿ");
    assert_eq!(exact.part_of_speech, "noun");
    assert!(exact.synonyms.contains(&"ನೆಮ್ಮದಿ".to_string()));
}
