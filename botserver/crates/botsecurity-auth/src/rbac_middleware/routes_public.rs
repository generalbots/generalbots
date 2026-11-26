use super::*;

pub(crate) fn anonymous_routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/healthz", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/health", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/version", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/product", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/weather", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/bot/config", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/i18n/**", "GET", "").with_anonymous(true),
        // App catalog drives every launcher — must be reachable by any user
        RoutePermission::new("/api/apps/catalog", "GET", "").with_anonymous(true),
        // Sandbox resources are protected by their own short-lived signed
        // token, not the suite session (#1674). Only the resource route is
        // public (#2760): config/status/revoke are management surfaces registered
        // in the authenticated catalogue (routes_user/app_routes.rs).
        RoutePermission::new("/sandbox/ui/**", "GET", "").with_anonymous(true),

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

        // Integration OAuth: the provider redirects the user's browser here
        // with no Authorization header. The request is authenticated by the
        // HMAC-signed `state` issued by the start handler, which also carries
        // the bot id — hence one stable callback path for the whole platform.
        RoutePermission::new("/api/integrations/oauth/**", "GET", "").with_anonymous(true),

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

        // #2723 — the advisory API is public, read-only and contains no
        // personal data. Operators and air-gapped CLIs must be able to query
        // it without a suite session.
        RoutePermission::new("/api/advisories", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/advisories/{id}", "GET", "").with_anonymous(true),

        // Client error reporting (#2962). #2133 made this endpoint
        // authenticated-only on the client: a signed-in page sends its suite
        // token, a logged-out one keeps errors local and gets a 401. Both
        // verbs therefore need an explicit authenticated entry — without one
        // RBAC's default-deny answers 401 (anonymous) or 403 (signed-in) on
        // the notepad sweep's GET probe. Anonymous stays denied.
        RoutePermission::new("/api/client-errors", "GET", ""),
        RoutePermission::new("/api/client-errors", "POST", ""),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_errors_has_authenticated_get_and_post_entries() {
        let routes = anonymous_routes();
        let entries: Vec<&RoutePermission> = routes
            .iter()
            .filter(|r| r.path_pattern == "/api/client-errors")
            .collect();
        assert_eq!(entries.len(), 2, "GET and POST must both be registered");
        for entry in &entries {
            assert!(
                !entry.allow_anonymous,
                "{} {} must deny anonymous callers",
                entry.method, entry.path_pattern
            );
            assert!(
                entry.required_permission.is_empty(),
                "any authenticated suite user may report a client error"
            );
        }
        assert!(entries.iter().any(|r| r.method == "GET"));
        assert!(entries.iter().any(|r| r.method == "POST"));

        // Unrelated public routes must not have been tightened by #2962.
        assert!(routes
            .iter()
            .any(|r| { r.path_pattern == "/api/auth" && r.method == "GET" && r.allow_anonymous }));
        assert!(routes
            .iter()
            .any(|r| { r.path_pattern == "/health" && r.method == "GET" && r.allow_anonymous }));
    }

    #[tokio::test]
    async fn client_errors_denies_anonymous_and_allows_signed_in() {
        let manager = RbacManager::with_defaults();
        manager.register_routes(anonymous_routes()).await;
        let anonymous = AuthenticatedUser::anonymous();
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
            .with_role(Role::Admin);
        for method in ["GET", "POST"] {
            assert!(
                !manager
                    .check_route_access("/api/client-errors", method, &anonymous)
                    .await
                    .is_allowed(),
                "{method} must be denied without a session"
            );
            assert!(
                manager
                    .check_route_access("/api/client-errors", method, &admin)
                    .await
                    .is_allowed(),
                "{method} must be allowed for a signed-in admin"
            );
        }
    }
}
