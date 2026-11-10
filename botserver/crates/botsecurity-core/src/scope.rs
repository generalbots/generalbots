//! Shared branch-scope resolution for tenant-scoped write handlers (#1347).
//!
//! This module replaces the nine per-crate `scope.rs` copies (billing,
//! attendant, marketing, people, products, tickets carried one identical
//! variant; contacts and workspaces carried a second variant that also
//! tolerated multi-token Authorization headers). One implementation now
//! guards every tenant boundary: scope is the most security-relevant helper
//! in the tree, and a copy that drifts is the easiest way to get an
//! authorization rule wrong.
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

use crate::verified_jwt::verify_saas_jwt;

/// Extracts the `email` claim from a **verified** Bearer JWT.
///
/// #2112 — the payload used to be decoded on its own, so a forged token could
/// present any email and have it resolved to another tenant's CRM contact. The
/// signature is now checked against the configured SaaS secret first.
pub fn email_from_jwt(headers: &HeaderMap) -> Option<String> {
    let token = bearer_token(headers)?;
    verify_saas_jwt(token).ok()?.email
}

/// Extracts the user email from an opaque suite session token (`gb_*`).
/// The suite UI authenticates with a random opaque token that the auth
/// middleware resolves via the session cache (populated at login); the
/// cached entry carries the real user email used to scope CRM data.
pub fn email_from_session(headers: &HeaderMap) -> Option<String> {
    let token = bearer_token(headers)?;
    if token.contains('.') {
        return None;
    }
    crate::lookup_session_cache(token).map(|u| u.email)
}

/// Extracts the server-minted `branch_id` JWT claim. Minted at login from
/// the user's verified tenant binding (issue #736), it is the authoritative
/// branch scope; never derived from client input.
///
/// #2112 — the claim is only read from a token whose HS256 signature verifies
/// against the configured SaaS secret (constant-time HMAC compare) and whose
/// expiry is still valid. A forged, tampered, `alg=none` or expired token
/// yields `None` and callers fall back to their crate default context.
pub fn branch_from_claim(headers: &HeaderMap) -> Option<Uuid> {
    let token = bearer_token(headers)?;
    verify_saas_jwt(token).ok()?.branch_id
}

/// The Bearer token carried by the `Authorization` header.
///
/// Some proxies append a second comma-separated credential to the header, so
/// the value is trimmed and only the first entry is used (behaviour shared by
/// the contacts/workspaces variants; strictly more permissive than the
/// single-crate copies and safe for both).
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    auth.strip_prefix("Bearer ")
        .map(|t| t.split(',').next().unwrap_or(t).trim())
}

/// Resolves the branch id for the authenticated user. The server-minted
/// `branch_id` claim wins when present (issue #736); otherwise the JWT email
/// (or the session-cache email for opaque suite tokens) is looked up in
/// `crm_contacts` to derive the contact's `branch_id`. Only verified
/// credentials are consulted — never the caller-supplied `X-User-ID` (#2122).
pub fn branch_from_jwt(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Option<Uuid> {
    if let Some(claim) = branch_from_claim(headers) {
        return Some(claim);
    }
    let email = email_from_jwt(headers).or_else(|| email_from_session(headers))?;
    let contact_branch = {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: Uuid,
        }
        diesel::sql_query(
            "SELECT branch_id FROM crm_contacts WHERE email = $1 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&email)
        .get_result::<Row>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.branch_id)
    };
    // Fall back to the user→org→branch binding when no CRM contact owns the
    // email (issue #808: prod admins without a crm_contacts row were scoped
    // to the nil branch → empty grids).
    contact_branch.or_else(|| crate::tenant::branch_from_user_binding(conn, &email))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verified_jwt::test_support::{mint_hs256, signed, tenant_claims};

    fn header_with(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", value.parse().expect("valid header value"));
        headers
    }

    fn claims_with(branch: Uuid, email: &str) -> serde_json::Value {
        tenant_claims(&branch.to_string(), &Uuid::nil().to_string(), "user-1", email)
    }

    #[test]
    fn email_from_jwt_reads_the_verified_claim() {
        let token = signed(&claims_with(Uuid::new_v4(), "admin@pragmatismo.com.br"));
        assert_eq!(
            email_from_jwt(&header_with(&format!("Bearer {token}"))).as_deref(),
            Some("admin@pragmatismo.com.br")
        );
    }

    #[test]
    fn email_from_jwt_tolerates_a_comma_separated_header() {
        let token = signed(&claims_with(Uuid::new_v4(), "user@example.com"));
        let headers = header_with(&format!("Bearer {token},extra-credential"));
        assert_eq!(email_from_jwt(&headers).as_deref(), Some("user@example.com"));
    }

    #[test]
    fn email_from_jwt_rejects_malformed_tokens() {
        assert!(email_from_jwt(&header_with("Bearer not-a-jwt")).is_none());
        assert!(email_from_jwt(&header_with("Bearer a.@@@.c")).is_none());
        assert!(email_from_jwt(&header_with("Basic dXNlcjpwYXNz")).is_none());
    }

    #[test]
    fn branch_from_claim_reads_the_server_minted_branch() {
        let branch = Uuid::new_v4();
        let token = signed(&claims_with(branch, "user@example.com"));
        assert_eq!(
            branch_from_claim(&header_with(&format!("Bearer {token}"))),
            Some(branch)
        );
    }

    #[test]
    fn a_forged_signature_cannot_select_a_branch() {
        // #2112 — the payload alone must never resolve a tenant.
        let branch = Uuid::new_v4();
        let mut parts: Vec<String> = signed(&claims_with(branch, "user@example.com"))
            .split('.')
            .map(str::to_string)
            .collect();
        parts[2] = "definitely-not-a-valid-signature".to_string();
        let forged = parts.join(".");
        let headers = header_with(&format!("Bearer {forged}"));
        assert_eq!(branch_from_claim(&headers), None);
        assert_eq!(email_from_jwt(&headers), None);
    }

    #[test]
    fn a_token_signed_with_another_key_cannot_select_a_branch() {
        let branch = Uuid::new_v4();
        let token = mint_hs256(
            &claims_with(branch, "user@example.com"),
            "an-attacker-controlled-secret",
        );
        assert_eq!(
            branch_from_claim(&header_with(&format!("Bearer {token}"))),
            None
        );
    }

    #[test]
    fn an_unsecured_token_cannot_select_a_branch() {
        use base64::Engine as _;
        let encode = |bytes: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        let branch = Uuid::new_v4();
        let payload = encode(format!(r#"{{"branch_id":"{branch}","sub":"u","exp":4102444800}}"#).as_bytes());
        for header in [r#"{"alg":"none","typ":"JWT"}"#, r#"{"alg":"NONE"}"#] {
            let forged = format!("{}.{payload}.", encode(header.as_bytes()));
            assert_eq!(
                branch_from_claim(&header_with(&format!("Bearer {forged}"))),
                None
            );
        }
    }

    #[test]
    fn an_expired_token_cannot_select_a_branch() {
        let mut claims = claims_with(Uuid::new_v4(), "user@example.com");
        claims["exp"] = serde_json::Value::from(1);
        let token = signed(&claims);
        assert_eq!(
            branch_from_claim(&header_with(&format!("Bearer {token}"))),
            None
        );
    }

    #[test]
    fn malformed_tokens_never_panic() {
        for value in ["", "a", "a.b", "!!!.payload.sig", ".."] {
            assert_eq!(
                branch_from_claim(&header_with(&format!("Bearer {value}"))),
                None,
                "{value:?}"
            );
        }
    }
}
