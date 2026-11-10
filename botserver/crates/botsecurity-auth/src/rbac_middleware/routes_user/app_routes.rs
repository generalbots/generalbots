//! Authenticated application routes: chat, drive, editors, docs and projects.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        // Chat history: authenticated users manage their own conversations.
        // PUT was missing, so renaming a conversation was rejected by the
        // default-deny catalogue with 403 before reaching the handler.
        RoutePermission::new("/api/chat/history/**", "GET", ""),
        RoutePermission::new("/api/chat/history/**", "PUT", ""),
        RoutePermission::new("/api/chat/history/**", "DELETE", ""),
        // #1682 — lock/unlock a conversation behind a PIN.
        RoutePermission::new("/api/chat/history/**", "POST", ""),

        // Drive / Files
        RoutePermission::new("/api/drive/**", "GET", ""),
        RoutePermission::new("/api/drive/**", "POST", ""),
        RoutePermission::new("/api/drive/**", "PUT", ""),
        RoutePermission::new("/api/drive/**", "DELETE", ""),
        RoutePermission::new("/api/files/**", "GET", ""),
        RoutePermission::new("/api/files/**", "POST", ""),
        RoutePermission::new("/api/files/**", "PUT", ""),
        RoutePermission::new("/api/files/**", "DELETE", ""),
        RoutePermission::new("/api/ai-accounts/**", "GET", ""),
        RoutePermission::new("/api/ai-accounts/**", "POST", ""),
        // Sandbox configuration for the chat client (#1674)
        RoutePermission::new("/api/sandbox/config", "GET", ""),
        // Tool manifests for the native surface tools (#1678)
        RoutePermission::new("/api/runs/tools", "GET", ""),
        // Built-in MCP server for generated apps (#1676)
        RoutePermission::new("/api/mcp/**", "GET", ""),
        RoutePermission::new("/api/mcp/**", "POST", ""),

        // Computer use (browser + terminal consent and execution)
        RoutePermission::new("/api/computer/**", "GET", ""),
        RoutePermission::new("/api/computer/**", "POST", ""),
        RoutePermission::new("/api/terminal/**", "GET", ""),
        RoutePermission::new("/api/terminal/**", "POST", ""),
        // Consent prompts for terminal/browser/API-tool actions are per-user.
        // Without these rows the default-deny catalogue rejected every
        // pending/resolve call with 403 before the handler ran.
        RoutePermission::new("/api/consent/**", "GET", ""),
        RoutePermission::new("/api/consent/**", "POST", ""),
        RoutePermission::new("/api/consent/**", "DELETE", ""),
        RoutePermission::new("/api/ui/consent/**", "GET", ""),
        // Conversation autotasks (#1613): per-user cron/systemd jobs owned by
        // the caller's conversation. Missing from the catalogue, so every
        // GET/POST answered 403 and the chat indicator was permanently dark.
        RoutePermission::new("/api/conversations/**", "GET", ""),
        RoutePermission::new("/api/conversations/**", "POST", ""),
        RoutePermission::new("/api/conversation-autotasks/**", "POST", ""),
        // Workspace tab mirror (chat tab strip).
        RoutePermission::new("/api/user/workspace/**", "GET", ""),
        RoutePermission::new("/api/user/workspace/**", "PUT", ""),
        // Connected external drives (compiled only with external_sync)
        RoutePermission::new("/api/external-drives/**", "GET", ""),
        RoutePermission::new("/api/external-drives/**", "POST", ""),

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
        // #1695 — cross-app collaboration: threaded comments, reactions,
        // presence, permissions, versions and links are shared by docs,
        // sheets, drive and the calendar. Missing from the catalogue, so
        // default-deny rejected every collaborative call with
        // "No matching route permission found" before the handler ran and
        // the docs editor could never collaborate at all.
        RoutePermission::new("/api/collab/**", "GET", ""),
        RoutePermission::new("/api/collab/**", "POST", ""),
        RoutePermission::new("/api/collab/**", "PUT", ""),
        RoutePermission::new("/api/collab/**", "DELETE", ""),
        RoutePermission::new("/api/activity", "GET", ""),
        RoutePermission::new("/api/activity", "POST", ""),
        // Browser-action ledger (#1702): the read endpoint behind the
        // Settings → Ledger → Browser tab.
        RoutePermission::new("/api/browser/**", "GET", ""),
        // First-token latency summary (#1700).
        RoutePermission::new("/api/runs/ttft", "GET", ""),
        // Parallel sub-agent fan-out (#1694).
        RoutePermission::new("/api/runs/subagents", "GET", ""),
        RoutePermission::new("/api/runs/subagents", "POST", ""),
        // Pooled group budgets, usage reports and cap policies (#1696/#1697).
        RoutePermission::new("/api/groups/**", "GET", ""),
        RoutePermission::new("/api/groups/**", "POST", ""),
        RoutePermission::new("/api/groups/**", "PUT", ""),
        RoutePermission::new("/api/groups/**", "DELETE", ""),
        RoutePermission::new("/api/paper/**", "GET", ""),
        RoutePermission::new("/api/paper/**", "POST", ""),
        RoutePermission::new("/api/paper/**", "PUT", ""),
        RoutePermission::new("/api/paper/**", "DELETE", ""),

        // Sheet — personal workbooks. Never anonymous (#2127): every route
        // under /api/sheet is scoped by the verified user id, so the RBAC gate
        // must require authentication just like the auth middleware does.
        RoutePermission::new("/api/sheet/**", "GET", ""),
        RoutePermission::new("/api/sheet/**", "POST", ""),
        RoutePermission::new("/api/sheet/**", "PUT", ""),
        RoutePermission::new("/api/sheet/**", "DELETE", ""),

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
        // Connectors (#1703): without catalogue entries default-deny answered
        // every /api/connectors call with "No matching route permission found"
        // even for admins, leaving the feature the comparison page advertises
        // (grants ledger) unusable while it is compiled in.
        RoutePermission::new("/api/connectors/**", "GET", ""),
        RoutePermission::new("/api/connectors/**", "POST", ""),
        RoutePermission::new("/api/connectors/**", "PUT", ""),
        RoutePermission::new("/api/connectors/**", "DELETE", ""),

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

    ]
}
