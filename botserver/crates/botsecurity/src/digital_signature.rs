use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    SHA256withRSA,
    SHA512withRSA,
    SHA256withECDSA,
    SHA512withECDSA,
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Uuid,
    pub document_hash: String,
    pub algorithm: SignatureAlgorithm,
    pub certificate_id: Option<Uuid>,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: SignatureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureStatus {
    Pending,
    Signed,
    Declined,
    Expired,
    Revoked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDocument {
    pub id: Uuid,
    pub signature_request_id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Uuid,
    pub document_hash_sha256: String,
    pub document_hash_sha512: String,
    pub signed_at: DateTime<Utc>,
    pub algorithm: SignatureAlgorithm,
    pub signature_value: String,
    pub certificate_chain: Vec<String>,
    pub timestamp_token: Option<String>,
    pub ocsp_response: Option<String>,
    pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Unknown,
    Expired,
    Revoked,
}

pub struct SignatureService {
    pub default_algorithm: SignatureAlgorithm,
}

impl SignatureService {
    pub fn new() -> Self {
        Self {
            default_algorithm: SignatureAlgorithm::SHA512withECDSA,
        }
    }

    pub fn hash_document(content: &[u8]) -> (String, String) {
        let sha256 = {
            let mut h = Sha256::new();
            h.update(content);
            hex_encode(&h.finalize())
        };
        let sha512 = {
            let mut h = Sha512::new();
            h.update(content);
            hex_encode(&h.finalize())
        };
        (sha256, sha512)
    }

    pub fn create_request(
        &self,
        tenant_id: Uuid,
        document_id: Uuid,
        signer_id: Uuid,
        document_content: &[u8],
        certificate_id: Option<Uuid>,
        reason: Option<String>,
    ) -> SignatureRequest {
        let (sha256, _sha512) = Self::hash_document(document_content);
        SignatureRequest {
            id: Uuid::new_v4(),
            tenant_id,
            document_id,
            signer_id,
            document_hash: sha256,
            algorithm: self.default_algorithm.clone(),
            certificate_id,
            reason,
            location: None,
            contact_info: None,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
            status: SignatureStatus::Pending,
        }
    }

    pub fn complete_signature(
        &self,
        request: SignatureRequest,
        signature_value: String,
        certificate_chain: Vec<String>,
    ) -> SignedDocument {
        let (_, sha512) = Self::hash_from_hash(&request.document_hash);
        SignedDocument {
            id: Uuid::new_v4(),
            signature_request_id: request.id,
            document_id: request.document_id,
            signer_id: request.signer_id,
            document_hash_sha256: request.document_hash.clone(),
            document_hash_sha512: sha512,
            signed_at: Utc::now(),
            algorithm: request.algorithm,
            signature_value,
            certificate_chain,
            timestamp_token: None,
            ocsp_response: None,
            validation_status: ValidationStatus::Valid,
        }
    }

    fn hash_from_hash(prev: &str) -> (String, String) {
        let bytes = hex_decode(prev).unwrap_or_default();
        let sha512 = {
            let mut h = Sha512::new();
            h.update(&bytes);
            hex_encode(&h.finalize())
        };
        (prev.to_string(), sha512)
    }

    pub fn validate(signed: &SignedDocument) -> ValidationStatus {
        if signed.validation_status != ValidationStatus::Valid {
            return signed.validation_status.clone();
        }
        if signed.signature_value.is_empty() {
            return ValidationStatus::Invalid;
        }
        if signed.certificate_chain.is_empty() {
            return ValidationStatus::Unknown;
        }
        ValidationStatus::Valid
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks(2) {
        let hex = std::str::from_utf8(chunk).map_err(|_| ())?;
        out.push(u8::from_str_radix(hex, 16).map_err(|_| ())?);
    }
    Ok(out)
}

impl Default for SignatureService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_produces_64_byte_sha512() {
        let (_, sha512) = SignatureService::hash_document(b"hello");
        assert_eq!(sha512.len(), 128);
    }

    #[test]
    fn complete_signature_is_valid() {
        let svc = SignatureService::new();
        let req = svc.create_request(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            b"contract",
            None,
            Some("Approval".into()),
        );
        let signed = svc.complete_signature(req, "sig-abc".into(), vec!["cert1".into()]);
        assert_eq!(SignatureService::validate(&signed), ValidationStatus::Valid);
    }
}
