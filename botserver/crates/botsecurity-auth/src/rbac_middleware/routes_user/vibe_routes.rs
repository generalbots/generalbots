//! Vibe platform routes for logged-in users.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        // Vibe platform (dashboard reads for any logged-in user)
        RoutePermission::new("/api/vibe/graph/**", "GET", ""),
        RoutePermission::new("/api/vibe/capabilities/**", "GET", ""),
        RoutePermission::new("/api/vibe/pipeline/**", "GET", ""),
        RoutePermission::new("/api/vibe/metrics/**", "GET", ""),
        RoutePermission::new("/api/vibe/runs/**", "GET", ""),
        RoutePermission::new("/api/vibe/tools/**", "GET", ""),
        // #1290 — tool execution endpoints (e.g. publish/project, domain/bind)
        // live under /api/vibe/tools/**; without a POST rule the default-deny
        // catalog rejects every tool call from the UI with "No matching route
        // permission found". Role enforcement for deploy-grade tools happens
        // inside the handlers (RBAC + metering), so the route opens to any
        // authenticated user here.
        RoutePermission::new("/api/vibe/tools/**", "POST", ""),
        RoutePermission::new("/api/vibe/events/**", "GET", ""),
        RoutePermission::new("/api/vibe/teams/**", "GET", ""),
        RoutePermission::new("/api/vibe/run/**", "POST", ""),
        RoutePermission::new("/api/vibe/run/**", "GET", ""),
        RoutePermission::new("/api/vibe/runs", "POST", ""),
        RoutePermission::new("/api/vibe/teams", "POST", ""),
        // Vibe project registry + VM lifecycle (#743/#744): creating a custom
        // app (e.g. a node calculator) and raising its dev VM must work for
        // any logged-in user.
        RoutePermission::new("/api/vibe/projects/**", "GET", ""),
        RoutePermission::new("/api/vibe/projects/**", "POST", ""),
        RoutePermission::new("/api/vibe/projects/**", "PUT", ""),
        RoutePermission::new("/api/vibe/projects/**", "DELETE", ""),
        // Vibe project-scoped resources (members/backups/deployments/envs/
        // metering/vms): authenticated users reach them; per-project RBAC
        // (member/developer/owner) still gates sensitive operations.
        // User typeahead for the members dialog (no raw UUID pasting).
        RoutePermission::new("/api/vibe/users/search", "GET", ""),
        RoutePermission::new("/api/vibe/projects/:project_id/**", "GET", ""),
        RoutePermission::new("/api/vibe/projects/:project_id/**", "POST", ""),
        RoutePermission::new("/api/vibe/projects/:project_id/**", "PUT", ""),
        RoutePermission::new("/api/vibe/projects/:project_id/**", "DELETE", ""),
        // AI-OS wave (#1171/#1172/#1173/#1175/#1182/#1185) — planner,
        // agent API, mixture-of-agents, browser memory, browser driver,
        // proactivity scheduler. The MOA share route is anonymous so
        // published deliverables are linkable without auth (#1180-style).
        RoutePermission::new("/api/vibe/planner/**", "GET", ""),
        RoutePermission::new("/api/vibe/planner/**", "POST", ""),
        RoutePermission::new("/api/vibe/agents/**", "GET", ""),
        RoutePermission::new("/api/vibe/agents/**", "POST", ""),
        RoutePermission::new("/api/vibe/agents/**", "PUT", ""),
        RoutePermission::new("/api/vibe/agents/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/moa/runs/**", "GET", ""),
        RoutePermission::new("/api/vibe/moa/route", "POST", ""),
        RoutePermission::new("/api/vibe/moa/share/:token", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/vibe/browser-memory/**", "GET", ""),
        RoutePermission::new("/api/vibe/browser-memory/**", "POST", ""),
        RoutePermission::new("/api/vibe/browser-memory/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/browser-driver/**", "GET", ""),
        RoutePermission::new("/api/vibe/browser-driver/**", "POST", ""),
        RoutePermission::new("/api/vibe/proactivity/**", "GET", ""),
        RoutePermission::new("/api/vibe/proactivity/**", "POST", ""),
        RoutePermission::new("/api/vibe/proactivity/**", "DELETE", ""),
        // Vibe canvas + collaboration surfaces for logged-in users.
        RoutePermission::new("/api/vibe/canvases/**", "GET", ""),
        RoutePermission::new("/api/vibe/canvases/**", "POST", ""),
        RoutePermission::new("/api/vibe/canvases/**", "PUT", ""),
        RoutePermission::new("/api/vibe/canvases/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/teams/**", "GET", ""),
        RoutePermission::new("/api/vibe/teams/**", "POST", ""),
        RoutePermission::new("/api/vibe/teams/**", "PUT", ""),
        RoutePermission::new("/api/vibe/teams/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/issues/**", "GET", ""),
        RoutePermission::new("/api/vibe/issues/**", "POST", ""),
        RoutePermission::new("/api/vibe/issues/**", "PUT", ""),
        RoutePermission::new("/api/vibe/issues/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/permissions/**", "GET", ""),
        RoutePermission::new("/api/vibe/permissions/**", "POST", ""),
        RoutePermission::new("/api/vibe/permissions/**", "PUT", ""),
        RoutePermission::new("/api/vibe/permissions/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/vms/**", "GET", ""),
        RoutePermission::new("/api/vibe/vms/**", "POST", ""),
        RoutePermission::new("/api/vibe/vms/**", "DELETE", ""),
        RoutePermission::new("/api/vibe/metering/**", "GET", ""),
        RoutePermission::new("/api/vibe/metering/**", "POST", ""),
        // Vibe sessions, skills and diagnostics for logged-in users.
        RoutePermission::new("/api/vibe/sessions/**", "GET", ""),
        RoutePermission::new("/api/vibe/sessions/**", "POST", ""),
        RoutePermission::new("/api/vibe/skills/**", "GET", ""),
        RoutePermission::new("/api/vibe/skills/**", "POST", ""),
        RoutePermission::new("/api/vibe/doctor", "GET", ""),
    ]
}
