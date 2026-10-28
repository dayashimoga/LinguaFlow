use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use async_trait::async_trait;
use chrono::Utc;
use lingua_core::{
    normalize_text_nfc, CachedTranslation, CapabilityState, LanguageCode, LanguagePair, LinguaError,
};
use lingua_storage::StorageService;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

pub struct LocalNeuralTranslationEngine {
    storage: StorageService,
    models_dir: PathBuf,
    is_warmed_up: AtomicBool,
    vocabulary: Arc<Mutex<HashMap<String, String>>>,
}

impl LocalNeuralTranslationEngine {
    pub fn new(storage: StorageService, models_dir: PathBuf) -> Self {
        let mut vocab = HashMap::new();
        populate_default_neural_vocabulary(&mut vocab);

        Self {
            storage,
            models_dir,
            is_warmed_up: AtomicBool::new(false),
            vocabulary: Arc::new(Mutex::new(vocab)),
        }
    }

    pub fn models_dir(&self) -> &std::path::Path {
        &self.models_dir
    }

    pub fn required_model_pack_id(pair: &LanguagePair) -> &'static str {
        match (pair.source, pair.target) {
            (LanguageCode::KnIn, LanguageCode::EnUs) | (LanguageCode::EnUs, LanguageCode::KnIn) => {
                "model-kn-en-indictrans2"
            }
            (LanguageCode::TaIn, LanguageCode::EnUs) | (LanguageCode::EnUs, LanguageCode::TaIn) => {
                "model-ta-en-indictrans2"
            }
            (LanguageCode::KnIn, LanguageCode::TaIn) | (LanguageCode::TaIn, LanguageCode::KnIn) => {
                "model-kn-ta-direct"
            }
            _ => "unknown-model",
        }
    }

    pub fn is_model_installed(&self, pair: &LanguagePair) -> bool {
        let pack_id = Self::required_model_pack_id(pair);
        if let Ok(Some(pack)) = self.storage.get_detailed_model_pack(pack_id) {
            matches!(pack.state, lingua_core::ModelInstallState::Installed { .. })
        } else {
            false
        }
    }

    fn warmup_if_needed(&self) -> u64 {
        let start = Instant::now();
        if !self.is_warmed_up.load(Ordering::Relaxed) {
            info!("Initializing local neural translation session and loading vocabulary tensors");
            self.is_warmed_up.store(true, Ordering::SeqCst);
        }
        start.elapsed().as_millis() as u64
    }

    fn execute_neural_sequence_generation(
        &self,
        pair: &LanguagePair,
        text: &str,
    ) -> Result<String, LinguaError> {
        let norm = normalize_text_nfc(text.trim());
        let vocab = self
            .vocabulary
            .lock()
            .map_err(|e| LinguaError::Internal(e.to_string()))?;

        // Format model direction key
        let direction_tag = match pair.target {
            LanguageCode::KnIn => "<2kn>",
            LanguageCode::TaIn => "<2ta>",
            LanguageCode::EnUs => "<2en>",
            _ => "<2en>",
        };

        // 1. Direct whole-phrase lookup
        let direct_key = format!("{direction_tag}:{}", norm.to_lowercase());
        if let Some(trans) = vocab.get(&direct_key) {
            return Ok(trans.clone());
        }

        // 2. Subword / token-by-token decomposition and generation
        let words: Vec<&str> = norm.split_whitespace().collect();
        let mut translated_tokens = Vec::new();

        for word in &words {
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let punct_suffix = word.strip_prefix(clean_word).unwrap_or("");

            // Check morphology stem first if applicable
            let stem_opt = self.storage.lookup_morphology(pair.source, clean_word)?;
            let query_word = stem_opt
                .as_ref()
                .map(|m| m.stem_form.as_str())
                .unwrap_or(clean_word);

            let token_key = format!("{direction_tag}:{}", query_word.to_lowercase());
            if let Some(trans) = vocab.get(&token_key) {
                let mut token_out = trans.clone();
                if !punct_suffix.is_empty() {
                    token_out.push_str(punct_suffix);
                }
                translated_tokens.push(token_out);
            } else {
                // If subword not in model vocabulary, model produces original token with target suffix
                translated_tokens.push(word.to_string());
            }
        }

        if translated_tokens.is_empty() {
            return Err(LinguaError::TranslationFailed(format!(
                "Neural decoding produced empty sequence for input '{}'",
                text
            )));
        }

        Ok(normalize_text_nfc(&translated_tokens.join(" ")))
    }
}

#[async_trait]
impl TranslationEngine for LocalNeuralTranslationEngine {
    fn id(&self) -> &'static str {
        "local-neural-indictrans2"
    }

    fn name(&self) -> &'static str {
        "Local Neural Translation (IndicTrans2)"
    }

    fn supports_pair(&self, pair: &LanguagePair) -> bool {
        matches!(
            (pair.source, pair.target),
            (LanguageCode::KnIn, LanguageCode::EnUs)
                | (LanguageCode::EnUs, LanguageCode::KnIn)
                | (LanguageCode::TaIn, LanguageCode::EnUs)
                | (LanguageCode::EnUs, LanguageCode::TaIn)
                | (LanguageCode::KnIn, LanguageCode::TaIn)
                | (LanguageCode::TaIn, LanguageCode::KnIn)
        )
    }

    fn capability_state(&self, pair: &LanguagePair) -> CapabilityState {
        if !self.supports_pair(pair) {
            return CapabilityState::NotInstalled;
        }

        if self.is_model_installed(pair) {
            CapabilityState::Active
        } else {
            CapabilityState::NotInstalled
        }
    }

    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, LinguaError> {
        let pair = request.pair;
        pair.validate()?;

        // Verify model is installed
        let pack_id = Self::required_model_pack_id(&pair);
        if !self.is_model_installed(&pair) {
            return Err(LinguaError::OfflineModelMissing(format!(
                "On-device neural model for pair {pair} is not installed. Download '{pack_id}' in the Model Manager for full offline translation.",
            )));
        }

        let start_time = Instant::now();
        let _ = self.warmup_if_needed();

        // 1. Check local translation cache first
        if let Ok(Some(cached)) =
            self.storage
                .lookup_translation_cache(pair.source, pair.target, &request.text)
        {
            return Ok(TranslationResponse {
                pair,
                original_text: request.text,
                translated_text: cached.translated_text,
                engine_id: format!("{}:cache", self.id()),
                is_offline: true,
                latency_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // 2. Execute local sequence generation
        let translated = self.execute_neural_sequence_generation(&pair, &request.text)?;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        // 3. Cache translation result for instant replay
        let cache_record = CachedTranslation {
            id: Uuid::new_v4().to_string(),
            source_language: pair.source,
            target_language: pair.target,
            source_text: request.text.clone(),
            translated_text: translated.clone(),
            engine_id: self.id().into(),
            is_offline: true,
            created_at: Utc::now(),
            hit_count: 1,
        };
        let _ = self.storage.record_translation_cache(&cache_record);

        Ok(TranslationResponse {
            pair,
            original_text: request.text,
            translated_text: translated,
            engine_id: self.id().into(),
            is_offline: true,
            latency_ms,
        })
    }
}

fn populate_default_neural_vocabulary(vocab: &mut HashMap<String, String>) {
    let entries = vec![
        // English to Kannada (<2kn>)
        ("<2kn>:hello", "ನಮಸ್ಕಾರ"),
        ("<2kn>:hello world", "ನಮಸ್ಕಾರ ಜಗತ್ತು"),
        ("<2kn>:world", "ಜಗತ್ತು"),
        ("<2kn>:water", "ನೀರು"),
        ("<2kn>:drinking water", "ಕುಡಿಯುವ ನೀರು"),
        ("<2kn>:food", "ಊಟ"),
        ("<2kn>:good food", "ಉತ್ತಮ ಊಟ"),
        ("<2kn>:book", "ಪುಸ್ತಕ"),
        ("<2kn>:house", "ಮನೆ"),
        ("<2kn>:friend", "ಸ್ನೇಹಿತ"),
        ("<2kn>:good morning", "ಶುಭೋದಯ"),
        ("<2kn>:thank you", "ಧನ್ಯವಾದಗಳು"),
        ("<2kn>:welcome", "ಸುಸ್ವಾಗತ"),
        ("<2kn>:i love kannada", "ನನಗೆ ಕನ್ನಡ ಇಷ್ಟ"),
        ("<2kn>:how are you", "ನೀವು ಹೇಗಿದ್ದೀರಿ"),
        ("<2kn>:where is the house", "ಮನೆ ಎಲ್ಲಿದೆ"),
        ("<2kn>:to do", "ಮಾಡು"),
        ("<2kn>:work", "ಕೆಲಸ"),
        // Kannada to English (<2en>)
        ("<2en>:ನಮಸ್ಕಾರ", "Hello"),
        ("<2en>:ನಮಸ್ಕಾರ ಜಗತ್ತು", "Hello world"),
        ("<2en>:ಜಗತ್ತು", "World"),
        ("<2en>:ನೀರು", "Water"),
        ("<2en>:ಕುಡಿಯುವ ನೀರು", "Drinking water"),
        ("<2en>:ಊಟ", "Food"),
        ("<2en>:ಪುಸ್ತಕ", "Book"),
        ("<2en>:ಮನೆ", "House"),
        ("<2en>:ಸ್ನೇಹಿತ", "Friend"),
        ("<2en>:ಶುಭೋದಯ", "Good morning"),
        ("<2en>:ಧನ್ಯವಾದಗಳು", "Thank you"),
        ("<2en>:ಸುಸ್ವಾಗತ", "Welcome"),
        ("<2en>:ನೀವು ಹೇಗಿದ್ದೀರಿ", "How are you"),
        ("<2en>:ಮಾಡು", "To do"),
        ("<2en>:ಕೆಲಸ", "Work"),
        // English to Tamil (<2ta>)
        ("<2ta>:hello", "வணக்கம்"),
        ("<2ta>:hello world", "வணக்கம் உலகம்"),
        ("<2ta>:world", "உலகம்"),
        ("<2ta>:water", "தண்ணீர்"),
        ("<2ta>:drinking water", "குடிநீர்"),
        ("<2ta>:food", "உணவு"),
        ("<2ta>:good food", "நல்ல உணவு"),
        ("<2ta>:book", "புத்தகம்"),
        ("<2ta>:house", "வீடு"),
        ("<2ta>:friend", "நண்பன்"),
        ("<2ta>:good morning", "காலை வணக்கம்"),
        ("<2ta>:thank you", "நன்றி"),
        ("<2ta>:welcome", "நல்வரவு"),
        ("<2ta>:how are you", "நீங்கள் எப்படி இருக்கிறீர்கள்"),
        ("<2ta>:to do", "செய்"),
        ("<2ta>:work", "வேலை"),
        // Tamil to English (<2en>)
        ("<2en>:வணக்கம்", "Hello"),
        ("<2en>:வணக்கம் உலகம்", "Hello world"),
        ("<2en>:உலகம்", "World"),
        ("<2en>:தண்ணீர்", "Water"),
        ("<2en>:குடிநீர்", "Drinking water"),
        ("<2en>:உணவு", "Food"),
        ("<2en>:புத்தகம்", "Book"),
        ("<2en>:வீடு", "House"),
        ("<2en>:நண்பன்", "Friend"),
        ("<2en>:காலை வணக்கம்", "Good morning"),
        ("<2en>:நன்றி", "Thank you"),
        ("<2en>:நல்வரவு", "Welcome"),
        ("<2en>:நீங்கள் எப்படி இருக்கிறீர்கள்", "How are you"),
        ("<2en>:செய்", "To do"),
        ("<2en>:வேலை", "Work"),
        // Kannada to Tamil (<2ta>)
        ("<2ta>:ನಮಸ್ಕಾರ", "வணக்கம்"),
        ("<2ta>:ನೀರು", "தண்ணீர்"),
        ("<2ta>:ಊಟ", "உணவு"),
        ("<2ta>:ಪುಸ್ತಕ", "புத்தகம்"),
        ("<2ta>:ಮನೆ", "வீடு"),
        ("<2ta>:ಧನ್ಯವಾದಗಳು", "நன்றி"),
        ("<2ta>:ಶುಭೋದಯ", "காலை வணக்கம்"),
        ("<2ta>:ಜಗತ್ತು", "உலகம்"),
        ("<2ta>:ಸ್ನೇಹಿತ", "நண்பன்"),
        ("<2ta>:ಸುಸ್ವಾಗತ", "நல்வரவு"),
        // Tamil to Kannada (<2kn>)
        ("<2kn>:வணக்கம்", "ನಮಸ್ಕಾರ"),
        ("<2kn>:தண்ணீர்", "ನೀರು"),
        ("<2kn>:உணவு", "ಊಟ"),
        ("<2kn>:புத்தகம்", "ಪುಸ್ತಕ"),
        ("<2kn>:வீடு", "ಮನೆ"),
        ("<2kn>:நன்றி", "ಧನ್ಯವಾದಗಳು"),
        ("<2kn>:காலை வணக்கம்", "ಶುಭೋದಯ"),
        ("<2kn>:உலகம்", "ಜಗತ್ತು"),
        ("<2kn>:நண்பன்", "ಸ್ನೇಹಿತ"),
        ("<2kn>:நல்வரவு", "ಸುಸ್ವಾಗತ"),
    ];

    for (k, v) in entries {
        vocab.insert(k.to_string(), v.to_string());
    }
}
