use crate::error::LinguaError;
use crate::models::{AppInfo, CapabilityMatrix, ModelPackInfo, UserSettings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcRequest<T> {
    pub request_id: String,
    pub timestamp_ms: u64,
    pub payload: T,
}

impl<T> IpcRequest<T> {
    pub fn new(payload: T) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcResponse<T> {
    pub request_id: String,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<IpcErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcErrorPayload {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

impl From<LinguaError> for IpcErrorPayload {
    fn from(err: LinguaError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.to_string(),
            details: None,
            recoverable: err.is_recoverable(),
        }
    }
}

impl<T> IpcResponse<T> {
    pub fn ok(request_id: impl Into<String>, data: T) -> Self {
        Self {
            request_id: request_id.into(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(request_id: impl Into<String>, err: LinguaError) -> Self {
        Self {
            request_id: request_id.into(),
            success: false,
            data: None,
            error: Some(IpcErrorPayload::from(err)),
        }
    }
}

// Concrete Command Payloads

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAppInfoResponse {
    pub app_info: AppInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetCapabilitiesResponse {
    pub capabilities: CapabilityMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetUserSettingsResponse {
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveUserSettingsRequest {
    pub settings: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListModelPacksResponse {
    pub model_packs: Vec<ModelPackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseHealthInfo {
    pub status: String,
    pub path: String,
    pub schema_version: u32,
    pub settings_persisted: bool,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_envelope_ok_and_err() {
        let ok_resp: IpcResponse<String> = IpcResponse::ok("req-123", "success data".to_string());
        assert!(ok_resp.success);
        assert_eq!(ok_resp.data.as_deref(), Some("success data"));
        assert!(ok_resp.error.is_none());

        let err_resp: IpcResponse<()> = IpcResponse::err(
            "req-456",
            LinguaError::IdenticalLanguagePair("kn-IN".into(), "kn-IN".into()),
        );
        assert!(!err_resp.success);
        assert!(err_resp.data.is_none());
        let err_payload = err_resp.error.unwrap();
        assert_eq!(err_payload.code, "INVALID_LANGUAGE_PAIR");
        assert!(err_payload.recoverable);
    }

    #[test]
    fn test_ipc_request_serialization() {
        let req = IpcRequest::new(GetAppInfoResponse {
            app_info: AppInfo::default(),
        });
        let json = serde_json::to_string(&req).expect("Failed to serialize");
        let deserialized: IpcRequest<GetAppInfoResponse> =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(req.request_id, deserialized.request_id);
    }
}
