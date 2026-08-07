//! Branch-scope resolution for CRM write handlers.
//!
//! Data is owned by the branch (workspace) which is owned by the org (the
//! `.gborg` tenant). When an authenticated request carries a JWT whose email
//! maps to a CRM contact, the branch is derived from that contact so writes
//! land in the caller's own workspace — never in an arbitrary "default" bot
//! branch (issue #730). When no JWT/email is present, callers fall back to
//! their crate-level default context.

use axum::http::HeaderMap;
use diesel::prelude::*;
use uuid::Uuid;

/// Minimal URL-safe base64 decoder (JWT payloads). Returns `None` on invalid
/// input so a malformed header never aborts a request.
fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input.as_bytes() {
        let v = match TABLE.iter().position(|&x| x == b) {
            Some(i) => i as u32,
            None => return None,
        };
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

/// Extracts the `email` claim from a Bearer JWT Authorization header.
pub fn email_from_jwt(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = base64url_decode(parts[1])?;
    let json: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    json.get("email").and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extracts the user email from an opaque suite session token (`gb_*`).
/// The suite UI authenticates with a random opaque token that the auth
/// middleware resolves via the session cache (populated at login); the
/// cached entry carries the real user email used to scope CRM data.
pub fn email_from_session(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    if token.contains('.') {
        return None;
    }
    botsecurity_core::lookup_session_cache(token).map(|u| u.email)
}

/// Resolves a suite user's email from the `X-User-ID` header used by the
/// chat/WhatsApp loopback executor (`api.exec`). The loopback hop carries no
/// Authorization header, so the account is identified by id only; its email
/// comes from the `users` table.
pub fn email_from_user_id(headers: &HeaderMap, conn: &mut diesel::PgConnection) -> Option<String> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        email: String,
    }
    let uid = headers.get("x-user-id")?.to_str().ok()?;
    let uid = uuid::Uuid::parse_str(uid).ok()?;
    diesel::sql_query("SELECT email FROM users WHERE id = $1 LIMIT 1")
        .bind::<SqlUuid, _>(uid)
        .get_result::<Row>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.email)
}

/// Resolves the branch id for the authenticated user: looks up the JWT email
/// (or the session-cache email for opaque suite tokens, or the `X-User-ID`
/// email for the chat loopback) in `crm_contacts` and returns that contact's
/// `branch_id`.
pub fn branch_from_jwt(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Option<Uuid> {
    let email = email_from_jwt(headers)
        .or_else(|| email_from_session(headers))
        .or_else(|| email_from_user_id(headers, conn))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    diesel::sql_query(
        "SELECT branch_id FROM crm_contacts WHERE email = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<Row>(conn)
    .optional()
    .ok()
    .flatten()
    .map(|r| r.branch_id)
}

/// Resolves the branch for the caller using a connection from the pool,
/// for handlers that hold a `DbPool` instead of a live connection.
pub fn branch_from_jwt_pool(
    headers: &HeaderMap,
    pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> Option<Uuid> {
    let mut conn = pool.get().ok()?;
    branch_from_jwt(headers, &mut conn)
}
