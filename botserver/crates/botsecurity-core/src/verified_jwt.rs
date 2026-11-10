//! Verified JWT identity (issue #2123).
//!
//! Handlers that need the caller identity out of a Bearer token used to
//! base64url-decode the payload and trust `sub`/`user_id`. Nothing checked the
//! signature, so a forged `{"alg":"none"}` token — or one signed with any
//! other key — identified the caller as an arbitrary user. This module is the
//! single place that turns a token into claims, and it refuses everything it
//! cannot verify:
//!
//! * the header must declare `HS256` (`none` and asymmetric algorithms are
//!   refused, so an unsigned token can never be accepted);
//! * the HMAC-SHA256 signature over `header.payload` must match, compared in
//!   constant time;
//! * the payload must carry a non-empty `sub` (or legacy `user_id`) and an
//!   `exp` that has not passed.
//!
//! Role claims (`role`/`roles`) are surfaced only after the signature checks
//! pass, so authorization decisions that read [`VerifiedJwt::roles`] can never
//! be driven by a payload the caller wrote (issues #2112/#2125).
//!
//! Cloud-login tokens (`botcloud::api`) and suite JWTs (`JwtManager`) are both
//! HS256 tokens signed with the persisted `saas_jwt_secret`, so one verifier
//! covers both. Opaque suite tokens (`gb_*`) are not JWTs and are resolved
//! through the session cache by their consumers instead.

use axum::http::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Claims of a token whose signature and expiry have been verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedJwt {
    /// Verified identity subject (Zitadel user id or a UUID).
    pub sub: String,
    pub email: Option<String>,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    /// Roles the issuer signed into the token (`role`, plus the `roles`
    /// array), trimmed and lowercased, without duplicates.
    ///
    /// Empty when the token carries no role claim — cloud-login tokens do
    /// not (#736), so callers resolve the role from RBAC in that case. A
    /// non-empty list is authoritative: it was signed by the server.
    pub roles: Vec<String>,
    /// Expiry in epoch seconds (always present on a verified token).
    pub exp: i64,
}

/// Why a token was refused. Callers must never fall back to the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtVerifyError {
    /// Not three dot-separated segments.
    Malformed,
    /// The header is not valid JSON or declares no algorithm.
    InvalidHeader,
    /// Only `HS256` is minted by this server; anything else is refused.
    UnsupportedAlgorithm,
    /// Signature mismatch — forged, tampered, or signed with another key.
    InvalidSignature,
    /// No signing secret is configured.
    MissingSecret,
    /// The payload is not valid JSON.
    InvalidPayload,
    /// No usable `sub`/`user_id` claim.
    MissingSubject,
    /// No `exp` claim; an unbounded token cannot be trusted.
    MissingExpiry,
    /// The `exp` claim is in the past.
    Expired,
    /// The `nbf` claim is in the future.
    NotYetValid,
}

/// Clock-skew tolerance applied to `nbf`, in seconds.
const NBF_LEEWAY_SECONDS: i64 = 60;

impl std::fmt::Display for JwtVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Malformed => "malformed token",
            Self::InvalidHeader => "invalid token header",
            Self::UnsupportedAlgorithm => "unsupported token algorithm",
            Self::InvalidSignature => "invalid token signature",
            Self::MissingSecret => "no token signing secret configured",
            Self::InvalidPayload => "invalid token payload",
            Self::MissingSubject => "token carries no identity claim",
            Self::MissingExpiry => "token carries no expiry",
            Self::Expired => "token expired",
            Self::NotYetValid => "token is not valid yet",
        };
        f.write_str(text)
    }
}

impl std::error::Error for JwtVerifyError {}

/// The Bearer token carried by the `Authorization` header.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    auth.strip_prefix("Bearer ")
        .map(|token| token.split(',').next().unwrap_or(token).trim())
}

fn decode_segment(segment: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(segment.trim_end_matches('='))
        .ok()
}

/// Verifies an HS256 JWT against an explicit secret and returns its claims.
///
/// This is the only function that may produce a [`VerifiedJwt`]; the secret
/// overload exists so tests (and callers holding the secret already) verify
/// deterministically without touching the process environment.
pub fn verify_hs256_jwt(token: &str, secret: &str) -> Result<VerifiedJwt, JwtVerifyError> {
    if secret.is_empty() {
        return Err(JwtVerifyError::MissingSecret);
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtVerifyError::Malformed);
    }
    let (header_b64, payload_b64, signature_b64) = (parts[0], parts[1], parts[2]);

    let header_raw = decode_segment(header_b64).ok_or(JwtVerifyError::InvalidHeader)?;
    let header: serde_json::Value =
        serde_json::from_slice(&header_raw).map_err(|_| JwtVerifyError::InvalidHeader)?;
    match header.get("alg").and_then(|v| v.as_str()) {
        Some("HS256") => {}
        Some(_) => return Err(JwtVerifyError::UnsupportedAlgorithm),
        None => return Err(JwtVerifyError::InvalidHeader),
    }

    let signature = decode_segment(signature_b64).ok_or(JwtVerifyError::InvalidSignature)?;
    let signing_input = format!("{header_b64}.{payload_b64}");
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| JwtVerifyError::MissingSecret)?;
    mac.update(signing_input.as_bytes());
    // `verify_slice` compares in constant time.
    mac.verify_slice(&signature)
        .map_err(|_| JwtVerifyError::InvalidSignature)?;

    let payload_raw = decode_segment(payload_b64).ok_or(JwtVerifyError::InvalidPayload)?;
    let payload: serde_json::Value =
        serde_json::from_slice(&payload_raw).map_err(|_| JwtVerifyError::InvalidPayload)?;

    let sub = payload
        .get("sub")
        .or_else(|| payload.get("user_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(JwtVerifyError::MissingSubject)?
        .to_string();

    let exp = payload
        .get("exp")
        .and_then(|v| v.as_i64())
        .ok_or(JwtVerifyError::MissingExpiry)?;
    let now = chrono::Utc::now().timestamp();
    if now >= exp {
        return Err(JwtVerifyError::Expired);
    }
    // #2112 — `nbf` is honoured when present. A 60-second leeway absorbs clock
    // skew between the minting host and this process, the same tolerance the
    // suite `JwtConfig` uses.
    if let Some(nbf) = payload.get("nbf").and_then(|v| v.as_i64()) {
        if nbf > now.saturating_add(NBF_LEEWAY_SECONDS) {
            return Err(JwtVerifyError::NotYetValid);
        }
    }

    Ok(VerifiedJwt {
        sub,
        email: string_claim(&payload, "email"),
        // The legacy `organization_id` alias is honoured for the same claim;
        // both spellings name the `.gborg` tenant.
        org_id: uuid_claim(&payload, "org_id")
            .or_else(|| uuid_claim(&payload, "organization_id")),
        branch_id: uuid_claim(&payload, "branch_id"),
        roles: role_claims(&payload),
        exp,
    })
}

/// The role claims of a verified payload, normalized for comparison.
fn role_claims(payload: &serde_json::Value) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();
    if let Some(role) = payload.get("role").and_then(|v| v.as_str()) {
        push_role(&mut roles, role);
    }
    if let Some(list) = payload.get("roles").and_then(|v| v.as_array()) {
        for entry in list.iter().filter_map(|v| v.as_str()) {
            push_role(&mut roles, entry);
        }
    }
    roles
}

fn push_role(roles: &mut Vec<String>, raw: &str) {
    let role = raw.trim().to_lowercase();
    if !role.is_empty() && !roles.contains(&role) {
        roles.push(role);
    }
}

fn string_claim(payload: &serde_json::Value, name: &str) -> Option<String> {
    payload
        .get(name)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn uuid_claim(payload: &serde_json::Value, name: &str) -> Option<Uuid> {
    payload
        .get(name)
        .and_then(|v| v.as_str())
        .and_then(|value| Uuid::parse_str(value.trim()).ok())
}

/// The single resolver for the SaaS JWT signing secret (#2140).
///
/// Re-exported here because this module is the documented JWT entry point; the
/// implementation lives in [`crate::jwt_secret`] so every crate — minter and
/// verifier alike — resolves the same value and a public fallback cannot be
/// reintroduced by a copy.
pub use crate::jwt_secret::{
    resolve_saas_jwt_secret, JwtSecret, JwtSecretError, JwtSecretPolicy, JwtSecretSource,
};

/// Verifies a token against the server's SaaS JWT secret.
///
/// The secret is resolved once per process by
/// [`crate::jwt_secret::process_saas_jwt_secret`]: the login path and every
/// verifier therefore observe the same value, and a process that could not
/// obtain a secret fails closed with [`JwtVerifyError::MissingSecret`] instead
/// of accepting a token signed with a public default.
pub fn verify_saas_jwt(token: &str) -> Result<VerifiedJwt, JwtVerifyError> {
    let secret = crate::jwt_secret::process_saas_jwt_secret()
        .map_err(|_| JwtVerifyError::MissingSecret)?;
    verify_hs256_jwt(token, secret)
}

#[cfg(test)]
#[path = "verified_jwt_tests.rs"]
mod tests;

/// Test-only token minting, shared by the verifier and tenant tests.
///
/// Tokens are signed with the secret the process resolves, so a test exercises
/// exactly the accept/reject boundary production uses.
#[cfg(test)]
pub(crate) mod test_support {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    /// Mints an HS256 token the way the login path does.
    pub(crate) fn mint_hs256(claims: &serde_json::Value, secret: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
        let signing_input = format!("{header}.{payload}");
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return String::new();
        };
        mac.update(signing_input.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{signing_input}.{signature}")
    }

    /// The secret the verifier uses in this process.
    pub(crate) fn process_secret() -> String {
        super::resolve_saas_jwt_secret().expect("saas jwt secret resolves in tests")
    }

    /// A valid token for `claims`, signed with the process secret.
    pub(crate) fn signed(claims: &serde_json::Value) -> String {
        mint_hs256(claims, &process_secret())
    }

    /// Claims that carry every tenant field plus a live expiry.
    pub(crate) fn tenant_claims(
        branch: &str,
        org: &str,
        sub: &str,
        email: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "branch_id": branch,
            "org_id": org,
            "sub": sub,
            "email": email,
            "exp": chrono::Utc::now().timestamp() + 3600,
        })
    }
}

