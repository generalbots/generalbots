//! Offline tests for the mail OAuth2 helpers (issue #2075): provider aliases,
//! consent URL, token expiry decision and the stored-credential encoding.

use super::*;

#[test]
fn provider_aliases_cover_every_supported_spelling() {
    let cases = [
        ("microsoft", MailOAuthProvider::Microsoft),
        ("M365", MailOAuthProvider::Microsoft),
        ("outlook", MailOAuthProvider::Microsoft),
        ("office365", MailOAuthProvider::Microsoft),
        ("exchange", MailOAuthProvider::Microsoft),
        ("google", MailOAuthProvider::Google),
        ("Gmail", MailOAuthProvider::Google),
    ];
    for (value, expected) in cases {
        assert_eq!(MailOAuthProvider::parse(value), Some(expected), "{value}");
    }
}

#[test]
fn unknown_provider_names_are_rejected() {
    for value in ["", "yahoo", "imap"] {
        assert_eq!(MailOAuthProvider::parse(value), None, "{value}");
    }
}

#[test]
fn provider_strings_and_default_servers_are_stable() {
    assert_eq!(MailOAuthProvider::Microsoft.as_str(), "microsoft");
    assert_eq!(MailOAuthProvider::Google.as_str(), "google");
    assert_eq!(
        MailOAuthProvider::Microsoft.default_imap_server(),
        "outlook.office365.com"
    );
    assert_eq!(
        MailOAuthProvider::Microsoft.default_smtp_server(),
        "smtp.office365.com"
    );
    assert_eq!(MailOAuthProvider::Google.default_imap_server(), "imap.gmail.com");
    assert_eq!(MailOAuthProvider::Google.default_smtp_server(), "smtp.gmail.com");
}

fn query_value(url: &str, key: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()?
        .query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

#[test]
fn microsoft_consent_url_carries_every_required_parameter() {
    let url = consent_url(
        MailOAuthProvider::Microsoft,
        "client-1",
        "https://app.test/api/email/oauth/callback",
        "state-9",
    );

    assert!(url.starts_with(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?"
    ));
    assert_eq!(query_value(&url, "client_id").as_deref(), Some("client-1"));
    assert_eq!(query_value(&url, "response_type").as_deref(), Some("code"));
    assert_eq!(
        query_value(&url, "redirect_uri").as_deref(),
        Some("https://app.test/api/email/oauth/callback")
    );
    assert_eq!(query_value(&url, "response_mode").as_deref(), Some("query"));
    assert_eq!(query_value(&url, "state").as_deref(), Some("state-9"));
    assert_eq!(query_value(&url, "prompt").as_deref(), Some("consent"));
    let scope = query_value(&url, "scope").expect("scope");
    assert!(scope.contains("offline_access"));
    assert!(scope.contains("IMAP.AccessAsUser.All"));
}

#[test]
fn google_consent_url_uses_the_google_endpoint_and_mail_scope() {
    let url = consent_url(
        MailOAuthProvider::Google,
        "client-2",
        "https://app.test/cb",
        "s",
    );

    assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
    assert_eq!(
        query_value(&url, "scope").as_deref(),
        Some("https://mail.google.com/")
    );
    assert_eq!(query_value(&url, "client_id").as_deref(), Some("client-2"));
}

#[test]
fn consent_url_percent_encodes_the_redirect_uri() {
    let url = consent_url(
        MailOAuthProvider::Google,
        "client",
        "https://app.test/cb?x=1&y=2",
        "state with space",
    );

    assert!(url.contains("redirect_uri=https%3A%2F%2Fapp.test%2Fcb%3Fx%3D1%26y%3D2"));
    assert_eq!(query_value(&url, "state").as_deref(), Some("state with space"));
}

#[test]
fn tokens_are_refreshed_when_they_are_expired_or_about_to_expire() {
    let expired = TokenSet {
        access_token: "a".to_string(),
        refresh_token: None,
        expires_at: Utc::now() - Duration::seconds(1),
    };
    let within_skew = TokenSet {
        access_token: "a".to_string(),
        refresh_token: None,
        expires_at: Utc::now() + Duration::seconds(30),
    };
    let fresh = TokenSet {
        access_token: "a".to_string(),
        refresh_token: None,
        expires_at: Utc::now() + Duration::seconds(3600),
    };

    assert!(expired.needs_refresh());
    assert!(within_skew.needs_refresh());
    assert!(!fresh.needs_refresh());
}

#[test]
fn encode_secret_matches_the_rfc_4648_standard_vectors() {
    let vectors = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];
    for (plain, encoded) in vectors {
        assert_eq!(encode_secret(plain), encoded, "plain {plain}");
        assert_eq!(decode_secret(encoded), Ok(plain.to_string()));
    }
}

#[test]
fn decode_secret_rejects_invalid_base64() {
    let error = decode_secret("not base64!!").expect_err("invalid base64");
    assert!(error.contains("could not be decoded"), "{error}");
}

#[test]
fn oauth2_credentials_are_flagged_by_mechanism() {
    let oauth = MailboxCredentials {
        username: "user@example.com".to_string(),
        secret: "token".to_string(),
        mechanism: MailMechanism::Xoauth2,
    };
    let password = MailboxCredentials {
        username: "user@example.com".to_string(),
        secret: "secret".to_string(),
        mechanism: MailMechanism::Password,
    };

    assert!(oauth.is_oauth2());
    assert!(!password.is_oauth2());
}
