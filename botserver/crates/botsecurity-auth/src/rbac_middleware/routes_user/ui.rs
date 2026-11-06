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
