use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use async_trait::async_trait;
use lingua_core::{normalize_text_nfc, CapabilityState, LanguageCode, LanguagePair, LinguaError};
use lingua_storage::StorageService;
use std::time::Instant;

pub struct LexiconTranslationEngine {
    storage: StorageService,
}

impl LexiconTranslationEngine {
    pub fn new(storage: StorageService) -> Self {
        Self { storage }
    }

    /// Map a concept key between Kannada, Tamil, and English.
    fn resolve_concept_translation(
        &self,
        word: &str,
        source: LanguageCode,
        target: LanguageCode,
    ) -> Option<String> {
        let norm = normalize_text_nfc(word.trim()).to_lowercase();

        // Authentic bilingual lexicon mappings for core vocabulary
        let mappings: &[(&[&str], LanguageCode, &str, LanguageCode)] = &[
            // (source words, src_lang, target_text, tgt_lang, concept_id)
            (
                &["ನಮಸ್ಕಾರ", "namaskara", "namaskāra"],
                LanguageCode::KnIn,
                "Hello",
                LanguageCode::EnUs,
            ),
            (
                &["ನಮಸ್ಕಾರ", "namaskara", "namaskāra"],
                LanguageCode::KnIn,
                "வணக்கம்",
                LanguageCode::TaIn,
            ),
            (
                &["வணக்கம்", "vanakkam", "vaṇakkam"],
                LanguageCode::TaIn,
                "Hello",
                LanguageCode::EnUs,
            ),
            (
                &["வணக்கம்", "vanakkam", "vaṇakkam"],
                LanguageCode::TaIn,
                "ನಮಸ್ಕಾರ",
                LanguageCode::KnIn,
            ),
            (
                &["hello", "hi", "greetings"],
                LanguageCode::EnUs,
                "ನಮಸ್ಕಾರ",
                LanguageCode::KnIn,
            ),
            (
                &["hello", "hi", "greetings"],
                LanguageCode::EnUs,
                "வணக்கம்",
                LanguageCode::TaIn,
            ),
            (
                &["ಶಾಂತಿ", "shanti", "śānti"],
                LanguageCode::KnIn,
                "Peace",
                LanguageCode::EnUs,
            ),
            (
                &["ಶಾಂತಿ", "shanti", "śānti"],
                LanguageCode::KnIn,
                "அமைதி",
                LanguageCode::TaIn,
            ),
            (
                &["அமைதி", "amaidhi", "amaidi"],
                LanguageCode::TaIn,
                "Peace",
                LanguageCode::EnUs,
            ),
            (
                &["அமைதி", "amaidhi", "amaidi"],
                LanguageCode::TaIn,
                "ಶಾಂತಿ",
                LanguageCode::KnIn,
            ),
            (
                &["peace", "calm", "tranquility"],
                LanguageCode::EnUs,
                "ಶಾಂತಿ",
                LanguageCode::KnIn,
            ),
            (
                &["peace", "calm", "tranquility"],
                LanguageCode::EnUs,
                "அமைதி",
                LanguageCode::TaIn,
            ),
            (
                &["ಸ್ನೇಹ", "sneha", "snēha"],
                LanguageCode::KnIn,
                "Friendship",
                LanguageCode::EnUs,
            ),
            (
                &["ಸ್ನೇಹ", "sneha", "snēha"],
                LanguageCode::KnIn,
                "அன்பு",
                LanguageCode::TaIn,
            ),
            (
                &["அன்பு", "anbu"],
                LanguageCode::TaIn,
                "Love / Friendship",
                LanguageCode::EnUs,
            ),
            (
                &["அன்பு", "anbu"],
                LanguageCode::TaIn,
                "ಸ್ನೇಹ",
                LanguageCode::KnIn,
            ),
            (
                &["friendship", "friend"],
                LanguageCode::EnUs,
                "ಸ್ನೇಹ",
                LanguageCode::KnIn,
            ),
            (
                &["friendship", "friend"],
                LanguageCode::EnUs,
                "அன்பு",
                LanguageCode::TaIn,
            ),
            (
                &["ಭಾಷೆ", "bhashe", "bhāṣe"],
                LanguageCode::KnIn,
                "Language",
                LanguageCode::EnUs,
            ),
            (
                &["ಭಾಷೆ", "bhashe", "bhāṣe"],
                LanguageCode::KnIn,
                "மொழி",
                LanguageCode::TaIn,
            ),
            (
                &["மொழி", "mozhi", "moḻi"],
                LanguageCode::TaIn,
                "Language",
                LanguageCode::EnUs,
            ),
            (
                &["மொழி", "mozhi", "moḻi"],
                LanguageCode::TaIn,
                "ಭಾಷೆ",
                LanguageCode::KnIn,
            ),
            (
                &["language", "tongue"],
                LanguageCode::EnUs,
                "ಭಾಷೆ",
                LanguageCode::KnIn,
            ),
            (
                &["language", "tongue"],
                LanguageCode::EnUs,
                "மொழி",
                LanguageCode::TaIn,
            ),
            (
                &["ವಿಶ್ವ", "vishwa", "viśva", "ಜಗತ್ತು"],
                LanguageCode::KnIn,
                "World",
                LanguageCode::EnUs,
            ),
            (
                &["ವಿಶ್ವ", "vishwa", "viśva", "ಜಗತ್ತು"],
                LanguageCode::KnIn,
                "உலகம்",
                LanguageCode::TaIn,
            ),
            (
                &["உலகம்", "ulagam", "வையகம்"],
                LanguageCode::TaIn,
                "World",
                LanguageCode::EnUs,
            ),
            (
                &["உலகம்", "ulagam", "வையகம்"],
                LanguageCode::TaIn,
                "ವಿಶ್ವ",
                LanguageCode::KnIn,
            ),
            (
                &["world", "earth", "globe"],
                LanguageCode::EnUs,
                "ವಿಶ್ವ",
                LanguageCode::KnIn,
            ),
            (
                &["world", "earth", "globe"],
                LanguageCode::EnUs,
                "உலகம்",
                LanguageCode::TaIn,
            ),
            (
                &["ನೀರು", "neeru", "nīru", "ಜಲ"],
                LanguageCode::KnIn,
                "Water",
                LanguageCode::EnUs,
            ),
            (
                &["ನೀರು", "neeru", "nīru", "ಜಲ"],
                LanguageCode::KnIn,
                "தண்ணீர்",
                LanguageCode::TaIn,
            ),
            (
                &["தண்ணீர்", "thanneer", "taṇṇīr", "நீர்"],
                LanguageCode::TaIn,
                "Water",
                LanguageCode::EnUs,
            ),
            (
                &["தண்ணீர்", "thanneer", "taṇṇīr", "நீர்"],
                LanguageCode::TaIn,
                "ನೀರು",
                LanguageCode::KnIn,
            ),
            (
                &["water", "aqua"],
                LanguageCode::EnUs,
                "ನೀರು",
                LanguageCode::KnIn,
            ),
            (
                &["water", "aqua"],
                LanguageCode::EnUs,
                "தண்ணீர்",
                LanguageCode::TaIn,
            ),
        ];

        for (src_tokens, src_lang, tgt_val, tgt_lang) in mappings {
            if *src_lang == source && *tgt_lang == target {
                for token in *src_tokens {
                    if normalize_text_nfc(token).to_lowercase() == norm {
                        return Some((*tgt_val).to_string());
                    }
                }
            }
        }

        // Check if database exact word or definition matches
        if let Ok(Some(entry)) = self.storage.get_entry_by_exact_word(word, source) {
            if target == LanguageCode::EnUs {
                // Return primary definition or alternate meaning
                return Some(entry.definition);
            }
        }

        None
    }
}

#[async_trait]
impl TranslationEngine for LexiconTranslationEngine {
    fn id(&self) -> &'static str {
        "verified-lexicon-engine"
    }

    fn name(&self) -> &'static str {
        "Verified Bilingual Lexicon Engine"
    }

    fn supports_pair(&self, pair: &LanguagePair) -> bool {
        let s = pair.source;
        let t = pair.target;
        matches!(
            (s, t),
            (LanguageCode::KnIn, LanguageCode::EnUs)
                | (LanguageCode::EnUs, LanguageCode::KnIn)
                | (LanguageCode::TaIn, LanguageCode::EnUs)
                | (LanguageCode::EnUs, LanguageCode::TaIn)
                | (LanguageCode::KnIn, LanguageCode::TaIn)
                | (LanguageCode::TaIn, LanguageCode::KnIn)
        )
    }

    fn capability_state(&self, pair: &LanguagePair) -> CapabilityState {
        if self.supports_pair(pair) {
            CapabilityState::Ready
        } else {
            CapabilityState::NotInstalled
        }
    }

    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, LinguaError> {
        let start = Instant::now();
        let cleaned = normalize_text_nfc(request.text.trim());

        if cleaned.is_empty() {
            return Err(LinguaError::Validation(
                "Cannot translate empty text".into(),
            ));
        }

        if let Some(translated) =
            self.resolve_concept_translation(&cleaned, request.pair.source, request.pair.target)
        {
            let latency_ms = start.elapsed().as_millis() as u64;
            return Ok(TranslationResponse {
                pair: request.pair,
                original_text: request.text,
                translated_text: translated,
                engine_id: self.id().into(),
                is_offline: true,
                latency_ms: latency_ms.max(1),
            });
        }

        Err(LinguaError::OfflineModelMissing(format!(
            "Term '{}' is not found in the verified offline dictionary for pair {}. Use online provider or download additional offline models in Stage 2.",
            cleaned, request.pair
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lexicon_bidirectional_translations() {
        let storage = StorageService::open_in_memory().unwrap();
        let engine = LexiconTranslationEngine::new(storage);

        // 1. Kannada -> English
        let pair_kn_en = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
        let resp = engine
            .translate(TranslationRequest {
                pair: pair_kn_en,
                text: "ನಮಸ್ಕಾರ".into(),
                allow_cloud: false,
            })
            .await
            .unwrap();
        assert_eq!(resp.translated_text, "Hello");
        assert!(resp.is_offline);

        // 2. English -> Kannada
        let pair_en_kn = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
        let resp2 = engine
            .translate(TranslationRequest {
                pair: pair_en_kn,
                text: "water".into(),
                allow_cloud: false,
            })
            .await
            .unwrap();
        assert_eq!(resp2.translated_text, "ನೀರು");

        // 3. Kannada -> Tamil
        let pair_kn_ta = LanguagePair::new(LanguageCode::KnIn, LanguageCode::TaIn).unwrap();
        let resp3 = engine
            .translate(TranslationRequest {
                pair: pair_kn_ta,
                text: "ಶಾಂತಿ".into(),
                allow_cloud: false,
            })
            .await
            .unwrap();
        assert_eq!(resp3.translated_text, "அமைதி");

        // 4. Tamil -> English
        let pair_ta_en = LanguagePair::new(LanguageCode::TaIn, LanguageCode::EnUs).unwrap();
        let resp4 = engine
            .translate(TranslationRequest {
                pair: pair_ta_en,
                text: "வணக்கம்".into(),
                allow_cloud: false,
            })
            .await
            .unwrap();
        assert_eq!(resp4.translated_text, "Hello");

        // 5. Tamil -> Kannada
        let pair_ta_kn = LanguagePair::new(LanguageCode::TaIn, LanguageCode::KnIn).unwrap();
        let resp5 = engine
            .translate(TranslationRequest {
                pair: pair_ta_kn,
                text: "தண்ணீர்".into(),
                allow_cloud: false,
            })
            .await
            .unwrap();
        assert_eq!(resp5.translated_text, "ನೀರು");

        // 6. Missing term reports explicit offline missing error
        let res_missing = engine
            .translate(TranslationRequest {
                pair: pair_kn_en,
                text: "ಅಪರಿಚಿತ ಪದ".into(),
                allow_cloud: false,
            })
            .await;
        assert!(res_missing.is_err());
        match res_missing {
            Err(LinguaError::OfflineModelMissing(msg)) => {
                assert!(msg.contains("ಅಪರಿಚಿತ ಪದ"));
            }
            other => panic!("Expected OfflineModelMissing, got {:?}", other),
        }
    }
}
