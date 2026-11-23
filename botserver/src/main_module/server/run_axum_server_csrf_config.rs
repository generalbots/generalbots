//! #2683 — pure builder for the production CSRF configuration.
//!
//! Kept in a sibling module so `run_axum_server.rs` stays below the project's
//! 450-line limit. The module is declared inline beside its only caller.

use super::*;

/// Builds the production CSRF configuration.
///
/// Pure — no I/O and no `AppState` — so the exemption list can be asserted in
/// unit tests. Rule: an exemption is only valid when the endpoint cannot be
/// reached from a cross-origin page with an ambient credential (a session
/// cookie). Bearer-authenticated callers are stateless and exempted by the
/// middleware itself, so a path exemption for them would only widen CSRF to
/// cookie-authenticated state changes.
pub(super) fn build_csrf_config(no_security: bool) -> CsrfConfig {
    CsrfConfig {
        // Keep CSRF active in all modes. In single-user desktop (`no_security`),
        // the middleware enforces loopback Origin/Referer checks so a cross-site
        // page cannot drive the localhost API, while trusted loopback origins
        // (the desktop WebView) can proceed without a double-submit token pair.
        enabled: true,
        no_security_mode: no_security,
        exempt_paths: vec![
            // Read-only probes and static metadata: no state changes.
            "/api/health".into(),
            "/api/setup/status".into(),
            "/api/product".into(),
            "/api/manifest".into(),
            "/api/i18n".into(),
            // Credential-entry routes: the request carries the credential it
            // establishes. Login/signup additionally require same-origin via
            // `origin_check_paths`; no ambient session is trusted here.
            "/api/auth".into(),
            "/api/auth/login".into(),
            "/api/auth/refresh".into(),
            "/api/auth/bootstrap".into(),
            "/api/auth/cloud-sso".into(),
            "/api/auth/suite-sso".into(),
            "/api/auth/unified-login".into(),
            // Anonymous client-side error telemetry; mutates no session state.
            "/api/client-errors".into(),
            // Peer machine-to-machine routes (#2141): a peer presents the shared
            // node token, not a CSRF cookie, so only these exact routes are
            // exempt. `enable`/`disable` are admin controls and stay under the
            // normal CSRF check.
            "/api/peer/status".into(),
            "/api/peer/register".into(),
            "/api/peer/heartbeat".into(),
            "/api/peer/home-site".into(),
            "/api/peer/payloads".into(),
            // #2815 — login/signup stay exempt from the double-submit token
            // (the JS forms post JSON), but the middleware requires a
            // same-origin Origin/Referer on these paths before honouring the
            // exemption. Bearer/other API paths keep the exemption.
            "/api/cloud/auth*".into(),
            // #2683 — `/api/sheet*` and `/suite/sheet*` are deliberately NOT
            // exempt. They are browser-reached, cookie/session-capable,
            // state-changing endpoints (`POST /api/sheet/save`, `/delete`,
            // `/cells`, and the HTMX forms under `/suite/sheet/...`). Legitimate
            // suite requests send `Authorization: Bearer` through the
            // security-bootstrap interceptor and are already exempt in the
            // middleware; exempting the paths would instead let a cross-origin
            // page with an ambient cookie change workbook data.
            // `/api/catalog` is likewise NOT exempt: its routes are public
            // GET-only reads (`/products`, `/plans`, `/prices.json`) that the
            // exempt GET method already covers, so a path entry would be dead
            // and misleading.
            // WebSocket upgrades are GET requests, not state-changing verbs.
            "/ws".into(),
            "/ws/".into(),
            // Inbound channel webhooks arrive from the provider, which cannot
            // hold a CSRF token; each handler verifies the provider signature
            // or shared secret itself (#1327).
            "/webhook/whatsapp".into(),
            "/api/whatsapp/webhook".into(),
            "/api/facebook/webhook".into(),
            "/webhook".into(),
            "/webhook/telegram*".into(),
            "/api/instagram/webhook".into(),
            "/api/msteams/messages".into(),
            // #2538 — published-site forms are anonymous by design; the
            // handler enforces a honeypot, per-e-mail rate limit and field
            // validation instead of a session cookie. The slug addresses one
            // published site, never a tenant namespace.
            "/forms/*".into(),
            // CalDAV clients (Thunderbird, Apple Calendar, DAVx5) authenticate
            // with HTTP Basic and their write verbs — PROPFIND, REPORT, PUT,
            // DELETE — are not in the exempt method list, so a CSRF check
            // rejected every discovery request and no client could sync. The
            // DAV router authenticates and authorizes each request itself
            // (#1335).
            "/caldav*".into(),
            "/.well-known/caldav".into(),
        ],
        // #2815: enforce same-origin Origin/Referer on the two anonymous auth
        // POSTs instead of leaving them wholly unprotected by the exemption.
        origin_check_paths: vec![
            "/api/cloud/auth/login".into(),
            "/api/cloud/auth/signup".into(),
        ],
        ..Default::default()
    }
}
