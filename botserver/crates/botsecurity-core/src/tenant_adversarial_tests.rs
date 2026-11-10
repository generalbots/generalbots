//! Adversarial matrix for the JWT claim → tenant resolver (#2066).
//!
//! `branch_from_claims` / `org_from_claims` decide which tenant every
//! CRM/Drive/KYC/compliance handler reads and writes. These tests pin the
//! accept/reject boundary of the segment, header and claim parser, including
//! the cases that are **known defects** and must not be mistaken for a fix.

use super::*;
use axum::http::{HeaderMap, HeaderValue};
use uuid::Uuid;

fn b64(input: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
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

fn token(header: &str, payload: &str, signature: &str) -> String {
    format!("{}.{}.{}", b64(header), b64(payload), signature)
}

fn bearer(header: &str, payload: &str) -> HeaderMap {
    let mut map = HeaderMap::new();
    let value = format!("Bearer {}", token(header, payload, "signature"));
    map.insert("authorization", HeaderValue::from_str(&value).expect("header"));
    map
}

const SECURED: &str = r#"{"alg":"HS256","typ":"JWT"}"#;
const BRANCH: Uuid = Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111);
const ORG: Uuid = Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222);

fn claims() -> String {
    format!(
        r#"{{"branch_id":"{BRANCH}","org_id":"{ORG}","sub":"user-1","email":"u@example.com"}}"#
    )
}

#[test]
fn a_well_formed_secured_token_is_fully_accepted() {
    let map = bearer(SECURED, &claims());
    assert_eq!(branch_from_claims(&map), Some(BRANCH));
    assert_eq!(org_from_claims(&map), Some(ORG));
    assert_eq!(user_id_from_claims(&map).as_deref(), Some("user-1"));
    assert_eq!(email_from_claims(&map).as_deref(), Some("u@example.com"));
}

#[test]
fn the_organization_id_alias_is_honoured() {
    let payload = format!(r#"{{"organization_id":"{ORG}"}}"#);
    let map = bearer(SECURED, &payload);
    assert_eq!(org_from_claims(&map), Some(ORG));
}

#[test]
fn a_payload_without_org_or_branch_yields_none() {
    let map = bearer(SECURED, r#"{"sub":"only-sub"}"#);
    assert_eq!(branch_from_claims(&map), None);
    assert_eq!(org_from_claims(&map), None);
    assert_eq!(user_id_from_claims(&map).as_deref(), Some("only-sub"));
}

#[test]
fn wrong_typed_claims_are_ignored_not_coerced() {
    let payload = r#"{"branch_id":123,"org_id":true,"sub":42,"email":["a"]}"#;
    let map = bearer(SECURED, payload);
    assert_eq!(branch_from_claims(&map), None);
    assert_eq!(org_from_claims(&map), None);
    assert_eq!(user_id_from_claims(&map), None);
    assert_eq!(email_from_claims(&map), None);

    let nested = format!(r#"{{"branch_id":{{"id":"{BRANCH}"}}}}"#);
    assert_eq!(branch_from_claims(&bearer(SECURED, &nested)), None);
}

#[test]
fn only_three_segments_are_accepted() {
    let good = b64(SECURED);
    let payload = b64(&claims());
    let cases = [
        format!("{good}.{payload}"),
        format!("{good}.{payload}.sig.extra"),
        format!(".{payload}.sig"),
        format!("{good}..sig"),
        format!("{good}.{payload}."),
        String::new(),
    ];
    for raw in cases {
        let mut map = HeaderMap::new();
        map.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {raw}")).expect("header"),
        );
        assert_eq!(branch_from_claims(&map), None, "{raw:?} must be refused");
    }
}

#[test]
fn payloads_that_are_not_json_objects_are_refused() {
    for payload in ["null", "true", "[1,2]", r#""branch_id""#, "not json"] {
        let map = bearer(SECURED, payload);
        assert_eq!(branch_from_claims(&map), None, "{payload} must be refused");
    }
}

#[test]
fn headers_that_are_not_json_or_lack_an_algorithm_are_refused() {
    for header in [
        "not json",
        "null",
        r#"{"typ":"JWT"}"#,
        r#"{"alg":null}"#,
        r#"{"alg":256}"#,
        r#"{"alg":""}"#,
        r#"{"alg":"   "}"#,
    ] {
        let map = bearer(header, &claims());
        assert_eq!(
            branch_from_claims(&map),
            None,
            "header {header} must be refused"
        );
    }
}

#[test]
fn any_algorithm_other_than_none_is_read() {
    for header in [
        r#"{"alg":"HS256"}"#,
        r#"{"alg":"RS256"}"#,
        r#"{"alg":"ES256"}"#,
        r#"{"alg":"PS512"}"#,
        r#"{"alg":"EdDSA"}"#,
        r#"{"alg":"hs256"}"#,
    ] {
        assert_eq!(
            branch_from_claims(&bearer(header, &claims())),
            Some(BRANCH),
            "header {header} must be accepted"
        );
    }
}

#[test]
fn padded_segments_are_tolerated() {
    let padded = format!(
        "{}.{}.sig",
        format!("{}=", b64(SECURED)),
        format!("{}=", b64(&claims()))
    );
    let mut map = HeaderMap::new();
    map.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {padded}")).expect("header"),
    );
    assert_eq!(branch_from_claims(&map), Some(BRANCH));
}

#[test]
fn a_non_utf8_payload_is_refused() {
    let mut map = HeaderMap::new();
    let raw = format!("Bearer {}.{}.sig", b64(SECURED), b64("no"));
    map.insert("authorization", HeaderValue::from_str(&raw).expect("header"));
    assert_eq!(branch_from_claims(&map), None);

    let invalid = format!("Bearer {}.@@@.sig", b64(SECURED));
    map.insert("authorization", HeaderValue::from_str(&invalid).expect("header"));
    assert_eq!(branch_from_claims(&map), None);
}

#[test]
fn the_bearer_scheme_is_case_insensitive() {
    let jwt = token(SECURED, &claims(), "sig");
    for scheme in ["Bearer", "bearer", "BEARER", "BeArEr"] {
        let mut map = HeaderMap::new();
        map.insert(
            "authorization",
            HeaderValue::from_str(&format!("{scheme} {jwt}")).expect("header"),
        );
        assert_eq!(
            user_id_from_claims(&map).as_deref(),
            Some("user-1"),
            "scheme {scheme} must be accepted"
        );
    }
}

#[test]
fn basic_transport_without_a_password_field_is_refused() {
    let encoded = base64::engine::general_purpose::STANDARD.encode("user-only");
    let mut map = HeaderMap::new();
    map.insert(
        "authorization",
        HeaderValue::from_str(&format!("Basic {encoded}")).expect("header"),
    );
    assert_eq!(branch_from_claims(&map), None);

    let empty = base64::engine::general_purpose::STANDARD.encode("user:");
    map.insert(
        "authorization",
        HeaderValue::from_str(&format!("Basic {empty}")).expect("header"),
    );
    assert_eq!(branch_from_claims(&map), None);
}

#[test]
fn a_basic_header_with_non_ascii_prefix_never_panics() {
    // The prefix check must stay on a character boundary; a multi-byte char in
    // the first six bytes used to be a slicing panic.
    for value in ["Básico abc", "ééééééé", "Bearer"] {
        let mut map = HeaderMap::new();
        if let Ok(header) = HeaderValue::from_str(value) {
            map.insert("authorization", header);
        }
        assert_eq!(branch_from_claims(&map), None, "{value:?} must be refused");
    }
}

#[test]
fn the_raw_token_subject_path_applies_the_same_header_gate() {
    // #2066 — the SSE `?token=` path read `sub` from an unsigned token. It now
    // refuses `alg=none` like the header path does.
    let forged = token(r#"{"alg":"none"}"#, &claims(), "");
    assert_eq!(user_id_from_claims_subject(&forged), None);

    let secured = token(SECURED, &claims(), "sig");
    assert_eq!(
        user_id_from_claims_subject(&secured).as_deref(),
        Some("user-1")
    );
    assert_eq!(
        user_id_from_claims_subject(&format!("Bearer {secured}")).as_deref(),
        Some("user-1")
    );
    assert_eq!(user_id_from_claims_subject("a.b"), None);
    assert_eq!(user_id_from_claims_subject(""), None);
}

#[test]
fn a_forged_signature_is_accepted_by_the_claim_reader_known_defect() {
    // KNOWN DEFECT (#2066 SEC-CORE-15, follow-up: verify the signature before
    // reading claims). `branch_from_claims` only decodes; the request is
    // authenticated elsewhere. A forged HS256 token therefore still selects a
    // branch here. This test pins the current behaviour so a future signature
    // check is a deliberate, visible change rather than a silent regression.
    let forged = token(SECURED, &claims(), "definitely-not-a-valid-signature");
    let mut map = HeaderMap::new();
    map.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {forged}")).expect("header"),
    );
    assert_eq!(branch_from_claims(&map), Some(BRANCH));
}

#[test]
fn an_expired_token_is_accepted_by_the_claim_reader_known_defect() {
    // KNOWN DEFECT (#2066 SEC-CORE-15): `exp`/`nbf` are not evaluated here;
    // expiry is enforced by the JWT validator on the authentication path.
    let expired = format!(
        r#"{{"branch_id":"{BRANCH}","exp":1,"nbf":9999999999}}"#
    );
    assert_eq!(branch_from_claims(&bearer(SECURED, &expired)), Some(BRANCH));
}

#[test]
fn an_unsigned_token_can_never_select_a_tenant() {
    // The one case that must stay refused at every entry point.
    for header in [r#"{"alg":"none"}"#, r#"{"alg":"NONE"}"#, r#"{"alg":"None"}"#] {
        let map = bearer(header, &claims());
        assert_eq!(branch_from_claims(&map), None);
        assert_eq!(org_from_claims(&map), None);
        assert_eq!(user_id_from_claims(&map), None);
        assert_eq!(email_from_claims(&map), None);
    }
}
