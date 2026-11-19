use super::{decode_jwt_payload, pkce_challenge};

#[test]
fn pkce_challenge_is_url_safe() {
    let challenge = pkce_challenge("verifier");
    assert!(!challenge.contains('='));
    assert!(!challenge.contains('+'));
    assert!(!challenge.contains('/'));
}

#[test]
fn jwt_payload_decodes() {
    // {"sub":"1"} in base64url, unsigned test token.
    let token = "x.eyJzdWIiOiIxIn0.y";
    let payload = decode_jwt_payload(token).expect("payload");
    assert_eq!(payload["sub"], "1");
}
