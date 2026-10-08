use super::*;

pub(crate) fn authenticated_routes() -> Vec<RoutePermission> {
    vec![

        // Drive / Files
        RoutePermission::new("/api/drive/**", "GET", ""),
        RoutePermission::new("/api/drive/**", "POST", ""),
        RoutePermission::new("/api/drive/**", "PUT", ""),
        RoutePermission::new("/api/drive/**", "DELETE", ""),
        RoutePermission::new("/api/files/**", "GET", ""),
        RoutePermission::new("/api/files/**", "POST", ""),
        RoutePermission::new("/api/files/**", "PUT", ""),
        RoutePermission::new("/api/files/**", "DELETE", ""),

        // Editor
        RoutePermission::new("/api/editor/**", "GET", ""),
        RoutePermission::new("/api/editor/**", "POST", ""),
        RoutePermission::new("/api/editor/**", "PUT", ""),
        RoutePermission::new("/api/editor/**", "DELETE", ""),

        // Database
        RoutePermission::new("/api/database/**", "GET", ""),
        RoutePermission::new("/api/database/**", "POST", ""),
        RoutePermission::new("/api/database/**", "PUT", ""),
        RoutePermission::new("/api/database/**", "DELETE", ""),

        // Git
        RoutePermission::new("/api/git/**", "GET", ""),
        RoutePermission::new("/api/git/**", "POST", ""),
        RoutePermission::new("/api/git/**", "PUT", ""),
        RoutePermission::new("/api/git/**", "DELETE", ""),

        // Mail
        RoutePermission::new("/api/mail/**", "GET", ""),
        RoutePermission::new("/api/mail/**", "POST", ""),
        RoutePermission::new("/api/mail/**", "PUT", ""),
        RoutePermission::new("/api/mail/**", "DELETE", ""),

        // Notifications — Web Push subscription backend (#1247): the Settings
        // app registers/unregisters the browser PushSubscription.
        RoutePermission::new("/api/notifications/push/**", "POST", ""),
        RoutePermission::new("/api/notifications/**", "GET", ""),

        // Calendar
        RoutePermission::new("/api/calendar/**", "GET", ""),
        RoutePermission::new("/api/calendar/**", "POST", ""),
        RoutePermission::new("/api/calendar/**", "PUT", ""),
        RoutePermission::new("/api/calendar/**", "DELETE", ""),

        // Tasks
        RoutePermission::new("/api/tasks/**", "GET", ""),
        RoutePermission::new("/api/tasks/**", "POST", ""),
        RoutePermission::new("/api/tasks/**", "PUT", ""),
        RoutePermission::new("/api/tasks/**", "PATCH", ""),
        RoutePermission::new("/api/tasks/**", "DELETE", ""),

        // Docs / Paper
        RoutePermission::new("/api/docs/**", "GET", ""),
        RoutePermission::new("/api/docs/**", "POST", ""),
        RoutePermission::new("/api/docs/**", "PUT", ""),
        RoutePermission::new("/api/docs/**", "DELETE", ""),
        RoutePermission::new("/api/paper/**", "GET", ""),
        RoutePermission::new("/api/paper/**", "POST", ""),
        RoutePermission::new("/api/paper/**", "PUT", ""),
        RoutePermission::new("/api/paper/**", "DELETE", ""),

        // Sheet
        RoutePermission::new("/api/sheet/**", "GET", "").with_anonymous(true),
        RoutePermission::new("/api/sheet/**", "POST", "").with_anonymous(true),
        RoutePermission::new("/api/sheet/**", "PUT", "").with_anonymous(true),
        RoutePermission::new("/api/sheet/**", "DELETE", "").with_anonymous(true),

        // Jukebox — was missing from the registry, so default_deny 403'd every
        // request (including /api/jukebox/health from the jukebox app).
        RoutePermission::new("/api/jukebox/**", "GET", ""),
        RoutePermission::new("/api/jukebox/**", "POST", ""),
        RoutePermission::new("/api/jukebox/**", "PUT", ""),
        RoutePermission::new("/api/jukebox/**", "DELETE", ""),

        // Slides
        RoutePermission::new("/api/slides/**", "GET", ""),
        RoutePermission::new("/api/slides/**", "POST", ""),
        RoutePermission::new("/api/slides/**", "PUT", ""),
        RoutePermission::new("/api/slides/**", "DELETE", ""),

        // Meet
        RoutePermission::new("/api/meet/**", "GET", ""),
        RoutePermission::new("/api/meet/**", "POST", ""),
        RoutePermission::new("/api/meet/**", "PUT", ""),
        RoutePermission::new("/api/meet/**", "DELETE", ""),

        // Research
        RoutePermission::new("/api/research/**", "GET", ""),
        RoutePermission::new("/api/research/**", "POST", ""),
        RoutePermission::new("/api/research/**", "PUT", ""),
        RoutePermission::new("/api/research/**", "DELETE", ""),

        // Sources
        RoutePermission::new("/api/sources/**", "GET", ""),
        RoutePermission::new("/api/sources/**", "POST", ""),
        RoutePermission::new("/api/sources/**", "PUT", ""),
        RoutePermission::new("/api/sources/**", "DELETE", ""),

        // Canvas
        RoutePermission::new("/api/canvas/**", "GET", ""),
        RoutePermission::new("/api/canvas/**", "POST", ""),
        RoutePermission::new("/api/canvas/**", "PUT", ""),
        RoutePermission::new("/api/canvas/**", "DELETE", ""),

        // Video / Player
        RoutePermission::new("/api/video/**", "GET", ""),
        RoutePermission::new("/api/video/**", "POST", ""),
        RoutePermission::new("/api/player/**", "GET", ""),
        RoutePermission::new("/api/player/**", "POST", ""),

        // Workspaces
        RoutePermission::new("/api/workspaces/**", "GET", ""),
        RoutePermission::new("/api/workspaces/**", "POST", ""),
        RoutePermission::new("/api/workspaces/**", "PUT", ""),
        RoutePermission::new("/api/workspaces/**", "DELETE", ""),

        // Projects
        RoutePermission::new("/api/projects/**", "GET", ""),
        RoutePermission::new("/api/projects/**", "POST", ""),
        RoutePermission::new("/api/projects/**", "PUT", ""),
        RoutePermission::new("/api/projects/**", "DELETE", ""),

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

        // Goals
        RoutePermission::new("/api/goals/**", "GET", ""),
        RoutePermission::new("/api/goals/**", "POST", ""),
        RoutePermission::new("/api/goals/**", "PUT", ""),
        RoutePermission::new("/api/goals/**", "DELETE", ""),

        // Settings (user's own settings)
        RoutePermission::new("/api/settings/**", "GET", ""),
        RoutePermission::new("/api/settings/**", "POST", ""),
        RoutePermission::new("/api/settings/**", "PUT", ""),

        // Bots (read for all authenticated users)
        RoutePermission::new("/api/bots", "GET", ""),
        RoutePermission::new("/api/bots/{id}", "GET", ""),
        RoutePermission::new("/api/bots/{id}/**", "GET", ""),

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
        RoutePermission::new("/api/billing/**", "GET", ""),
        RoutePermission::new("/api/billing/**", "POST", ""),
        RoutePermission::new("/api/products/**", "GET", ""),
        RoutePermission::new("/api/products/**", "POST", ""),
        RoutePermission::new("/api/products/**", "PUT", ""),
        RoutePermission::new("/api/products/**", "DELETE", ""),

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

    ]
}

pub(crate) fn ui_routes() -> Vec<RoutePermission> {
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
