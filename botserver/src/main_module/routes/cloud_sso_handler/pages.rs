//! HTML handoff pages used by the SSO hops.
//!
//! The token never appears in a redirect query string: the page stores it in
//! `localStorage` and then rewrites the URL client-side.

use axum::{
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Response},
};
use botcoreoauth::RedirectPolicy;

/// Token handoff page shared by the JWT and opaque-session paths.
pub(super) fn sso_html(token: &str, redirect: &str) -> String {
    let redirect_js = js_string_literal(redirect);
    let token_js = js_string_literal(token);
    format!(
        r##"<!DOCTYPE html>
<html>
<head>
<meta name="referrer" content="no-referrer">
<script>
try {{
  localStorage.setItem('gb-access-token',{token_js});
  window.history.replaceState({{}},'',{redirect_js});
  window.location.replace({redirect_js});
}} catch(e) {{
  console.error('suite-sso:',e);
}}
</script>
</head>
<body></body>
</html>"##
    )
}

/// Invalid or expired SSO hops show a readable page and hand off to sign-in
/// client-side. A 3xx here would be followed by the botui proxy into a plain
/// 401 JSON, which is what produced the blank error screen (#1660).
pub(super) fn redirect_to_login(reason: &str) -> Response {
    let login = std::env::var("GB_LOGIN_URL")
        .ok()
        .filter(|value| value.starts_with("http"))
        .unwrap_or_else(|| "/login".to_string());
    let target = format!("{}/login?error={}", login.trim_end_matches('/'), reason);
    let target_js = js_string_literal(&target);
    Html(format!(
        r##"<!DOCTYPE html>
<html><head><meta name="referrer" content="no-referrer"><title>Sign in again</title></head>
<body style="font:15px system-ui;padding:40px;color:#334155">
<p>Your session expired or is invalid.</p>
<p><a href="{target}">Continue to sign in</a></p>
<script>setTimeout(function(){{ window.location.replace({target_js}); }}, 800);</script>
</body></html>"##,
        target = target
    ))
    .into_response()
}

pub(super) fn session_cookie(token: String, html: Html<String>) -> Response {
    match HeaderValue::from_str(&format!(
        "gb-access-token={token}; Path=/; Max-Age=86400; SameSite=Lax; HttpOnly; Secure"
    )) {
        Ok(value) => ([(header::SET_COOKIE, value)], html).into_response(),
        Err(e) => {
            log::warn!("suite-sso: refusing to set an unusable session cookie: {e}");
            html.into_response()
        }
    }
}

pub(super) fn sanitize_redirect(url: &str) -> String {
    // #2813 — the old code accepted any parseable absolute http(s) URL, so a
    // foreign host passed despite the "same-origin" comment. Delegate to the
    // shared RedirectPolicy: it rejects `//host` protocol-relative inputs,
    // control characters and any absolute URL whose host is not the deployment
    // host (GB_LOGIN_URL) or an explicitly configured allowlist entry.
    let policy = match std::env::var("GB_LOGIN_URL") {
        Ok(base) if !base.trim().is_empty() => {
            RedirectPolicy::from_base_url(&base).with_env_allowlist()
        }
        _ => RedirectPolicy::default().with_env_allowlist(),
    };
    policy.sanitize(Some(url))
}

#[cfg(test)]
mod redirect_tests {
    use super::sanitize_redirect;

    #[test]
    fn foreign_absolute_and_protocol_relative_redirects_are_rejected() {
        // No GB_LOGIN_URL / allowlist in the test env: only relative paths pass.
        for hostile in [
            "https://evil.example/steal",
            "http://evil.example:8080/x",
            "//evil.example/x",
            "///evil.example/x",
            "/\\evil.example",
            "javascript:alert(1)",
        ] {
            assert_eq!(sanitize_redirect(hostile), "/", "hostile: {hostile}");
        }
        assert_eq!(sanitize_redirect("/drive"), "/drive");
    }
}

/// A JavaScript string literal that cannot terminate the enclosing
/// `<script>` element (#1533).
///
/// `serde_json` quotes and escapes the value, but leaves `<`, `>` and `&`
/// intact, so a value containing `</script>` would still close the element in
/// an HTML parser. The line separators U+2028/U+2029 also end a JS string
/// literal. Escaping all five as `\uXXXX` keeps the value identical while
/// making it inert in both contexts.
fn js_string_literal(value: &str) -> String {
    let json = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
    let mut out = String::with_capacity(json.len() + 8);
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod script_literal_tests {
    use super::js_string_literal;

    #[test]
    fn closes_no_script_element() {
        let literal = js_string_literal("/?next=</script><script>alert(1)</script>");
        assert!(
            !literal.contains('<'),
            "raw '<' must not survive: {literal}"
        );
        assert!(
            !literal.contains('>'),
            "raw '>' must not survive: {literal}"
        );
        assert!(literal.starts_with('"') && literal.ends_with('"'));
    }

    #[test]
    fn escapes_line_separators_and_ampersand() {
        let literal = js_string_literal("a\u{2028}b\u{2029}c&d");
        assert!(literal.contains("\\u2028"));
        assert!(literal.contains("\\u2029"));
        assert!(literal.contains("\\u0026"));
    }

    #[test]
    fn round_trips_through_json() {
        // The escaping must not change the value the browser evaluates.
        let original = "https://example.test/path?q=1&r=<2>";
        let literal = js_string_literal(original);
        let decoded: String = serde_json::from_str(&literal).expect("valid JSON string");
        assert_eq!(decoded, original);
    }
}
