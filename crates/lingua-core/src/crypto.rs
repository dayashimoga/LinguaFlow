use crate::error::LinguaError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Fixed 32-byte Ed25519 public key for LinguaFlow Official Model Authority.
pub const LINGUAFLOW_MODEL_AUTHORITY_PUBKEY: &str =
    "a4d7b328905b768e7ec839a8c7b801a610f443b7e61e05d0cb0047e157778da9";

/// Fixed 32-byte Ed25519 secret seed used by LinguaFlow authority to sign official model manifests.
pub const LINGUAFLOW_MODEL_AUTHORITY_TEST_SEED: [u8; 32] = [
    0x4c, 0x69, 0x6e, 0x67, 0x75, 0x61, 0x46, 0x6c, 0x6f, 0x77, 0x41, 0x75, 0x74, 0x68, 0x6f, 0x72,
    0x69, 0x74, 0x79, 0x53, 0x65, 0x65, 0x64, 0x56, 0x31, 0x30, 0x32, 0x30, 0x32, 0x36, 0x21, 0x24,
];

/// Offline Entitlement Tiers for LinguaFlow platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementTier {
    Free,
    Pro,
    Enterprise,
}

impl std::fmt::Display for EntitlementTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntitlementTier::Free => write!(f, "Free"),
            EntitlementTier::Pro => write!(f, "Pro"),
            EntitlementTier::Enterprise => write!(f, "Enterprise"),
        }
    }
}

/// Offline License Token payload verified cryptographically via Ed25519.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseClaims {
    pub org_id: String,
    pub org_name: String,
    pub tier: EntitlementTier,
    pub max_devices: u32,
    pub expires_at_timestamp: i64,
    pub zero_log_required: bool,
    pub features: Vec<String>,
}

/// Generate Ed25519 signature in hex format using a 32-byte secret key seed.
pub fn sign_ed25519(data: &[u8], secret_key_hex: &str) -> Result<String, LinguaError> {
    let secret_bytes = hex::decode(secret_key_hex)
        .map_err(|e| LinguaError::Validation(format!("Invalid hex string for secret key: {e}")))?;

    if secret_bytes.len() != 32 {
        return Err(LinguaError::Validation(format!(
            "Secret key must be 32 bytes, got {}",
            secret_bytes.len()
        )));
    }

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&secret_bytes);
    let signing_key = SigningKey::from_bytes(&seed);
    let sig: Signature = signing_key.sign(data);
    Ok(hex::encode(sig.to_bytes()))
}

/// Verify an Ed25519 signature in hex format against raw data and public key in hex format.
pub fn verify_ed25519(
    data: &[u8],
    signature_hex: &str,
    pubkey_hex: &str,
) -> Result<bool, LinguaError> {
    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| LinguaError::Validation(format!("Invalid hex string for public key: {e}")))?;

    if pubkey_bytes.len() != 32 {
        return Err(LinguaError::Validation(format!(
            "Public key must be 32 bytes, got {}",
            pubkey_bytes.len()
        )));
    }

    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&key_arr).map_err(|e| {
        LinguaError::Validation(format!("Failed to parse Ed25519 verifying key: {e}"))
    })?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| LinguaError::Validation(format!("Invalid hex string for signature: {e}")))?;

    let sig = Signature::from_slice(&sig_bytes)
        .map_err(|e| LinguaError::Validation(format!("Invalid Ed25519 signature format: {e}")))?;

    match verifying_key.verify(data, &sig) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Deterministically derive the official public key from a 32-byte seed.
pub fn get_pubkey_from_seed(seed: &[u8; 32]) -> String {
    let signing_key = SigningKey::from_bytes(seed);
    hex::encode(signing_key.verifying_key().to_bytes())
}

/// Sign a license claim structure and return a tamper-evident token: `base64(json).hex(sig)`.
pub fn issue_license_token(claims: &LicenseClaims, seed: &[u8; 32]) -> Result<String, LinguaError> {
    let json = serde_json::to_string(claims)
        .map_err(|e| LinguaError::Validation(format!("Failed to serialize license claims: {e}")))?;

    let secret_hex = hex::encode(seed);
    let sig_hex = sign_ed25519(json.as_bytes(), &secret_hex)?;

    let payload_hex = hex::encode(json.as_bytes());
    Ok(format!("{payload_hex}.{sig_hex}"))
}

/// Verify an offline license token using an Ed25519 public key.
pub fn verify_license_token(token: &str, pubkey_hex: &str) -> Result<LicenseClaims, LinguaError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return Err(LinguaError::Validation(
            "Malformed license token format; expected payload.signature".into(),
        ));
    }

    let payload_bytes = hex::decode(parts[0]).map_err(|e| {
        LinguaError::Validation(format!("Invalid hex payload in license token: {e}"))
    })?;

    let is_valid = verify_ed25519(&payload_bytes, parts[1], pubkey_hex)?;
    if !is_valid {
        return Err(LinguaError::Validation(
            "Cryptographic signature check failed on license token".into(),
        ));
    }

    let claims: LicenseClaims = serde_json::from_slice(&payload_bytes).map_err(|e| {
        LinguaError::Validation(format!("Failed to deserialize license claims: {e}"))
    })?;

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_and_verify_roundtrip() {
        let seed = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
        let secret_hex = hex::encode(seed);

        let message = b"LinguaFlow Model Pack Manifest Verification b94d27b9934d";
        let sig_hex = sign_ed25519(message, &secret_hex).expect("Signing failed");

        let is_valid = verify_ed25519(message, &sig_hex, &pubkey_hex).expect("Verify failed");
        assert!(is_valid, "Valid signature must verify successfully");

        let is_invalid = verify_ed25519(b"tampered message", &sig_hex, &pubkey_hex)
            .expect("Verify failed on tamper");
        assert!(!is_invalid, "Tampered message must fail verification");
    }

    #[test]
    fn test_official_authority_pubkey_derivation() {
        let official_pubkey = get_pubkey_from_seed(&LINGUAFLOW_MODEL_AUTHORITY_TEST_SEED);
        assert!(!official_pubkey.is_empty());
        assert_eq!(official_pubkey.len(), 64);
    }

    #[test]
    fn test_license_token_offline_verification() {
        let seed = LINGUAFLOW_MODEL_AUTHORITY_TEST_SEED;
        let pubkey = get_pubkey_from_seed(&seed);

        let claims = LicenseClaims {
            org_id: "org_enterprise_apollo".into(),
            org_name: "Apollo Healthcare Systems".into(),
            tier: EntitlementTier::Enterprise,
            max_devices: 50,
            expires_at_timestamp: 1893456000, // 2030-01-01
            zero_log_required: true,
            features: vec![
                "offline_neural_translation".into(),
                "hipaa_audit_trail".into(),
                "priority_queue".into(),
            ],
        };

        let token = issue_license_token(&claims, &seed).expect("License issuance failed");
        let verified = verify_license_token(&token, &pubkey).expect("License verification failed");

        assert_eq!(verified.org_name, "Apollo Healthcare Systems");
        assert_eq!(verified.tier, EntitlementTier::Enterprise);
        assert!(verified.zero_log_required);
    }
}
