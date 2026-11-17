//! BotUI-only refinement of the shared IMP-09 header layer (#2376).
//!
//! `harden()` in `mod.rs` applies `with_security_headers()` — the exact layer
//! botserver uses — to the suite (:3000), cloud (:4000) and login (:5000)
//! routers. This module keeps the two BotUI rollout rules that a shared,
//! transport-agnostic layer cannot express:
//!
//! * CSP is delivered as `Content-Security-Policy-Report-Only`. The suite still
//!   ships roughly a thousand inline `on*=` handlers (tracked by the
//!   `scripts/check_inline_handlers.py` ratchet), so enforcing
//!   `script-src 'self'` today would blank the app. Report-only gives us the
//!   violations without breaking chat, Drive or the desktop shell.
//! * HSTS is emitted only when the request arrived over TLS. BotUI listens on
//!   plain HTTP behind a TLS-terminating proxy, so the proxy's
//!   `X-Forwarded-Proto: https` is the signal; plain-HTTP local dev must never
//!   pin the host to HTTPS.

use axum::{
    extract::Request,
    http::{header, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use botsecurity_core::headers::SecurityHeadersConfig;

static X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");

/// Frame protection stays effective while `frame-ancestors` lives only in a
/// report-only CSP: `DENY` would also break same-origin previews.
const FRAME_OPTIONS: &str = "SAMEORIGIN";

/// Builds the browser-facing profile consumed by the shared header layer.
///
/// The shared permissions policy is kept as-is: it already disables
/// geolocation and deliberately delegates camera/microphone to `(self)` for
/// chat voice, camera capture and Meet (`getUserMedia` in
/// `suite/chat/modules/76_voice.js`, `77_camera.js` and `suite/meet/`).
/// Overriding those two to `()` as the literal #2376 text suggests would
/// silently kill the browser prompts (#1691 follow-up).
pub fn ui_security_headers_config() -> SecurityHeadersConfig {
    SecurityHeadersConfig::from_env().with_frame_options(FRAME_OPTIONS)
}

/// Rewrites the shared layer's output after the response has been produced, so
/// every status — 200, static 404, proxy error and fallback — is covered.
pub async fn ui_header_policy(request: Request, next: Next) -> Response {
    let tls = request.uri().scheme_str() == Some("https")
        || forwarded_proto_is_https(request.headers().get(&X_FORWARDED_PROTO));
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    if let Some(csp) = headers.remove(header::CONTENT_SECURITY_POLICY) {
        headers.insert(
            HeaderName::from_static("content-security-policy-report-only"),
            csp,
        );
    }

    if !tls {
        headers.remove(header::STRICT_TRANSPORT_SECURITY);
    }

    response
}

/// Proxies may set a chain such as `https, http`; any hop over HTTPS is enough.
fn forwarded_proto_is_https(value: Option<&HeaderValue>) -> bool {
    value
        .and_then(|raw| raw.to_str().ok())
        .is_some_and(|raw| raw.split(',').any(|hop| hop.trim().eq_ignore_ascii_case("https")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwarded_proto_accepts_https_and_chains() {
        assert!(forwarded_proto_is_https(Some(&HeaderValue::from_static(
            "https"
        ))));
        assert!(forwarded_proto_is_https(Some(&HeaderValue::from_static(
            "https, http"
        ))));
        assert!(!forwarded_proto_is_https(Some(&HeaderValue::from_static(
            "http"
        ))));
        assert!(!forwarded_proto_is_https(None));
    }

    #[test]
    fn ui_profile_pins_the_issue_headers() {
        let config = ui_security_headers_config();
        assert_eq!(config.x_frame_options.as_deref(), Some(FRAME_OPTIONS));
        assert!(config.content_security_policy.is_some());
        let policy = config.permissions_policy.as_deref().unwrap_or_default();
        assert!(policy.contains("geolocation=()"));
        assert!(policy.contains("camera=(self)"));
        assert!(policy.contains("microphone=(self)"));
    }
}
