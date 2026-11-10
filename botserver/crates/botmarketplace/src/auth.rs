use axum::http::HeaderMap;
use serde_json::Value;
use uuid::Uuid;

use crate::b64;

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

pub fn jwt_claims(headers: &HeaderMap) -> Option<Value> {
    let token = bearer_token(headers)?;
    if !jwt_header_is_secured(&token) {
        return None;
    }
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _signature = parts.next()?;
    let decoded = b64::decode_flexible(payload)?;
    serde_json::from_slice(&decoded).ok()
}

/// #2066 — the payload was decoded with no look at the header, so a forged
/// unsecured token (`{"alg":"none"}`) could claim `role: admin`. An unsecured
/// token can never have been minted by the server, so it is refused outright.
fn jwt_header_is_secured(token: &str) -> bool {
    let Some(header_segment) = token.split('.').next() else {
        return false;
    };
    let Some(decoded) = b64::decode_flexible(header_segment) else {
        return false;
    };
    let Ok(header) = serde_json::from_slice::<Value>(&decoded) else {
        return false;
    };
    match header.get("alg").and_then(Value::as_str) {
        Some(alg) => !alg.trim().eq_ignore_ascii_case("none"),
        None => false,
    }
}

fn claims_role_is_admin(claims: &Value) -> bool {
    if claims
        .get("role")
        .and_then(Value::as_str)
        .map(|r| r.eq_ignore_ascii_case("admin"))
        .unwrap_or(false)
    {
        return true;
    }
    claims
        .get("roles")
        .and_then(Value::as_array)
        .map(|roles| {
            roles.iter().any(|r| {
                r.as_str()
                    .map(|s| s.eq_ignore_ascii_case("admin"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

pub fn jwt_is_admin(headers: &HeaderMap) -> bool {
    jwt_claims(headers)
        .map(|c| claims_role_is_admin(&c))
        .unwrap_or(false)
}

pub fn jwt_org_id(headers: &HeaderMap) -> Option<Uuid> {
    let claims = jwt_claims(headers)?;
    for key in ["org_id", "publisher_org_id"] {
        if let Some(raw) = claims.get(key).and_then(Value::as_str) {
            if let Ok(id) = Uuid::parse_str(raw) {
                return Some(id);
            }
        }
    }
    None
}

pub fn jwt_user_id(headers: &HeaderMap) -> Option<Uuid> {
    let claims = jwt_claims(headers)?;
    for key in ["user_id", "sub"] {
        if let Some(raw) = claims.get(key).and_then(Value::as_str) {
            if let Ok(id) = Uuid::parse_str(raw) {
                return Some(id);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with_payload(payload: &str) -> HeaderMap {
        let token = format!(
            "eyJhbGciOiJIUzI1NiJ9.{}.c2ln",
            crate::b64::encode_standard(payload.as_bytes()).replace('+', "-").replace('/', "_")
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers
    }

    #[test]
    fn extracts_admin_from_single_role() {
        const ORG: &str = "00000000-0000-0000-0000-000000000001";
        let headers =
            header_with_payload(&format!(r#"{{"role":"admin","org_id":"{ORG}"}}"#));
        assert!(jwt_is_admin(&headers));
        assert_eq!(
            jwt_org_id(&headers),
            Some(Uuid::parse_str(ORG).expect("uuid"))
        );
    }

    #[test]
    fn extracts_admin_from_roles_array_and_publisher_fallback() {
        let headers = header_with_payload(r#"{"roles":["publisher"]}"#);
        assert!(!jwt_is_admin(&headers));
        let headers = header_with_payload(
            r#"{"roles":["user","ADMIN"],"publisher_org_id":"00000000-0000-0000-0000-00000000000a"}"#,
        );
        assert!(jwt_is_admin(&headers));
        assert_eq!(
            jwt_org_id(&headers),
            Some(Uuid::parse_str("00000000-0000-0000-0000-00000000000a").unwrap())
        );
    }

    #[test]
    fn missing_or_malformed_token_yields_none() {
        assert!(!jwt_is_admin(&HeaderMap::new()));
        let mut bad = HeaderMap::new();
        bad.insert("authorization", axum::http::HeaderValue::from_static("Bearer a.b"));
        assert!(!jwt_is_admin(&bad));
    }

    #[test]
    fn unsecured_tokens_are_refused() {
        let payload = crate::b64::encode_standard(br#"{"role":"admin"}"#)
            .replace('+', "-")
            .replace('/', "_");
        let forged = format!("{}.{}.", crate::b64::encode_standard(br#"{"alg":"none"}"#), payload);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            axum::http::HeaderValue::from_str(&format!("Bearer {forged}")).unwrap(),
        );

        assert!(jwt_claims(&headers).is_none());
        assert!(!jwt_is_admin(&headers));
        assert!(jwt_user_id(&headers).is_none());
        assert!(jwt_org_id(&headers).is_none());
    }

    #[test]
    fn headers_without_an_algorithm_are_refused() {
        let payload = crate::b64::encode_standard(br#"{"role":"admin"}"#);
        let forged = format!("{}.{}.c2ln", crate::b64::encode_standard(br#"{"typ":"JWT"}"#), payload);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            axum::http::HeaderValue::from_str(&format!("Bearer {forged}")).unwrap(),
        );

        assert!(!jwt_is_admin(&headers));
    }
}
