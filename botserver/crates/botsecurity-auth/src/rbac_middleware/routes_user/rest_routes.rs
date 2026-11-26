//! Remaining authenticated routes: goals, bots, CRM, billing and profile.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        // Goals
        RoutePermission::new("/api/goals/**", "GET", ""),
        RoutePermission::new("/api/goals/**", "POST", ""),
        RoutePermission::new("/api/goals/**", "PUT", ""),
        RoutePermission::new("/api/goals/**", "DELETE", ""),

        // Settings (user's own settings). #2775 — the
        // /api/settings/security/* handlers (botsettings::security_admin) are
        // admin-shaped: TLS, rate-limit, CORS, MFA, password policy, API keys
        // and session revocation. The broad /api/settings/** entries below
        // also match that prefix, and manager.rs returns on the first match,
        // so the admin-gated rules must come FIRST or any authenticated user
        // slips through to the stubs. Keep the remaining settings self-service
        // scope unchanged.
        RoutePermission::new("/api/settings/security/**", "GET", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/security/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/settings/**", "GET", ""),
        RoutePermission::new("/api/settings/**", "POST", ""),
        RoutePermission::new("/api/settings/**", "PUT", ""),

        // #2960 — the Integrations app opens its connected-services list with
        // a roleless authenticated GET on this exact prefix. Keep explicit
        // entries ahead of the broad `/api/bots/{id}/**` GET below: the route
        // table matches in order, so these rules name the audited pattern and
        // cannot be silently dropped by a future tightening of the catch-all.
        // Tenant binding stays in `botintegrations::scope::resolve_scope`: a
        // foreign or unresolvable bot id is rejected there (403/404), never
        // by a blanket role gate in front of a caller's own bot.
        RoutePermission::new("/api/bots/{id}/integration-connections", "GET", ""),
        RoutePermission::new("/api/bots/{id}/integration-connections/**", "GET", ""),

        // Bots (read for all authenticated users)
        RoutePermission::new("/api/bots", "GET", ""),
        RoutePermission::new("/api/bots/{id}", "GET", ""),
        RoutePermission::new("/api/bots/{id}/**", "GET", ""),
        // Integration connections and actions. The handlers resolve the
        // tenant scope from the bot id and the caller's membership, and the
        // credential material stays in Vault, so any authenticated user may
        // manage their own bot's connections. Without these rules the
        // "add a Gmail/OneDrive account" flow answered 403 before reaching
        // the handler.
        RoutePermission::new("/api/bots/{id}/integration-connections", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-connections/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-connections/**", "DELETE", ""),
        RoutePermission::new("/api/bots/{id}/integration-actions/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-automations/**", "POST", ""),
        RoutePermission::new("/api/bots/{id}/integration-automations/**", "DELETE", ""),
        RoutePermission::new("/api/bots/{id}/integrations/oauth/**", "GET", ""),
        // Saving the OAuth client (the "configure once" form) is a POST.
        RoutePermission::new("/api/bots/{id}/integrations/oauth/**", "POST", ""),

        // Autotask
        RoutePermission::new("/api/autotask/**", "GET", ""),
        RoutePermission::new("/api/autotask/**", "POST", ""),
        RoutePermission::new("/api/autotask/**", "PUT", ""),
        RoutePermission::new("/api/autotask/**", "DELETE", ""),

        // Designer
        RoutePermission::new("/api/designer/**", "GET", ""),
        RoutePermission::new("/api/designer/**", "POST", ""),
        RoutePermission::new("/api/designer/**", "PUT", ""),
        RoutePermission::new("/api/designer/**", "DELETE", ""),

        // Dashboards
        RoutePermission::new("/api/dashboards/**", "GET", ""),
        RoutePermission::new("/api/dashboards/**", "POST", ""),
        RoutePermission::new("/api/dashboards/**", "PUT", ""),
        RoutePermission::new("/api/dashboards/**", "DELETE", ""),

        // DB/Table access
        RoutePermission::new("/api/db/**", "GET", ""),
        RoutePermission::new("/api/db/**", "POST", ""),
        RoutePermission::new("/api/db/**", "PUT", ""),
        RoutePermission::new("/api/db/**", "DELETE", ""),

        // CRM / Contacts
        RoutePermission::new("/api/crm/**", "GET", ""),
        RoutePermission::new("/api/crm/**", "POST", ""),
        RoutePermission::new("/api/crm/**", "PUT", ""),
        RoutePermission::new("/api/crm/**", "DELETE", ""),
        RoutePermission::new("/api/contacts/**", "GET", ""),
        RoutePermission::new("/api/contacts/**", "POST", ""),
        RoutePermission::new("/api/contacts/**", "PUT", ""),
        RoutePermission::new("/api/contacts/**", "DELETE", ""),

        // Marketing / Campaigns
        RoutePermission::new("/api/marketing/**", "GET", ""),
        RoutePermission::new("/api/marketing/**", "POST", ""),
        RoutePermission::new("/api/marketing/**", "PUT", ""),
        RoutePermission::new("/api/marketing/**", "DELETE", ""),

        // CRM Campaigns
        RoutePermission::new("/api/crm/campaigns/**", "GET", ""),
        RoutePermission::new("/api/crm/campaigns/**", "POST", ""),
        RoutePermission::new("/api/crm/campaigns/**", "PUT", ""),
        RoutePermission::new("/api/crm/campaigns/**", "DELETE", ""),
        RoutePermission::new("/api/crm/lists/**", "GET", ""),
        RoutePermission::new("/api/crm/lists/**", "POST", ""),
        RoutePermission::new("/api/crm/lists/**", "PUT", ""),
        RoutePermission::new("/api/crm/lists/**", "DELETE", ""),
        RoutePermission::new("/api/crm/templates/**", "GET", ""),
        RoutePermission::new("/api/crm/templates/**", "POST", ""),
        RoutePermission::new("/api/crm/templates/**", "PUT", ""),
        RoutePermission::new("/api/crm/templates/**", "DELETE", ""),

        // Billing / Products
        // #2671 — reads stay available to every authenticated tenant user;
        // invoice/payment/product state changes are admin-only. These entries
        // must carry the roles HERE (not in admin_routes): the route table is
        // matched in registration order and returns on the first hit.
        RoutePermission::new("/api/billing/**", "GET", ""),
        RoutePermission::new("/api/billing/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/billing/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/billing/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "GET", ""),
        RoutePermission::new("/api/products/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/products/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        // #2671 — cloud store checkout, vouchers and organization writes.
        RoutePermission::new("/api/cloud/store/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/vouchers/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/cloud/organizations/**", "DELETE", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/billing/**", "POST", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),
        RoutePermission::new("/api/ui/billing/**", "PUT", "")
            .with_roles(vec!["Admin".into(), "SuperAdmin".into()]),

        // Tickets
        RoutePermission::new("/api/tickets/**", "GET", ""),
        RoutePermission::new("/api/tickets/**", "POST", ""),
        RoutePermission::new("/api/tickets/**", "PUT", ""),
        RoutePermission::new("/api/tickets/**", "DELETE", ""),

        // Learn
        RoutePermission::new("/api/learn/**", "GET", ""),
        RoutePermission::new("/api/learn/**", "POST", ""),

        // Social
        RoutePermission::new("/api/social/**", "GET", ""),
        RoutePermission::new("/api/social/**", "POST", ""),

        // LLM
        RoutePermission::new("/api/llm/**", "GET", ""),
        RoutePermission::new("/api/llm/**", "POST", ""),
        RoutePermission::new("/api/llm/**", "PUT", ""),
        RoutePermission::new("/api/llm/**", "DELETE", ""),

        // Email
        RoutePermission::new("/api/email/**", "GET", ""),
        RoutePermission::new("/api/email/**", "POST", ""),
        RoutePermission::new("/api/email/**", "PUT", ""),
        RoutePermission::new("/api/email/**", "DELETE", ""),

        // Messaging channels
        RoutePermission::new("/api/telegram/**", "GET", ""),
        RoutePermission::new("/api/telegram/**", "POST", ""),
        RoutePermission::new("/api/whatsapp/**", "GET", ""),
        RoutePermission::new("/api/whatsapp/**", "POST", ""),
        RoutePermission::new("/api/msteams/**", "GET", ""),
        RoutePermission::new("/api/msteams/**", "POST", ""),
        RoutePermission::new("/api/instagram/**", "GET", ""),
        RoutePermission::new("/api/instagram/**", "POST", ""),

        // Pages
        RoutePermission::new("/api/pages/**", "GET", ""),
        RoutePermission::new("/api/pages/**", "POST", ""),
        RoutePermission::new("/api/pages/**", "PUT", ""),
        RoutePermission::new("/api/pages/**", "DELETE", ""),

        // Insights
        RoutePermission::new("/api/insights/**", "GET", ""),
        RoutePermission::new("/api/insights/**", "POST", ""),

        // App logs
        RoutePermission::new("/api/app-logs/**", "GET", ""),
        RoutePermission::new("/api/app-logs/**", "POST", ""),

        // User profile (own user)
        RoutePermission::new("/api/user/**", "GET", ""),
        RoutePermission::new("/api/user/**", "PUT", ""),

        // Personal memory (#2952). The botmemory router scopes every row to the
        // verified JWT `sub` and only widens a listing to branch-shared rows
        // when the verified `branch_id` claim matches; writes derive org/branch
        // from that same verified token. No bot/session context is required, so
        // any authenticated caller (admin or regular user) may reach the
        // handler, which fails closed on foreign tenants. Without these rows
        // the default-deny RBAC gate answered 403 before the handler ran.
        RoutePermission::new("/api/memory/**", "GET", ""),
        RoutePermission::new("/api/memory/**", "POST", ""),
        RoutePermission::new("/api/memory/**", "PUT", ""),
        RoutePermission::new("/api/memory/**", "DELETE", ""),

        // Agent VM (per-user VM, #1580)
        RoutePermission::new("/api/agent/**", "GET", ""),
        RoutePermission::new("/api/agent/**", "POST", ""),
        RoutePermission::new("/api/agent/**", "PUT", ""),
        RoutePermission::new("/api/agent/**", "DELETE", ""),

    ]
}

#[cfg(test)]
mod memory_routes_tests {
    use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
    use crate::{AuthenticatedUser, Role};
    use uuid::Uuid;

    /// Pure catalogue check: every verb mounted by `botmemory::api` must be
    /// present, authenticated-only and free of extra role/permission gates —
    /// the handler owns owner/tenant scoping, not the route table.
    #[test]
    fn memory_prefix_is_registered_for_every_mounted_verb() {
        let routes = super::routes();
        for method in ["GET", "POST", "PUT", "DELETE"] {
            let route = routes
                .iter()
                .find(|r| r.path_pattern == "/api/memory/**" && r.method == method);
            assert!(
                route.is_some(),
                "/api/memory/** {method} must be in the authenticated catalogue"
            );
            let route = route.expect("memory route");
            assert!(route.required_permission.is_empty(), "{method} permission gate");
            assert!(route.required_roles.is_empty(), "{method} role gate");
            assert!(!route.allow_anonymous, "{method} must require a session");
        }
    }

    #[test]
    fn the_memory_route_matches_only_its_own_prefix() {
        let route = super::routes()
            .into_iter()
            .find(|r| r.path_pattern == "/api/memory/**" && r.method == "GET")
            .expect("memory GET route");
        assert!(route.matches_path("/api/memory/items"));
        assert!(route.matches_path("/api/memory/items/42"));
        assert!(route.matches_path("/api/memory/export"));
        assert!(!route.matches_path("/api/memoryx/items"));
        assert!(!route.matches_path("/api/memorying/items"));
    }

    #[tokio::test]
    async fn memory_routes_admit_authenticated_users_and_admins_but_not_anonymous() {
        let manager = RbacManager::with_defaults();
        manager
            .register_routes(build_default_route_permissions())
            .await;
        let user = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string())
            .with_role(Role::User);
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
            .with_role(Role::Admin);

        for (method, path) in [
            ("GET", "/api/memory/items"),
            ("GET", "/api/memory/export"),
            ("POST", "/api/memory/items"),
            ("PUT", "/api/memory/items/00000000-0000-0000-0000-000000000001"),
            ("DELETE", "/api/memory/items/00000000-0000-0000-0000-000000000001"),
            ("POST", "/api/memory/import"),
        ] {
            for principal in [&user, &admin] {
                let decision = manager.check_route_access(path, method, principal).await;
                assert!(
                    decision.is_allowed(),
                    "{method} {path} must be allowed for {} : {}",
                    principal.username,
                    decision.reason
                );
            }
            let denied = manager
                .check_route_access(path, method, &AuthenticatedUser::anonymous())
                .await;
            assert!(
                !denied.is_allowed(),
                "{method} {path} must stay closed to anonymous callers"
            );
        }
    }
}

#[cfg(test)]
mod integration_connections_routes_tests {
    use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
    use crate::{AuthenticatedUser, Role};
    use uuid::Uuid;

    const BOT_ID: &str = "f47ac10b-58cc-4372-a567-0e02b2c3d480";
    const BASE: &str = "/api/bots/{id}/integration-connections";
    const NESTED: &str = "/api/bots/{id}/integration-connections/**";

    fn find(pattern: &str, method: &str) -> crate::rbac_middleware::RoutePermission {
        super::routes()
            .into_iter()
            .find(|r| r.path_pattern == pattern && r.method == method)
            .expect("route must be in the catalogue")
    }

    #[test]
    fn connections_read_routes_are_explicit_and_roleless() {
        for pattern in [BASE, NESTED] {
            let route = find(pattern, "GET");
            assert!(route.required_roles.is_empty(), "{pattern} role gate");
            assert!(
                route.required_permission.is_empty(),
                "{pattern} permission gate"
            );
            assert!(!route.allow_anonymous, "{pattern} must require a session");
        }
        // The write verbs stay roleless too; the handler owns tenant scoping.
        assert!(find(BASE, "POST").required_roles.is_empty());
        assert!(find(NESTED, "DELETE").required_roles.is_empty());
    }

    #[test]
    fn connections_read_patterns_cover_only_their_own_prefix() {
        let base = find(BASE, "GET");
        let nested = find(NESTED, "GET");
        let url = format!("/api/bots/{BOT_ID}/integration-connections");

        assert!(base.matches_path(&url));
        assert!(!base.matches_path(&format!("{url}/{BOT_ID}")));
        assert!(!base.matches_path(&format!("{url}-extra")));

        assert!(nested.matches_path(&format!("{url}/{BOT_ID}")));
        assert!(nested.matches_path(&format!("{url}/{BOT_ID}/events")));
        assert!(!nested.matches_path(&format!("{url}-extra/{BOT_ID}")));
    }

    #[tokio::test]
    async fn connections_reads_admit_members_and_admins_but_not_anonymous() {
        let manager = RbacManager::with_defaults();
        manager
            .register_routes(build_default_route_permissions())
            .await;
        let member = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string())
            .with_role(Role::User);
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
            .with_role(Role::Admin);
        let url = format!("/api/bots/{BOT_ID}/integration-connections");

        for (method, path) in [
            ("GET", url.as_str()),
            ("GET", format!("{url}/{BOT_ID}").as_str()),
            ("GET", format!("{url}/{BOT_ID}/events").as_str()),
            ("POST", url.as_str()),
            ("DELETE", format!("{url}/{BOT_ID}").as_str()),
        ] {
            for principal in [&member, &admin] {
                let decision = manager.check_route_access(path, method, principal).await;
                assert!(
                    decision.is_allowed(),
                    "{method} {path} must be allowed for {} : {}",
                    principal.username,
                    decision.reason
                );
            }
            let denied = manager
                .check_route_access(path, method, &AuthenticatedUser::anonymous())
                .await;
            assert!(
                !denied.is_allowed(),
                "{method} {path} must stay closed anonymously"
            );
        }
    }
}
