//! Tenant-scope resolution for multi-tenant data isolation (issue #734).
//!
//! All reads and writes of branch-scoped tables MUST be constrained to the
//! caller's tenant. The tenant is resolved exclusively from the server-minted
//! JWT claims (`branch_id` is the workspace branch, `org_id` the `.gborg`
//! tenant), never from client-supplied params or headers.
//!
//! Handlers receive a `HeaderMap` and use one of the `*_from_claims` helpers
//! to obtain the authoritative tenant, then apply `branch_id = ?` in queries.

use axum::http::HeaderMap;
use base64::Engine as _;
use uuid::Uuid;

/// Minimal URL-safe base64 decoder (JWT payload decoding).
/// Returns `None` on invalid input so a malformed header never aborts a
/// request or panics in a production path (AGENTS.md).
pub fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input.as_bytes() {
        let v = {
            let i = TABLE.iter().position(|&x| x == b)?;
            i as u32
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

/// Extracts the caller's token from an `Authorization` header value.
///
/// `Bearer <token>` is the native form. `Basic <base64>` is accepted as well
/// because native clients (calendar, mail) authenticate with HTTP Basic and
/// have no field for a bearer token: the password field is expected to carry a
/// GeneralBots token, and the username is ignored because the token already
/// identifies the caller. The returned token is still verified on the normal
/// authentication path, so this widens the transport, not the trust.
pub fn token_from_authorization(auth: &str) -> Option<String> {
    if let Some(token) = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))
    {
        let token = token.trim();
        return if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        };
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

/// Decodes and validates the JWT header segment.
///
/// #2066 — the payload was decoded with no look at the header, so a forged
/// unsecured token (`{"alg":"none"}`) supplied whatever `branch_id` it liked
/// and every caller treated it as authoritative. An unsecured token can never
/// have been minted by the server, so it is refused outright.
fn jwt_header_is_secured(token: &str) -> bool {
    let Some(header_segment) = token.split('.').next() else {
        return false;
    };
    let Some(decoded) = base64url_decode(header_segment.trim_end_matches('=')) else {
        return false;
    };
    let Ok(header) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return false;
    };
    match header.get("alg").and_then(|v| v.as_str()) {
        // `none` (any casing) means "no signature"; HS/RS/ES/PS algorithms are
        // verified on the normal authentication path before a handler runs.
        Some(alg) => !alg.trim().eq_ignore_ascii_case("none"),
        None => false,
    }
}

/// Extracts the JWT payload object (middle segment) from the Authorization
/// header, for either the `Bearer` or the `Basic` transport. Invalid tokens
/// yield `None`.
fn jwt_payload(headers: &HeaderMap) -> Option<serde_json::Value> {
    let auth = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))?
        .to_str()
        .ok()?;
    let token = token_from_authorization(auth)?;
    if !token.contains('.') {
        return None;
    }
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    if !jwt_header_is_secured(&token) {
        return None;
    }
    let payload = base64url_decode(parts[1].trim_end_matches('='))?;
    serde_json::from_slice(&payload).ok()
}

/// Returns the server-minted `branch_id` claim, the authoritative tenant
/// workspace branch. Never derived from client input.
pub fn branch_from_claims(headers: &HeaderMap) -> Option<Uuid> {
    jwt_payload(headers)?
        .get("branch_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Returns the owning tenant organization id from the JWT claims.
pub fn org_from_claims(headers: &HeaderMap) -> Option<Uuid> {
    let payload = jwt_payload(headers)?;
    payload
        .get("org_id")
        .or_else(|| payload.get("organization_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Returns the `sub` (user id) from the JWT claims.
pub fn user_id_from_claims(headers: &HeaderMap) -> Option<String> {
    jwt_payload(headers)?
        .get("sub")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Returns the `sub` claim from a raw token string. Needed by transports
/// that cannot set headers (SSE via EventSource passes `?token=`).
pub fn user_id_from_claims_subject(token: &str) -> Option<String> {
    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let payload = base64url_decode(parts[1].trim_end_matches('='))?;
    serde_json::from_slice::<serde_json::Value>(&payload)
        .ok()?
        .get("sub")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Returns the email claim from the JWT, if present.
pub fn email_from_claims(headers: &HeaderMap) -> Option<String> {
    jwt_payload(headers)?
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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
mod tenant_claim_tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    fn b64(input: &str) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let bytes = input.as_bytes();
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[(triple >> 18) as usize & 0x3F] as char);
            out.push(TABLE[(triple >> 12) as usize & 0x3F] as char);
            if chunk.len() > 1 {
                out.push(TABLE[(triple >> 6) as usize & 0x3F] as char);
            }
            if chunk.len() > 2 {
                out.push(TABLE[triple as usize & 0x3F] as char);
            }
        }
        out
    }

    fn token(header: &str, payload: &str) -> String {
        format!("{}.{}.{}", b64(header), b64(payload), "signature")
    }

    fn headers(auth: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("authorization", HeaderValue::from_str(auth).expect("header"));
        map
    }

    const SECURED: &str = r#"{"alg":"HS256","typ":"JWT"}"#;
    const UNSECURED: &str = r#"{"alg":"none","typ":"JWT"}"#;
    const CLAIMS: &str = r#"{"branch_id":"11111111-1111-1111-1111-111111111111","org_id":"22222222-2222-2222-2222-222222222222","sub":"user-1"}"#;

    #[test]
    fn a_secured_token_yields_every_claim() {
        let map = headers(&format!("Bearer {}", token(SECURED, CLAIMS)));
        assert_eq!(
            branch_from_claims(&map),
            Some(Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("uuid"))
        );
        assert_eq!(
            org_from_claims(&map),
            Some(Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("uuid"))
        );
        assert_eq!(user_id_from_claims(&map).as_deref(), Some("user-1"));
    }

    #[test]
    fn a_forged_alg_none_token_is_refused() {
        // #2066 — an unsecured token must not select a tenant branch.
        let map = headers(&format!("Bearer {}", token(UNSECURED, CLAIMS)));
        assert_eq!(branch_from_claims(&map), None);
        assert_eq!(org_from_claims(&map), None);
        assert_eq!(user_id_from_claims(&map), None);
    }

    #[test]
    fn an_alg_none_header_in_any_casing_is_refused() {
        for header in [r#"{"alg":"NONE"}"#, r#"{"alg":"None"}"#, r#"{"alg":" none "}"#] {
            let map = headers(&format!("Bearer {}", token(header, CLAIMS)));
            assert_eq!(branch_from_claims(&map), None, "{header} must be refused");
        }
    }

    #[test]
    fn a_token_without_an_algorithm_is_refused() {
        let map = headers(&format!("Bearer {}", token(r#"{"typ":"JWT"}"#, CLAIMS)));
        assert_eq!(branch_from_claims(&map), None);
    }

    #[test]
    fn malformed_authorization_headers_never_panic() {
        for auth in ["", "Bearer", "Bearer ", "Basic !!!notbase64", "garbage", "a.b"] {
            let map = headers(auth);
            assert_eq!(branch_from_claims(&map), None, "{auth:?} must be refused");
        }
        assert_eq!(branch_from_claims(&HeaderMap::new()), None);
    }

    #[test]
    fn basic_transport_carries_the_token_in_the_password_field() {
        let jwt = token(SECURED, CLAIMS);
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

    #[test]
    fn a_claim_that_is_not_a_uuid_is_ignored() {
        let payload = r#"{"branch_id":"not-a-uuid","org_id":42}"#;
        let map = headers(&format!("Bearer {}", token(SECURED, payload)));
        assert_eq!(branch_from_claims(&map), None);
        assert_eq!(org_from_claims(&map), None);
    }

    #[test]
    fn base64url_decode_round_trips_ascii() {
        let encoded = b64("hello world");
        let decoded = base64url_decode(&encoded).expect("decode");
        assert_eq!(String::from_utf8(decoded).expect("utf8"), "hello world");
        assert!(base64url_decode("!!!").is_none());
    }
}
