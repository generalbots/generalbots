//! #2036 — endpoint-prefix → app map, and the guard that keeps the two in sync.
//!
//! The inventory lists every mounted REST endpoint (`endpoint_inventory`), but
//! an endpoint is only reachable from chat when a curated verb covers its
//! app. Without this table a new crate could add thirty endpoints and no verb,
//! and nothing would notice: `api.find` would return paths the model cannot
//! explain. The test at the bottom fails the build for any prefix that is
//! neither mapped to an app with at least one verb nor declared a platform
//! surface (auth, health, setup, HTMX fragments, …).

/// Endpoint path prefix → owning app id. The app must expose at least one
/// curated command; the guard test below enforces that.
pub(crate) static ENDPOINT_PREFIX_APPS: &[(&str, &str)] = &[
    ("/api/admin", "settings"),
    ("/api/ai", "chat"),
    ("/api/analytics", "analytics"),
    ("/api/apps", "desktop"),
    ("/api/attendance", "timeclock"),
    ("/api/attendant", "attendant"),
    ("/api/autotask", "automations"),
    ("/api/banking", "banking"),
    ("/api/billing", "billing"),
    ("/api/bots", "settings"),
    ("/api/brazil", "tax"),
    ("/api/browser", "browser"),
    ("/api/calendar", "calendar"),
    ("/api/canvas", "canvas"),
    ("/api/catalog", "products"),
    ("/api/chat", "chat"),
    ("/api/cloud", "store"),
    ("/api/compliance", "compliance"),
    ("/api/contacts", "people"),
    ("/api/crm", "crm"),
    ("/api/dashboards", "analytics"),
    ("/api/database", "database"),
    ("/api/deployment", "vibe"),
    ("/api/directory", "settings"),
    ("/api/dns", "settings"),
    ("/api/domains", "settings"),
    ("/api/docs", "docs"),
    ("/api/editor", "editor"),
    ("/api/email", "mail"),
    ("/api/erp", "products"),
    ("/api/files", "drive"),
    ("/api/fraud", "fraud"),
    ("/api/git", "vibe"),
    ("/api/goals", "goals"),
    ("/api/governance", "compliance"),
    ("/api/groups", "settings"),
    ("/api/handoff", "handoff"),
    ("/api/hr", "hr"),
    ("/api/insights", "analytics"),
    ("/api/integrations", "integrations"),
    ("/api/instagram", "social"),
    ("/api/itsm", "tickets"),
    ("/api/jukebox", "jukebox"),
    ("/api/kyc", "kyc"),
    ("/api/learn", "learn"),
    ("/api/legal", "compliance"),
    ("/api/m365", "o365"),
    ("/api/marketing", "campaigns"),
    ("/api/mcp", "integrations"),
    ("/api/meet", "meet"),
    ("/api/minutes", "minutes"),
    ("/api/monitoring", "monitoring"),
    ("/api/msteams", "o365"),
    ("/api/o365", "o365"),
    ("/api/ops", "monitoring"),
    ("/api/organizations", "settings"),
    ("/api/pages", "workspace"),
    ("/api/people", "people"),
    ("/api/plan", "plan"),
    ("/api/player", "player"),
    ("/api/pos", "pos"),
    ("/api/products", "products"),
    ("/api/projects", "project"),
    ("/api/rbac", "settings"),
    ("/api/reports", "analytics"),
    ("/api/retail", "retail"),
    ("/api/runs", "vibe"),
    ("/api/sales", "crm"),
    ("/api/search", "research"),
    ("/api/security", "settings"),
    ("/api/settings", "settings"),
    ("/api/sheet", "sheet"),
    ("/api/slides", "slides"),
    ("/api/social", "social"),
    ("/api/tasks", "tasks"),
    ("/api/tax", "tax"),
    ("/api/templates", "templates"),
    ("/api/tickets", "tickets"),
    ("/api/timeclock", "timeclock"),
    ("/api/user", "settings"),
    ("/api/users", "settings"),
    ("/api/vibe", "vibe"),
    ("/api/video", "video"),
    ("/api/vision", "vision"),
    ("/api/voice", "chat"),
    ("/api/website", "templates"),
    ("/api/workspaces", "workspace"),
    ("/api-keys", "integrations"),
    ("/conversations", "chat"),
];

/// Prefixes that are infrastructure rather than an app surface: they have no
/// owning application, so no curated verb is expected for them.
pub(crate) static PLATFORM_ENDPOINT_PREFIXES: &[&str] = &[
    "/api/activity",
    "/api/auth",
    "/api/bot",
    "/api/client-errors",
    "/api/config",
    "/api/features",
    "/api/i18n",
    "/api/manifest",
    "/api/oauth",
    "/api/product",
    "/api/services",
    "/api/sessions",
    "/api/setup",
    "/api/system",
    "/api/telegram",
    "/api/ui",
    "/api/v2",
    "/cloud",
];

/// First path segment after `/api/` (or the leading segment of a root path).
pub(crate) fn endpoint_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("/api/") {
        let segment = rest.split('/').next().unwrap_or("");
        return format!("/api/{segment}");
    }
    if path == "/api" {
        return "/api".to_string();
    }
    let segment = path.trim_start_matches('/').split('/').next().unwrap_or("");
    format!("/{segment}")
}

#[cfg(test)]
mod endpoint_prefix_guard {
    use super::*;
    use crate::apps::commands::commands_for_app;
    use crate::core::bot::endpoint_inventory::all_routes;

    /// #2036 — every mounted endpoint must belong to an app the catalogue can
    /// act on, or be declared platform infrastructure. This is the assertion
    /// that stops a new crate from adding endpoints nobody can reach.
    #[test]
    fn every_endpoint_prefix_maps_to_a_curated_verb() {
        for endpoint in all_routes() {
            let prefix = endpoint_prefix(&endpoint.path);
            if PLATFORM_ENDPOINT_PREFIXES.contains(&prefix.as_str()) {
                continue;
            }
            let app = ENDPOINT_PREFIX_APPS
                .iter()
                .find(|(mapped, _)| *mapped == prefix)
                .map(|(_, app)| *app)
                .unwrap_or_else(|| {
                    panic!(
                        "endpoint prefix {prefix} ({} {}) has no app mapping — add it to \
                         ENDPOINT_PREFIX_APPS or to PLATFORM_ENDPOINT_PREFIXES",
                        endpoint.method, endpoint.path
                    )
                });
            assert!(
                !commands_for_app(app).is_empty(),
                "endpoint prefix {prefix} maps to app {app}, which has no curated verb"
            );
        }
    }

    #[test]
    fn prefix_extraction_matches_the_inventory_shape() {
        assert_eq!(endpoint_prefix("/api/database/table/:name/row"), "/api/database");
        assert_eq!(endpoint_prefix("/api-keys"), "/api-keys");
        assert_eq!(endpoint_prefix("/conversations/:id"), "/conversations");
        assert_eq!(endpoint_prefix("/cloud/billing"), "/cloud");
    }
}
