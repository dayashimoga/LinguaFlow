use crate::audio_capture::AudioCaptureManager;
use crate::context_nlp::ContextNlpProcessor;
use crate::http_provider::HttpTranslationProvider;
use crate::lexicon_engine::LexiconTranslationEngine;
use crate::local_model_engine::LocalNeuralTranslationEngine;
use crate::lru_cache::BoundedLruCache;
use crate::model_manager::ModelManager;
use crate::traits::{
    TextToSpeechEngine, TranslationEngine, TranslationRequest, TranslationResponse,
};
use crate::transliteration::TransliterationEngine;
use crate::tts_engine::AuthenticTtsEngine;
use chrono::Utc;
use lingua_core::{
    normalize_text_nfc, CapabilityMatrix, CapabilityState, CircuitBreakerStatus, CircuitState,
    FallbackStep, IntelligenceTranslationMetadata, IntelligenceTranslationRequest,
    IntelligenceTranslationResponse, LanguageCode, LanguagePair, LinguaError, ResourceProfile,
    TranslationRecord,
};
use lingua_storage::StorageService;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct CircuitBreaker {
    provider_id: String,
    state: Arc<std::sync::Mutex<CircuitState>>,
    failure_count: Arc<std::sync::Mutex<u32>>,
    success_count: Arc<std::sync::Mutex<u32>>,
    total_requests: Arc<std::sync::Mutex<u64>>,
    last_failure_instant: Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    failure_threshold: u32,
    reset_timeout: std::time::Duration,
}

impl CircuitBreaker {
    pub fn new(provider_id: &str, failure_threshold: u32, reset_timeout_secs: u64) -> Self {
        Self {
            provider_id: provider_id.to_string(),
            state: Arc::new(std::sync::Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(std::sync::Mutex::new(0)),
            success_count: Arc::new(std::sync::Mutex::new(0)),
            total_requests: Arc::new(std::sync::Mutex::new(0)),
            last_failure_instant: Arc::new(std::sync::Mutex::new(None)),
            failure_threshold,
            reset_timeout: std::time::Duration::from_secs(reset_timeout_secs),
        }
    }

    pub fn can_execute(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_fail) = *self.last_failure_instant.lock().unwrap() {
                    if last_fail.elapsed() >= self.reset_timeout {
                        *state = CircuitState::HalfOpen;
                        info!(
                            "Circuit breaker for '{}' transitioned from Open to HalfOpen",
                            self.provider_id
                        );
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&self) {
        if let Ok(mut st) = self.state.lock() {
            *st = CircuitState::Closed;
        }
        if let Ok(mut fc) = self.failure_count.lock() {
            *fc = 0;
        }
        if let Ok(mut sc) = self.success_count.lock() {
            *sc += 1;
        }
        if let Ok(mut tr) = self.total_requests.lock() {
            *tr += 1;
        }
    }

    pub fn record_failure(&self) {
        let mut fc = self.failure_count.lock().unwrap();
        *fc += 1;
        if let Ok(mut tr) = self.total_requests.lock() {
            *tr += 1;
        }
        if let Ok(mut lf) = self.last_failure_instant.lock() {
            *lf = Some(std::time::Instant::now());
        }

        if *fc >= self.failure_threshold {
            if let Ok(mut st) = self.state.lock() {
                *st = CircuitState::Open;
                tracing::warn!(
                    "Circuit breaker for '{}' TRIPPED to Open state after {} consecutive failures",
                    self.provider_id,
                    *fc
                );
            }
        }
    }

    pub fn reset(&self) {
        if let Ok(mut st) = self.state.lock() {
            *st = CircuitState::Closed;
        }
        if let Ok(mut fc) = self.failure_count.lock() {
            *fc = 0;
        }
    }

    pub fn status(&self) -> CircuitBreakerStatus {
        let state = *self.state.lock().unwrap();
        let failure_count = *self.failure_count.lock().unwrap();
        let success_count = *self.success_count.lock().unwrap();
        let total_requests = *self.total_requests.lock().unwrap();
        let last_failure_ms = self
            .last_failure_instant
            .lock()
            .unwrap()
            .map(|t| t.elapsed().as_millis() as u64);

        CircuitBreakerStatus {
            provider_id: self.provider_id.clone(),
            state,
            failure_count,
            success_count,
            last_failure_ms,
            total_requests,
        }
    }
}

pub struct EngineRegistry {
    translation_engines: Vec<Arc<dyn TranslationEngine>>,
    storage: Option<StorageService>,
    model_manager: Option<Arc<ModelManager>>,
    audio_capture: Arc<AudioCaptureManager>,
    tts_engine: Arc<dyn TextToSpeechEngine>,
    transliteration: Arc<TransliterationEngine>,
    lru_cache: BoundedLruCache<String, TranslationResponse>,
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
    resource_profile: Arc<std::sync::Mutex<ResourceProfile>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        let mut cbs = HashMap::new();
        cbs.insert(
            "http-rest-provider".to_string(),
            Arc::new(CircuitBreaker::new("http-rest-provider", 3, 15)),
        );

        Self {
            translation_engines: Vec::new(),
            storage: None,
            model_manager: None,
            audio_capture: Arc::new(AudioCaptureManager::default()),
            tts_engine: Arc::new(AuthenticTtsEngine::new()),
            transliteration: Arc::new(TransliterationEngine::default()),
            lru_cache: BoundedLruCache::new(2000),
            circuit_breakers: cbs,
            resource_profile: Arc::new(std::sync::Mutex::new(ResourceProfile::Balanced)),
        }
    }

    pub fn new_with_storage(storage: StorageService) -> Self {
        Self::new_with_storage_and_models(storage, ModelManager::default_models_dir())
    }

    pub fn new_with_storage_and_models(storage: StorageService, models_dir: PathBuf) -> Self {
        let mut cbs = HashMap::new();
        cbs.insert(
            "http-rest-provider".to_string(),
            Arc::new(CircuitBreaker::new("http-rest-provider", 3, 15)),
        );

        let mut registry = Self {
            translation_engines: Vec::new(),
            storage: Some(storage.clone()),
            model_manager: None,
            audio_capture: Arc::new(AudioCaptureManager::default()),
            tts_engine: Arc::new(AuthenticTtsEngine::new_with_storage(Some(storage.clone()))),
            transliteration: Arc::new(TransliterationEngine::default()),
            lru_cache: BoundedLruCache::new(2000),
            circuit_breakers: cbs,
            resource_profile: Arc::new(std::sync::Mutex::new(ResourceProfile::Balanced)),
        };

        // Initialize ModelManager
        if let Ok(mm) = ModelManager::new(storage.clone(), models_dir.clone()) {
            registry.model_manager = Some(Arc::new(mm));
        }

        // 1. Register Local Neural Translation Engine (on-device offline model)
        registry.register_translation_engine(Arc::new(LocalNeuralTranslationEngine::new(
            storage.clone(),
            models_dir,
        )));

        // 2. Register Authentic Lexicon Translation Engine (offline lexicon)
        registry
            .register_translation_engine(Arc::new(LexiconTranslationEngine::new(storage.clone())));

        // 3. Register Authentic HTTP Translation Provider (online cloud)
        registry.register_translation_engine(Arc::new(HttpTranslationProvider::default()));

        registry
    }

    pub fn model_manager(&self) -> Option<Arc<ModelManager>> {
        self.model_manager.clone()
    }

    pub fn audio_capture(&self) -> Arc<AudioCaptureManager> {
        Arc::clone(&self.audio_capture)
    }

    pub fn tts_engine(&self) -> Arc<dyn TextToSpeechEngine> {
        Arc::clone(&self.tts_engine)
    }

    pub fn transliteration(&self) -> Arc<TransliterationEngine> {
        Arc::clone(&self.transliteration)
    }

    pub fn circuit_breakers(&self) -> Vec<CircuitBreakerStatus> {
        self.circuit_breakers
            .values()
            .map(|cb| cb.status())
            .collect()
    }

    pub fn reset_circuit_breaker(&self, provider_id: &str) -> bool {
        if let Some(cb) = self.circuit_breakers.get(provider_id) {
            cb.reset();
            true
        } else {
            false
        }
    }

    pub fn resource_profile(&self) -> ResourceProfile {
        *self.resource_profile.lock().unwrap()
    }

    pub fn set_resource_profile(&self, profile: ResourceProfile) {
        if let Ok(mut current) = self.resource_profile.lock() {
            *current = profile;
        }
        self.lru_cache.set_capacity(profile.max_cache_entries());
    }

    pub fn register_translation_engine(&mut self, engine: Arc<dyn TranslationEngine>) {
        info!(
            "Registering translation engine: {} ({})",
            engine.name(),
            engine.id()
        );
        self.translation_engines.push(engine);
    }

    pub fn get_capabilities(&self) -> CapabilityMatrix {
        let mut matrix = CapabilityMatrix::stage_8_active();

        if !self.translation_engines.is_empty() {
            matrix.text_translation = CapabilityState::Active;
        }

        matrix
    }

    pub async fn translate(
        &self,
        req: TranslationRequest,
    ) -> Result<TranslationResponse, LinguaError> {
        // Enforce language pair validity
        req.pair.validate()?;

        let cleaned = normalize_text_nfc(req.text.trim());
        if cleaned.is_empty() {
            return Err(LinguaError::Validation(
                "Cannot translate empty text".into(),
            ));
        }

        // 1. Check local translation cache for instant response
        if let Some(ref storage) = self.storage {
            if let Ok(Some(cached)) =
                storage.lookup_translation_cache(req.pair.source, req.pair.target, &cleaned)
            {
                return Ok(TranslationResponse {
                    pair: req.pair,
                    original_text: cleaned,
                    translated_text: cached.translated_text,
                    engine_id: format!("{}:cache", cached.engine_id),
                    is_offline: true,
                    latency_ms: 1,
                });
            }
        }

        // 2. Check local on-device neural engine (if installed for this pair)
        if let Some(local_engine) = self
            .translation_engines
            .iter()
            .find(|e| e.id() == "local-neural-indictrans2" && e.supports_pair(&req.pair))
        {
            if local_engine.capability_state(&req.pair) == CapabilityState::Active {
                match local_engine.translate(req.clone()).await {
                    Ok(res) => {
                        self.record_history(&res);
                        return Ok(res);
                    }
                    Err(LinguaError::OfflineModelMissing(_)) => {
                        // Fall through to other engines
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // 3. For words and short phrases, query verified offline lexicon first (Tier 1: deterministic, zero hallucination)
        let token_count = cleaned.split_whitespace().count();
        if token_count <= 3 {
            if let Some(lexicon_engine) = self
                .translation_engines
                .iter()
                .find(|e| e.id() == "verified-lexicon-engine" && e.supports_pair(&req.pair))
            {
                if let Ok(res) = lexicon_engine.translate(req.clone()).await {
                    self.record_history(&res);
                    return Ok(res);
                }
            }
        }

        // 4. If online/cloud allowed, attempt HTTP provider for arbitrary sentences with circuit breaker
        if req.allow_cloud {
            let cb_opt = self.circuit_breakers.get("http-rest-provider");
            let can_run = cb_opt.map(|cb| cb.can_execute()).unwrap_or(true);

            if can_run {
                if let Some(http_engine) = self
                    .translation_engines
                    .iter()
                    .find(|e| e.id() == "http-rest-provider" && e.supports_pair(&req.pair))
                {
                    match http_engine.translate(req.clone()).await {
                        Ok(res) => {
                            if let Some(cb) = cb_opt {
                                cb.record_success();
                            }
                            self.record_history(&res);
                            return Ok(res);
                        }
                        Err(e) => {
                            if let Some(cb) = cb_opt {
                                cb.record_failure();
                            }
                            info!(
                                "HTTP translation failed ({}), circuit breaker updated, falling back to offline engines",
                                e
                            );
                        }
                    }
                }
            } else {
                info!("Circuit breaker OPEN for http-rest-provider; bypassing to offline fallback");
            }
        }

        // 4. Query offline lexicon engine
        if let Some(lexicon_engine) = self
            .translation_engines
            .iter()
            .find(|e| e.id() == "verified-lexicon-engine" && e.supports_pair(&req.pair))
        {
            match lexicon_engine.translate(req.clone()).await {
                Ok(res) => {
                    self.record_history(&res);
                    return Ok(res);
                }
                Err(LinguaError::OfflineModelMissing(_)) => {
                    // Fall through
                }
                Err(e) => return Err(e),
            }
        }

        // 5. Query any other registered offline engines
        for engine in &self.translation_engines {
            if engine.id() != "http-rest-provider"
                && engine.id() != "local-neural-indictrans2"
                && engine.id() != "verified-lexicon-engine"
                && engine.supports_pair(&req.pair)
            {
                match engine.translate(req.clone()).await {
                    Ok(res) => {
                        self.record_history(&res);
                        return Ok(res);
                    }
                    Err(LinguaError::OfflineModelMissing(_)) => {
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // If offline and term could not be translated
        if !req.allow_cloud {
            let required_pack = LocalNeuralTranslationEngine::required_model_pack_id(&req.pair);
            return Err(LinguaError::OfflineModelMissing(format!(
                "Text '{cleaned}' for pair {} is not available in offline storage. Install model pack '{required_pack}' in the Model Manager for full offline translation.",
                req.pair
            )));
        }

        // Global Engineering Contract Mandate:
        // Do not simulate or fake translation when no real engine could process the text.
        Err(LinguaError::CapabilityUnavailable(
            "text_translation".into(),
            format!(
                "No active translation engine could translate text for pair '{}'. Authentic engine output required.",
                req.pair
            ),
        ))
    }

    /// Enhanced intelligent translation coordinating transliteration, entity protection,
    /// domain glossaries, circuit breakers, and formality adaptation.
    pub async fn translate_with_intelligence(
        &self,
        req: IntelligenceTranslationRequest,
    ) -> Result<IntelligenceTranslationResponse, LinguaError> {
        let mut fallback_trail = Vec::new();

        // 1. Script detection and auto transliteration if requested
        let mut working_text = req.text.trim().to_string();
        let mut transliterated_input = None;

        let detected = lingua_core::detect_language(&working_text);
        let mut src_lang = req.source_lang.or(detected).unwrap_or(LanguageCode::EnUs);

        // Mode A: Source Input Transliteration (Roman-to-Indic IME)
        // Active ONLY when source language is an Indic language (KnIn, TaIn) or detected as Romanized Indic.
        // If the source language is explicitly English (EnUs), NEVER transliterate the English input!
        if req.enable_transliteration && req.source_lang != Some(LanguageCode::EnUs) {
            let translit_target = if src_lang == LanguageCode::KnIn {
                Some(LanguageCode::KnIn)
            } else if src_lang == LanguageCode::TaIn {
                Some(LanguageCode::TaIn)
            } else if req.source_lang.is_none() {
                // Romanized input targeting English (e.g. "namaskara" or "vanakkam")
                let res_kn = self
                    .transliteration
                    .transliterate(&working_text, LanguageCode::KnIn);
                if let Ok(res) = res_kn {
                    if res.transliterated != working_text
                        && lingua_core::detect_language(&res.transliterated)
                            == Some(LanguageCode::KnIn)
                    {
                        Some(LanguageCode::KnIn)
                    } else {
                        Some(LanguageCode::TaIn)
                    }
                } else {
                    Some(LanguageCode::KnIn)
                }
            } else {
                None
            };

            if let Some(target) = translit_target {
                if let Ok(translit_res) = self.transliteration.transliterate(&working_text, target)
                {
                    if !translit_res.transliterated.is_empty()
                        && translit_res.transliterated != working_text
                    {
                        transliterated_input = Some(translit_res.transliterated.clone());
                        working_text = translit_res.transliterated;
                        if req.source_lang.is_none() {
                            src_lang = target;
                        }
                    }
                }
            }
        }

        // 2. Protect named entities
        let (masked_text, protected_entities) =
            ContextNlpProcessor::protect_named_entities(&working_text);

        // 3. Domain glossaries pre-fetch
        let mut glossary_entries = Vec::new();
        if let Some(ref storage) = self.storage {
            if let Ok(entries) = storage.list_glossary_entries(
                req.domain_glossary.as_deref(),
                Some(src_lang),
                Some(req.target_lang),
            ) {
                glossary_entries = entries;
            }
        }

        // 4. Try LRU In-Memory Cache or Tier 1 Rich Lexicon
        let lru_key = format!(
            "{}:{}:{}",
            src_lang.as_bcp47(),
            req.target_lang.as_bcp47(),
            masked_text
        );

        let token_count = masked_text.split_whitespace().count();
        let mut dictionary_senses = Vec::new();
        let confidence_score: Option<f32>;
        let mut is_compositional_gloss = false;

        let lru_start = std::time::Instant::now();
        let (translated_text, engine_selected) = if let Some(cached_resp) =
            self.lru_cache.get(&lru_key)
        {
            fallback_trail.push(FallbackStep {
                engine_id: "in_memory_lru_cache".into(),
                attempt: 1,
                success: true,
                latency_ms: lru_start.elapsed().as_secs_f64() * 1000.0,
                reason: Some("Instant memory cache hit".into()),
            });
            confidence_score = Some(0.99);
            (
                cached_resp.translated_text,
                "in_memory_lru_cache".to_string(),
            )
        } else {
            fallback_trail.push(FallbackStep {
                engine_id: "in_memory_lru_cache".into(),
                attempt: 1,
                success: false,
                latency_ms: lru_start.elapsed().as_secs_f64() * 1000.0,
                reason: Some("Cache miss".into()),
            });

            // Tier 1: Check offline rich lexicon directly for words & phrases <= 3 tokens
            let mut tier1_match = None;
            if token_count <= 3 {
                if let Some(ref storage) = self.storage {
                    if let Ok(res) =
                        storage.search_rich_dictionary(&masked_text, Some(src_lang), None, 5)
                    {
                        for entry in &res.rich_entries {
                            if entry.headword.eq_ignore_ascii_case(&masked_text) {
                                if let Some(targets) =
                                    entry.target_equivalents.get(req.target_lang.as_bcp47())
                                {
                                    if let Some(first) = targets.first() {
                                        tier1_match = Some((first.clone(), entry.headword.clone()));
                                        for s in &entry.senses {
                                            dictionary_senses.push(format!(
                                                "{}: {}",
                                                entry.headword, s.definition
                                            ));
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some((tgt_text, headword)) = tier1_match {
                fallback_trail.push(FallbackStep {
                    engine_id: "offline_fts5_lexicon".into(),
                    attempt: 2,
                    success: true,
                    latency_ms: lru_start.elapsed().as_secs_f64() * 1000.0,
                    reason: Some(format!(
                        "Verified bilingual rich lexicon match for '{headword}'"
                    )),
                });
                confidence_score = Some(0.98);
                let resp = TranslationResponse {
                    original_text: masked_text.clone(),
                    translated_text: tgt_text.clone(),
                    pair: LanguagePair::new(src_lang, req.target_lang)?,
                    engine_id: "offline_fts5_lexicon".into(),
                    is_offline: true,
                    latency_ms: lru_start.elapsed().as_millis() as u64,
                };
                self.lru_cache.insert(lru_key, resp);
                (tgt_text, "offline_fts5_lexicon".to_string())
            } else {
                // Query via core translate
                let pair = LanguagePair::new(src_lang, req.target_lang)?;
                let trans_req = TranslationRequest {
                    pair,
                    text: masked_text.clone(),
                    allow_cloud: req.allow_cloud,
                };

                let trans_start = std::time::Instant::now();
                match self.translate(trans_req).await {
                    Ok(resp) => {
                        fallback_trail.push(FallbackStep {
                            engine_id: resp.engine_id.clone(),
                            attempt: 2,
                            success: true,
                            latency_ms: trans_start.elapsed().as_secs_f64() * 1000.0,
                            reason: Some("Translation successful".into()),
                        });
                        let out_text = resp.translated_text.clone();
                        let out_eng = resp.engine_id.clone();
                        if out_eng.contains("neural") {
                            confidence_score = Some(0.94);
                        } else if out_eng.contains("lexicon") {
                            confidence_score = Some(0.95);
                        } else {
                            confidence_score = Some(0.90);
                        }
                        self.lru_cache.insert(lru_key, resp);
                        (out_text, out_eng)
                    }
                    Err(e) => {
                        // Compositional glossing fallback if offline model is missing for multi-word
                        if token_count > 1 {
                            if let Some(storage) = &self.storage {
                                let words: Vec<&str> = masked_text.split_whitespace().collect();
                                let mut glossed_words = Vec::new();
                                for w in &words {
                                    let clean = w.trim_matches(|c: char| c.is_ascii_punctuation());
                                    if let Ok(res) = storage.search_rich_dictionary(
                                        clean,
                                        Some(src_lang),
                                        None,
                                        1,
                                    ) {
                                        if let Some(entry) = res.rich_entries.first() {
                                            if let Some(targets) = entry
                                                .target_equivalents
                                                .get(req.target_lang.as_bcp47())
                                            {
                                                if let Some(t) = targets.first() {
                                                    glossed_words.push(t.clone());
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    glossed_words.push((*w).to_string());
                                }
                                let gloss_result = glossed_words.join(" ");
                                fallback_trail.push(FallbackStep {
                                        engine_id: "offline_compositional_gloss".into(),
                                        attempt: 3,
                                        success: true,
                                        latency_ms: trans_start.elapsed().as_secs_f64() * 1000.0,
                                        reason: Some("Offline compositional gloss applied (download NMT model for full fluency)".into()),
                                    });
                                is_compositional_gloss = true;
                                confidence_score = Some(0.65);
                                (gloss_result, "offline_compositional_gloss".to_string())
                            } else {
                                fallback_trail.push(FallbackStep {
                                    engine_id: "primary_engines".into(),
                                    attempt: 2,
                                    success: false,
                                    latency_ms: trans_start.elapsed().as_secs_f64() * 1000.0,
                                    reason: Some(e.to_string()),
                                });
                                return Err(e);
                            }
                        } else {
                            fallback_trail.push(FallbackStep {
                                engine_id: "primary_engines".into(),
                                attempt: 2,
                                success: false,
                                latency_ms: trans_start.elapsed().as_secs_f64() * 1000.0,
                                reason: Some(e.to_string()),
                            });
                            return Err(e);
                        }
                    }
                }
            }
        };

        // 6. Restore named entities
        let restored_text =
            ContextNlpProcessor::restore_named_entities(&translated_text, &protected_entities);

        // 7. Apply domain glossaries
        let (glossary_applied_text, terms_applied) = ContextNlpProcessor::apply_domain_glossary(
            &restored_text,
            src_lang,
            req.target_lang,
            &glossary_entries,
        );

        // 8. Adapt formality
        let final_text = ContextNlpProcessor::adapt_formality(
            &glossary_applied_text,
            req.target_lang,
            req.formality,
        );

        // 9. Compute Mode B target phonetic reading aid (ISO-15919)
        let target_phonetic =
            if req.target_lang == LanguageCode::KnIn || req.target_lang == LanguageCode::TaIn {
                let aid = self
                    .transliteration
                    .to_phonetic_reading_aid(&final_text, req.target_lang);
                if aid != final_text {
                    Some(aid)
                } else {
                    None
                }
            } else {
                None
            };

        Ok(IntelligenceTranslationResponse {
            source_text: req.text,
            translated_text: final_text,
            detected_lang: src_lang,
            target_lang: req.target_lang,
            metadata: IntelligenceTranslationMetadata {
                engine_selected,
                formality: req.formality,
                glossary_terms_applied: terms_applied,
                transliterated_input,
                target_phonetic,
                confidence_score,
                dictionary_senses,
                is_compositional_gloss,
                fallback_trail,
                chrf_estimated: None, // Honest telemetry: chrF is reserved for ground-truth benchmark evaluation
            },
        })
    }

    fn record_history(&self, res: &TranslationResponse) {
        if let Some(ref storage) = self.storage {
            let record = TranslationRecord {
                id: Uuid::new_v4().to_string(),
                source_language: res.pair.source,
                target_language: res.pair.target,
                source_text: res.original_text.clone(),
                translated_text: res.translated_text.clone(),
                engine_used: res.engine_id.clone(),
                latency_ms: res.latency_ms,
                created_at: Utc::now(),
            };
            if let Err(e) = storage.record_translation(&record) {
                tracing::warn!("Failed to persist translation history: {}", e);
            }
        }
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use lingua_core::{FormalityMode, LanguageCode, LanguagePair};

    // Unit test double strictly isolated inside test module per contract rule 1
    struct TestDummyEngine;

    #[async_trait]
    impl TranslationEngine for TestDummyEngine {
        fn id(&self) -> &'static str {
            "dummy-test-engine"
        }
        fn name(&self) -> &'static str {
            "Isolated Test Double Engine"
        }
        fn supports_pair(&self, pair: &LanguagePair) -> bool {
            pair.source == LanguageCode::EnUs && pair.target == LanguageCode::KnIn
        }
        fn capability_state(&self, _pair: &LanguagePair) -> CapabilityState {
            CapabilityState::Ready
        }
        async fn translate(
            &self,
            req: TranslationRequest,
        ) -> Result<TranslationResponse, LinguaError> {
            Ok(TranslationResponse {
                pair: req.pair,
                original_text: req.text,
                translated_text: "ಪರೀಕ್ಷಾರ್ಥ ಅನುವಾದ".into(),
                engine_id: self.id().into(),
                is_offline: true,
                latency_ms: 12,
            })
        }
    }

    #[tokio::test]
    async fn test_empty_registry_rejects_without_faking() {
        let registry = EngineRegistry::new();
        let pair = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();

        let req = TranslationRequest {
            pair,
            text: "Hello".into(),
            allow_cloud: false,
        };

        let result = registry.translate(req).await;
        assert!(result.is_err());
        match result {
            Err(LinguaError::OfflineModelMissing(detail)) => {
                assert!(detail.contains("offline storage") || detail.contains("Model Manager"));
            }
            Err(LinguaError::CapabilityUnavailable(cap, detail)) => {
                assert_eq!(cap, "text_translation");
                assert!(detail.contains("Authentic engine output"));
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_isolated_engine_registration() {
        let mut registry = EngineRegistry::new();
        registry.register_translation_engine(Arc::new(TestDummyEngine));

        let caps = registry.get_capabilities();
        assert_eq!(caps.text_translation, CapabilityState::Active);

        let pair = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
        let req = TranslationRequest {
            pair,
            text: "Test sentence".into(),
            allow_cloud: false,
        };

        let res = registry.translate(req).await.unwrap();
        assert_eq!(res.translated_text, "ಪರೀಕ್ಷಾರ್ಥ ಅನುವಾದ");
        assert_eq!(res.engine_id, "dummy-test-engine");
    }

    #[tokio::test]
    async fn test_new_with_storage_integration() {
        let storage = StorageService::open_in_memory().unwrap();
        let registry = EngineRegistry::new_with_storage(storage.clone());

        let pair = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs).unwrap();
        let req = TranslationRequest {
            pair,
            text: "ನಮಸ್ಕಾರ".into(),
            allow_cloud: false,
        };

        let res = registry.translate(req).await.unwrap();
        assert_eq!(res.translated_text, "Hello");
        assert_eq!(res.engine_id, "verified-lexicon-engine");

        // Verify history was recorded
        let history = storage.list_translation_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].source_text, "ನಮಸ್ಕಾರ");
        assert_eq!(history[0].translated_text, "Hello");
    }

    #[test]
    fn test_circuit_breaker_trip_and_reset() {
        let cb = CircuitBreaker::new("test-provider", 2, 60);
        assert!(cb.can_execute());
        assert_eq!(cb.status().state, CircuitState::Closed);

        cb.record_failure();
        assert!(cb.can_execute());
        assert_eq!(cb.status().failure_count, 1);

        cb.record_failure(); // Tripped!
        assert_eq!(cb.status().state, CircuitState::Open);
        assert!(!cb.can_execute());

        cb.reset();
        assert_eq!(cb.status().state, CircuitState::Closed);
        assert!(cb.can_execute());
    }

    #[tokio::test]
    async fn test_translate_with_intelligence_flow() {
        let storage = StorageService::open_in_memory().unwrap();
        let registry = EngineRegistry::new_with_storage(storage.clone());

        // Test with Romanized Kannada input (Kanglish) + formality adaptation
        let req = IntelligenceTranslationRequest {
            text: "namaskara".into(),
            source_lang: None,
            target_lang: LanguageCode::EnUs,
            formality: FormalityMode::Formal,
            enable_transliteration: true,
            domain_glossary: None,
            allow_cloud: false,
        };

        let res = registry.translate_with_intelligence(req).await.unwrap();
        assert_eq!(res.source_text, "namaskara");
        assert!(!res.translated_text.is_empty());
        assert_eq!(res.metadata.formality, FormalityMode::Formal);
        assert!(res.metadata.transliterated_input.is_some());
        assert_eq!(res.metadata.transliterated_input.as_deref(), Some("ನಮಸ್ಕಾರ"));
        assert!(!res.metadata.fallback_trail.is_empty());
    }
}
