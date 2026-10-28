use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use lingua_core::{CapabilityMatrix, CapabilityState, LinguaError};
use std::sync::Arc;
use tracing::info;

pub struct EngineRegistry {
    translation_engines: Vec<Arc<dyn TranslationEngine>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            translation_engines: Vec::new(),
        }
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
        let mut matrix = CapabilityMatrix::stage_0_baseline();

        if !self.translation_engines.is_empty() {
            matrix.text_translation = CapabilityState::Ready;
        }

        matrix
    }

    pub async fn translate(
        &self,
        req: TranslationRequest,
    ) -> Result<TranslationResponse, LinguaError> {
        // Enforce language pair validity
        let _ = req.pair;

        for engine in &self.translation_engines {
            if engine.supports_pair(&req.pair) {
                return engine.translate(req).await;
            }
        }

        // Global Engineering Contract Mandate:
        // Do not simulate or fake translation when no real engine is active.
        Err(LinguaError::CapabilityUnavailable(
            "text_translation".into(),
            format!(
                "Translation engine for pair '{}' is scheduled for Stage 1. No mock or fake translation permitted.",
                req.pair
            ),
        ))
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
    async fn test_stage_0_baseline_rejects_unimplemented_translation_without_faking() {
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
            Err(LinguaError::CapabilityUnavailable(cap, detail)) => {
                assert_eq!(cap, "text_translation");
                assert!(detail.contains("Stage 1"));
            }
            _ => panic!("Expected CapabilityUnavailable error"),
        }
    }

    #[tokio::test]
    async fn test_isolated_engine_registration() {
        let mut registry = EngineRegistry::new();
        registry.register_translation_engine(Arc::new(TestDummyEngine));

        let caps = registry.get_capabilities();
        assert_eq!(caps.text_translation, CapabilityState::Ready);

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
}
