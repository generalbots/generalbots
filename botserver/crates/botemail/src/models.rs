use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{
    Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid, Varchar,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;


pub use botlib::db_pool::DbPool;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut PgConnection) -> (Uuid, String) + Send + Sync>;

pub type SecretsProvider = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
    pub secrets_provider: SecretsProvider,
}

#[derive(Debug, QueryableByName)]
pub struct EmailAccountBasicRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
    /// Failure reported by the last sync pass, surfaced next to the account so
    /// an unreachable mailbox is distinguishable from an empty one.
    #[diesel(sql_type = Nullable<Text>)]
    pub last_error: Option<String>,
}

#[derive(Debug, QueryableByName)]
pub struct ImapCredentialsRow {
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
}

#[derive(Debug, QueryableByName)]
pub struct SmtpCredentialsRow {
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Text)]
    pub display_name: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
    #[diesel(sql_type = Text)]
    pub auth_mode: String,
}

#[derive(Debug, QueryableByName)]
pub struct EmailSearchRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub from_address: String,
    #[diesel(sql_type = Text)]
    pub to_addresses: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub body_text: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct EmailSignatureRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    pub bot_id: Option<Uuid>,
    #[diesel(sql_type = Varchar)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub content_html: String,
    #[diesel(sql_type = Text)]
    pub content_plain: String,
    #[diesel(sql_type = Bool)]
    pub is_default: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, QueryableByName)]
pub struct EmailAccountRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Text)]
    pub imap_server: String,
    #[diesel(sql_type = Integer)]
    pub imap_port: i32,
    #[diesel(sql_type = Text)]
    pub smtp_server: String,
    #[diesel(sql_type = Integer)]
    pub smtp_port: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub password_encrypted: String,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

pub struct EmailError(pub String);

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for EmailError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct EmailMessageRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    pub account_id: Uuid,
    #[diesel(sql_type = Nullable<Varchar>)]
    pub message_id_header: Option<String>,
    #[diesel(sql_type = Nullable<Varchar>)]
    pub in_reply_to: Option<String>,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub normalized_subject: String,
    #[diesel(sql_type = Varchar)]
    pub from_address: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub to_addresses: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub body_text: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub body_html: Option<String>,
    #[diesel(sql_type = Bool)]
    pub has_attachments: bool,
    #[diesel(sql_type = Varchar)]
    pub folder: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub uid: i64,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub flags: serde_json::Value,
    #[diesel(sql_type = Bool)]
    pub is_read: bool,
    #[diesel(sql_type = Bool)]
    pub is_flagged: bool,
    #[diesel(sql_type = Timestamptz)]
    pub received_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct EmailAccountColorRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub email: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub display_name: Option<String>,
    #[diesel(sql_type = Bool)]
    pub is_primary: bool,
}

pub struct EmailData {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct EmailSummary {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub from_name: String,
    #[diesel(sql_type = Text)]
    pub from_email: String,
    #[diesel(sql_type = Text)]
    pub subject: String,
    #[diesel(sql_type = Text)]
    pub preview: String,
    #[diesel(sql_type = Text)]
    pub date: String,
    #[diesel(sql_type = Bool)]
    pub read: bool,
}

pub struct EmailContent {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

/// Maps a raw identity subject to the stable database UUID used for mail
/// accounts. Zitadel/OIDC numeric user ids are not valid UUIDs; routing them
/// through `UUIDv5(zitadel:{id})` produces the same deterministic value that
/// the RBAC layer derives, keeping `user_email_accounts` in the caller scope.
pub fn stable_user_uuid(raw: &str) -> Uuid {
    match Uuid::parse_str(raw) {
        Ok(uuid) => uuid,
        Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{raw}").as_bytes()),
    }
}

/// Resolves the authenticated user id from the request headers.
///
/// Supported identities, in priority order:
/// 1. Opaque suite session token (`gb_*`) resolved through the shared session
///    cache populated at login.
/// 2. Bearer JWT — **verified** before any claim is read (#2123). The payload
///    is never parsed on its own: a forged `alg=none` token, a token signed
///    with another key, an expired token or one without `exp` is rejected.
///
/// `X-User-ID` is caller-supplied and is not an identity (#2122): the chat
/// executor authenticates its loopback hop with a short-lived signed JWT.
///
/// Never falls back to `Uuid::nil()` for an authenticated request: if the
/// identity cannot be resolved this fails so callers can reject the request.
pub fn extract_user_from_session(headers: &HeaderMap) -> Result<Uuid, String> {
    extract_user_with_secret(
        headers,
        &botsecurity_core::verified_jwt::resolve_saas_jwt_secret(),
    )
}

/// Identity resolution with an explicit signing secret, so tests can verify
/// tokens deterministically without touching the process environment.
pub(crate) fn extract_user_with_secret(headers: &HeaderMap, secret: &str) -> Result<Uuid, String> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing Authorization header".to_string())?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or_else(|| "Authorization header is not a Bearer token".to_string())?;

    if !token.contains('.') {
        let entry = botsecurity_core::lookup_session_cache(token)
            .ok_or_else(|| "Unknown or expired session token".to_string())?;
        return Ok(stable_user_uuid(&entry.user_id));
    }

    // #2123 — the identity must come from a signature-verified token. Any
    // failure (alg=none, wrong key, tampered payload, expired/missing exp,
    // no subject) rejects the request; the decoded payload is never used as
    // a fallback.
    let claims = botsecurity_core::verified_jwt::verify_hs256_jwt(token, secret)
        .map_err(|e| format!("Invalid JWT token: {e}"))?;
    Ok(stable_user_uuid(&claims.sub))
}

#[derive(Debug, QueryableByName, serde::Serialize)]
pub struct EmailLabelRow {
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Varchar)]
    pub name: String,
    #[diesel(sql_type = Varchar)]
    pub color: String,
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;
