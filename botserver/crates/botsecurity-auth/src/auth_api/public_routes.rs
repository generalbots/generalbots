//! The platform's anonymous/public route declarations (#2133).
//!
//! Every entry is an exact `(method, path)` pair, a parameterized path
//! (`:id`) or a declared trailing `/*` wildcard. There is no bare-prefix
//! entry: `/api/auth` does not admit `/api/auth/directory-config`, and
//! `/api/products/items` does not admit `POST /api/products/items`.
//!
//! Adding a route here is a security decision, so each block states why the
//! route may run without a session. If a route cannot be justified with one of
//! the [`PublicAccess`] kinds, it stays authenticated.

use axum::http::Method;

use super::config::{AuthConfigError, PublicAccess, PublicRoute};

/// Every route reachable without a session in a botserver deployment.
///
/// Invalid declarations are refused (fail closed) and logged instead of being
/// silently widened into a namespace.
#[must_use]
pub fn platform_public_routes() -> Vec<PublicRoute> {
    let mut routes = RouteList::new();

    // ── Health and static, caller-independent metadata ──────────────────────
    routes.read("/health");
    routes.read("/api/health");
    routes.read("/api/product");
    routes.read("/api/whats-new");
    routes.read("/api/manifest");
    routes.read("/api/setup/status");
    // Public bot profile: the chat shell asks whether a bot exists before the
    // visitor has an account.
    routes.read("/api/bot/public");
    routes.read("/api/weather");

    // ── Translations (no tenant data) ───────────────────────────────────────
    routes.read("/api/i18n/locales");
    // `/api/i18n/preference` is NOT here: it is caller-scoped and already
    // rejects a caller without a verified token.
    routes.read("/api/i18n/:locale");

    // ── Launcher and integration catalogs ───────────────────────────────────
    // The desktop launcher renders before login; the handlers serve the app
    // registry and the built-in provider seeds only (#1342, #1428).
    routes.read("/api/apps/catalog");
    routes.read("/api/apps/integrations/catalog");
    routes.read("/api/apps/integrations/catalog/:provider");

    // ── Public bot access check ─────────────────────────────────────────────
    // Answers "is this bot public?" before any token exists. Only the exact
    // access route: bot config and mutations stay authenticated.
    routes.read("/api/bots/:bot_name/access");

    // ── Host and TLS lookups used by the UI server and the proxy ────────────
    routes.read("/api/domains/resolve");
    // Caddy's on-demand TLS decision runs during the TLS handshake, before any
    // HTTP request (and therefore any Authorization header) exists.
    routes.read("/api/domains/tls-ask");

    // ── CalDAV discovery ────────────────────────────────────────────────────
    // Native clients probe the well-known URL and authenticate with HTTP
    // Basic; the credential is resolved into a user by the auth middleware.
    routes.read_method(Method::GET, "/.well-known/caldav");
    // PROPFIND is a WebDAV verb, not one of the standard `Method` constants.
    routes.read_token("PROPFIND", "/.well-known/caldav");

    // ── Public store catalog (read only) ────────────────────────────────────
    // Browsing plans/offers/store needs no account. Writes (checkout,
    // purchase, payment cards) are deliberately absent.
    routes.read("/api/cloud/store");
    routes.read("/api/cloud/plans");
    routes.read("/api/cloud/plans/:plan_id");
    routes.read("/api/cloud/offers");
    routes.read("/api/cloud/llm-providers");
    routes.read("/api/catalog/products");
    routes.read("/api/catalog/products/:sku");
    routes.read("/api/catalog/plans");
    routes.read("/api/catalog/prices.json");
    // Store catalog only: GET on the exact item routes. `POST /api/products/items`,
    // `PUT /api/products/items/:id/stock` and `DELETE /api/products/items/:id`
    // are writes and stay authenticated (#2128).
    routes.read("/api/products/items");
    routes.read("/api/products/items/:id");

    // ── Authentication entry points ─────────────────────────────────────────
    // The request carries the credential being verified; there is no session
    // yet. These are the only write routes allowed without a signature.
    routes.read("/api/auth");
    routes.read("/api/auth/me");
    routes.credential(Method::POST, "/api/auth/login");
    routes.credential(Method::POST, "/api/auth/logout");
    routes.credential(Method::POST, "/api/auth/refresh");
    routes.credential(Method::POST, "/api/auth/2fa/verify");
    routes.credential(Method::POST, "/api/auth/2fa/resend");
    routes.credential(Method::POST, "/api/auth/bootstrap");
    routes.credential(Method::POST, "/api/cloud/auth/login");
    routes.credential(Method::POST, "/api/cloud/auth/signup");
    // Browser hop from the login domain; the handler verifies the SSO token.
    routes.webhook(Method::GET, "/api/auth/suite-sso");
    // `/api/auth/directory-config` is intentionally absent: it exposes the
    // identity-provider URL and must not be reachable anonymously (#2133).

    // ── Capability-token routes (read only) ─────────────────────────────────
    // The token is an unguessable value carried in the path/query and verified
    // by the handler before any tenant data is read.
    routes.capability("/api/files/public/:token");
    routes.capability("/api/terminal/ws");
    routes.capability("/api/marketing/track/open/:token");
    // Email click tracking (#2139): the destination is a signed query
    // parameter (`u`/`e`/`s`), so the only place this route redirects to is a
    // URL the server signed into the sent email. The legacy path shape
    // (`…/click/:id/<destination>`) is declared so an old link answers an
    // explicit 400 instead of being widened into a namespace.
    routes.capability("/api/marketing/track/click/:id");
    routes.capability("/api/marketing/track/click/:id/*");
    // Caddy forward-auth: the proxy calls it without a token; the handler
    // validates the domain cookie/JWT itself.
    routes.capability("/api/vibe/domain-auth");
    // Attachment preview and agent workspace reads cannot send an
    // Authorization header from an <iframe>/<img>; the handler verifies the
    // `access_token` query parameter (#1582).
    routes.capability("/api/agent/vm/file");
    routes.capability("/api/agent/vm/files");
    routes.capability("/api/chat/attachments/preview");

    // ── Inbound provider webhooks (handler verifies the call) ───────────────
    // Meta verification challenge (GET) and message delivery (POST, signed
    // with X-Hub-Signature-256).
    routes.webhook(Method::GET, "/api/whatsapp/webhook");
    routes.webhook(Method::POST, "/api/whatsapp/webhook");
    // Instagram: verify-token challenge (GET) and signed delivery (POST).
    routes.webhook(Method::GET, "/api/instagram/webhook");
    routes.webhook(Method::POST, "/api/instagram/webhook");
    // Teams Bot Framework HMAC.
    routes.webhook(Method::POST, "/api/msteams/messages");
    // Telegram secret token, global and per-bot.
    routes.webhook(Method::POST, "/webhook/telegram");
    routes.webhook(Method::POST, "/webhook/telegram/:bot_name");

    // ── Browser redirects authenticated by a signed state ───────────────────
    routes.webhook(Method::GET, "/api/email/oauth/callback");
    routes.webhook(Method::GET, "/api/integrations/oauth/:provider/callback");
    routes.webhook(
        Method::GET,
        "/api/bots/:bot_id/integrations/oauth/:provider/callback",
    );
    routes.webhook(Method::GET, "/api/external-drives/callback");
    routes.webhook(Method::GET, "/auth/callback");

    // ── Peer network (machine callers, shared GB_PEER_TOKEN) ────────────────
    // Peers hold no user session. #2141 — only the machine-to-machine routes
    // are listed here, and every one of them requires the shared token: when
    // GB_PEER_TOKEN is unset they fail closed (503) instead of accepting
    // anonymous payloads. `enable`/`disable` are admin controls and stay
    // authenticated (admin session), so they are deliberately absent.
    routes.read("/api/peer/status");
    routes.read("/api/peer/home-site");
    routes.read("/api/peer/payloads");
    routes.webhook(Method::POST, "/api/peer/register");
    routes.webhook(Method::POST, "/api/peer/heartbeat");
    routes.webhook(Method::POST, "/api/peer/payloads");

    // ── Chat and progress WebSockets ────────────────────────────────────────
    // A browser WebSocket cannot set an Authorization header; the handlers
    // resolve the optional `?token=` and the bot's public flag themselves.
    // Only these four exact sockets are admitted — `/ws/logs`, `/ws/sheet/:id`,
    // `/ws/docs/:id`, `/ws/plan/:id` stay authenticated (see the deny list).
    routes.read("/ws");
    routes.read("/ws/:bot_name");
    routes.read("/ws/task-progress");
    routes.read("/ws/task-progress/:task_id");

    // ── Static asset trees (the one legitimate prefix) ──────────────────────
    // Only file-like paths (a dotted final segment) match, so a server route
    // registered under the same prefix — e.g. `/suite/sheet/fragments/...` —
    // stays authenticated.
    routes.asset("/suite/*");
    routes.asset("/static/*");
    routes.asset("/themes/*");
    routes.asset("/auth/*");
    routes.read("/favicon.ico");
    routes.read("/robots.txt");

    routes.finish()
}

/// Routes that must require a session even though a broader public pattern
/// (a `:param` or an asset tree) would match them.
///
/// Deny wins, so this list can only remove access — a new route added under a
/// public pattern stays authenticated until someone adds it here or the
/// pattern is narrowed.
#[must_use]
pub fn platform_private_routes() -> Vec<PublicRoute> {
    let mut routes = RouteList::new();
    // `/ws/:bot_name` admits public-bot chat; the monitoring log stream shares
    // that shape and has no handler-level gate.
    routes.deny(Method::GET, "/ws/logs");
    // `/api/i18n/:locale` serves translation catalogues; the caller-scoped
    // preference endpoint shares that shape and already answers 401, but the
    // transport gate must not treat it as anonymous.
    routes.deny(Method::GET, "/api/i18n/preference");
    routes.finish()
}

/// Collects declarations, refusing an invalid one instead of widening access.
struct RouteList {
    routes: Vec<PublicRoute>,
}

impl RouteList {
    fn new() -> Self {
        Self { routes: Vec::new() }
    }

    fn read(&mut self, pattern: &str) {
        self.push(PublicRoute::read(Method::GET, pattern));
    }

    fn read_method(&mut self, method: Method, pattern: &str) {
        self.push(PublicRoute::new(method, pattern, PublicAccess::AnonymousRead));
    }

    /// Declares a read route for a verb that is not a `Method` constant
    /// (WebDAV's PROPFIND).
    fn read_token(&mut self, token: &str, pattern: &str) {
        match Method::from_bytes(token.as_bytes()) {
            Ok(method) => {
                self.push(PublicRoute::new(method, pattern, PublicAccess::AnonymousRead));
            }
            Err(error) => {
                log::error!("invalid method {token:?} for public route {pattern:?}: {error}");
            }
        }
    }

    fn capability(&mut self, pattern: &str) {
        self.push(PublicRoute::capability(pattern));
    }

    fn credential(&mut self, method: Method, pattern: &str) {
        self.push(PublicRoute::credential_entry(method, pattern));
    }

    fn webhook(&mut self, method: Method, pattern: &str) {
        self.push(PublicRoute::webhook(method, pattern));
    }

    fn asset(&mut self, pattern: &str) {
        self.push(PublicRoute::asset(pattern));
    }

    /// Declares a route that always requires a session.
    fn deny(&mut self, method: Method, pattern: &str) {
        self.push(PublicRoute::deny(method, pattern));
    }

    fn push(&mut self, route: Result<PublicRoute, AuthConfigError>) {
        match route {
            Ok(route) => self.routes.push(route),
            // Fail closed: a malformed declaration must never become a
            // namespace grant. The route stays authenticated and the operator
            // sees the mistake at boot.
            Err(error) => log::error!("public route declaration refused: {error}"),
        }
    }

    fn finish(self) -> Vec<PublicRoute> {
        self.routes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_api::config::AuthConfig;

    /// The same policy the middleware receives: allow list + deny list.
    fn is_public(method: Method, path: &str) -> bool {
        AuthConfig::default()
            .with_public_routes(platform_public_routes())
            .with_private_routes(platform_private_routes())
            .is_public_route(&method, path)
    }

    #[test]
    fn a_new_route_under_an_allowlisted_prefix_is_not_anonymous() {
        // The old prefix model made every one of these anonymous.
        assert!(!is_public(Method::GET, "/api/auth/directory-config"));
        assert!(!is_public(Method::GET, "/api/auth/me/extra"));
        assert!(!is_public(Method::GET, "/api/products/items/x/stock"));
        assert!(!is_public(Method::GET, "/api/sheet/list"));
        assert!(!is_public(Method::GET, "/api/ui/sheet/fragments/sheet-list"));
        assert!(!is_public(Method::GET, "/api/cloud/auth/anything"));
        assert!(!is_public(Method::GET, "/api/bots/foo/config"));
        // A server-rendered route under the `/suite` asset tree stays private.
        assert!(!is_public(
            Method::GET,
            "/suite/sheet/fragments/sheet-list"
        ));
        assert!(!is_public(Method::GET, "/suite/dashboards"));
    }

    #[test]
    fn a_write_method_is_never_anonymous_by_prefix() {
        assert!(!is_public(Method::POST, "/api/products/items"));
        assert!(!is_public(Method::PUT, "/api/products/items/:id"));
        assert!(!is_public(Method::DELETE, "/api/products/items/:id"));
        assert!(!is_public(Method::POST, "/api/products/items/x/stock"));
        assert!(!is_public(Method::PUT, "/api/products/items/x/stock"));
        // Every other products write stays authenticated too (#2128): the
        // legacy `/api/products/items` prefix entry opened the whole namespace.
        assert!(!is_public(Method::POST, "/api/products/services"));
        assert!(!is_public(Method::PUT, "/api/products/services/x"));
        assert!(!is_public(Method::DELETE, "/api/products/services/x"));
        assert!(!is_public(Method::POST, "/api/products/categories"));
        assert!(!is_public(Method::POST, "/api/products/price-lists"));
        assert!(!is_public(Method::POST, "/api/products/inventory/adjust"));
        assert!(!is_public(Method::POST, "/api/products/inventory/build"));
        assert!(!is_public(Method::POST, "/api/products/services/x/tax"));
        assert!(!is_public(Method::POST, "/api/sheet/save"));
        assert!(!is_public(Method::POST, "/api/cloud/store/purchase"));
        assert!(!is_public(Method::POST, "/api/auth/directory-config"));
        assert!(!is_public(Method::POST, "/health"));
    }

    #[test]
    fn the_real_public_endpoints_still_match() {
        assert!(is_public(Method::GET, "/health"));
        assert!(is_public(Method::GET, "/api/health"));
        assert!(is_public(Method::GET, "/api/product"));
        // Login/signup are explicit credential entry points.
        assert!(is_public(Method::POST, "/api/auth/login"));
        assert!(is_public(Method::POST, "/api/auth/refresh"));
        assert!(is_public(Method::POST, "/api/auth/bootstrap"));
        assert!(is_public(Method::POST, "/api/cloud/auth/login"));
        assert!(is_public(Method::POST, "/api/cloud/auth/signup"));
        // Provider webhooks remain reachable so the signature check can run.
        assert!(is_public(Method::POST, "/api/whatsapp/webhook"));
        assert!(is_public(Method::GET, "/api/whatsapp/webhook"));
        assert!(is_public(Method::POST, "/api/msteams/messages"));
        assert!(is_public(Method::POST, "/webhook/telegram/acme"));
        // Public catalog reads stay public, writes do not.
        assert!(is_public(Method::GET, "/api/products/items"));
        assert!(is_public(Method::GET, "/api/products/items/abc"));
        assert!(is_public(Method::GET, "/api/catalog/products/widget"));
        // Static UI files are public; server fragments are not.
        assert!(is_public(Method::GET, "/suite/desktop.html"));
        assert!(is_public(Method::GET, "/suite/sheet/sheet.js"));
        assert!(is_public(Method::GET, "/themes/dark.css"));
    }

    #[test]
    fn email_tracking_routes_are_anonymous_only_in_their_declared_shapes() {
        // A mail client cannot send a bearer token, so the pixel and the click
        // redirect stay anonymous (#2139) — but only in the declared shapes.
        assert!(is_public(Method::GET, "/api/marketing/track/open/abc"));
        let click = "/api/marketing/track/click/6f1f6f9a-0d1a-4c02-9a53-2f7f4b0c1d11";
        assert!(is_public(Method::GET, click));
        // The legacy path shape must still reach the handler so it can answer
        // an explicit 400 instead of redirecting.
        assert!(is_public(
            Method::GET,
            "/api/marketing/track/click/6f1f6f9a-0d1a-4c02-9a53-2f7f4b0c1d11/https://evil.example"
        ));
        // Nothing else under the prefix is public, and no write is.
        assert!(!is_public(Method::GET, "/api/marketing/track/open"));
        assert!(!is_public(Method::GET, "/api/marketing/track/click"));
        assert!(!is_public(Method::GET, "/api/marketing/track/open/abc/extra"));
        assert!(!is_public(Method::GET, "/api/marketing/campaigns"));
        assert!(!is_public(Method::POST, click));
    }

    #[test]
    fn a_head_request_follows_its_get_declaration() {
        assert!(is_public(Method::HEAD, "/health"));
        assert!(is_public(Method::HEAD, "/suite/desktop.html"));
    }

    #[test]
    fn a_parameterized_route_matches_exactly_one_segment() {
        assert!(is_public(Method::GET, "/api/bots/acme/access"));
        assert!(!is_public(Method::GET, "/api/bots/acme/access/extra"));
        assert!(!is_public(Method::GET, "/api/bots/access"));
        assert!(is_public(Method::GET, "/api/files/public/abc"));
        assert!(!is_public(Method::GET, "/api/files/public/abc/extra"));
    }

    #[test]
    fn every_declaration_is_valid() {
        // `platform_public_routes` drops invalid declarations; a short list
        // means a declaration was silently refused.
        let routes = platform_public_routes();
        let count = routes.len();
        assert!(count > 40, "declarations were refused: {count}");
        for route in &routes {
            assert!(route.pattern.starts_with('/'));
            assert!(route.access.allows_method(&route.method));
        }
    }
}
