//! Signed bounce capabilities (#2886).
//!
//! The bounce webhook is anonymous (a mail provider cannot present a bearer
//! token), so the request itself must carry a capability that only this server
//! can issue. The capability is an HMAC-SHA256 over the canonical claim that
//! decides the state change — `(branch_id, recipient_id, address, channel,
//! bounce_type, reason)`. A forged or transplanted body therefore cannot
//! suppress a contact, and a capability issued for tenant A cannot be replayed
//! against tenant B: changing any claim changes the signature.
//!
//! The key is derived from the same persisted deployment secret as click
//! tracking and unsubscribe, with a distinct domain label, so one key rotation
//! covers all three features and no key is reused across them.

use axum::http::StatusCode;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::OnceLock;
use uuid::Uuid;

use super::bounce::BounceWebhookPayload;
use crate::click_sign;

type HmacSha256 = Hmac<Sha256>;

/// Domain separation: a bounce capability can never be a valid click or
/// unsubscribe token even though all three share the base deployment secret.
const KEY_DERIVATION_LABEL: &[u8] = b"general-bots/marketing-bounce/v1";

/// The claim covered by a bounce capability. Every field that can change the
/// suppression decision is included so none of them can be tampered with in
/// transit. `branch_id` is the tenant the capability was issued for and is the
/// only tenant an update may touch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BounceClaims {
    pub branch_id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub address: Option<String>,
    pub channel: Option<String>,
    pub reason: Option<String>,
    pub bounce_type: Option<String>,
}

impl BounceClaims {
    /// Builds the claim from the wire body. Only the signature-bearing request
    /// forwarded by [`verify_bounce_signature`] becomes authoritative.
    fn from_payload(body: &BounceWebhookPayload) -> Self {
        Self {
            branch_id: body.branch_id.unwrap_or_else(Uuid::nil),
            recipient_id: body.recipient_id,
            address: body.address.clone(),
            channel: body.channel.clone(),
            reason: body.reason.clone(),
            bounce_type: body.bounce_type.clone(),
        }
    }

    fn validate(&self) -> Result<(), SignatureError> {
        if self.branch_id.is_nil() {
            return Err(SignatureError::MissingTenant);
        }
        let has_recipient = self.recipient_id.is_some();
        let has_address = self
            .address
            .as_deref()
            .map(|address| !address.trim().is_empty())
            .unwrap_or(false);
        if !has_recipient && !has_address {
            return Err(SignatureError::MissingIdentifier);
        }
        Ok(())
    }
}

/// Why a bounce request was rejected. `MissingCapability` is unauthenticated
/// (401); everything else is a present but unusable capability (403). No
/// variant echoes caller input, so reasons are safe to return verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureError {
    MissingCapability,
    MalformedCapability,
    InvalidSignature,
    SigningUnavailable,
    MissingTenant,
    MissingIdentifier,
    TenantMismatch,
}

impl SignatureError {
    pub fn status(self) -> StatusCode {
        match self {
            Self::MissingCapability => StatusCode::UNAUTHORIZED,
            Self::MalformedCapability
            | Self::InvalidSignature
            | Self::SigningUnavailable
            | Self::MissingTenant
            | Self::MissingIdentifier
            | Self::TenantMismatch => StatusCode::FORBIDDEN,
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::MissingCapability => "missing bounce capability",
            Self::MalformedCapability => "malformed bounce capability",
            Self::InvalidSignature => "bounce signature does not match",
            Self::SigningUnavailable => "bounce signing key unavailable",
            Self::MissingTenant => "bounce capability has no tenant",
            Self::MissingIdentifier => "recipient_id or address is required",
            Self::TenantMismatch => "bounce capability tenant does not match recipient",
        }
    }
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(message);
    let bytes = mac.finalize().into_bytes();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

/// Process-wide bounce capability key, derived from the persisted secret.
/// An empty base means "fail closed": no capability can be signed or accepted
/// until the deployment secret is configured.
pub fn signing_key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(|| {
        let base = click_sign::signing_key();
        if base.is_empty() {
            log::error!("bounce capability key unavailable: no signing secret configured");
            return String::new();
        }
        match hmac_sha256(base.as_bytes(), KEY_DERIVATION_LABEL) {
            Some(digest) => hex::encode(digest),
            None => String::new(),
        }
    })
    .as_str()
}

/// Signs a claim with an explicit key. Exposed so the sending path can issue
/// capabilities and tests can exercise the lifecycle without a deployment
/// secret. Returns `None` (fail closed) for an empty key.
pub fn sign_bounce_with_key(key: &str, claims: &BounceClaims) -> Option<String> {
    if key.is_empty() {
        return None;
    }
    let payload = hex::encode(serde_json::to_vec(claims).ok()?);
    hmac_sha256(key.as_bytes(), payload.as_bytes()).map(hex::encode)
}

/// Canonicalizes the wire body into a claim and verifies its HMAC in constant
/// time. No caller is identified by the body alone: the returned claim is the
/// only value the webhook handler may use for the state change.
pub fn verify_bounce_signature(
    key: &str,
    body: &BounceWebhookPayload,
) -> Result<BounceClaims, SignatureError> {
    let signature = body
        .signature
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(SignatureError::MissingCapability)?;
    if signature.len() != 64 || !signature.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SignatureError::MalformedCapability);
    }
    if key.is_empty() {
        return Err(SignatureError::SigningUnavailable);
    }
    let claims = BounceClaims::from_payload(body);
    let expected = sign_bounce_with_key(key, &claims).ok_or(SignatureError::SigningUnavailable)?;
    if !botsecurity_core::constant_time::eq_str(&expected, signature) {
        return Err(SignatureError::InvalidSignature);
    }
    claims.validate()?;
    Ok(claims)
}

/// Confirms that the recipient row the capability selected belongs to the
/// tenant named by the capability. The authoritative branch always comes from
/// the persisted recipient mapping, never from the request body.
pub fn authorized_branch(
    claims: &BounceClaims,
    recipient_branch: Uuid,
) -> Result<Uuid, SignatureError> {
    if claims.branch_id != recipient_branch {
        return Err(SignatureError::TenantMismatch);
    }
    Ok(recipient_branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "unit-test-bounce-key-not-a-production-secret";

    fn sample_claims(branch: Uuid, recipient: Uuid) -> BounceClaims {
        BounceClaims {
            branch_id: branch,
            recipient_id: Some(recipient),
            address: Some("victim@example.test".to_string()),
            channel: Some("email".to_string()),
            reason: Some("550 user unknown".to_string()),
            bounce_type: Some("hard".to_string()),
        }
    }

    fn payload(claims: &BounceClaims, signature: Option<String>) -> BounceWebhookPayload {
        BounceWebhookPayload {
            recipient_id: claims.recipient_id,
            branch_id: Some(claims.branch_id),
            address: claims.address.clone(),
            channel: claims.channel.clone(),
            reason: claims.reason.clone(),
            bounce_type: claims.bounce_type.clone(),
            signature,
        }
    }

    #[test]
    fn valid_capability_verifies_and_is_bound_to_claims() {
        let tenant = Uuid::new_v4();
        let recipient = Uuid::new_v4();
        let claims = sample_claims(tenant, recipient);
        let signature = sign_bounce_with_key(KEY, &claims).expect("test key signs");
        let body = payload(&claims, Some(signature));

        let verified = verify_bounce_signature(KEY, &body).expect("valid capability verifies");
        assert_eq!(verified, claims);
        assert_eq!(authorized_branch(&verified, tenant), Ok(tenant));
    }

    #[test]
    fn forged_signature_is_rejected() {
        let tenant = Uuid::new_v4();
        let claims = sample_claims(tenant, Uuid::new_v4());
        let body = payload(
            &claims,
            Some("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string()),
        );

        assert_eq!(
            verify_bounce_signature(KEY, &body),
            Err(SignatureError::InvalidSignature)
        );
    }

    #[test]
    fn malformed_signature_is_rejected() {
        let claims = sample_claims(Uuid::new_v4(), Uuid::new_v4());
        let body = payload(&claims, Some("not-a-hex-hmac".to_string()));
        let error = verify_bounce_signature(KEY, &body).expect_err("malformed capability");
        assert_eq!(error, SignatureError::MalformedCapability);
        assert_eq!(error.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn two_tenant_tamper_is_rejected() {
        // A capability issued for tenant A cannot be replayed for tenant B:
        // changing branch_id (or recipient_id) changes the signed claim.
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let claims_a = sample_claims(tenant_a, Uuid::new_v4());
        let signature = sign_bounce_with_key(KEY, &claims_a).expect("test key signs");

        let mut forged_for_b = claims_a.clone();
        forged_for_b.branch_id = tenant_b;
        let body_b = payload(&forged_for_b, Some(signature));

        assert_eq!(
            verify_bounce_signature(KEY, &body_b),
            Err(SignatureError::InvalidSignature)
        );
    }

    #[test]
    fn bounce_type_tamper_is_rejected() {
        let claims = sample_claims(Uuid::new_v4(), Uuid::new_v4());
        let signature = sign_bounce_with_key(KEY, &claims).expect("test key signs");

        let mut forged = claims.clone();
        forged.bounce_type = Some("soft".to_string());
        let body = payload(&forged, Some(signature));

        assert_eq!(
            verify_bounce_signature(KEY, &body),
            Err(SignatureError::InvalidSignature)
        );
    }

    #[test]
    fn missing_capability_is_unauthorized() {
        let claims = sample_claims(Uuid::new_v4(), Uuid::new_v4());
        let body = payload(&claims, None);
        let error = verify_bounce_signature(KEY, &body).expect_err("no capability");
        assert_eq!(error, SignatureError::MissingCapability);
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn cross_tenant_recipient_is_rejected() {
        // Even a correctly signed capability cannot suppress a recipient whose
        // persisted row belongs to a different tenant.
        let claim_tenant = Uuid::new_v4();
        let other_tenant = Uuid::new_v4();
        let claims = sample_claims(claim_tenant, Uuid::new_v4());
        assert_eq!(
            authorized_branch(&claims, other_tenant),
            Err(SignatureError::TenantMismatch)
        );
    }

    #[test]
    fn empty_key_fails_closed() {
        let claims = sample_claims(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(sign_bounce_with_key("", &claims), None);
        let signature = sign_bounce_with_key(KEY, &claims).expect("test key signs");
        let body = payload(&claims, Some(signature));
        assert_eq!(
            verify_bounce_signature("", &body),
            Err(SignatureError::SigningUnavailable)
        );
    }

    #[test]
    fn capability_without_tenant_or_identifier_is_rejected() {
        let tenant = Uuid::new_v4();
        let mut no_tenant = sample_claims(tenant, Uuid::new_v4());
        no_tenant.branch_id = Uuid::nil();
        let signature = sign_bounce_with_key(KEY, &no_tenant).expect("test key signs");
        let body = payload(&no_tenant, Some(signature));
        assert_eq!(
            verify_bounce_signature(KEY, &body),
            Err(SignatureError::MissingTenant)
        );

        let mut no_id = sample_claims(tenant, Uuid::new_v4());
        no_id.recipient_id = None;
        no_id.address = None;
        let signature = sign_bounce_with_key(KEY, &no_id).expect("test key signs");
        let body = payload(&no_id, Some(signature));
        assert_eq!(
            verify_bounce_signature(KEY, &body),
            Err(SignatureError::MissingIdentifier)
        );
    }
}
