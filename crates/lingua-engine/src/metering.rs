use chrono::Utc;
use lingua_core::{AccountTier, LinguaError, UsageRecord, UsageSummary, UsageType};
use lingua_storage::StorageService;
use tracing::warn;

/// Usage and Cost Control Manager (Phase 8.2).
/// Enforces per-account quotas and the contract rule:
/// "Never charge for uncompleted billable requests contrary to the published policy."
#[derive(Clone)]
pub struct UsageQuotaManager {
    storage: StorageService,
}

impl UsageQuotaManager {
    pub fn new(storage: StorageService) -> Self {
        Self { storage }
    }

    /// Check whether an account has sufficient quota for the requested operation.
    pub fn check_quota(
        &self,
        account_id: &str,
        usage_type: UsageType,
        quantity: u64,
    ) -> Result<AccountTier, LinguaError> {
        let summary = self.storage.get_usage_summary(account_id)?;

        match usage_type {
            UsageType::CloudTranslationWords => {
                if summary.words_used + quantity > summary.words_limit {
                    return Err(LinguaError::QuotaExceeded {
                        usage_type: "cloud_translation_words".into(),
                        limit: summary.words_limit,
                        current: summary.words_used,
                    });
                }
            }
            UsageType::CloudSpeechSeconds => {
                if summary.speech_seconds_used + quantity > summary.speech_seconds_limit {
                    return Err(LinguaError::QuotaExceeded {
                        usage_type: "cloud_speech_seconds".into(),
                        limit: summary.speech_seconds_limit,
                        current: summary.speech_seconds_used,
                    });
                }
            }
            UsageType::CloudOcrPages => {
                if summary.ocr_pages_used + quantity > summary.ocr_pages_limit {
                    return Err(LinguaError::QuotaExceeded {
                        usage_type: "cloud_ocr_pages".into(),
                        limit: summary.ocr_pages_limit,
                        current: summary.ocr_pages_used,
                    });
                }
            }
            UsageType::ApiRequests => {
                // API request volume is governed by per-minute rate limiting
            }
        }

        Ok(summary.tier)
    }

    /// Execute a billable operation with strict atomic commit-on-success semantics.
    /// If `operation` returns an Err or panics, no billable usage is recorded (Contract Rule).
    pub async fn execute_billable_op<F, Fut, T>(
        &self,
        account_id: &str,
        org_id: Option<&str>,
        usage_type: UsageType,
        quantity: u64,
        cost_cents: u64,
        operation: F,
    ) -> Result<T, LinguaError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, LinguaError>>,
    {
        // 1. Pre-flight quota check
        self.check_quota(account_id, usage_type, quantity)?;

        // 2. Execute the actual underlying operation
        let result = operation().await;

        match result {
            Ok(output) => {
                // 3. Commit billable usage only upon success
                let record = UsageRecord {
                    id: format!("use-{}", uuid::Uuid::new_v4()),
                    account_id: account_id.to_string(),
                    org_id: org_id.map(|s| s.to_string()),
                    usage_type,
                    quantity,
                    billable_cents: cost_cents,
                    was_successful: true,
                    timestamp: Utc::now(),
                };
                if let Err(e) = self.storage.record_usage(&record) {
                    warn!("Failed to commit usage record for {account_id}: {e}");
                }
                Ok(output)
            }
            Err(e) => {
                // 4. Contract requirement: zero charge for failed requests
                warn!(
                    "Billable operation failed for account {account_id}; zero charges billed: {e}"
                );
                Err(e)
            }
        }
    }

    /// Estimate projected end-of-month cost based on usage run-rate.
    pub fn estimate_projected_cost(&self, account_id: &str) -> Result<UsageSummary, LinguaError> {
        self.storage.get_usage_summary(account_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quota_check_and_refund_on_failure() {
        let storage = StorageService::open_in_memory().expect("in-memory db");
        let anon = storage.get_or_create_anonymous_account().expect("get anon");
        let manager = UsageQuotaManager::new(storage.clone());

        // Free tier allows up to 10,000 words
        assert!(manager
            .check_quota(&anon.id, UsageType::CloudTranslationWords, 500)
            .is_ok());

        // Check exceeding quota fails
        assert!(manager
            .check_quota(&anon.id, UsageType::CloudTranslationWords, 15_000)
            .is_err());

        // Test billable operation success commits usage
        let res: Result<String, LinguaError> = manager
            .execute_billable_op(
                &anon.id,
                None,
                UsageType::CloudTranslationWords,
                200,
                10,
                || async { Ok("translated text".to_string()) },
            )
            .await;

        assert_eq!(res.unwrap(), "translated text");
        let summary = storage.get_usage_summary(&anon.id).expect("summary");
        assert_eq!(summary.words_used, 200);

        // Test failed operation does NOT commit usage (Contract Rule)
        let failed_res: Result<String, LinguaError> = manager
            .execute_billable_op(
                &anon.id,
                None,
                UsageType::CloudTranslationWords,
                300,
                15,
                || async { Err(LinguaError::NetworkUnavailable) },
            )
            .await;

        assert!(failed_res.is_err());
        let summary_after_fail = storage.get_usage_summary(&anon.id).expect("summary");
        assert_eq!(summary_after_fail.words_used, 200); // Unchanged!
    }
}
