//! HTMX UI fragment routes for authenticated users.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        RoutePermission::new("/api/ui/tasks/**", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/ui/tasks/**", "POST", ""),
        RoutePermission::new("/api/ui/tasks/**", "PUT", ""),
        RoutePermission::new("/api/ui/tasks/**", "PATCH", ""),
        RoutePermission::new("/api/ui/tasks/**", "DELETE", ""),
        RoutePermission::new("/api/ui/calendar/**", "GET", ""),
        RoutePermission::new("/api/ui/calendar/**", "POST", ""),
        RoutePermission::new("/api/ui/drive/**", "GET", ""),
        RoutePermission::new("/api/ui/drive/**", "POST", ""),
        RoutePermission::new("/api/ui/mail/**", "GET", ""),
        RoutePermission::new("/api/ui/mail/**", "POST", ""),
        RoutePermission::new("/api/ui/docs/**", "GET", ""),
        RoutePermission::new("/api/ui/docs/**", "POST", ""),
        RoutePermission::new("/api/ui/paper/**", "GET", ""),
        RoutePermission::new("/api/ui/paper/**", "POST", ""),
        RoutePermission::new("/api/ui/sheet/**", "GET", ""),
        RoutePermission::new("/api/ui/sheet/**", "POST", ""),
        RoutePermission::new("/api/ui/slides/**", "GET", ""),
        RoutePermission::new("/api/ui/slides/**", "POST", ""),
        RoutePermission::new("/api/ui/meet/**", "GET", ""),
        RoutePermission::new("/api/ui/meet/**", "POST", ""),
        RoutePermission::new("/api/ui/research/**", "GET", ""),
        RoutePermission::new("/api/ui/research/**", "POST", ""),
        RoutePermission::new("/api/ui/search", "GET", ""),
        RoutePermission::new("/api/ui/sources/**", "GET", ""),
        RoutePermission::new("/api/ui/sources/**", "POST", ""),
        RoutePermission::new("/api/ui/canvas/**", "GET", ""),
        RoutePermission::new("/api/ui/video/**", "GET", ""),
        RoutePermission::new("/api/ui/player/**", "GET", ""),
        RoutePermission::new("/api/ui/workspaces/**", "GET", ""),
        RoutePermission::new("/api/ui/projects/**", "GET", ""),
        // Project app UI fragments (singular `/api/ui/project/**` — must match
        // the routes registered in src/project/project_ui.rs; the plural
        // `/api/ui/projects/**` entry above does NOT cover these).
        RoutePermission::new("/api/ui/project/**", "GET", ""),
        RoutePermission::new("/api/ui/project/**", "POST", ""),
        RoutePermission::new("/api/ui/goals/**", "GET", ""),
        RoutePermission::new("/api/ui/designer/**", "GET", ""),
        // #2963 — Plugins app (`botdesigner::plugin_manifest::handlers`) mounts
        // GET /api/ui/plugins, POST .../upload, POST .../:id/toggle and
        // DELETE /api/ui/plugins/:id. The prefix was missing from the
        // default-deny matrix, so `handle_list_plugins` never ran and the app
        // opened with a 403. Every handler scopes manifests to the
        // server-owned bucket (`state.bucket_name`), so the matrix only has to
        // gate authentication: authenticated callers pass, anonymous stay out.
        RoutePermission::new("/api/ui/plugins/**", "GET", ""),
        RoutePermission::new("/api/ui/plugins/**", "POST", ""),
        RoutePermission::new("/api/ui/plugins/**", "DELETE", ""),
        RoutePermission::new("/api/ui/dashboards/**", "GET", ""),
        RoutePermission::new("/api/ui/crm/**", "GET", ""),
        RoutePermission::new("/api/ui/billing/**", "GET", ""),
        RoutePermission::new("/api/ui/products/**", "GET", ""),
        RoutePermission::new("/api/ui/tickets/**", "GET", ""),
        RoutePermission::new("/api/ui/learn/**", "GET", ""),
        RoutePermission::new("/api/ui/social/**", "GET", ""),
        RoutePermission::new("/api/ui/settings/**", "GET", ""),
        RoutePermission::new("/api/ui/autotask/**", "GET", ""),
        RoutePermission::new("/api/ui/email/**", "GET", ""),
        RoutePermission::new("/api/ui/email/**", "POST", ""),

        // Search (global FTS index) — any authenticated user
        RoutePermission::new("/api/search/**", "GET", ""),
        RoutePermission::new("/api/search/**", "POST", ""),

        // DNS records (infrastructure) — any authenticated user
        RoutePermission::new("/api/dns/**", "GET", ""),
        RoutePermission::new("/api/dns/**", "POST", ""),
        RoutePermission::new("/api/dns/**", "PUT", ""),

        // o365 (unified office integration) — any authenticated user
        RoutePermission::new("/api/o365/**", "GET", ""),
        RoutePermission::new("/api/o365/**", "POST", ""),
        RoutePermission::new("/api/o365/**", "PUT", ""),
        RoutePermission::new("/api/m365/**", "GET", ""),
        RoutePermission::new("/api/m365/**", "POST", ""),
        RoutePermission::new("/api/m365/**", "PUT", ""),
        RoutePermission::new("/api/meet/**", "GET", ""),
        RoutePermission::new("/api/meet/**", "POST", ""),
        RoutePermission::new("/api/voice/**", "POST", ""),
    ]
}

#[cfg(test)]
mod tests {
    use crate::rbac_middleware::{RbacManager, build_default_route_permissions};
    use crate::{AuthenticatedUser, Role};
    use uuid::Uuid;

    /// The catalogue must admit every verb mounted by
    /// `plugin_manifest::handlers::configure_plugin_routes` to an authenticated
    /// caller, without a role/permission gate and without anonymous access —
    /// tenant scoping is owned by the handler's server-owned bucket.
    #[test]
    fn plugins_prefix_covers_mounted_verbs_without_anonymous() {
        let routes = build_default_route_permissions();
        for method in ["GET", "POST", "DELETE"] {
            let route = routes
                .iter()
                .find(|r| r.path_pattern == "/api/ui/plugins/**" && r.method == method);
            assert!(
                route.is_some(),
                "/api/ui/plugins/** {method} must be in the RBAC catalogue"
            );
            if let Some(route) = route {
                assert!(route.required_roles.is_empty(), "{method} role gate");
                assert!(
                    route.required_permission.is_empty(),
                    "{method} permission gate"
                );
                assert!(!route.allow_anonymous, "{method} must require a session");
            }
        }
    }

    /// Offline decision table: a signed-in admin reaches each endpoint the
    /// Plugins app polls or mutates; the same rules deny an anonymous caller.
    #[tokio::test]
    async fn plugins_endpoints_allow_admin_and_deny_anonymous() {
        let manager = RbacManager::with_defaults();
        manager
            .register_routes(build_default_route_permissions())
            .await;
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin@example.com".to_string())
            .with_role(Role::Admin);
        let anonymous = AuthenticatedUser::anonymous();

        let cases = [
            ("GET", "/api/ui/plugins"),
            ("POST", "/api/ui/plugins/upload"),
            ("POST", "/api/ui/plugins/sample/toggle"),
            ("DELETE", "/api/ui/plugins/sample"),
        ];
        for (method, path) in cases {
            let allowed = manager.check_route_access(path, method, &admin).await;
            assert!(
                allowed.is_allowed(),
                "{method} {path} must be allowed for an admin: {}",
                allowed.reason
            );
            let denied = manager.check_route_access(path, method, &anonymous).await;
            assert!(
                !denied.is_allowed(),
                "{method} {path} must stay closed to anonymous callers"
            );
        }
    }

    /// The wildcard must match the bare list path and its sub-paths, but not a
    /// look-alike prefix that would widen the grant to unrelated endpoints.
    #[test]
    fn plugins_prefix_does_not_match_lookalikes() {
        let routes = build_default_route_permissions();
        let route = routes
            .iter()
            .find(|r| r.path_pattern == "/api/ui/plugins/**" && r.method == "GET");
        assert!(route.is_some(), "plugins GET catalogue entry");
        if let Some(route) = route {
            for path in [
                "/api/ui/plugins",
                "/api/ui/plugins/upload",
                "/api/ui/plugins/sample/toggle",
            ] {
                assert!(route.matches_path(path), "GET {path} must match");
            }
            assert!(!route.matches_path("/api/ui/pluginsx/upload"));
        }
    }
}
