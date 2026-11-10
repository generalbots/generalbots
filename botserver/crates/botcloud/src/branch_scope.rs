use diesel::prelude::*;
use uuid::Uuid;

use crate::api::base64_url_decode;

#[derive(Debug, Default, PartialEq)]
struct Claims {
    subject: Option<Uuid>,
    email: Option<String>,
    branch_id: Option<Uuid>,
}

fn parse_uuid(value: &serde_json::Value) -> Option<Uuid> {
    value.as_str().and_then(|s| Uuid::parse_str(s).ok())
}

/// Rejects an unsecured JWT header (`alg: none`) or one with no algorithm.
///
/// #2069 — the payload used to be decoded with no look at the header, so a
/// forged unsecured token could select any tenant branch. An unsecured token
/// can never have been minted by the server.
fn header_is_secured(bearer: &str) -> bool {
    let Some(header) = bearer.split('.').next() else {
        return false;
    };
    let Some(decoded) = base64_url_decode(header).ok() else {
        return false;
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return false;
    };
    match json.get("alg").and_then(|v| v.as_str()) {
        Some(alg) => !alg.trim().eq_ignore_ascii_case("none"),
        None => false,
    }
}

fn claims_from_bearer(bearer: &str) -> Option<Claims> {
    if !header_is_secured(bearer) {
        return None;
    }
    let payload = bearer.split('.').nth(1)?;
    let decoded = base64_url_decode(payload).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    Some(Claims {
        subject: json.get("sub").and_then(parse_uuid),
        email: json
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        branch_id: json.get("branch_id").and_then(parse_uuid),
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

    fn b64url(bytes: &[u8]) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    fn bearer(payload: serde_json::Value) -> String {
        let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
        let body = b64url(payload.to_string().as_bytes());
        format!("{header}.{body}.signature")
    }

    fn headers_with(token: &str) -> axum::http::HeaderMap {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    #[test]
    fn reads_branch_claim() {
        let branch = Uuid::new_v4();
        let token = bearer(serde_json::json!({
            "sub": Uuid::new_v4().to_string(),
            "email": "owner@example.com",
            "branch_id": branch.to_string(),
        }));
        let claims = claims_from_bearer(&token).expect("claims");
        assert_eq!(claims.branch_id, Some(branch));
        assert_eq!(claims.email.as_deref(), Some("owner@example.com"));
    }

    #[test]
    fn ignores_absent_or_malformed_branch_claim() {
        let token = bearer(serde_json::json!({"email": "owner@example.com"}));
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
        let token = bearer(serde_json::json!({}));
        let headers = headers_with(&token);
        assert_eq!(bearer_from_headers(&headers).as_deref(), Some(token.as_str()));
    }

    #[test]
    fn an_unsecured_token_is_refused() {
        // #2069 — a forged alg=none token must not select a tenant branch.
        let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
        let body = b64url(serde_json::json!({"branch_id": Uuid::new_v4().to_string()}).to_string().as_bytes());
        let token = format!("{header}.{body}.signature");

        assert_eq!(claims_from_bearer(&token), None);
        assert!(!header_is_secured(&token));
    }

    #[test]
    fn an_unsecured_header_in_any_casing_is_refused() {
        for header in [r#"{"alg":"NONE"}"#, r#"{"alg":"None"}"#, r#"{"alg":" none "}"#] {
            let encoded = b64url(header.as_bytes());
            let body = b64url(br#"{"sub":"x"}"#);
            let token = format!("{encoded}.{body}.sig");
            assert!(!header_is_secured(&token), "{header} must be refused");
        }
    }

    #[test]
    fn a_token_without_an_algorithm_is_refused() {
        let header = b64url(br#"{"typ":"JWT"}"#);
        let body = b64url(br#"{"sub":"x"}"#);
        let token = format!("{header}.{body}.sig");
        assert!(!header_is_secured(&token));
    }

    #[test]
    fn a_secured_token_is_accepted() {
        let token = bearer(serde_json::json!({"sub": Uuid::new_v4().to_string()}));
        assert!(header_is_secured(&token));
        assert!(claims_from_bearer(&token).is_some());
    }

    #[test]
    fn a_malformed_header_never_panics() {
        for token in ["", "a", "a.b", "!!!.payload.sig", ".."] {
            assert!(!header_is_secured(token), "{token:?} must be refused");
            assert_eq!(claims_from_bearer(token), None, "{token:?}");
        }
    }
}