use super::*;

fn b64url(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 63) as usize] as char);
        }
    }
    out
}

fn token(header: &str, payload: &str) -> String {
    format!(
        "{}.{}.{}",
        b64url(header.as_bytes()),
        b64url(payload.as_bytes()),
        b64url(b"signature")
    )
}

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        axum::http::HeaderValue::from_str(&format!("Bearer {token}")).expect("header"),
    );
    headers
}

#[test]
fn base64url_decodes_url_safe_and_ignores_padding() {
    let encoded = b64url(b"{\"sub\":\"user\"}");
    assert_eq!(
        base64_url_decode(&encoded).expect("decoded"),
        b"{\"sub\":\"user\"}"
    );
    assert_eq!(
        base64_url_decode(&format!("{encoded}==")).expect("decoded"),
        b"{\"sub\":\"user\"}"
    );
    assert!(base64_url_decode("!!!").is_err());
}

#[test]
fn secured_tokens_expose_their_payload() {
    let payload = r#"{"uid":"00000000-0000-0000-0000-000000000001"}"#;
    let jwt = token(r#"{"alg":"HS256","typ":"JWT"}"#, payload);

    assert_eq!(jwt_payload(&jwt).and_then(|p| p.get("uid").cloned()), Some(serde_json::json!("00000000-0000-0000-0000-000000000001")));
}

#[test]
fn unsecured_tokens_are_refused() {
    let payload = r#"{"uid":"00000000-0000-0000-0000-000000000001"}"#;

    assert!(jwt_payload(&token(r#"{"alg":"none"}"#, payload)).is_none());
    assert!(jwt_payload(&token(r#"{"alg":"NONE"}"#, payload)).is_none());
    assert!(jwt_payload(&token(r#"{"typ":"JWT"}"#, payload)).is_none());
}

#[test]
fn malformed_tokens_are_refused() {
    assert!(jwt_payload("a.b").is_none());
    assert!(jwt_payload("a.b.c.d").is_none());
    assert!(jwt_payload("not-a-token").is_none());
    assert!(jwt_payload(&format!("{}.!!!.sig", b64url(br#"{"alg":"HS256"}"#))).is_none());
}

#[test]
fn resolve_user_id_prefers_the_uid_claim() {
    let headers = headers_with(&token(
        r#"{"alg":"HS256"}"#,
        r#"{"uid":"00000000-0000-0000-0000-00000000000a","sub":"00000000-0000-0000-0000-00000000000b"}"#,
    ));

    assert_eq!(
        resolve_user_id(&headers).expect("user"),
        Uuid::parse_str("00000000-0000-0000-0000-00000000000a").expect("uuid")
    );
}

#[test]
fn resolve_user_id_falls_back_to_sub() {
    let headers = headers_with(&token(
        r#"{"alg":"HS256"}"#,
        r#"{"sub":"00000000-0000-0000-0000-00000000000b"}"#,
    ));

    assert_eq!(
        resolve_user_id(&headers).expect("user"),
        Uuid::parse_str("00000000-0000-0000-0000-00000000000b").expect("uuid")
    );
}

#[test]
fn resolve_user_id_rejects_missing_invalid_and_unsecured_tokens() {
    assert!(resolve_user_id(&HeaderMap::new()).is_err());

    let no_subject = headers_with(&token(r#"{"alg":"HS256"}"#, r#"{"role":"admin"}"#));
    assert!(resolve_user_id(&no_subject).is_err());

    let invalid_uuid = headers_with(&token(r#"{"alg":"HS256"}"#, r#"{"uid":"nope"}"#));
    assert!(resolve_user_id(&invalid_uuid).is_err());

    let forged = headers_with(&token(
        r#"{"alg":"none"}"#,
        r#"{"uid":"00000000-0000-0000-0000-000000000001"}"#,
    ));
    assert!(resolve_user_id(&forged).is_err());
}

#[test]
fn sse_resolution_prefers_the_header_over_the_query_token() {
    let header_jwt = token(
        r#"{"alg":"HS256"}"#,
        r#"{"uid":"00000000-0000-0000-0000-00000000000a"}"#,
    );
    let query_jwt = token(
        r#"{"alg":"HS256"}"#,
        r#"{"uid":"00000000-0000-0000-0000-00000000000b"}"#,
    );

    let resolved = resolve_user_id_sse(&headers_with(&header_jwt), &query_jwt).expect("header wins");
    assert_eq!(
        resolved,
        Uuid::parse_str("00000000-0000-0000-0000-00000000000a").expect("uuid")
    );

    let resolved = resolve_user_id_sse(&HeaderMap::new(), &query_jwt).expect("query token");
    assert_eq!(
        resolved,
        Uuid::parse_str("00000000-0000-0000-0000-00000000000b").expect("uuid")
    );
}

#[test]
fn sse_resolution_requires_some_token() {
    assert!(resolve_user_id_sse(&HeaderMap::new(), "").is_err());
}
