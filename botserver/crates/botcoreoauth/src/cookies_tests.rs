//! Offline tests for the OAuth `Set-Cookie` attributes (#2130).
//!
//! The issue's acceptance list is exactly what is asserted here: `Secure`,
//! `HttpOnly` and `SameSite` on both the session and the browser-binding
//! nonce cookie.

use crate::cookies::{
    clear_nonce_cookie, cookie_value, nonce_cookie, session_cookie, LINK_NONCE_PATH,
    LOGIN_NONCE_PATH, NONCE_COOKIE, SESSION_COOKIE,
};
use axum::http::{header, HeaderMap, HeaderValue};

fn value_of(cookie: Option<HeaderValue>) -> String {
    let cookie = cookie.expect("cookie must be buildable");
    cookie.to_str().expect("cookie must be valid ascii").to_string()
}

fn assert_secure_flags(cookie: &str) {
    assert!(cookie.contains("Secure"), "missing Secure: {cookie}");
    assert!(cookie.contains("HttpOnly"), "missing HttpOnly: {cookie}");
    assert!(cookie.contains("SameSite=Lax"), "missing SameSite=Lax: {cookie}");
    assert!(
        !cookie.contains("SameSite=None"),
        "SameSite=None is never needed for a top-level callback: {cookie}"
    );
}

#[test]
fn the_session_cookie_is_secure_httponly_and_lax() {
    let cookie = value_of(session_cookie("abc-123"));
    assert!(cookie.starts_with("session=abc-123;"), "{cookie}");
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("Max-Age=86400"));
    assert_secure_flags(&cookie);
}

#[test]
fn the_nonce_cookie_is_secure_httponly_and_lax() {
    let cookie = value_of(nonce_cookie("nonce-value", LOGIN_NONCE_PATH));
    assert!(cookie.starts_with(&format!("{NONCE_COOKIE}=nonce-value;")), "{cookie}");
    assert!(cookie.contains(&format!("Path={LOGIN_NONCE_PATH}")), "{cookie}");
    assert!(cookie.contains("Max-Age=600"), "{cookie}");
    assert_secure_flags(&cookie);
}

#[test]
fn the_link_flow_uses_its_own_cookie_path() {
    let cookie = value_of(nonce_cookie("n", LINK_NONCE_PATH));
    assert!(cookie.contains(&format!("Path={LINK_NONCE_PATH}")), "{cookie}");
    assert_secure_flags(&cookie);
    // Login and link flows must not clobber each other's nonce.
    assert_ne!(LOGIN_NONCE_PATH, LINK_NONCE_PATH);
}

#[test]
fn clearing_the_nonce_cookie_expires_it() {
    let cookie = value_of(clear_nonce_cookie(LOGIN_NONCE_PATH));
    assert!(cookie.starts_with(&format!("{NONCE_COOKIE}=;")), "{cookie}");
    assert!(cookie.contains("Max-Age=0"), "{cookie}");
    assert_secure_flags(&cookie);
}

#[test]
fn the_session_cookie_name_is_the_one_the_server_reads() {
    assert_eq!(SESSION_COOKIE, "session");
}

#[test]
fn a_cookie_value_is_read_from_the_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_static("session=abc; gb_oauth_nonce=xyz; theme=dark"),
    );
    assert_eq!(cookie_value(&headers, NONCE_COOKIE).as_deref(), Some("xyz"));
    assert_eq!(cookie_value(&headers, SESSION_COOKIE).as_deref(), Some("abc"));
    assert_eq!(cookie_value(&headers, "absent"), None);
}

#[test]
fn a_missing_or_empty_cookie_is_none() {
    assert_eq!(cookie_value(&HeaderMap::new(), NONCE_COOKIE), None);

    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, HeaderValue::from_static("gb_oauth_nonce="));
    assert_eq!(cookie_value(&headers, NONCE_COOKIE), None);

    headers.insert(header::COOKIE, HeaderValue::from_static("gb_oauth_nonce"));
    assert_eq!(cookie_value(&headers, NONCE_COOKIE), None);
}

#[test]
fn the_cookie_parser_does_not_confuse_similar_names() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_static("xgb_oauth_nonce=evil; gb_oauth_nonce_real=evil2"),
    );
    assert_eq!(cookie_value(&headers, NONCE_COOKIE), None);
}

#[test]
fn the_cookie_parser_reads_multiple_cookie_headers() {
    let mut headers = HeaderMap::new();
    headers.append(header::COOKIE, HeaderValue::from_static("a=1"));
    headers.append(header::COOKIE, HeaderValue::from_static("gb_oauth_nonce=second"));
    assert_eq!(
        cookie_value(&headers, NONCE_COOKIE).as_deref(),
        Some("second")
    );
}
