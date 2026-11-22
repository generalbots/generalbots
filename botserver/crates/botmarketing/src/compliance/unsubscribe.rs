//! Signed one-click unsubscribe (#2536).
//!
//! The endpoint is anonymous by design (a mail client cannot present a bearer
//! token). CSRF protection is the HMAC token: it is issued by this server,
//! bound to a `(branch_id, channel, address)` claim and cannot be forged or
//! replayed for another recipient. The token never expires because an opt-out
//! link in an old email must keep working.
//!
//! Recording an opt-out is idempotent: a repeated call returns `200` with
//! `newly_added = false`.

use axum::{extract::Path, extract::State, http::StatusCode, Json};
use chrono::Utc;
use diesel::prelude::*;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

use super::suppression::{
    persist_suppression, MarketingChannel, SuppressionKey, SuppressionStore,
};
use crate::click_sign;
use crate::schema::{marketing_contacts, marketing_recipients};
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Domain separation: an unsubscribe token can never be a valid click token.
const KEY_DERIVATION_LABEL: &[u8] = b"general-bots/marketing-unsubscribe/v1";

/// The claim carried by a signed unsubscribe link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsubscribeClaims {
    pub branch_id: Uuid,
    pub channel: MarketingChannel,
    pub address: String,
}

/// Why a token was rejected. All variants map to a caller-safe `400`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsubscribeError {
    MalformedToken,
    BadSignature,
    SigningUnavailable,
    MissingAddress,
    StorageUnavailable,
}

impl UnsubscribeError {
    pub fn reason(self) -> &'static str {
        match self {
            Self::MalformedToken => "malformed unsubscribe token",
            Self::BadSignature => "unsubscribe signature does not match",
            Self::SigningUnavailable => "unsubscribe signing key unavailable",
            Self::MissingAddress => "missing recipient address",
            Self::StorageUnavailable => "suppression store unavailable",
        }
    }
}

/// Typed result of recording an opt-out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeResult {
    pub success: bool,
    pub newly_added: bool,
    pub channel: String,
    pub branch_id: Uuid,
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(message);
    let bytes = mac.finalize().into_bytes();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

/// Derived from the persisted signing key used by click tracking, with a
/// separate domain label. An empty base means "fail closed": no token can be
/// issued or accepted until the deployment secret is configured.
fn signing_key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(|| {
        let base = click_sign::signing_key();
        if base.is_empty() {
            log::error!("unsubscribe key unavailable: no signing secret configured");
            return String::new();
        }
        hmac_sha256(base.as_bytes(), KEY_DERIVATION_LABEL)
            .map(hex::encode)
            .unwrap_or_default()
    })
    .as_str()
}

/// Signs claims with an explicit key. Exposed so token lifecycle can be
/// tested without depending on a configured deployment secret.
pub fn sign_claims_with_key(key: &str, claims: &UnsubscribeClaims) -> Option<String> {
    if key.is_empty() || claims.address.trim().is_empty() {
        return None;
    }
    let payload = hex::encode(serde_json::to_vec(claims).ok()?);
    let signature = hmac_sha256(key.as_bytes(), payload.as_bytes())?;
    Some(format!("{}.{payload}", hex::encode(signature)))
}

/// Creates the opaque token embedded in the unsubscribe URL.
pub fn issue_unsubscribe_token(claims: &UnsubscribeClaims) -> Option<String> {
    sign_claims_with_key(signing_key(), claims)
}

/// Verifies a token against the process signing key. Exposed separately so
/// callers can test malformed/tampered tokens without a running server.
pub fn verify_unsubscribe_token(
    key: &str,
    token: &str,
) -> Result<UnsubscribeClaims, UnsubscribeError> {
    if key.is_empty() {
        return Err(UnsubscribeError::SigningUnavailable);
    }
    let (signature, payload) = token
        .trim()
        .split_once('.')
        .ok_or(UnsubscribeError::MalformedToken)?;
    if signature.is_empty() || payload.is_empty() {
        return Err(UnsubscribeError::MalformedToken);
    }
    let expected = hmac_sha256(key.as_bytes(), payload.as_bytes())
        .ok_or(UnsubscribeError::SigningUnavailable)?;
    if !botsecurity_core::constant_time::eq_str(&hex::encode(expected), signature) {
        return Err(UnsubscribeError::BadSignature);
    }
    let bytes = hex::decode(payload).map_err(|_| UnsubscribeError::MalformedToken)?;
    let claims: UnsubscribeClaims =
        serde_json::from_slice(&bytes).map_err(|_| UnsubscribeError::MalformedToken)?;
    if claims.address.trim().is_empty() {
        return Err(UnsubscribeError::MissingAddress);
    }
    Ok(claims)
}

/// First-party fallback when the deployment has no `server-url` configured.
/// It is a fixed constant, never a request `Host` or caller-supplied URL.
pub const FIRST_PARTY_UNSUBSCRIBE_BASE: &str = "https://chat.pragmatismo.com.br";

/// Rejects anything that is not an absolute http(s) URL without credentials.
fn is_safe_base_url(raw: &str) -> bool {
    if raw.is_empty() || raw == "stub" {
        return false;
    }
    match url::Url::parse(raw) {
        Ok(parsed) => {
            matches!(parsed.scheme(), "http" | "https")
                && parsed.host_str().is_some()
                && parsed.username().is_empty()
                && parsed.password().is_none()
        }
        Err(_) => false,
    }
}

/// Resolves the base URL for first-party marketing links. `configured` comes
/// from the deployment config; `env_self` is the operator-set self URL. Both
/// are validated — an unsafe value is replaced by a fixed first-party URL.
pub fn resolve_unsubscribe_base(configured: &str, env_self: Option<&str>) -> String {
    let configured = configured.trim().trim_end_matches('/');
    if is_safe_base_url(configured) {
        return configured.to_string();
    }
    if let Some(env_base) = env_self {
        let env_base = env_base.trim().trim_end_matches('/');
        if is_safe_base_url(env_base) {
            return env_base.to_string();
        }
    }
    FIRST_PARTY_UNSUBSCRIBE_BASE.to_string()
}

/// Convenience wrapper reading the operator-configured self URL from the env.
pub fn unsubscribe_base_url(configured: &str) -> String {
    let env_self = match std::env::var("GB_SELF_URL") {
        Ok(value) => value,
        Err(error) => {
            log::warn!("GB_SELF_URL unavailable, using first-party fallback: {error}");
            String::new()
        }
    };
    resolve_unsubscribe_base(configured, Some(env_self.as_str()))
}

/// Absolute URL placed in the `List-Unsubscribe` header.
pub fn build_unsubscribe_url(base_url: &str, claims: &UnsubscribeClaims) -> Option<String> {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return None;
    }
    let token = issue_unsubscribe_token(claims)?;
    Some(format!("{base}/api/marketing/unsubscribe/{token}"))
}

/// RFC 8058 header pair for marketing email. The 5th argument of
/// `SendEmailFn` is the raw header block; the integrator's provider adapter
/// must forward it (see the report for the remaining wiring gap).
pub fn unsubscribe_headers(url: &str) -> String {
    format!(
        "List-Unsubscribe: <{url}>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click"
    )
}

/// Best-effort durable opt-out signals. The suppression store is the
/// authoritative immediate gate; these updates keep opt-outs effective after
/// a restart even before a `marketing_suppressions` migration exists.
fn persist_opt_out(state: &Arc<AppState>, claims: &UnsubscribeClaims) {
    let Ok(mut conn) = state.conn.get() else {
        log::error!("unsubscribe: DB pool unavailable; opt-out kept in memory only");
        return;
    };
    // Authoritative durable record: the in-memory store alone would forget
    // this opt-out on restart. Unique index makes the write idempotent.
    let key = SuppressionKey::new(claims.branch_id, claims.channel, &claims.address);
    match persist_suppression(&mut conn, &key, "unsubscribed") {
        Ok(_) => log::info!("unsubscribe persisted for {}", claims.channel.as_str()),
        Err(error) => log::error!("unsubscribe could not persist suppression: {error}"),
    }

    let now = Utc::now();
    if let Err(error) = diesel::update(
        marketing_contacts::table
            .filter(marketing_contacts::branch_id.eq(claims.branch_id))
            .filter(marketing_contacts::email.eq(&claims.address)),
    )
    .set((
        marketing_contacts::subscribed.eq(Some(false)),
        marketing_contacts::updated_at.eq(now),
    ))
    .execute(&mut conn)
    {
        log::error!("unsubscribe: could not mark marketing contact opted out: {error}");
    }
    if let Err(error) = diesel::update(
        marketing_recipients::table
            .filter(marketing_recipients::branch_id.eq(claims.branch_id))
            .filter(marketing_recipients::email.eq(&claims.address)),
    )
    .set((
        marketing_recipients::status.eq(Some("unsubscribed".to_string())),
        marketing_recipients::error_message.eq(Some("recipient opted out".to_string())),
    ))
    .execute(&mut conn)
    {
        log::error!("unsubscribe: could not mark recipients opted out: {error}");
    }
}

/// Records an opt-out in the suppression store. Pure with respect to the HTTP
/// layer so idempotency can be tested without a running server.
pub fn record_unsubscribe(claims: &UnsubscribeClaims) -> Result<UnsubscribeResult, UnsubscribeError> {
    let key = SuppressionKey::new(claims.branch_id, claims.channel, &claims.address);
    let outcome = SuppressionStore::global()
        .add(key)
        .map_err(|_| UnsubscribeError::StorageUnavailable)?;
    let channel = claims.channel.as_str().to_string();
    let branch_id = claims.branch_id;
    log::info!("unsubscribe recorded: channel={channel} newly_added={}", outcome.newly_added);
    Ok(UnsubscribeResult {
        success: true,
        newly_added: outcome.newly_added,
        channel,
        branch_id,
    })
}

/// `GET|POST /api/marketing/unsubscribe/:token`
pub async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<UnsubscribeResult>, (StatusCode, String)> {
    let mut claims = verify_unsubscribe_token(signing_key(), &token).map_err(|error| {
        log::warn!("unsubscribe rejected: {}", error.reason());
        (StatusCode::BAD_REQUEST, error.reason().to_string())
    })?;
    // The suppression key normalizes internally; normalize here as well so the
    // durable `marketing_contacts` / `marketing_recipients` updates use the same
    // spelling and the opt-out still blocks sends after a process restart.
    claims.address = claims.address.trim().to_ascii_lowercase();

    let result = record_unsubscribe(&claims).map_err(|error| {
        log::error!("unsubscribe suppression failed: {}", error.reason());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            error.reason().to_string(),
        )
    })?;
    persist_opt_out(&state, &claims);

    Ok(Json(result))
}
