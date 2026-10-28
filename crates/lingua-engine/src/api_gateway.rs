use chrono::Utc;
use lingua_core::{ApiKeyRecord, LinguaError};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Rate Limiter state per API key (token bucket / sliding minute window).
#[derive(Debug, Clone)]
struct RateBucket {
    window_minute: u64,
    request_count: u32,
}

/// Developer API Authentication & Rate Limiting Engine (Phase 8.4).
#[derive(Clone)]
pub struct ApiGateway {
    rate_buckets: Arc<Mutex<HashMap<String, RateBucket>>>,
}

impl Default for ApiGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiGateway {
    pub fn new() -> Self {
        Self {
            rate_buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Compute cryptographic SHA-256 digest of an API key for secure storage.
    pub fn hash_api_key(raw_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a cryptographically random, prefixed live API key.
    pub fn generate_raw_api_key(environment: &str) -> (String, String, String) {
        let prefix = format!("lf_{environment}");
        let secret = uuid::Uuid::new_v4().to_string().replace('-', "");
        let raw_key = format!("{prefix}_{secret}");
        let key_hash = Self::hash_api_key(&raw_key);
        (raw_key, prefix, key_hash)
    }

    /// Check and advance rate limiting for a validated API key.
    pub fn check_rate_limit(&self, key_record: &ApiKeyRecord) -> Result<(), LinguaError> {
        let now_sec = Utc::now().timestamp() as u64;
        let current_window = now_sec / 60;

        let mut buckets = self
            .rate_buckets
            .lock()
            .map_err(|e| LinguaError::Internal(format!("Lock failure: {e}")))?;

        let bucket = buckets.entry(key_record.id.clone()).or_insert(RateBucket {
            window_minute: current_window,
            request_count: 0,
        });

        if bucket.window_minute < current_window {
            bucket.window_minute = current_window;
            bucket.request_count = 0;
        }

        bucket.request_count += 1;

        if bucket.request_count > key_record.rate_limit_per_min {
            let retry_after = 60 - (now_sec % 60);
            return Err(LinguaError::RateLimitExceeded {
                limit_per_minute: key_record.rate_limit_per_min,
                retry_after_secs: retry_after as u32,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_generation_and_hashing() {
        let (raw_key, prefix, key_hash) = ApiGateway::generate_raw_api_key("live");
        assert!(raw_key.starts_with("lf_live_"));
        assert_eq!(prefix, "lf_live");
        assert_eq!(key_hash.len(), 64);
        assert_eq!(ApiGateway::hash_api_key(&raw_key), key_hash);
    }

    #[test]
    fn test_token_bucket_rate_limiter() {
        let gateway = ApiGateway::new();
        let key_record = ApiKeyRecord {
            id: "key-test-1".into(),
            user_id: "user-1".into(),
            org_id: None,
            key_hash: "hash123".into(),
            key_prefix: "lf_live".into(),
            name: "Test Key".into(),
            rate_limit_per_min: 5,
            is_active: true,
            created_at: Utc::now(),
            expires_at: None,
        };

        // First 5 requests must pass
        for _ in 0..5 {
            assert!(gateway.check_rate_limit(&key_record).is_ok());
        }

        // 6th request must be rejected with RateLimitExceeded
        let overflow = gateway.check_rate_limit(&key_record);
        assert!(overflow.is_err());
        match overflow {
            Err(LinguaError::RateLimitExceeded {
                limit_per_minute, ..
            }) => {
                assert_eq!(limit_per_minute, 5);
            }
            _ => panic!("Expected RateLimitExceeded"),
        }
    }
}
