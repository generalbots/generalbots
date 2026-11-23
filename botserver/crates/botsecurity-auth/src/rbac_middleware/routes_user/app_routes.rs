//! Authenticated application routes: chat, drive, editors, docs and projects.

use super::super::RoutePermission;

pub(super) fn routes() -> Vec<RoutePermission> {
    vec![
        // Chat history: authenticated users manage their own conversations.
        // PUT was missing, so renaming a conversation was rejected by the
        // default-deny catalogue with 403 before reaching the handler.
        // #2562 — PATCH added: the assistant sidebar sends PATCH for rename.
        RoutePermission::new("/api/chat/history/**", "GET", ""),
        RoutePermission::new("/api/chat/history/**", "PUT", ""),
        RoutePermission::new("/api/chat/history/**", "PATCH", ""),
        RoutePermission::new("/api/chat/history/**", "DELETE", ""),
        // #1682 — lock/unlock a conversation behind a PIN.
        RoutePermission::new("/api/chat/history/**", "POST", ""),

        // i18n preference: the POST was in the public routes (GET only), so
        // setting a locale returned 403 for authenticated users (#2562).
        RoutePermission::new("/api/i18n/preference", "POST", ""),

        // Appearance: read and change the accent colour after setup.
        RoutePermission::new("/api/appearance", "GET", ""),
        RoutePermission::new("/api/appearance", "PUT", ""),

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
        // Truthful preview status of a published sandbox resource. The handler
        // (`core/bot/sandbox/routes.rs` -> `/api/sandbox/status/:hash`) is
        // mounted behind auth, but its path was absent from the default-deny
        // catalogue, so authenticated users got 403 before reaching it.
        // Read-only and scoped to a caller-known resource id: never anonymous.
        RoutePermission::new("/api/sandbox/status/**", "GET", ""),
        // Tool manifests for the native surface tools (#1678)
        RoutePermission::new("/api/runs/tools", "GET", ""),

        // Usage-ranked command/app statistics (#2372). Mounted by apps::installs::register
        // but were absent from the default-deny catalogue, so even admins got
        // 403 "No matching route permission found" before reaching the handler.
        RoutePermission::new("/api/commands/stats", "GET", ""),
        RoutePermission::new("/api/apps/install/stats", "GET", ""),
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
        // #2501 — approvals workflow (botconsent/src/approvals/api.rs): the
        // router is mounted behind auth, but its prefix was absent from the
        // default-deny catalogue, so every dashboard/list/decide/sign call
        // returned 403 "No matching route permission found" before reaching
        // the handler. GET reads the queue; mutations use POST/PUT/DELETE.
        // Keep allow_anonymous=false: approvals are per-user, never public.
        RoutePermission::new("/api/approvals/**", "GET", ""),
        RoutePermission::new("/api/approvals/**", "POST", ""),
        RoutePermission::new("/api/approvals/**", "PUT", ""),
        RoutePermission::new("/api/approvals/**", "DELETE", ""),
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

        // HTMX app fragments. /suite/docs/fragments/*, /suite/slides/fragments/*
        // and friends are rendered by botserver behind the botui proxy, but no
        // /suite prefix existed in the catalogue — default-deny answered
        // "No matching route permission found" and the Docs app could not even
        // list its own documents (#2159).
        RoutePermission::new("/suite/**", "GET", ""),
        RoutePermission::new("/suite/**", "POST", ""),

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
        // #2538 — site builder (WebA) routes. `/render/**` and `/forms/**` are
        // public-facing; the builder routes are authenticated.
        RoutePermission::new("/apps", "GET", ""),
        RoutePermission::new("/apps", "POST", ""),
        RoutePermission::new("/apps/**", "GET", ""),
        RoutePermission::new("/apps/**", "POST", ""),
        RoutePermission::new("/apps/**", "PUT", ""),
        RoutePermission::new("/apps/**", "DELETE", ""),
        RoutePermission::new("/render/**", "GET", "").with_anonymous(true),
        RoutePermission::new("/forms/**", "POST", "").with_anonymous(true),
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

        // Office event stream (#2503). The JSON surface (DLQ summary, replay
        // and metadata-only publish) is mounted behind require_auth in
        // feature_merge_platform/commerce.rs. Without catalogue rows the
        // default-deny gate rejected every call with "No matching route
        // permission found" before the handler ran. The /ws/events sockets are
        // deliberately absent here: they are declared as handler-token
        // capabilities in auth_api/public_routes.rs and are already admitted
        // anonymously by routes_public.rs's /ws/** entry.
        RoutePermission::new("/api/events/**", "GET", ""),
        RoutePermission::new("/api/events/**", "POST", ""),

    ]
}

#[cfg(test)]
mod approvals_tests {
    use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
    use crate::{AuthenticatedUser, Role};
    use uuid::Uuid;

    /// #2501 — the approvals prefix must carry every verb in the authenticated
    /// catalogue with anonymous access disabled; otherwise default-deny 403s
    /// the botconsent approvals router before its handler runs.
    #[test]
    fn approvals_prefix_is_in_authenticated_catalogue_with_all_verbs() {
        let routes = super::routes();
        for method in ["GET", "POST", "PUT", "DELETE"] {
            assert!(
                routes.iter().any(|r| {
                    r.path_pattern == "/api/approvals/**"
                        && r.method == method
                        && !r.allow_anonymous
                }),
                "/api/approvals/** {method} must be in the authenticated catalogue with anonymous disabled"
            );
        }
    }

    #[tokio::test]
    async fn approvals_routes_admit_authenticated_and_deny_anonymous() {
        let manager = RbacManager::with_defaults();
        manager
            .register_routes(build_default_route_permissions())
            .await;
        let user = AuthenticatedUser::new(Uuid::new_v4(), "approver@example.com".to_string())
            .with_role(Role::User);
        let anonymous = AuthenticatedUser::anonymous();

        // Covers all 12 route/method pairs mounted by botconsent's approvals
        // api.rs router, plus the catalogue-only PUT/DELETE verbs.
        let cases = [
            ("GET", "/api/approvals"),
            ("POST", "/api/approvals"),
            ("POST", "/api/approvals/sla-sweep"),
            ("GET", "/api/approvals/a1"),
            ("POST", "/api/approvals/a1/route"),
            ("POST", "/api/approvals/a1/delegate"),
            ("POST", "/api/approvals/a1/decide"),
            ("POST", "/api/approvals/a1/sign"),
            ("POST", "/api/approvals/a1/update-content"),
            ("POST", "/api/approvals/a1/check-sla"),
            ("POST", "/api/approvals/a1/publish"),
            ("GET", "/api/approvals/a1/verify"),
            ("PUT", "/api/approvals/a1/decide"),
            ("DELETE", "/api/approvals/a1"),
        ];

        for (method, path) in cases {
            let allowed = manager.check_route_access(path, method, &user).await;
            assert!(
                allowed.is_allowed(),
                "{method} {path} must be allowed for an authenticated user: {}",
                allowed.reason
            );
            let denied = manager.check_route_access(path, method, &anonymous).await;
            assert!(
                !denied.is_allowed(),
                "{method} {path} must be denied for an anonymous caller: {}",
                denied.reason
            );
        }
    }
}

#[cfg(test)]
mod sandbox_status_tests {
    use crate::rbac_middleware::{build_default_route_permissions, RbacManager};
    use crate::{AuthenticatedUser, Role};
    use uuid::Uuid;

    /// Sandbox status is a read-only status of a published resource: it must be
    /// in the authenticated catalogue with anonymous access disabled, or the
    /// default-deny gate 403s the handler before it can report the truth.
    #[test]
    fn sandbox_status_prefix_is_in_authenticated_catalogue_without_anonymous() {
        let routes = super::routes();
        assert!(
            routes.iter().any(|r| {
                r.path_pattern == "/api/sandbox/status/**"
                    && r.method == "GET"
                    && !r.allow_anonymous
            }),
            "/api/sandbox/status/** GET must be in the authenticated catalogue with anonymous disabled"
        );
    }

    /// The mounted route is `/api/sandbox/status/:hash`, so a real hash path
    /// must be admitted for an authenticated user and denied for an anonymous
    /// caller through the full default-deny catalogue.
    #[tokio::test]
    async fn sandbox_status_admits_authenticated_and_denies_anonymous() {
        let manager = RbacManager::with_defaults();
        manager
            .register_routes(build_default_route_permissions())
            .await;
        let user = AuthenticatedUser::new(Uuid::new_v4(), "viewer@example.com".to_string())
            .with_role(Role::User);
        let anonymous = AuthenticatedUser::anonymous();
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let path = format!("/api/sandbox/status/{hash}");

        let allowed = manager.check_route_access(&path, "GET", &user).await;
        assert!(
            allowed.is_allowed(),
            "GET {path} must be allowed for an authenticated user: {}",
            allowed.reason
        );
        let denied = manager.check_route_access(&path, "GET", &anonymous).await;
        assert!(
            !denied.is_allowed(),
            "GET {path} must be denied for an anonymous caller: {}",
            denied.reason
        );
    }
}
