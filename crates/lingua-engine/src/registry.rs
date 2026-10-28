use crate::http_provider::HttpTranslationProvider;
use crate::lexicon_engine::LexiconTranslationEngine;
use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use chrono::Utc;
use lingua_core::{
    normalize_text_nfc, CapabilityMatrix, CapabilityState, LinguaError, TranslationRecord,
};
use lingua_storage::StorageService;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct EngineRegistry {
    translation_engines: Vec<Arc<dyn TranslationEngine>>,
    storage: Option<StorageService>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            translation_engines: Vec::new(),
            storage: None,
        }
    }

    pub fn new_with_storage(storage: StorageService) -> Self {
        let mut registry = Self {
            translation_engines: Vec::new(),
            storage: Some(storage.clone()),
        };

        // Register authentic Lexicon Translation Engine (offline)
        registry
            .register_translation_engine(Arc::new(LexiconTranslationEngine::new(storage.clone())));

        // Register authentic HTTP Translation Provider (online)
        registry.register_translation_engine(Arc::new(HttpTranslationProvider::default()));

        registry
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
        let mut matrix = CapabilityMatrix::stage_1_active();

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

        // 1. If online/cloud allowed, attempt HTTP provider first for arbitrary text
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
                            "HTTP translation failed ({}), attempting offline lexicon fallback",
                            e
                        );
                        // Fall through to offline lexicon
                    }
                }
            }
        }

        // 2. Query offline engines (e.g. Lexicon or local models)
        for engine in &self.translation_engines {
            if engine.id() != "http-rest-provider" && engine.supports_pair(&req.pair) {
                match engine.translate(req.clone()).await {
                    Ok(res) => {
                        self.record_history(&res);
                        return Ok(res);
                    }
                    Err(LinguaError::OfflineModelMissing(_)) => {
                        // Continue to other engines if any
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // If cloud was not allowed and lexicon didn't contain the term
        if !req.allow_cloud {
            return Err(LinguaError::OfflineModelMissing(format!(
                "Text '{}' for pair {} is not available in the offline dictionary. Enable online translation or download additional models in Stage 2.",
                cleaned, req.pair
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
                assert!(detail.contains("offline dictionary"));
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
