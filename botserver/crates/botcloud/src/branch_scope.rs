use diesel::prelude::*;
use uuid::Uuid;

use botsecurity::botsecurity_core::verified_jwt::verify_saas_jwt;

#[derive(Debug, Default, PartialEq)]
struct Claims {
    subject: Option<Uuid>,
    email: Option<String>,
    branch_id: Option<Uuid>,
}

/// Reads the tenant identity out of a **verified** bearer token.
///
/// #2112 — the payload used to be decoded on its own, so a token signed with
/// any key (or none) supplied whatever `branch_id` it liked. `verify_saas_jwt`
/// checks the HS256 signature in constant time against the configured SaaS
/// secret, the algorithm, and the expiry before any claim is returned.
fn claims_from_bearer(bearer: &str) -> Option<Claims> {
    let verified = verify_saas_jwt(bearer).ok()?;
    Some(Claims {
        subject: Uuid::parse_str(&verified.sub).ok(),
        email: verified.email,
        branch_id: verified.branch_id,
    })
}

fn bearer_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

#[derive(QueryableByName)]
struct BranchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

fn branch_from_crm_contact(
    conn: &mut diesel::PgConnection,
    contact_email: &str,
) -> Result<Option<Uuid>, String> {
    use crate::schema_ext::crm_contacts::dsl::{branch_id, crm_contacts, email};

    crm_contacts
        .filter(email.eq(contact_email))
        .select(branch_id)
        .first(conn)
        .optional()
        .map_err(|e| format!("crm_contacts query: {e}"))
}

fn branch_from_membership(
    conn: &mut diesel::PgConnection,
    user_id: Uuid,
) -> Result<Option<Uuid>, String> {
    let row: Option<BranchRow> = diesel::sql_query(
        "SELECT b.id
           FROM user_organizations uo
           JOIN branches b ON b.org_id = uo.org_id
          WHERE uo.user_id = $1
          ORDER BY uo.is_default DESC, b.created_at ASC
          LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("user_organizations query: {e}"))?;

    Ok(row.map(|r| r.id))
}

pub fn resolve_branch_id(
    headers: &axum::http::HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<Option<Uuid>, String> {
    let Some(bearer) = bearer_from_headers(headers) else {
        return Ok(None);
    };
    let Some(claims) = claims_from_bearer(&bearer) else {
        return Ok(None);
    };

    if let Some(branch_id) = claims.branch_id {
        return Ok(Some(branch_id));
    }

    if let Some(email) = claims.email.as_deref() {
        if let Some(branch_id) = branch_from_crm_contact(conn, email)? {
            return Ok(Some(branch_id));
        }
    }

    match claims.subject {
        Some(user_id) => branch_from_membership(conn, user_id),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    fn b64url(bytes: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Mints an HS256 token the way cloud login does, signed with the secret
    /// the verifier resolves in this process.
    fn bearer(payload: serde_json::Value) -> String {
        let secret = botsecurity::botsecurity_core::verified_jwt::resolve_saas_jwt_secret();
        let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
        let body = b64url(payload.to_string().as_bytes());
        let signing_input = format!("{header}.{body}");
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
        mac.update(signing_input.as_bytes());
        format!("{signing_input}.{}", b64url(&mac.finalize().into_bytes()))
    }

    fn live_claims(value: serde_json::Value) -> serde_json::Value {
        let mut claims = value;
        if let Some(object) = claims.as_object_mut() {
            object.insert(
                "exp".to_string(),
                serde_json::json!(chrono::Utc::now().timestamp() + 3600),
            );
        }
        claims
    }

    fn headers_with(token: &str) -> axum::http::HeaderMap {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {token}").parse().expect("header"),
        );
        headers
    }

    #[test]
    fn reads_branch_claim() {
        let branch = Uuid::new_v4();
        let token = bearer(live_claims(serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "email": "owner@example.com",
            "branch_id": branch.to_string(),
        })));
        let claims = claims_from_bearer(&token).expect("claims");
        assert_eq!(claims.branch_id, Some(branch));
        assert_eq!(claims.email.as_deref(), Some("owner@example.com"));
    }

    #[test]
    fn ignores_absent_or_malformed_branch_claim() {
        let token = bearer(live_claims(
            serde_json::json!({"email": "owner@example.com"}),
        ));
        let claims = claims_from_bearer(&token).expect("claims");
        assert_eq!(claims.branch_id, None);

        let garbage = claims_from_bearer("not-a-jwt");
        assert_eq!(garbage, None);
    }

    #[test]
    fn no_authorization_header_yields_no_identity() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(bearer_from_headers(&headers), None);
    }

    #[test]
    fn header_parsing_strips_the_bearer_prefix() {
        let token = bearer(live_claims(serde_json::json!({})));
        let headers = headers_with(&token);
        assert_eq!(bearer_from_headers(&headers).as_deref(), Some(token.as_str()));
    }

    #[test]
    fn an_unsecured_token_is_refused() {
        let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
        let body = b64url(
            live_claims(serde_json::json!({"branch_id": Uuid::new_v4().to_string()}))
                .to_string()
                .as_bytes(),
        );
        let token = format!("{header}.{body}.");
        assert_eq!(claims_from_bearer(&token), None);
    }

    #[test]
    fn a_forged_signature_is_refused() {
        let branch = Uuid::new_v4();
        let valid = bearer(live_claims(serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "branch_id": branch.to_string(),
        })));
        let mut parts: Vec<String> = valid.split('.').map(str::to_string).collect();
        parts[2] = b64url(b"definitely-not-a-valid-signature");
        let forged = parts.join(".");
        assert_eq!(claims_from_bearer(&forged), None);
    }

    #[test]
    fn a_token_signed_with_another_key_is_refused() {
        let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
        let body = b64url(
            live_claims(serde_json::json!({"branch_id": Uuid::new_v4().to_string()}))
                .to_string()
                .as_bytes(),
        );
        let signing_input = format!("{header}.{body}");
        let mut mac =
            Hmac::<Sha256>::new_from_slice(b"attacker-controlled-secret").expect("hmac key");
        mac.update(signing_input.as_bytes());
        let forged = format!("{signing_input}.{}", b64url(&mac.finalize().into_bytes()));
        assert_eq!(claims_from_bearer(&forged), None);
    }

    #[test]
    fn an_expired_token_is_refused() {
        let token = bearer(serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "branch_id": Uuid::new_v4().to_string(),
            "exp": 1,
        }));
        assert_eq!(claims_from_bearer(&token), None);
    }

    #[test]
    fn a_secured_token_is_accepted() {
        let token = bearer(live_claims(
            serde_json::json!({"sub": Uuid::new_v4().to_string()}),
        ));
        assert!(claims_from_bearer(&token).is_some());
    }

    #[test]
    fn a_malformed_header_never_panics() {
        for token in ["", "a", "a.b", "!!!.payload.sig", ".."] {
            assert_eq!(claims_from_bearer(token), None, "{token:?}");
        }
    }
}
