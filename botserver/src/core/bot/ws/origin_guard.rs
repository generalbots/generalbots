//! #2704 — cross-site WebSocket handshake guard.
//!
//! A browser always sends `Origin` on a WebSocket handshake. Without a check,
//! any web page could open a socket against a local botserver and consume
//! anonymous/public-bot LLM turns. This module decides whether a handshake is
//! trustworthy before `WebSocketUpgrade::on_upgrade` is reached.
//!
//! The policy deliberately keeps the documented carve-outs:
//! * no `Origin` (non-browser client, e.g. the desktop sidecar) — allowed;
//! * loopback origins (desktop edition / local suite) — allowed;
//! * same-origin with the request `Host` — allowed;
//! * a foreign origin — only via the canonical CORS origin policy
//!   (`resolve_origin_policy`), so the WS guard and the HTTP CORS layer cannot
//!   drift into disagreeing about which origins are trusted.

use botsecurity::cors::{resolve_origin_policy, OriginPolicy};

/// Pure predicate: may a handshake from `origin` be upgraded?
///
/// `configured` is the process-wide CORS allowlist snapshot; passing it in
/// keeps this function free of global state so it can be unit-tested directly.
pub(super) fn origin_allowed(
    origin: Option<&str>,
    host: Option<&str>,
    configured: &[String],
) -> bool {
    let Some(origin) = origin else {
        // No Origin means no foreign page initiated the handshake.
        return true;
    };
    if origin.is_empty() {
        return true;
    }
    let Some(origin_host_name) = origin_host(origin) else {
        return false;
    };

    // Same-origin and loopback desktop clients are trusted regardless of the
    // CORS allowlist: in a single-host deployment that list is normally empty,
    // yet the suite page must still be able to open its own bot's socket.
    if is_loopback_host(&origin_host_name) {
        return true;
    }
    if host
        .and_then(header_host)
        .is_some_and(|request_host| request_host.eq_ignore_ascii_case(&origin_host_name))
    {
        return true;
    }

    // Foreign origin: defer to the same policy the CORS layer resolves.
    match resolve_origin_policy(configured) {
        OriginPolicy::Deny => false,
        OriginPolicy::Allowlist(allowed) => allowed.iter().any(|entry| entry == origin),
        OriginPolicy::AnyNoCredentials => is_safe_origin_format(origin),
    }
}

/// Host portion of an `Origin` value (`scheme://host[:port]`), rejecting
/// userinfo, traversal and any value that is not a plain http(s) authority.
fn origin_host(origin: &str) -> Option<String> {
    let without_scheme = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))?;
    let authority = without_scheme.split('/').next()?;
    if authority.is_empty() || authority.contains('@') || authority.contains("..") {
        return None;
    }
    let host = if authority.starts_with('[') {
        authority.split(']').next()?.trim_start_matches('[')
    } else {
        authority.split(':').next()?
    };
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

/// Host portion of a `Host` header, tolerating an IPv6 literal in brackets.
fn header_host(host: &str) -> Option<&str> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('[') {
        trimmed.split(']').next().map(|h| h.trim_start_matches('['))
    } else {
        trimmed.split(':').next()
    }
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host == "127.0.0.1"
        || host == "::1"
        || host == "0.0.0.0"
}

/// Dev-only `CORS_ALLOW_ANY_ORIGIN` branch: accept a format-valid origin,
/// mirroring `botsecurity::cors::is_valid_origin_format` without reaching into
/// its crate-private helper.
fn is_safe_origin_format(origin: &str) -> bool {
    const DANGEROUS: [&str; 6] = ["<script", "javascript:", "data:", "vbscript:", "%3c", "%3e"];
    let lower = origin.to_ascii_lowercase();
    !DANGEROUS.iter().any(|pattern| lower.contains(pattern))
        && origin.parse::<axum::http::HeaderValue>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::origin_allowed;

    #[test]
    fn hostile_origin_is_rejected() {
        let configured = vec!["https://suite.example.org".to_string()];
        assert!(!origin_allowed(
            Some("https://evil.example"),
            Some("chat.example.com"),
            &configured
        ));
    }

    #[test]
    fn same_origin_is_allowed() {
        assert!(origin_allowed(
            Some("https://chat.example.com"),
            Some("chat.example.com"),
            &[]
        ));
    }

    #[test]
    fn loopback_desktop_origin_is_allowed() {
        assert!(origin_allowed(
            Some("http://localhost:3000"),
            Some("localhost:8080"),
            &[]
        ));
        assert!(origin_allowed(
            Some("http://127.0.0.1:3000"),
            Some("127.0.0.1:8080"),
            &[]
        ));
    }

    #[test]
    fn configured_origin_is_allowed() {
        let configured = vec![" https://suite.example.org ".to_string()];
        assert!(origin_allowed(
            Some("https://suite.example.org"),
            Some("api.example.org"),
            &configured
        ));
    }

    #[test]
    fn missing_origin_is_allowed() {
        assert!(origin_allowed(None, Some("chat.example.com"), &[]));
        assert!(origin_allowed(Some(""), Some("chat.example.com"), &[]));
    }

    #[test]
    fn malformed_origin_is_rejected() {
        let configured = vec!["https://suite.example.org".to_string()];
        for hostile in [
            "javascript:alert(1)",
            "https://user@evil.example",
            "null",
            "https://evil.example/../chat",
            "data:text/html,<script>alert(1)</script>",
        ] {
            assert!(
                !origin_allowed(Some(hostile), Some("chat.example.com"), &configured),
                "origin must be rejected: {hostile}"
            );
        }
    }

    #[test]
    fn host_header_is_required_for_same_origin_when_not_loopback() {
        assert!(!origin_allowed(Some("https://chat.example.com"), None, &[]));
    }
}
