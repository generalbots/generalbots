//! Service assertion for the `api.exec` loopback hop (#2122).
//!
//! The executor performs a real HTTP request back into the running server so
//! every call goes through the same auth, RBAC and CSRF layers as the browser.
//! It used to identify the caller with an `X-User-ID` header, which the auth
//! middleware accepted as a logged-in identity — so anyone who could reach the
//! API port could impersonate any user by setting one header.
//!
//! The hop now presents a short-lived HS256 JWT signed with the server's own
//! secret (`saas_jwt_secret`, the same one cloud login mints with). The auth
//! middleware verifies the signature and expiry before any handler runs, so
//! the assertion cannot be forged or replayed after it expires.

use std::sync::Arc;

use botcore::shared::state::AppState;
use serde_json::json;
use uuid::Uuid;

/// Lifetime of one assertion. The executor consumes it immediately after
/// minting, so this only bounds clock skew and in-flight requests.
const ASSERTION_TTL_SECS: i64 = 120;

/// Audience of the assertion, carried as a custom claim (`aud` is a registered
/// claim that the shared JWT validators reject when they expect no audience).
const ASSERTION_AUDIENCE: &str = "api-exec";

/// Mints the signed service assertion for `user_id`.
///
/// Claims: `sub` (the session user), `email` (tenant/drive scoping), `bot_id`
/// (when the call is bot-scoped), `gb_aud`/`scope` (this executor only), `iat`
/// and `exp`.
pub(super) fn mint(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Option<Uuid>,
) -> Result<String, String> {
    // #2140 — one resolver, failing closed: without a secret this refuses to
    // mint rather than signing with a public constant.
    let secret = botsecurity_core::verified_jwt::resolve_saas_jwt_secret()
        .map_err(|error| format!("service assertion: {error}"))?;
    let now = chrono::Utc::now().timestamp();
    let claims = json!({
        "sub": user_id.to_string(),
        "email": user_email(state, user_id).unwrap_or_default(),
        "bot_id": bot_uuid.map(|b| b.to_string()),
        "gb_aud": ASSERTION_AUDIENCE,
        "scope": ASSERTION_AUDIENCE,
        "iat": now,
        "exp": now + ASSERTION_TTL_SECS,
    });

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| format!("service assertion: {error}"))
}

/// The caller's account email, used by the tenant/drive scope resolution that
/// previously read it from the `X-User-ID` header.
fn user_email(state: &Arc<AppState>, user_id: Uuid) -> Option<String> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
    use diesel::{OptionalExtension, RunQueryDsl};

    #[derive(diesel::QueryableByName)]
    struct EmailRow {
        #[diesel(sql_type = Text)]
        email: String,
    }

    let mut conn = state.conn.get().ok()?;
    let row: Option<EmailRow> = diesel::sql_query("SELECT email FROM users WHERE id = $1 LIMIT 1")
        .bind::<SqlUuid, _>(user_id)
        .get_result::<EmailRow>(&mut conn)
        .optional()
        .ok()
        .flatten();
    let email = row?.email.trim().to_string();
    if email.is_empty() {
        return None;
    }
    Some(email)
}
