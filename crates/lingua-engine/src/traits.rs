use async_trait::async_trait;
use lingua_core::{CapabilityState, LanguageCode, LanguagePair, LinguaError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationRequest {
    pub pair: LanguagePair,
    pub text: String,
    pub allow_cloud: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationResponse {
    pub pair: LanguagePair,
    pub original_text: String,
    pub translated_text: String,
    pub engine_id: String,
    pub is_offline: bool,
    pub latency_ms: u64,
}

#[async_trait]
pub trait TranslationEngine: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn supports_pair(&self, pair: &LanguagePair) -> bool;
    fn capability_state(&self, pair: &LanguagePair) -> CapabilityState;
    async fn translate(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse, LinguaError>;
}

#[async_trait]
pub trait SpeechToTextEngine: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn supports_language(&self, lang: LanguageCode) -> bool;
    fn capability_state(&self, lang: LanguageCode) -> CapabilityState;
}

#[async_trait]
pub trait TextToSpeechEngine: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn supports_language(&self, lang: LanguageCode) -> bool;
    fn capability_state(&self, lang: LanguageCode) -> CapabilityState;
}
