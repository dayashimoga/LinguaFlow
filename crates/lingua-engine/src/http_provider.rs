use crate::traits::{TranslationEngine, TranslationRequest, TranslationResponse};
use async_trait::async_trait;
use lingua_core::{normalize_text_nfc, CapabilityState, LanguageCode, LanguagePair, LinguaError};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct MyMemoryResponseData {
    #[serde(rename = "translatedText")]
    translated_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MyMemoryResponse {
    #[serde(rename = "responseData")]
    response_data: Option<MyMemoryResponseData>,
    #[serde(rename = "responseDetails")]
    response_details: Option<String>,
}

pub struct HttpTranslationProvider {
    client: Client,
    endpoint: Option<String>,
    timeout: Duration,
}

impl HttpTranslationProvider {
    pub fn new(endpoint: Option<String>, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            endpoint,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn lang_code_to_api(code: LanguageCode) -> &'static str {
        code.short_code()
    }

    fn decode_html_entities(text: &str) -> String {
        text.replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
    }

    async fn execute_request(&self, req: &TranslationRequest) -> Result<String, LinguaError> {
        let src_code = Self::lang_code_to_api(req.pair.source);
        let tgt_code = Self::lang_code_to_api(req.pair.target);

        // Check for custom endpoint (e.g. self-hosted LibreTranslate)
        let endpoint_url = self
            .endpoint
            .clone()
            .or_else(|| std::env::var("LINGUAFLOW_TRANSLATE_ENDPOINT").ok())
            .unwrap_or_else(|| "https://api.mymemory.translated.net/get".to_string());

        let url = format!(
            "{}?q={}&langpair={}|{}",
            endpoint_url,
            urlencoding::encode(&req.text),
            src_code,
            tgt_code
        );

        let mut attempts = 0;
        let max_attempts = 2;
        let mut last_err = String::new();

        while attempts < max_attempts {
            attempts += 1;
            let resp_result = self
                .client
                .get(&url)
                .header(
                    "User-Agent",
                    "LinguaFlow/0.1.0 (Production Translation Client)",
                )
                .timeout(self.timeout)
                .send()
                .await;

            match resp_result {
                Ok(resp) => {
                    let status = resp.status();
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        last_err = format!("HTTP provider returned status {}: {}", status, body);
                        warn!("Attempt {}: {}", attempts, last_err);
                        tokio::time::sleep(Duration::from_millis(200 * attempts as u64)).await;
                        continue;
                    }

                    let parsed: MyMemoryResponse = resp.json().await.map_err(|e| {
                        LinguaError::TranslationFailed(format!(
                            "Failed to parse provider response: {e}"
                        ))
                    })?;

                    if let Some(data) = parsed.response_data {
                        if let Some(text) = data.translated_text {
                            if !text.trim().is_empty() {
                                return Ok(Self::decode_html_entities(&text));
                            }
                        }
                    }

                    let details = parsed
                        .response_details
                        .unwrap_or_else(|| "Unknown provider error".into());
                    return Err(LinguaError::TranslationFailed(format!(
                        "Translation provider did not return translated text: {details}"
                    )));
                }
                Err(e) => {
                    last_err = format!("Network failure to {}: {e}", endpoint_url);
                    warn!("Attempt {}: {}", attempts, last_err);
                    if attempts < max_attempts {
                        tokio::time::sleep(Duration::from_millis(250 * attempts as u64)).await;
                    }
                }
            }
        }

        warn!(
            "Translation request exhausted {} attempts. Last error: {}",
            max_attempts, last_err
        );
        Err(LinguaError::NetworkUnavailable)
    }
}

impl Default for HttpTranslationProvider {
    fn default() -> Self {
        Self::new(None, 5)
    }
}

#[async_trait]
impl TranslationEngine for HttpTranslationProvider {
    fn id(&self) -> &'static str {
        "http-rest-provider"
    }

    fn name(&self) -> &'static str {
        "Verified HTTP Translation Provider"
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

        info!(
            "Executing HTTP translation for '{}' ({})",
            cleaned, request.pair
        );
        let translated = self.execute_request(&request).await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(TranslationResponse {
            pair: request.pair,
            original_text: request.text,
            translated_text: translated,
            engine_id: self.id().into(),
            is_offline: false,
            latency_ms: latency_ms.max(1),
        })
    }
}

// Minimal urlencoding utility to avoid extra external crates
mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut encoded = String::with_capacity(data.len() * 3);
        for byte in data.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => encoded.push_str("%20"),
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_html_entities() {
        let raw = "&quot;Hello&quot; &amp; welcome &#39;world&#39;";
        let decoded = HttpTranslationProvider::decode_html_entities(raw);
        assert_eq!(decoded, "\"Hello\" & welcome 'world'");
    }

    #[test]
    fn test_urlencode() {
        let input = "Hello world! ನಮಸ್ಕಾರ";
        let enc = urlencoding::encode(input);
        assert!(enc.starts_with("Hello%20world%21%20"));
    }

    #[tokio::test]
    async fn test_http_provider_network_unavailable_handling() {
        // Test with invalid endpoint that immediately fails connection
        let provider = HttpTranslationProvider::new(
            Some("http://127.0.0.1:59999/non-existent-translate".into()),
            1,
        );
        let pair = LanguagePair::new(LanguageCode::EnUs, LanguageCode::KnIn).unwrap();
        let req = TranslationRequest {
            pair,
            text: "Hello".into(),
            allow_cloud: true,
        };

        let result = provider.translate(req).await;
        assert!(result.is_err());
        match result {
            Err(LinguaError::NetworkUnavailable) => {}
            other => panic!("Expected NetworkUnavailable, got {:?}", other),
        }
    }
}
