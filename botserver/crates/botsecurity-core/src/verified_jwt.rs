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
        exp,
    })
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

/// Resolves the secret that mints cloud-login and suite JWTs.
///
/// Same chain as `directory_setup::resolve_saas_jwt_secret` in the botserver
/// crate (persisted `directory_config.json` → `SAAS_JWT_SECRET` →
/// `JWT_SECRET` → development default), so a token verified here is the token
/// the login flow produced.
pub fn resolve_saas_jwt_secret() -> String {
    let stack_path = botlib::security::get_stack_path();
    let config_path = format!("{stack_path}/conf/system/directory_config.json");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(secret) = json
                .get("saas_jwt_secret")
                .and_then(|v| v.as_str())
                .filter(|value| !value.is_empty())
            {
                return secret.to_string();
            }
        }
    }
    std::env::var("SAAS_JWT_SECRET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("JWT_SECRET")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "dev-secret-key-change-in-production-minimum-32-chars".to_string())
}

/// Verifies a token against the server's SaaS JWT secret.
///
/// The secret is resolved once per process: it is persisted into
/// `directory_config.json` during bootstrap before any request is served, and
/// tenant resolution runs on every handler, so re-reading the file per call
/// would add a syscall to each request for a value that cannot change.
pub fn verify_saas_jwt(token: &str) -> Result<VerifiedJwt, JwtVerifyError> {
    verify_hs256_jwt(token, saas_jwt_secret())
}

/// The process-wide secret, resolved once.
fn saas_jwt_secret() -> &'static str {
    static SECRET: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SECRET.get_or_init(resolve_saas_jwt_secret).as_str()
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
        super::saas_jwt_secret().to_string()
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

