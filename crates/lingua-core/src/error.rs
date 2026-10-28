use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum LinguaError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid language pair: source '{0}' cannot be identical to target '{1}'")]
    IdenticalLanguagePair(String, String),

    #[error("Language '{0}' is unsupported for capability '{1}'")]
    UnsupportedLanguage(String, String),

    #[error("Capability '{0}' is unavailable: {1}")]
    CapabilityUnavailable(String, String),

    #[error("Network connection required for online operation")]
    NetworkUnavailable,

    #[error("Storage budget exceeded: requested {requested_bytes} bytes, but only {available_bytes} bytes available")]
    StorageBudgetExceeded {
        requested_bytes: u64,
        available_bytes: u64,
    },

    #[error("Audio device failure: {0}")]
    AudioDeviceError(String),

    #[error("Microphone permission denied by host OS")]
    AudioPermissionDenied,

    #[error("Streaming ASR engine error: {0}")]
    AsrEngineError(String),

    #[error("Speech session '{0}' not found")]
    SpeechSessionNotFound(String),

    #[error("Model verification failed for '{0}': checksum mismatch")]
    ModelIntegrityFailure(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Translation failed: {0}")]
    TranslationFailed(String),

    #[error("Offline model missing: {0}")]
    OfflineModelMissing(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Download cancelled: {0}")]
    DownloadCancelled(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("TTS engine error: {0}")]
    TtsEngineError(String),

    #[error("Conversation session '{0}' not found")]
    ConversationSessionNotFound(String),

    #[error("Speaker conflict or invalid turn state: {0}")]
    SpeakerConflict(String),

    #[error("Circuit breaker open for provider '{0}'; request diverted to fallback")]
    CircuitBreakerOpen(String),

    #[error("Glossary term '{0}' not found")]
    GlossaryTermNotFound(String),

    #[error("Media decoding error: {0}")]
    MediaDecodingError(String),

    #[error("Media session '{0}' not found")]
    MediaSessionNotFound(String),

    #[error("OCR extraction failed: {0}")]
    OcrError(String),

    #[error("Document parsing failed: {0}")]
    DocumentParsingError(String),

    #[error("Document translation job '{0}' not found")]
    DocumentJobNotFound(String),

    #[error(
        "File size limit exceeded: file is {requested_bytes} bytes, limit is {limit_bytes} bytes"
    )]
    FileSizeLimitExceeded {
        requested_bytes: u64,
        limit_bytes: u64,
    },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Learning deck '{0}' not found")]
    LearningDeckNotFound(String),

    #[error("Learning card '{0}' not found")]
    LearningCardNotFound(String),

    #[error("Security or privacy violation: {0}")]
    SecurityViolation(String),

    #[error("Privacy policy has not been accepted by user")]
    PrivacyPolicyNotAccepted,

    #[error("Data archive export failed: {0}")]
    DataExportError(String),

    #[error("User data purge failed: {0}")]
    DataPurgeError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Unauthorized request: {0}")]
    Unauthorized(String),

    #[error("Forbidden: insufficient permissions ({0})")]
    Forbidden(String),

    #[error("Usage quota exceeded for {usage_type}: current={current}, limit={limit}")]
    QuotaExceeded {
        usage_type: String,
        limit: u64,
        current: u64,
    },

    #[error(
        "Rate limit exceeded: max {limit_per_minute} req/min. Retry after {retry_after_secs}s"
    )]
    RateLimitExceeded {
        limit_per_minute: u32,
        retry_after_secs: u32,
    },

    #[error("Organization '{0}' not found")]
    OrganizationNotFound(String),

    #[error("Workspace member '{0}' not found")]
    WorkspaceMemberNotFound(String),

    #[error("Shared glossary '{0}' not found")]
    SharedGlossaryNotFound(String),

    #[error("API key '{0}' not found or inactive")]
    ApiKeyNotFound(String),

    #[error("Worker capacity exceeded: {0}")]
    WorkerCapacityExceeded(String),

    #[error("Anonymous mode restriction: {0}")]
    AnonymousModeRestriction(String),

    #[error("Internal system error: {0}")]
    Internal(String),
}

impl LinguaError {
    pub fn code(&self) -> &'static str {
        match self {
            LinguaError::Database(_) => "DATABASE_ERROR",
            LinguaError::IdenticalLanguagePair(_, _) => "INVALID_LANGUAGE_PAIR",
            LinguaError::UnsupportedLanguage(_, _) => "UNSUPPORTED_LANGUAGE",
            LinguaError::CapabilityUnavailable(_, _) => "CAPABILITY_UNAVAILABLE",
            LinguaError::NetworkUnavailable => "NETWORK_UNAVAILABLE",
            LinguaError::StorageBudgetExceeded { .. } => "STORAGE_BUDGET_EXCEEDED",
            LinguaError::AudioDeviceError(_) => "AUDIO_DEVICE_ERROR",
            LinguaError::AudioPermissionDenied => "AUDIO_PERMISSION_DENIED",
            LinguaError::AsrEngineError(_) => "ASR_ENGINE_ERROR",
            LinguaError::SpeechSessionNotFound(_) => "SPEECH_SESSION_NOT_FOUND",
            LinguaError::ModelIntegrityFailure(_) => "MODEL_INTEGRITY_FAILURE",
            LinguaError::Validation(_) => "VALIDATION_ERROR",
            LinguaError::TranslationFailed(_) => "TRANSLATION_FAILED",
            LinguaError::OfflineModelMissing(_) => "OFFLINE_MODEL_MISSING",
            LinguaError::ModelNotFound(_) => "MODEL_NOT_FOUND",
            LinguaError::DownloadCancelled(_) => "DOWNLOAD_CANCELLED",
            LinguaError::DownloadFailed(_) => "DOWNLOAD_FAILED",
            LinguaError::NotFound(_) => "NOT_FOUND",
            LinguaError::TtsEngineError(_) => "TTS_ENGINE_ERROR",
            LinguaError::ConversationSessionNotFound(_) => "CONVERSATION_SESSION_NOT_FOUND",
            LinguaError::SpeakerConflict(_) => "SPEAKER_CONFLICT",
            LinguaError::CircuitBreakerOpen(_) => "CIRCUIT_BREAKER_OPEN",
            LinguaError::GlossaryTermNotFound(_) => "GLOSSARY_TERM_NOT_FOUND",
            LinguaError::MediaDecodingError(_) => "MEDIA_DECODING_ERROR",
            LinguaError::MediaSessionNotFound(_) => "MEDIA_SESSION_NOT_FOUND",
            LinguaError::OcrError(_) => "OCR_ERROR",
            LinguaError::DocumentParsingError(_) => "DOCUMENT_PARSING_ERROR",
            LinguaError::DocumentJobNotFound(_) => "DOCUMENT_JOB_NOT_FOUND",
            LinguaError::FileSizeLimitExceeded { .. } => "FILE_SIZE_LIMIT_EXCEEDED",
            LinguaError::PermissionDenied(_) => "PERMISSION_DENIED",
            LinguaError::LearningDeckNotFound(_) => "LEARNING_DECK_NOT_FOUND",
            LinguaError::LearningCardNotFound(_) => "LEARNING_CARD_NOT_FOUND",
            LinguaError::SecurityViolation(_) => "SECURITY_VIOLATION",
            LinguaError::PrivacyPolicyNotAccepted => "PRIVACY_POLICY_NOT_ACCEPTED",
            LinguaError::DataExportError(_) => "DATA_EXPORT_ERROR",
            LinguaError::DataPurgeError(_) => "DATA_PURGE_ERROR",
            LinguaError::AuthenticationFailed(_) => "AUTHENTICATION_FAILED",
            LinguaError::Unauthorized(_) => "UNAUTHORIZED",
            LinguaError::Forbidden(_) => "FORBIDDEN",
            LinguaError::QuotaExceeded { .. } => "QUOTA_EXCEEDED",
            LinguaError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            LinguaError::OrganizationNotFound(_) => "ORGANIZATION_NOT_FOUND",
            LinguaError::WorkspaceMemberNotFound(_) => "WORKSPACE_MEMBER_NOT_FOUND",
            LinguaError::SharedGlossaryNotFound(_) => "SHARED_GLOSSARY_NOT_FOUND",
            LinguaError::ApiKeyNotFound(_) => "API_KEY_NOT_FOUND",
            LinguaError::WorkerCapacityExceeded(_) => "WORKER_CAPACITY_EXCEEDED",
            LinguaError::AnonymousModeRestriction(_) => "ANONYMOUS_MODE_RESTRICTION",
            LinguaError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            LinguaError::NetworkUnavailable
                | LinguaError::CapabilityUnavailable(_, _)
                | LinguaError::Validation(_)
                | LinguaError::TranslationFailed(_)
                | LinguaError::OfflineModelMissing(_)
                | LinguaError::IdenticalLanguagePair(_, _)
                | LinguaError::AudioDeviceError(_)
                | LinguaError::AudioPermissionDenied
                | LinguaError::AsrEngineError(_)
                | LinguaError::SpeechSessionNotFound(_)
                | LinguaError::ModelNotFound(_)
                | LinguaError::DownloadCancelled(_)
                | LinguaError::DownloadFailed(_)
                | LinguaError::NotFound(_)
                | LinguaError::TtsEngineError(_)
                | LinguaError::ConversationSessionNotFound(_)
                | LinguaError::SpeakerConflict(_)
                | LinguaError::CircuitBreakerOpen(_)
                | LinguaError::GlossaryTermNotFound(_)
                | LinguaError::MediaDecodingError(_)
                | LinguaError::MediaSessionNotFound(_)
                | LinguaError::OcrError(_)
                | LinguaError::DocumentParsingError(_)
                | LinguaError::DocumentJobNotFound(_)
                | LinguaError::FileSizeLimitExceeded { .. }
                | LinguaError::PermissionDenied(_)
                | LinguaError::LearningDeckNotFound(_)
                | LinguaError::LearningCardNotFound(_)
                | LinguaError::PrivacyPolicyNotAccepted
                | LinguaError::AuthenticationFailed(_)
                | LinguaError::Unauthorized(_)
                | LinguaError::Forbidden(_)
                | LinguaError::QuotaExceeded { .. }
                | LinguaError::RateLimitExceeded { .. }
                | LinguaError::OrganizationNotFound(_)
                | LinguaError::WorkspaceMemberNotFound(_)
                | LinguaError::SharedGlossaryNotFound(_)
                | LinguaError::ApiKeyNotFound(_)
                | LinguaError::WorkerCapacityExceeded(_)
                | LinguaError::AnonymousModeRestriction(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_and_recoverable() {
        let err = LinguaError::IdenticalLanguagePair("kn-IN".into(), "kn-IN".into());
        assert_eq!(err.code(), "INVALID_LANGUAGE_PAIR");
        assert!(err.is_recoverable());

        let internal_err = LinguaError::Internal("disk full".into());
        assert_eq!(internal_err.code(), "INTERNAL_ERROR");
        assert!(!internal_err.is_recoverable());

        let learn_err = LinguaError::LearningDeckNotFound("deck-1".into());
        assert_eq!(learn_err.code(), "LEARNING_DECK_NOT_FOUND");
        assert!(learn_err.is_recoverable());

        let priv_err = LinguaError::PrivacyPolicyNotAccepted;
        assert_eq!(priv_err.code(), "PRIVACY_POLICY_NOT_ACCEPTED");
        assert!(priv_err.is_recoverable());
    }

    #[test]
    fn test_error_serialization_roundtrip() {
        let err = LinguaError::UnsupportedLanguage("fr-FR".into(), "ASR".into());
        let json = serde_json::to_string(&err).expect("serialization failed");
        let deserialized: LinguaError =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(err, deserialized);
    }
}
