//! Adversarial matrix for the verified JWT claim → tenant resolver (#2112).
//!
//! `branch_from_claims` / `org_from_claims` decide which tenant every
//! CRM/Drive/KYC/compliance handler reads and writes. A claim is only returned
//! when the HS256 signature verifies (in constant time) against the configured
//! SaaS secret and the token has not expired, so every forgery strategy below
//! must yield `None` — never a fallback to the decoded payload.

use super::*;
use crate::verified_jwt::test_support::{mint_hs256, process_secret, signed, tenant_claims};
use axum::http::{HeaderMap, HeaderValue};
use uuid::Uuid;

const BRANCH: Uuid = Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111);
const ORG: Uuid = Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222);
const BRANCH_STR: &str = "11111111-1111-1111-1111-111111111111";
const ORG_STR: &str = "22222222-2222-2222-2222-222222222222";

fn claims() -> serde_json::Value {
    tenant_claims(BRANCH_STR, ORG_STR, "user-1", "u@example.com")
}

fn b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Mints a token over the exact segments given, so padding and header
/// tampering can be exercised without touching the fixture.
fn mint_segments(header_b64: &str, payload_b64: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let signing_input = format!("{header_b64}.{payload_b64}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(signing_input.as_bytes());
    let signature = b64(&mac.finalize().into_bytes());
    format!("{signing_input}.{signature}")
}

fn headers(auth: &str) -> HeaderMap {
    let mut map = HeaderMap::new();
    map.insert("authorization", HeaderValue::from_str(auth).expect("header"));
    map
}

fn bearer(token: &str) -> HeaderMap {
    headers(&format!("Bearer {token}"))
}

#[test]
fn a_well_formed_secured_token_is_fully_accepted() {
    let map = bearer(&signed(&claims()));
    assert_eq!(branch_from_claims(&map), Some(BRANCH));
    assert_eq!(org_from_claims(&map), Some(ORG));
    assert_eq!(user_id_from_claims(&map).as_deref(), Some("user-1"));
    assert_eq!(email_from_claims(&map).as_deref(), Some("u@example.com"));
}

#[test]
fn the_organization_id_alias_is_honoured() {
    let mut payload = claims();
    if let Some(object) = payload.as_object_mut() {
        object.remove("org_id");
        object.insert(
            "organization_id".to_string(),
            serde_json::Value::String(ORG_STR.to_string()),
        );
    }
    assert_eq!(org_from_claims(&bearer(&signed(&payload))), Some(ORG));
}

#[test]
fn a_payload_without_org_or_branch_yields_none() {
    let payload = serde_json::json!({
        "sub": "only-sub",
        "exp": chrono::Utc::now().timestamp() + 600,
    });
    let map = bearer(&signed(&payload));
    assert_eq!(branch_from_claims(&map), None);
    assert_eq!(org_from_claims(&map), None);
    assert_eq!(user_id_from_claims(&map).as_deref(), Some("only-sub"));
}

#[test]
fn wrong_typed_claims_are_ignored_not_coerced() {
    let payload = serde_json::json!({
        "branch_id": 123,
        "org_id": true,
        "sub": "user-1",
        "email": ["a"],
        "exp": chrono::Utc::now().timestamp() + 600,
    });
    let map = bearer(&signed(&payload));
    assert_eq!(branch_from_claims(&map), None);
    assert_eq!(org_from_claims(&map), None);
    assert_eq!(email_from_claims(&map), None);

    let nested = serde_json::json!({
        "branch_id": {"id": BRANCH_STR},
        "sub": "user-1",
        "exp": chrono::Utc::now().timestamp() + 600,
    });
    assert_eq!(branch_from_claims(&bearer(&signed(&nested))), None);
}

#[test]
fn only_three_segments_are_accepted() {
    let good = signed(&claims());
    let parts: Vec<&str> = good.split('.').collect();
    let (head, payload, signature) = (parts[0], parts[1], parts[2]);
    let cases = [
        format!("{head}.{payload}"),
        format!("{head}.{payload}.{signature}.extra"),
        format!(".{payload}.{signature}"),
        format!("{head}..{signature}"),
        format!("{head}.{payload}."),
        String::new(),
    ];
    for raw in cases {
        assert_eq!(branch_from_claims(&bearer(&raw)), None, "{raw:?} refused");
    }
}

#[test]
fn payloads_that_are_not_json_objects_are_refused() {
    let secret = process_secret();
    let header = b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    for payload in ["null", "true", "[1,2]", r#""branch_id""#, "not json"] {
        let token = mint_segments(&header, &b64(payload.as_bytes()), &secret);
        assert_eq!(
            branch_from_claims(&bearer(&token)),
            None,
            "{payload} must be refused"
        );
    }
}

#[test]
fn headers_that_are_not_json_or_lack_an_algorithm_are_refused() {
    let secret = process_secret();
    let payload = b64(claims().to_string().as_bytes());
    for header in [
        "not json",
        "null",
        r#"{"typ":"JWT"}"#,
        r#"{"alg":null}"#,
        r#"{"alg":256}"#,
        r#"{"alg":""}"#,
        r#"{"alg":"   "}"#,
    ] {
        let token = mint_segments(&b64(header.as_bytes()), &payload, &secret);
        assert_eq!(
            branch_from_claims(&bearer(&token)),
            None,
            "header {header} must be refused"
        );
    }
}

#[test]
fn only_hs256_is_accepted() {
    // Algorithm confusion: the server only mints HS256, so an asymmetric or
    // downgraded algorithm is refused even when the signature bytes are valid.
    let secret = process_secret();
    let payload = b64(claims().to_string().as_bytes());
    for header in [
        r#"{"alg":"RS256"}"#,
        r#"{"alg":"ES256"}"#,
        r#"{"alg":"PS512"}"#,
        r#"{"alg":"EdDSA"}"#,
        r#"{"alg":"hs256"}"#,
        r#"{"alg":"none"}"#,
        r#"{"alg":"NONE"}"#,
    ] {
        let token = mint_segments(&b64(header.as_bytes()), &payload, &secret);
        assert_eq!(
            branch_from_claims(&bearer(&token)),
            None,
            "header {header} must be refused"
        );
    }
}

#[test]
fn padded_segments_are_tolerated_when_the_signature_matches() {
    use base64::Engine as _;
    let padded_header = base64::engine::general_purpose::URL_SAFE
        .encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let padded_payload = base64::engine::general_purpose::URL_SAFE
        .encode(claims().to_string().as_bytes());
    let token = mint_segments(&padded_header, &padded_payload, &process_secret());
    assert_eq!(branch_from_claims(&bearer(&token)), Some(BRANCH));
}

#[test]
fn a_non_utf8_or_non_base64_payload_is_refused() {
    let secret = process_secret();
    let header = b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    let token = mint_segments(&header, &b64(&[0xFF, 0xFE]), &secret);
    assert_eq!(branch_from_claims(&bearer(&token)), None);

    let invalid = format!("{header}.@@@.{secret}");
    assert_eq!(branch_from_claims(&bearer(&invalid)), None);
}

#[test]
fn the_bearer_scheme_is_case_insensitive() {
    let jwt = signed(&claims());
    for scheme in ["Bearer", "bearer", "BEARER", "BeArEr"] {
        assert_eq!(
            user_id_from_claims(&headers(&format!("{scheme} {jwt}"))).as_deref(),
            Some("user-1"),
            "scheme {scheme} must be accepted"
        );
    }
}

#[test]
fn basic_transport_without_a_password_field_is_refused() {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode("user-only");
    assert_eq!(branch_from_claims(&headers(&format!("Basic {encoded}"))), None);

    let empty = base64::engine::general_purpose::STANDARD.encode("user:");
    assert_eq!(branch_from_claims(&headers(&format!("Basic {empty}"))), None);
}

#[test]
fn a_basic_header_with_non_ascii_prefix_never_panics() {
    // The prefix check must stay on a character boundary; a multi-byte char in
    // the first six bytes used to be a slicing panic.
    for value in ["Básico abc", "ééééééé", "Bearer"] {
        assert_eq!(
            branch_from_claims(&headers(value)),
            None,
            "{value:?} must be refused"
        );
    }
}

#[test]
fn the_raw_token_subject_path_verifies_the_signature() {
    let valid = signed(&claims());
    assert_eq!(
        user_id_from_claims_subject(&valid).as_deref(),
        Some("user-1")
    );
    assert_eq!(
        user_id_from_claims_subject(&format!("Bearer {valid}")).as_deref(),
        Some("user-1")
    );

    let forged = mint_hs256(&claims(), "definitely-not-the-server-key");
    assert_eq!(user_id_from_claims_subject(&forged), None);
    assert_eq!(user_id_from_claims_subject("a.b"), None);
    assert_eq!(user_id_from_claims_subject(""), None);
}

#[test]
fn a_forged_signature_is_refused() {
    // #2112 — the payload used to be trusted on its own; a token whose
    // signature does not match must never select a branch.
    let mut parts: Vec<String> = signed(&claims()).split('.').map(str::to_string).collect();
    parts[2] = "definitely-not-a-valid-signature".to_string();
    let forged = parts.join(".");
    let map = bearer(&forged);
    assert_eq!(branch_from_claims(&map), None);
    assert_eq!(org_from_claims(&map), None);
    assert_eq!(user_id_from_claims(&map), None);
    assert_eq!(email_from_claims(&map), None);
}

#[test]
fn a_token_signed_with_another_key_is_refused() {
    let other_key = format!("{}-attacker", process_secret());
    let token = mint_hs256(&claims(), &other_key);
    assert_eq!(branch_from_claims(&bearer(&token)), None);
}

#[test]
fn a_tampered_payload_is_refused() {
    // Re-encode the payload for a different tenant while keeping the original
    // signature: the HMAC no longer matches.
    let original = signed(&claims());
    let parts: Vec<&str> = original.split('.').collect();
    let mut stolen = claims();
    stolen["branch_id"] = serde_json::Value::String(Uuid::new_v4().to_string());
    let tampered = format!("{}.{}.{}", parts[0], b64(stolen.to_string().as_bytes()), parts[2]);
    assert_eq!(branch_from_claims(&bearer(&tampered)), None);
}

#[test]
fn an_expired_token_is_refused() {
    let mut payload = claims();
    payload["exp"] = serde_json::Value::from(1);
    assert_eq!(branch_from_claims(&bearer(&signed(&payload))), None);
}

#[test]
fn a_token_without_expiry_is_refused() {
    let mut payload = claims();
    if let Some(object) = payload.as_object_mut() {
        object.remove("exp");
    }
    assert_eq!(branch_from_claims(&bearer(&signed(&payload))), None);
}

#[test]
fn a_token_without_a_subject_cannot_scope_a_tenant() {
    let mut payload = claims();
    if let Some(object) = payload.as_object_mut() {
        object.remove("sub");
    }
    assert_eq!(branch_from_claims(&bearer(&signed(&payload))), None);
}

#[test]
fn an_unsigned_token_can_never_select_a_tenant() {
    let payload = b64(claims().to_string().as_bytes());
    for header in [r#"{"alg":"none"}"#, r#"{"alg":"NONE"}"#, r#"{"alg":"None"}"#] {
        let token = format!("{}.{payload}.", b64(header.as_bytes()));
        let map = bearer(&token);
        assert_eq!(branch_from_claims(&map), None);
        assert_eq!(org_from_claims(&map), None);
        assert_eq!(user_id_from_claims(&map), None);
        assert_eq!(email_from_claims(&map), None);
    }
}
