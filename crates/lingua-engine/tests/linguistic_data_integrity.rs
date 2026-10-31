use lingua_core::{LanguageCode, LanguagePair};
use lingua_engine::{LexiconTranslationEngine, TranslationEngine, TranslationRequest};
use lingua_storage::StorageService;

#[tokio::test]
async fn test_adventure_lexicon_entry_integrity() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");

    // Search for "adventure" in English
    let res = storage
        .search_rich_dictionary("adventure", Some(LanguageCode::EnUs), None, 10)
        .expect("search failed");

    assert!(
        !res.rich_entries.is_empty(),
        "Expected 'adventure' to be in the Stage 9 rich lexicon"
    );
    let entry = &res.rich_entries[0];
    assert_eq!(entry.headword, "adventure");
    assert_eq!(entry.part_of_speech, "noun");
    assert_eq!(entry.ipa_pronunciation.as_deref(), Some("/ədˈvɛntʃər/"));

    // Check Kannada target equivalents
    let kn_equivs = entry
        .target_equivalents
        .get("kn-IN")
        .expect("Missing kn-IN equivalents");
    assert!(
        kn_equivs.contains(&"ಸಾಹಸ".to_string()),
        "Target equivalents must contain 'ಸಾಹಸ'"
    );

    // Check Tamil target equivalents
    let ta_equivs = entry
        .target_equivalents
        .get("ta-IN")
        .expect("Missing ta-IN equivalents");
    assert!(
        ta_equivs.contains(&"சாகசம்".to_string()),
        "Target equivalents must contain 'சாகசம்'"
    );

    // Check senses and examples
    assert!(entry.senses.len() >= 2);
    assert!(!entry.bilingual_examples.is_empty());
}

#[tokio::test]
async fn test_kannada_sahasa_and_agglutinative_morphology() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");

    // 1. Direct root search
    let res_root = storage
        .search_rich_dictionary("ಸಾಹಸ", Some(LanguageCode::KnIn), None, 10)
        .expect("search failed");
    assert!(!res_root.rich_entries.is_empty());
    assert_eq!(res_root.rich_entries[0].headword, "ಸಾಹಸ");
    assert_eq!(
        res_root.rich_entries[0]
            .native_phonetic_respelling
            .as_deref(),
        Some("sāhasa")
    );

    // 2. Inflected accusative plural: ಸಾಹಸಗಳನ್ನು -> should resolve stem to ಸಾಹಸ
    let res_acc = storage
        .search_rich_dictionary("ಸಾಹಸಗಳನ್ನು", Some(LanguageCode::KnIn), None, 10)
        .expect("search failed");
    assert!(
        !res_acc.rich_entries.is_empty(),
        "Agglutinative search must resolve ಸಾಹಸಗಳನ್ನು"
    );
    assert_eq!(res_acc.resolved_stem, Some("ಸಾಹಸ".into()));
    assert_eq!(res_acc.rich_entries[0].headword, "ಸಾಹಸ");

    // 3. Inflected instrumental plural: ಸಾಹಸಗಳಿಂದ -> should resolve stem to ಸಾಹಸ
    let res_inst = storage
        .search_rich_dictionary("ಸಾಹಸಗಳಿಂದ", Some(LanguageCode::KnIn), None, 10)
        .expect("search failed");
    assert!(!res_inst.rich_entries.is_empty());
    assert_eq!(res_inst.resolved_stem, Some("ಸಾಹಸ".into()));

    // 4. Inflected locative: ಸಾಹಸದಲ್ಲಿ -> should resolve stem to ಸಾಹಸ
    let res_loc = storage
        .search_rich_dictionary("ಸಾಹಸದಲ್ಲಿ", Some(LanguageCode::KnIn), None, 10)
        .expect("search failed");
    assert!(!res_loc.rich_entries.is_empty());
    assert_eq!(res_loc.resolved_stem, Some("ಸಾಹಸ".into()));
}

#[tokio::test]
async fn test_tamil_cakacam_and_morphological_assimilation() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");

    // 1. Direct root search
    let res_root = storage
        .search_rich_dictionary("சாகசம்", Some(LanguageCode::TaIn), None, 10)
        .expect("search failed");
    assert!(!res_root.rich_entries.is_empty());
    assert_eq!(res_root.rich_entries[0].headword, "சாகசம்");
    assert_eq!(
        res_root.rich_entries[0]
            .native_phonetic_respelling
            .as_deref(),
        Some("cākacam")
    );

    // 2. Inflected neuter plural accusative: சாகசங்களை -> should resolve stem to சாகசம்
    let res_acc = storage
        .search_rich_dictionary("சாகசங்களை", Some(LanguageCode::TaIn), None, 10)
        .expect("search failed");
    assert!(
        !res_acc.rich_entries.is_empty(),
        "Agglutinative search must resolve சாகசங்களை"
    );
    assert_eq!(res_acc.resolved_stem, Some("சாகசம்".into()));
    assert_eq!(res_acc.rich_entries[0].headword, "சாகசம்");

    // 3. Inflected neuter singular accusative: சாகசத்தை -> should resolve stem to சாகசம்
    let res_sing = storage
        .search_rich_dictionary("சாகசத்தை", Some(LanguageCode::TaIn), None, 10)
        .expect("search failed");
    assert!(!res_sing.rich_entries.is_empty());
    assert_eq!(res_sing.resolved_stem, Some("சாகசம்".into()));

    // 4. Inflected dative: சாகசத்திற்கு -> should resolve stem to சாகசம்
    let res_dat = storage
        .search_rich_dictionary("சாகசத்திற்கு", Some(LanguageCode::TaIn), None, 10)
        .expect("search failed");
    assert!(!res_dat.rich_entries.is_empty());
    assert_eq!(res_dat.resolved_stem, Some("சாகசம்".into()));
}

#[tokio::test]
async fn test_english_lemmatization_and_derivation() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");

    // "adventures" -> should stem to "adventure"
    let res_pl = storage
        .search_rich_dictionary("adventures", Some(LanguageCode::EnUs), None, 10)
        .expect("search failed");
    assert!(!res_pl.rich_entries.is_empty());
    assert_eq!(res_pl.rich_entries[0].headword, "adventure");

    // "adventuring" -> should stem to "adventure"
    let res_prog = storage
        .search_rich_dictionary("adventuring", Some(LanguageCode::EnUs), None, 10)
        .expect("search failed");
    assert!(!res_prog.rich_entries.is_empty());
    assert_eq!(res_prog.rich_entries[0].headword, "adventure");
}

#[tokio::test]
async fn test_fuzzy_did_you_mean_suggestions() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");

    // Misspelled "advnture" (missing 'e')
    let res = storage
        .search_rich_dictionary("advnture", Some(LanguageCode::EnUs), None, 10)
        .expect("search failed");
    assert!(
        res.rich_entries.is_empty(),
        "Direct search for typo should return 0 entries"
    );
    assert!(
        !res.did_you_mean.is_empty(),
        "Expected did_you_mean suggestions for 'advnture'"
    );
    assert!(
        res.did_you_mean.contains(&"adventure".to_string()),
        "Did you mean must suggest 'adventure'"
    );
}

#[tokio::test]
async fn test_lexicon_translation_engine_adventure_resolution() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let engine = LexiconTranslationEngine::new(storage);

    // 1. English -> Kannada: "adventure" -> "ಸಾಹಸ" (and NEVER "ದೇವೇಗೌಡರು")
    let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let resp_kn = engine
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "adventure".into(),
            allow_cloud: false,
        })
        .await
        .expect("Translation failed");

    assert_eq!(resp_kn.translated_text, "ಸಾಹಸ");
    assert_ne!(resp_kn.translated_text, "ದೇವೇಗೌಡರು");

    // 2. English -> Tamil: "adventure" -> "சாகசம்"
    let pair_en_ta = LanguagePair::new(LanguageCode::EnUs, LanguageCode::TaIn).unwrap();
    let resp_ta = engine
        .translate(TranslationRequest {
            pair: pair_en_ta,
            text: "adventure".into(),
            allow_cloud: false,
        })
        .await
        .expect("Translation failed");

    assert_eq!(resp_ta.translated_text, "சாகசம்");

    // 3. Kannada -> English: "ಸಾಹಸ" -> "Adventure"
    let pair_kn_en = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
    let resp_kn_en = engine
        .translate(TranslationRequest {
            pair: pair_kn_en,
            text: "ಸಾಹಸ".into(),
            allow_cloud: false,
        })
        .await
        .expect("Translation failed");

    assert_eq!(resp_kn_en.translated_text, "Adventure");

    // 4. Tamil -> English: "சாகசம்" -> "Adventure"
    let pair_ta_en = LanguagePair::new(LanguageCode::TaIn, LanguageCode::EnUs).unwrap();
    let resp_ta_en = engine
        .translate(TranslationRequest {
            pair: pair_ta_en,
            text: "சாகசம்".into(),
            allow_cloud: false,
        })
        .await
        .expect("Translation failed");

    assert_eq!(resp_ta_en.translated_text, "Adventure");
}

#[tokio::test]
async fn test_medical_and_logistics_lexicon_integrity() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let engine = LexiconTranslationEngine::new(storage);

    // Doctor (Medical)
    let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
    let resp_doc_kn = engine
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "doctor".into(),
            allow_cloud: false,
        })
        .await
        .expect("doctor translation failed");
    assert_eq!(resp_doc_kn.translated_text, "ವೈದ್ಯ");

    let pair_en_ta = LanguagePair::new(LanguageCode::EnUs, LanguageCode::TaIn).unwrap();
    let resp_doc_ta = engine
        .translate(TranslationRequest {
            pair: pair_en_ta,
            text: "doctor".into(),
            allow_cloud: false,
        })
        .await
        .expect("doctor translation failed");
    assert_eq!(resp_doc_ta.translated_text, "மருத்துவர்");

    // Freight (Logistics)
    let resp_fr_kn = engine
        .translate(TranslationRequest {
            pair: pair_en_kn,
            text: "freight".into(),
            allow_cloud: false,
        })
        .await
        .expect("freight translation failed");
    assert_eq!(resp_fr_kn.translated_text, "ಸರಕು ಸಾಗಣೆ");
}

#[test]
fn test_migration_011_schema_version() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let health = storage.check_health().expect("health check failed");
    assert_eq!(
        health.schema_version, 11,
        "Schema version must be 11 after Migration 011 (show_developer_hud)"
    );
}
