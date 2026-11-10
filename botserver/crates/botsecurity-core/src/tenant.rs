//! Tenant-scope resolution for multi-tenant data isolation (issue #734).
//!
//! All reads and writes of branch-scoped tables MUST be constrained to the
//! caller's tenant. The tenant is resolved exclusively from **verified**
//! server-minted JWT claims (`branch_id` is the workspace branch, `org_id` the
//! `.gborg` tenant), never from client-supplied params or headers.
//!
//! #2112 — these helpers used to base64url-decode the payload segment and
//! return whatever `branch_id` it contained. Nothing checked the signature, so
//! a token signed with any key (or none at all) selected another tenant's
//! branch. Every claim now comes from [`crate::verified_jwt::VerifiedJwt`],
//! which is only produced after the HS256 signature is checked in constant
//! time against the configured SaaS secret and the expiry is validated.
//!
//! Handlers receive a `HeaderMap` and use one of the `*_from_claims` helpers
//! to obtain the authoritative tenant, then apply `branch_id = ?` in queries.

use axum::http::HeaderMap;
use base64::Engine as _;
use uuid::Uuid;

use crate::verified_jwt::{verify_saas_jwt, VerifiedJwt};

/// Extracts the caller's token from an `Authorization` header value.
///
/// `Bearer <token>` is the native form. `Basic <base64>` is accepted as well
/// because native clients (calendar, mail) authenticate with HTTP Basic and
/// have no field for a bearer token: the password field is expected to carry a
/// GeneralBots token, and the username is ignored because the token already
/// identifies the caller. The returned token is still verified on the normal
/// authentication path, so this widens the transport, not the trust.
pub fn token_from_authorization(auth: &str) -> Option<String> {
    // RFC 7235: the auth scheme is case-insensitive, so `BEARER` must work too.
    const BEARER_PREFIX_LEN: usize = "bearer ".len();
    if let Some(head) = auth.get(..BEARER_PREFIX_LEN) {
        if head.eq_ignore_ascii_case("bearer ") {
            let token = auth[BEARER_PREFIX_LEN..].trim();
            return if token.is_empty() {
                None
            } else {
                Some(token.to_string())
            };
        }
    }

    const BASIC_PREFIX_LEN: usize = "basic ".len();
    // `get` keeps the split on a character boundary, so a non-ASCII header is
    // refused instead of panicking.
    let head = auth.get(..BASIC_PREFIX_LEN)?;
    if !head.eq_ignore_ascii_case("basic ") {
        return None;
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(auth[BASIC_PREFIX_LEN..].trim())
        .ok()?;
    let credentials = String::from_utf8(decoded).ok()?;
    let (_, token) = credentials.split_once(':')?;
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

/// Verifies the request token and returns its claims.
///
/// This is the only path that may produce tenant claims: an unsigned,
/// tampered, wrong-key, non-HS256 or expired token yields `None`, and the
/// caller falls back to its crate default instead of trusting the payload.
fn verified_claims(headers: &HeaderMap) -> Option<VerifiedJwt> {
    let auth = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))?
        .to_str()
        .ok()?;
    let token = token_from_authorization(auth)?;
    match verify_saas_jwt(&token) {
        Ok(claims) => Some(claims),
        Err(reason) => {
            log::debug!("tenant claims refused: {reason}");
            None
        }
    }
}

/// Returns the verified `branch_id` claim, the authoritative tenant
/// workspace branch. Never derived from client input.
pub fn branch_from_claims(headers: &HeaderMap) -> Option<Uuid> {
    verified_claims(headers)?.branch_id
}

/// Returns the owning tenant organization id from the verified JWT claims.
pub fn org_from_claims(headers: &HeaderMap) -> Option<Uuid> {
    verified_claims(headers)?.org_id
}

/// Returns the verified `sub` (user id) from the JWT claims.
pub fn user_id_from_claims(headers: &HeaderMap) -> Option<String> {
    Some(verified_claims(headers)?.sub)
}

/// Returns the verified `sub` claim from a raw token string. Needed by
/// transports that cannot set headers (SSE via EventSource passes `?token=`).
pub fn user_id_from_claims_subject(token: &str) -> Option<String> {
    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    Some(verify_saas_jwt(token.trim()).ok()?.sub)
}

/// Returns the verified email claim from the JWT, if present.
pub fn email_from_claims(headers: &HeaderMap) -> Option<String> {
    verified_claims(headers)?.email
}

/// Resolves the user's workspace branch from their org membership binding
/// (users → user_organizations → branches) when no server-minted claim and no
/// CRM contact row are available. Returns `None` when the user has no verified
/// binding — callers then fall back to their crate-level default context.
pub fn branch_from_user_binding(
    conn: &mut diesel::PgConnection,
    email: &str,
) -> Option<Uuid> {
    use diesel::prelude::*;
    #[derive(diesel::QueryableByName)]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let user_id = diesel::sql_query("SELECT id FROM users WHERE email = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(email)
        .get_result::<UserRow>(conn)
        .optional()
        .ok()
        .flatten()?
        .id;

    #[derive(diesel::QueryableByName)]
    struct BindingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let org_id = diesel::sql_query(
        "SELECT org_id FROM user_organizations WHERE user_id = $1 ORDER BY is_default DESC, joined_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<BindingRow>(conn)
    .optional()
    .ok()
    .flatten()?
    .org_id;

    #[derive(diesel::QueryableByName)]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    diesel::sql_query(
        "SELECT id FROM branches WHERE org_id = $1 AND is_active = true ORDER BY created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .get_result::<BranchRow>(conn)
    .optional()
    .ok()
    .flatten()
    .map(|r| r.id)
}

#[cfg(test)]
#[path = "tenant_adversarial_tests.rs"]
mod tenant_adversarial_tests;

#[cfg(test)]
mod tenant_claim_tests {
    use super::*;
    use crate::verified_jwt::test_support::{signed, tenant_claims};
    use axum::http::{HeaderMap, HeaderValue};

    fn headers(auth: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("authorization", HeaderValue::from_str(auth).expect("header"));
        map
    }

    const BRANCH: &str = "11111111-1111-1111-1111-111111111111";
    const ORG: &str = "22222222-2222-2222-2222-222222222222";

    #[test]
    fn a_signed_token_yields_every_claim() {
        let jwt = signed(&tenant_claims(BRANCH, ORG, "user-1", "u@example.com"));
        let map = headers(&format!("Bearer {jwt}"));
        assert_eq!(
            branch_from_claims(&map),
            Some(Uuid::parse_str(BRANCH).expect("uuid"))
        );
        assert_eq!(
            org_from_claims(&map),
            Some(Uuid::parse_str(ORG).expect("uuid"))
        );
        assert_eq!(user_id_from_claims(&map).as_deref(), Some("user-1"));
        assert_eq!(email_from_claims(&map).as_deref(), Some("u@example.com"));
    }

    #[test]
    fn malformed_authorization_headers_never_panic() {
        for auth in [
            "",
            "Bearer",
            "Bearer ",
            "Basic !!!notbase64",
            "garbage",
            "a.b",
            "Bearer a.b.c",
        ] {
            let map = headers(auth);
            assert_eq!(branch_from_claims(&map), None, "{auth:?} must be refused");
        }
        assert_eq!(branch_from_claims(&HeaderMap::new()), None);
    }

    #[test]
    fn basic_transport_carries_the_token_in_the_password_field() {
        let jwt = signed(&tenant_claims(BRANCH, ORG, "user-1", "u@example.com"));
        let credentials = format!("ignored:{jwt}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        let map = headers(&format!("Basic {encoded}"));

        assert_eq!(user_id_from_claims(&map).as_deref(), Some("user-1"));
    }

    #[test]
    fn token_from_authorization_accepts_both_transports() {
        assert_eq!(
            token_from_authorization("Bearer abc").as_deref(),
            Some("abc")
        );
        assert_eq!(
            token_from_authorization("bearer abc").as_deref(),
            Some("abc")
        );
        assert_eq!(token_from_authorization("Bearer   "), None);
        let encoded = base64::engine::general_purpose::STANDARD.encode("u:tok");
        assert_eq!(
            token_from_authorization(&format!("Basic {encoded}")).as_deref(),
            Some("tok")
        );
    }
}
