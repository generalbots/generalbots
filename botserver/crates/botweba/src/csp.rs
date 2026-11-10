//! Isolation policy for published WebA documents (#2124).
//!
//! A published app is user-authored HTML with user-authored JavaScript. It is
//! never served as a trusted suite page: every document leaves the server with
//! a `Content-Security-Policy` whose `sandbox` directive (deliberately without
//! `allow-same-origin`) gives it an opaque origin. Script inside it therefore
//! cannot read the suite `localStorage` (where the chat session token lives),
//! send the suite cookies, or call authenticated APIs with the victim's
//! session. The embed shell repeats the isolation on the client with a
//! `sandbox` iframe attribute, so the document stays contained even when it is
//! reached through a cached or proxied copy that dropped the header.

use axum::body::Body;
use axum::http::{header, HeaderName, HeaderValue};
use axum::response::Response;

use crate::escape::escape_html;

/// `sandbox` attribute of the iframe in the embed shell. `allow-same-origin` is
/// absent on purpose: adding it would hand the app the suite origin back.
pub const WEBA_IFRAME_SANDBOX: &str = "allow-scripts allow-forms";

/// Policy served with every published and previewed WebA document.
///
/// * `sandbox allow-scripts allow-forms` — opaque origin, no same-origin access.
/// * `default-src 'none'` — nothing loads unless a directive allows it.
/// * `script-src 'self' https:` — only external, vetted scripts; no inline
///   script and no `eval`, so `content` can carry markup but not code.
/// * `connect-src 'none'` — the document cannot call back into the suite API.
/// * `frame-ancestors` is omitted: `'self'` matches nothing for an opaque
///   origin, and the app is public content that the suite frames on purpose.
///
/// Consequence for app authors: assets must be referenced by absolute `https:`
/// URL. A relative path resolves against the suite origin, which `'self'` can
/// never match once the document is sandboxed into an opaque origin.
pub const WEBA_CSP: &str = concat!(
    "sandbox allow-scripts allow-forms; ",
    "default-src 'none'; ",
    "script-src 'self' https:; ",
    "style-src 'self' 'unsafe-inline' https:; ",
    "img-src 'self' data: https:; ",
    "font-src 'self' data: https:; ",
    "media-src 'self' https:; ",
    "connect-src 'none'; ",
    "base-uri 'none'; ",
    "form-action 'self'"
);

/// Policy for the embed shell itself: it is a suite-origin page whose only job
/// is to host one isolated iframe.
pub const WEBA_EMBED_CSP: &str =
    "default-src 'none'; frame-src 'self'; style-src 'unsafe-inline'; base-uri 'none'";

/// Builds the HTTP response for a rendered app document.
///
/// Every render path goes through this function, so the policy cannot be lost
/// by adding a route later. The header is written with `from_static`, which
/// fails at compile time for an invalid policy instead of dropping it at
/// runtime.
#[must_use]
pub fn document_response(html: String, cache_control: &'static str) -> Response {
    document(html, WEBA_CSP, cache_control)
}

/// Response for the embed shell.
#[must_use]
pub fn embed_response(html: String) -> Response {
    document(html, WEBA_EMBED_CSP, "no-store")
}

fn document(html: String, csp: &'static str, cache_control: &'static str) -> Response {
    let mut response = Response::new(Body::from(html));
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(csp),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static(cache_control),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response
}

/// Builds the shell that renders a published app inside a sandboxed iframe.
///
/// `src` and `title` are escaped: both carry user data (the app name and the
/// requested page path).
#[must_use]
pub fn embed_shell(src: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        html, body {{ height: 100%; margin: 0; }}
        iframe {{ border: 0; width: 100%; height: 100%; display: block; }}
    </style>
</head>
<body>
    <iframe sandbox="{sandbox}" referrerpolicy="no-referrer" src="{src}"></iframe>
</body>
</html>"#,
        title = escape_html(title),
        sandbox = WEBA_IFRAME_SANDBOX,
        src = escape_html(src),
    )
}
