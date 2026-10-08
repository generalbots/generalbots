use super::*;

pub(crate) fn anonymous_routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/healthz", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/version", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/product", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/bot/config", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/i18n/**", "GET", "").with_anonymous(true),
        // App catalog drives every launcher — must be reachable by any user
        RoutePermission::new("/api/apps/catalog", "GET", "").with_anonymous(true),

        // WhatsApp webhook - anonymous for Meta verification and message delivery
        RoutePermission::new("/webhook/whatsapp/{bot_id}", "GET", "").with_anonymous(true),
        RoutePermission::new("/webhook/whatsapp/{bot_id}", "POST", "").with_anonymous(true),

        // Inbound channel webhooks - anonymous because the provider posts from
        // its own infrastructure and holds no token. The alternative is worse:
        // an unmatched path under `/api/...` is denied outright, so the
        // Instagram and Teams deliveries would never reach their handler. Each
        // handler verifies the call itself (#1327).
        RoutePermission::new("/webhook/telegram", "POST", "").with_anonymous(true),
        // Per-bot Telegram webhook: the bot name in the path selects the bot;
        // the handler proves the call with the secret-token header (#1327).
        RoutePermission::new("/webhook/telegram/{bot_name}", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/instagram/webhook", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/instagram/webhook", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/msteams/messages", "POST", "").with_anonymous(true),

        // CalDAV. Native clients (Thunderbird, Apple Calendar, DAVx5) probe
        // `/.well-known/caldav` and then speak the DAV verbs against `/caldav`,
        // authenticating with HTTP Basic — the only scheme they implement. The
        // DAV router authorizes each calendar itself, so these entries only need
        // to admit an authenticated caller: without them RBAC answers "No
        // matching route permission found" (403) even for a valid credential and
        // no client can ever sync (#1335, #1336).
        RoutePermission::new("/.well-known/caldav", "GET", ""),
        RoutePermission::new("/.well-known/caldav", "PROPFIND", ""),
        RoutePermission::new("/caldav/**", "OPTIONS", ""),
        RoutePermission::new("/caldav/**", "GET", ""),
        RoutePermission::new("/caldav/**", "HEAD", ""),
        RoutePermission::new("/caldav/**", "PUT", ""),
        RoutePermission::new("/caldav/**", "DELETE", ""),
        RoutePermission::new("/caldav/**", "PROPFIND", ""),
        RoutePermission::new("/caldav/**", "PROPPATCH", ""),
        RoutePermission::new("/caldav/**", "REPORT", ""),

        // Auth routes - login must be anonymous
        RoutePermission::new("/api/auth", "GET", "").with_anonymous(true),

        RoutePermission::new("/api/auth/login", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/cloud/auth/login", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/cloud/auth/signup", "POST", "").with_anonymous(true),
        // Cloud dashboard: list the caller's workspace bots (scoped by the
        // JWT claims inside the handler; any authenticated tenant user).
        RoutePermission::new("/api/cloud/bots", "GET", ""),
        RoutePermission::new("/api/cloud/organizations", "GET", ""),
        RoutePermission::new("/api/cloud/services", "GET", ""),
        RoutePermission::new("/api/cloud/invoices", "GET", ""),
        RoutePermission::new("/api/cloud/plans", "GET", ""),
        RoutePermission::new("/api/cloud/payment-cards", "GET", ""),
        RoutePermission::new("/api/cloud/payment-cards", "POST", ""),
        RoutePermission::new("/api/cloud/payment-cards/setup", "POST", ""),
        RoutePermission::new("/api/cloud/payment-cards/**", "DELETE", ""),
        RoutePermission::new("/api/cloud/tenant/settings/**", "GET", ""),
        RoutePermission::new("/api/cloud/tenant/settings/**", "POST", ""),
        RoutePermission::new("/api/cloud/tenant/settings/**", "PUT", ""),

        // Client error reporting - anonymous to catch all JS errors
        RoutePermission::new("/api/client-errors", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/bootstrap", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/refresh", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/auth/logout", "POST", ""),
        RoutePermission::new("/api/auth/me", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/auth/**", "GET", ""),
        RoutePermission::new("/api/auth/**", "POST", ""),

        // WebSocket - anonymous for chat support
        RoutePermission::new("/ws", "GET", "").with_anonymous(true),
        RoutePermission::new("/ws/**", "GET", "").with_anonymous(true),

        // Bot access check - anonymous so is_public bots can be accessed without login
        RoutePermission::new("/api/bots/{bot_name}/access", "GET", "").with_anonymous(true),

        // Chat - ANONYMOUS for customer support
        RoutePermission::new("/api/chat/**", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/chat/**", "POST", "").with_anonymous(true),

        // Sessions - anonymous can create sessions for chat
        RoutePermission::new("/api/sessions", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/sessions", "GET", ""),
        RoutePermission::new("/api/sessions/**", "GET", ""),
    ]
}
