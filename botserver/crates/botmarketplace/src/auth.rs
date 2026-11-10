//! Caller identity for marketplace handlers (issues #2112/#2125).
//!
//! These helpers used to base64url-decode the payload segment of the Bearer
//! token and trust whatever it said — including `role: admin`. Nothing checked
//! the signature, so a forged (`alg: none`), tampered, or foreign-key token was
//! an administrator. Identity now comes from
//! [`botsecurity::botsecurity_core::verified_jwt::verify_saas_jwt`]: HS256
//! pinned, HMAC-SHA256 compared in constant time, `exp`/`nbf` validated.
//!
//! A token that cannot be verified yields `None`; every helper here answers
//! "not authenticated" / "not admin" in that case, and the handlers turn that
//! into `401 Unauthorized`. No path derives an identity or a role from an
//! unverified payload.

use axum::http::HeaderMap;
use botsecurity::botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt, VerifiedJwt};
use diesel::prelude::*;
use uuid::Uuid;

use crate::DbPool;

/// Group name fragment that marks an administrator, the same rule
/// `resolve_user_role` applies for the rest of the API.
const ADMIN_ROLE: &str = "admin";

/// Verified claims of the request token.
///
/// `None` when the token is absent, malformed, unsigned, signed with another
/// key, expired, or not yet valid — the caller must treat that as anonymous.
pub fn jwt_claims(headers: &HeaderMap) -> Option<VerifiedJwt> {
    let token = bearer_token(headers)?;
    match verify_saas_jwt(token) {
        Ok(claims) => Some(claims),
        Err(reason) => {
            tracing::debug!("marketplace token refused: {reason}");
            None
        }
    }
}

/// Whether the caller is an administrator.
///
/// An unverifiable token is never an administrator; otherwise the decision is
/// [`claims_are_admin`] on the verified claims.
pub fn jwt_is_admin(headers: &HeaderMap, pool: &DbPool) -> bool {
    match jwt_claims(headers) {
        Some(claims) => claims_are_admin(&claims, pool),
        None => false,
    }
}

/// Whether verified claims grant the administrator role.
///
/// A token that carries role claims decides on its own: those claims were
/// signed by the server, so a token saying `user` is never upgraded by a
/// database lookup. Cloud-login tokens carry no role at all (#736), so for
/// those the RBAC groups decide, exactly like `resolve_user_role` does for the
/// rest of the API.
pub fn claims_are_admin(claims: &VerifiedJwt, pool: &DbPool) -> bool {
    if !claims.roles.is_empty() {
        return claims.roles.iter().any(|role| role == ADMIN_ROLE);
    }
    rbac_says_admin(pool, claims)
}

/// Resolves the role from the RBAC tables for a token without role claims.
fn rbac_says_admin(pool: &DbPool, claims: &VerifiedJwt) -> bool {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!("marketplace role resolution: database pool unavailable: {error}");
            return false;
        }
    };

    #[derive(diesel::QueryableByName)]
    struct GroupName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    // Same rule as `resolve_user_role`/`is_admin_user`: an active group whose
    // name contains "admin". The user row is matched by the stable UUID first
    // and by email second, because cloud `sub` claims are numeric Zitadel ids
    // and older rows were created under a different id derivation.
    let groups: Vec<GroupName> = diesel::sql_query(
        "SELECT g.name FROM rbac_groups g \
         JOIN rbac_user_groups ug ON ug.group_id = g.id \
         WHERE g.is_active = true \
           AND (ug.user_id = $1 \
                OR ug.user_id = (SELECT u.id FROM users u \
                                 WHERE lower(u.email) = lower($2) AND u.is_active = true \
                                 LIMIT 1))",
    )
    .bind::<diesel::sql_types::Uuid, _>(stable_user_uuid(&claims.sub))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(claims.email.as_deref())
    .load(&mut conn)
    .unwrap_or_else(|error| {
        tracing::error!("marketplace role resolution query failed: {error}");
        Vec::new()
    });

    groups
        .iter()
        .any(|group| group.name.to_lowercase().contains(ADMIN_ROLE))
}

/// Maps a verified `sub` to the stable user UUID the RBAC tables use, the same
/// derivation `resolve_user_role` and `SaasJwtAuthProvider` apply.
fn stable_user_uuid(sub: &str) -> Uuid {
    let sub = sub.trim();
    match Uuid::parse_str(sub) {
        Ok(id) => id,
        Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{sub}").as_bytes()),
    }
}

/// The verified tenant organization of the caller.
pub fn jwt_org_id(headers: &HeaderMap) -> Option<Uuid> {
    jwt_claims(headers)?.org_id
}

/// The verified subject as a UUID.
///
/// Marketplace tables store `installed_by` as a UUID, so a numeric Zitadel
/// subject (which has no such row) yields `None` and the column stays null.
pub fn jwt_user_id(headers: &HeaderMap) -> Option<Uuid> {
    Uuid::parse_str(jwt_claims(headers)?.sub.trim()).ok()
}

/// The verified email of the caller, used as the publisher display name.
pub fn jwt_email(headers: &HeaderMap) -> Option<String> {
    jwt_claims(headers)?.email
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use botsecurity::botsecurity_core::verified_jwt::resolve_saas_jwt_secret;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use std::time::Duration;

    /// Signs a token the way the login flow does, with the secret the verifier
    /// resolves in this process.
    fn sign(signing_input: &str) -> String {
        let secret = resolve_saas_jwt_secret();
        let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return String::new(),
        };
        mac.update(signing_input.as_bytes());
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }

    fn token_with_header(header: &str, payload: &str) -> String {
        let signing_input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.as_bytes()),
            URL_SAFE_NO_PAD.encode(payload.as_bytes())
        );
        let signature = sign(&signing_input);
        format!("{signing_input}.{signature}")
    }

    fn token(payload: &serde_json::Value) -> String {
        token_with_header(r#"{"alg":"HS256","typ":"JWT"}"#, &payload.to_string())
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(value) = axum::http::HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert("authorization", value);
        }
        headers
    }

    /// A pool that never connects: the RBAC lookup is only reached by tokens
    /// without role claims, and it must fail closed.
    fn offline_pool() -> DbPool {
        diesel::r2d2::Pool::builder()
            .connection_timeout(Duration::from_millis(1))
            .build_unchecked(diesel::r2d2::ConnectionManager::new(
                "postgres://offline:offline@127.0.0.1:1/offline",
            ))
    }

    fn in_an_hour() -> i64 {
        chrono::Utc::now().timestamp() + 3600
    }

    fn claims(role: &str) -> serde_json::Value {
        serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "email": "admin@example.com",
            "role": role,
            "exp": in_an_hour(),
        })
    }

    #[test]
    fn a_valid_admin_token_is_accepted() {
        let headers = bearer(&token(&claims("admin")));
        assert_eq!(
            jwt_claims(&headers).map(|c| c.roles),
            Some(vec!["admin".to_string()])
        );
        assert!(jwt_is_admin(&headers, &offline_pool()));
    }

    #[test]
    fn a_valid_non_admin_token_is_not_admin() {
        let headers = bearer(&token(&claims("user")));
        assert!(jwt_claims(&headers).is_some());
        assert!(!jwt_is_admin(&headers, &offline_pool()));
    }

    #[test]
    fn a_tampered_payload_is_rejected() {
        let valid = token(&claims("user"));
        let parts: Vec<&str> = valid.split('.').collect();
        let forged = URL_SAFE_NO_PAD.encode(claims("admin").to_string().as_bytes());
        let tampered = format!("{}.{}.{}", parts[0], forged, parts[2]);

        let headers = bearer(&tampered);
        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers, &offline_pool()));
        assert!(jwt_org_id(&headers).is_none());
        assert!(jwt_user_id(&headers).is_none());
        assert!(jwt_email(&headers).is_none());
    }

    #[test]
    fn an_alg_none_token_is_rejected() {
        // Signed-looking `alg: none` header: refused before any claim is read.
        let forged = token_with_header(r#"{"alg":"none","typ":"JWT"}"#, &claims("admin").to_string());
        let headers = bearer(&forged);
        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers, &offline_pool()));

        // The classic empty-signature form.
        let unsigned = format!(
            "{}.{}.",
            URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#),
            URL_SAFE_NO_PAD.encode(claims("admin").to_string().as_bytes())
        );
        let headers = bearer(&unsigned);
        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers, &offline_pool()));
    }

    #[test]
    fn a_token_without_a_signature_is_rejected() {
        let payload = URL_SAFE_NO_PAD.encode(claims("admin").to_string().as_bytes());
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);

        for unsigned in [format!("{header}.{payload}"), format!("{header}.{payload}.")] {
            let headers = bearer(&unsigned);
            assert!(jwt_claims(&headers).is_none(), "{unsigned}");
            assert!(!jwt_is_admin(&headers, &offline_pool()), "{unsigned}");
        }
    }

    #[test]
    fn a_token_signed_with_another_key_is_rejected() {
        let mut mac = match Hmac::<Sha256>::new_from_slice(b"not-the-saas-secret-0123456789") {
            Ok(mac) => mac,
            Err(_) => return,
        };
        let signing_input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#),
            URL_SAFE_NO_PAD.encode(claims("admin").to_string().as_bytes())
        );
        mac.update(signing_input.as_bytes());
        let foreign = format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
        );

        let headers = bearer(&foreign);
        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers, &offline_pool()));
    }

    #[test]
    fn missing_or_malformed_tokens_are_anonymous() {
        assert!(!jwt_is_admin(&HeaderMap::new(), &offline_pool()));
        let mut headers = HeaderMap::new();
        if let Ok(value) = axum::http::HeaderValue::from_str("Bearer a.b") {
            headers.insert("authorization", value);
        }
        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers, &offline_pool()));
    }

    #[test]
    fn a_token_without_role_claims_fails_closed_without_a_database() {
        let payload = serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "exp": in_an_hour(),
        });
        let headers = bearer(&token(&payload));
        assert!(jwt_claims(&headers).is_some());
        assert!(jwt_claims(&headers).map(|c| c.roles.is_empty()).unwrap_or(false));
        assert!(!jwt_is_admin(&headers, &offline_pool()));
    }
}
