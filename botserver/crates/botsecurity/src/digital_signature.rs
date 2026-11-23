//! Digital-signature primitives for the security facade.
//!
//! Verification is cryptographic and fail-closed. A document is reported
//! `Valid` only when an **authoritative** Ed25519 public key — registered out
//! of band for the signer, never supplied by the party being verified — proves
//! the signature over the SHA-256 digest of the **real** document.
//!
//! Nothing carried in the request itself is treated as proof: hashes and the
//! certificate chain are inputs to the check, and a caller-supplied
//! `validation_status` is ignored entirely (#2902).

use std::collections::HashMap;
use std::fmt;
use std::sync::{OnceLock, RwLock};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use ring::signature::{self as ring_signature, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

const ED25519_PUBLIC_KEY_LEN: usize = 32;
const ED25519_SIGNATURE_LEN: usize = 64;
const SHA256_DIGEST_LEN: usize = 32;

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
    /// Never trusted as input; verification always overwrites this.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureError {
    MalformedHash,
    HashMismatch,
    MalformedSignature,
    MissingSignature,
    MissingAuthoritativeKey,
    InvalidKey,
    InvalidSignature,
    UnsupportedAlgorithm,
    LockPoisoned,
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MalformedHash => "document hash is not a SHA-256 digest",
            Self::HashMismatch => "document hash fields are inconsistent",
            Self::MalformedSignature => "signature is not a valid base64 Ed25519 signature",
            Self::MissingSignature => "signature is missing",
            Self::MissingAuthoritativeKey => "no authoritative key registered for signer",
            Self::InvalidKey => "authoritative key must be a 32-byte Ed25519 public key",
            Self::InvalidSignature => "signature does not verify against the document",
            Self::UnsupportedAlgorithm => "unsupported signature algorithm",
            Self::LockPoisoned => "authoritative key store is unavailable",
        };
        f.write_str(message)
    }
}

impl std::error::Error for SignatureError {}

pub struct SignatureService {
    pub default_algorithm: SignatureAlgorithm,
    /// signer_id -> authoritative Ed25519 public key (raw 32 bytes).
    keys: HashMap<Uuid, Vec<u8>>,
}

impl SignatureService {
    pub fn new() -> Self {
        Self {
            default_algorithm: SignatureAlgorithm::Ed25519,
            keys: HashMap::new(),
        }
    }

    /// Register the authoritative public key for a signer. This is a trusted
    /// provisioning operation (boot / Vault / PKI); request payloads can only
    /// refer to a key by `signer_id`, never supply key material themselves.
    pub fn register_key(
        &mut self,
        signer_id: Uuid,
        public_key: &[u8],
    ) -> Result<(), SignatureError> {
        if public_key.len() != ED25519_PUBLIC_KEY_LEN {
            return Err(SignatureError::InvalidKey);
        }
        self.keys.insert(signer_id, public_key.to_vec());
        Ok(())
    }

    pub fn hash_document(content: &[u8]) -> (String, String) {
        let digest = Sha256::digest(content);
        let sha256 = hex_encode(&digest);
        // The SHA-512 field is the hash of the SHA-256 *digest bytes* —
        // `signed_message()` re-derives it by decoding the paired sha256 hex,
        // so both producers must use the same rule or genuine signatures fail
        // with HashMismatch (#2902).
        let sha512 = hex_encode(&Sha512::digest(&digest));
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

    /// Binds caller-supplied signature material to a real request and returns
    /// the *verified* status. An unverifiable signature can never come back as
    /// `Valid`; without a registered key the result is `Unknown`.
    pub fn complete_signature(
        &self,
        request: SignatureRequest,
        signature_value: String,
        certificate_chain: Vec<String>,
    ) -> Result<SignedDocument, SignatureError> {
        let mut signed = Self::build_signed_document(&request, signature_value, certificate_chain)?;
        signed.validation_status = self.validate(&signed);
        Ok(signed)
    }

    fn build_signed_document(
        request: &SignatureRequest,
        signature_value: String,
        certificate_chain: Vec<String>,
    ) -> Result<SignedDocument, SignatureError> {
        let sha512 = Self::sha512_of_digest(&request.document_hash)?;
        Ok(SignedDocument {
            id: Uuid::new_v4(),
            signature_request_id: request.id,
            document_id: request.document_id,
            signer_id: request.signer_id,
            document_hash_sha256: request.document_hash.clone(),
            document_hash_sha512: sha512,
            signed_at: Utc::now(),
            algorithm: request.algorithm.clone(),
            signature_value,
            certificate_chain,
            timestamp_token: None,
            ocsp_response: None,
            validation_status: ValidationStatus::Unknown,
        })
    }

    fn sha512_of_digest(sha256_hex: &str) -> Result<String, SignatureError> {
        let digest = hex_decode(sha256_hex)?;
        if digest.len() != SHA256_DIGEST_LEN {
            return Err(SignatureError::MalformedHash);
        }
        let mut hasher = Sha512::new();
        hasher.update(&digest);
        Ok(hex_encode(&hasher.finalize()))
    }

    /// The exact bytes covered by the signature: the raw SHA-256 digest of the
    /// document. The paired SHA-512 field must agree with it, so a swapped or
    /// tampered document hash is rejected before any signature check.
    fn signed_message(signed: &SignedDocument) -> Result<Vec<u8>, SignatureError> {
        let digest = hex_decode(&signed.document_hash_sha256)?;
        if digest.len() != SHA256_DIGEST_LEN {
            return Err(SignatureError::MalformedHash);
        }
        let expected_sha512 = Self::sha512_of_digest(&signed.document_hash_sha256)?;
        if expected_sha512 != signed.document_hash_sha512 {
            return Err(SignatureError::HashMismatch);
        }
        Ok(digest)
    }

    /// Cryptographic verdict. The `validation_status` carried on `signed` is
    /// deliberately ignored — a caller cannot assert its own validity.
    pub fn validate(&self, signed: &SignedDocument) -> ValidationStatus {
        match self.verify(signed) {
            Ok(()) => ValidationStatus::Valid,
            Err(SignatureError::MissingAuthoritativeKey) => ValidationStatus::Unknown,
            Err(_) => ValidationStatus::Invalid,
        }
    }

    fn verify(&self, signed: &SignedDocument) -> Result<(), SignatureError> {
        if signed.algorithm != SignatureAlgorithm::Ed25519 {
            return Err(SignatureError::UnsupportedAlgorithm);
        }
        if signed.signature_value.is_empty() {
            return Err(SignatureError::MissingSignature);
        }
        let public_key = self
            .keys
            .get(&signed.signer_id)
            .ok_or(SignatureError::MissingAuthoritativeKey)?;
        let signature_bytes = BASE64
            .decode(signed.signature_value.as_bytes())
            .map_err(|_| SignatureError::MalformedSignature)?;
        if signature_bytes.len() != ED25519_SIGNATURE_LEN {
            return Err(SignatureError::MalformedSignature);
        }
        let message = Self::signed_message(signed)?;
        UnparsedPublicKey::new(&ring_signature::ED25519, public_key.as_slice())
            .verify(&message, &signature_bytes)
            .map_err(|_| SignatureError::InvalidSignature)
    }
}

impl Default for SignatureService {
    fn default() -> Self {
        Self::new()
    }
}

// ── Process-wide authoritative key store ───────────────────────────────────
// The HTTP facade is registered without an AppState, so trusted boot code
// provisions keys here once and the handlers look them up by signer_id.

static SHARED_KEYS: OnceLock<RwLock<SignatureService>> = OnceLock::new();

fn shared_service() -> &'static RwLock<SignatureService> {
    SHARED_KEYS.get_or_init(|| RwLock::new(SignatureService::new()))
}

/// Provision an authoritative Ed25519 public key at boot. Never call this
/// from a request handler: key material must never arrive from a client.
pub fn register_authoritative_key(
    signer_id: Uuid,
    public_key: &[u8],
) -> Result<(), SignatureError> {
    shared_service()
        .write()
        .map_err(|_| SignatureError::LockPoisoned)?
        .register_key(signer_id, public_key)
}

pub fn validate_with_shared(signed: &SignedDocument) -> ValidationStatus {
    match shared_service().read() {
        Ok(service) => service.validate(signed),
        Err(_) => ValidationStatus::Unknown,
    }
}

pub fn complete_with_shared(
    request: SignatureRequest,
    signature_value: String,
    certificate_chain: Vec<String>,
) -> Result<SignedDocument, SignatureError> {
    let mut signed =
        SignatureService::build_signed_document(&request, signature_value, certificate_chain)?;
    signed.validation_status = validate_with_shared(&signed);
    Ok(signed)
}

// ── encoding helpers ────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn hex_decode(input: &str) -> Result<Vec<u8>, SignatureError> {
    if !input.len().is_multiple_of(2) {
        return Err(SignatureError::MalformedHash);
    }
    let mut decoded = Vec::with_capacity(input.len() / 2);
    for chunk in input.as_bytes().chunks(2) {
        let pair = std::str::from_utf8(chunk).map_err(|_| SignatureError::MalformedHash)?;
        decoded.push(u8::from_str_radix(pair, 16).map_err(|_| SignatureError::MalformedHash)?);
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests;
