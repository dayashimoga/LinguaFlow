use crate::audio_capture::AudioCaptureManager;
use crate::http_provider::HttpTranslationProvider;
use crate::lexicon_engine::LexiconTranslationEngine;
use crate::local_model_engine::LocalNeuralTranslationEngine;
use crate::model_manager::ModelManager;
use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use chrono::Utc;
use lingua_core::{
    normalize_text_nfc, CapabilityMatrix, CapabilityState, LinguaError, TranslationRecord,
};
use lingua_storage::StorageService;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct EngineRegistry {
    translation_engines: Vec<Arc<dyn TranslationEngine>>,
    storage: Option<StorageService>,
    model_manager: Option<Arc<ModelManager>>,
    audio_capture: Arc<AudioCaptureManager>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            translation_engines: Vec::new(),
            storage: None,
            model_manager: None,
            audio_capture: Arc::new(AudioCaptureManager::default()),
        }
    }

    pub fn new_with_storage(storage: StorageService) -> Self {
        Self::new_with_storage_and_models(storage, ModelManager::default_models_dir())
    }

    pub fn new_with_storage_and_models(storage: StorageService, models_dir: PathBuf) -> Self {
        let mut registry = Self {
            translation_engines: Vec::new(),
            storage: Some(storage.clone()),
            model_manager: None,
            audio_capture: Arc::new(AudioCaptureManager::default()),
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

    pub fn register_translation_engine(&mut self, engine: Arc<dyn TranslationEngine>) {
        info!(
            "Registering translation engine: {} ({})",
            engine.name(),
            engine.id()
        );
        self.translation_engines.push(engine);
    }

    pub fn get_capabilities(&self) -> CapabilityMatrix {
        let mut matrix = CapabilityMatrix::stage_3_active();

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

        // 3. If online/cloud allowed, attempt HTTP provider for arbitrary sentences
        if req.allow_cloud {
            if let Some(http_engine) = self
                .translation_engines
                .iter()
                .find(|e| e.id() == "http-rest-provider" && e.supports_pair(&req.pair))
            {
                match http_engine.translate(req.clone()).await {
                    Ok(res) => {
                        self.record_history(&res);
                        return Ok(res);
                    }
                    Err(e) => {
                        info!(
                            "HTTP translation failed ({}), falling back to offline engines",
                            e
                        );
                    }
                }
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
    use lingua_core::{LanguageCode, LanguagePair};

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
}
