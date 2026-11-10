//! #2134 — every provider must report whether it actually verified the email
//! it returned. `None` is treated as unverified by the identity resolver, so a
//! provider that cannot prove ownership can never match a local account.

use super::OAuthProvider;
use serde_json::json;

fn parse(provider: OAuthProvider, raw: serde_json::Value) -> super::super::OAuthUserInfo {
    match provider.parse_user_info(&raw) {
        Ok(info) => info,
        Err(error) => panic!("provider payload should parse: {error}"),
    }
}

#[test]
fn google_reports_the_verified_email_flag() {
    let verified = parse(
        OAuthProvider::Google,
        json!({ "id": "1", "email": "user@example.com", "verified_email": true }),
    );
    assert_eq!(verified.email_verified, Some(true));

    let unverified = parse(
        OAuthProvider::Google,
        json!({ "id": "1", "email": "user@example.com", "verified_email": false }),
    );
    assert_eq!(unverified.email_verified, Some(false));

    let silent = parse(
        OAuthProvider::Google,
        json!({ "id": "1", "email": "user@example.com" }),
    );
    assert_eq!(silent.email_verified, None);
}

#[test]
fn google_accepts_the_openid_email_verified_claim() {
    let info = parse(
        OAuthProvider::Google,
        json!({ "id": "1", "email": "user@example.com", "email_verified": true }),
    );
    assert_eq!(info.email_verified, Some(true));
}

#[test]
fn discord_reports_the_verified_flag() {
    let info = parse(
        OAuthProvider::Discord,
        json!({ "id": "1", "email": "user@example.com", "verified": false }),
    );
    assert_eq!(info.email_verified, Some(false));
}

#[test]
fn microsoft_never_claims_a_verified_email() {
    // `mail`/`userPrincipalName` are tenant assertions, not proof of ownership.
    let info = parse(
        OAuthProvider::Microsoft,
        json!({ "id": "1", "mail": "user@example.com", "userPrincipalName": "u@tenant" }),
    );
    assert_eq!(info.email.as_deref(), Some("user@example.com"));
    assert_eq!(info.email_verified, None);
}

#[test]
fn facebook_never_claims_a_verified_email() {
    let info = parse(
        OAuthProvider::Facebook,
        json!({ "id": "1", "email": "user@example.com" }),
    );
    assert_eq!(info.email_verified, None);
}

#[test]
fn github_user_payload_is_not_proof_of_ownership() {
    // `/user` may return a public profile address; verification comes from
    // `/user/emails` (see `github_email`), so the bare payload stays unverified.
    let info = parse(
        OAuthProvider::GitHub,
        json!({ "id": 42, "login": "octocat", "email": "public@example.com" }),
    );
    assert_eq!(info.provider_id, "42");
    assert_eq!(info.email.as_deref(), Some("public@example.com"));
    assert_eq!(info.email_verified, None);
}

#[test]
fn providers_without_email_support_report_none() {
    let reddit = parse(OAuthProvider::Reddit, json!({ "id": "1", "name": "r" }));
    assert_eq!(reddit.email, None);
    assert_eq!(reddit.email_verified, None);

    let twitter = parse(
        OAuthProvider::Twitter,
        json!({ "data": { "id": "1", "name": "t" } }),
    );
    assert_eq!(twitter.email, None);
    assert_eq!(twitter.email_verified, None);
}
