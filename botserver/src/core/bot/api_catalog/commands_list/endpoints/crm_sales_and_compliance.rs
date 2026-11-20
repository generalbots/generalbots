use super::super::ApiEndpoint;

pub(super) static CRM_SALES_AND_COMPLIANCE: &[ApiEndpoint] = &[
        ApiEndpoint {
            method: "GET",
            path: "/api/bots/:bot_id/integration-connections/:connection_id/events",
            summary: "List the sanitized audit event trail of an integration connection",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/bots/:bot_id/integration-actions/invoke",
            summary: "Execute an implemented provider action (provider, action, params); credentials load from Vault and never return",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/apps/integrations/context",
            summary: "Resolve the default active bot id, name and integration-connections base URL for suite apps",
        },
        // Conversation autotasks (#1613): work that outlives the chat turn
        ApiEndpoint {
            method: "GET",
            path: "/api/conversations/:conversation_id/autotasks",
            summary: "List the cron/systemd jobs a conversation owns, with live status, schedule and next run",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/conversations/:conversation_id/autotasks",
            summary: "Create a conversation autotask (title, schedule, command, kind=cron|systemd); installs a cron entry or systemd unit in the agent workspace",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/conversation-autotasks/:id/cancel",
            summary: "Stop a conversation autotask: removes the cron entry or disables the systemd unit and marks the row stopped",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/conversation-autotasks",
            summary: "List all conversation cron/systemd jobs owned by the current user, across conversations",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/conversation-autotasks/:id",
            summary: "Delete a conversation autotask: removes the workspace entry and deletes the row",
        },
        // File version history (GB7-14)
        ApiEndpoint {
            method: "POST",
            path: "/api/files/versions/list",
            summary: "List stored revisions of a Drive file (path, bucket, scope); every overwrite keeps the previous revision",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/files/versions/restore",
            summary: "Restore a Drive file revision by version key; the current content is snapshotted first",
        },
        // Computer use (GB7-19/GB7-20): proposal -> consent -> execution
        ApiEndpoint {
            method: "POST",
            path: "/api/computer/actions",
            summary: "Propose a browser or terminal action for a conversation (session_id, kind, action); the suite asks the user for consent",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/computer/actions/pending",
            summary: "Poll the actions awaiting consent for a session (session_id, kind=browser|terminal)",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/computer/actions/:id",
            summary: "Read one computer-use action with the user's decision and its result",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/terminal/exec",
            summary: "Execute an approved terminal command inside the user's workspace container; requires approved=true and refuses destructive patterns",
        },
        // Learning / LMS
        ApiEndpoint {
            method: "GET",
            path: "/api/learn/courses",
            summary: "List published courses with filters (category, difficulty, search)",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/learn/courses",
            summary: "Create a course draft",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/learn/courses/:id/enroll",
            summary: "Enroll the current user in a course",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/learn/progress",
            summary: "Current user learning progress and stats",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/learn/certifications",
            summary: "List certifications earned by the current user",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/learn/leaderboard",
            summary: "Leaderboard of learners by XP",
        },
        // Sales / CRM pipeline — #2540: one canonical prefix. The old
        // /api/sales/* aliases are 410 tombstones and are not advertised.
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/deals",
            summary: "List CRM deals with stage, value, probability and owner",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/crm/deals",
            summary: "Create a CRM deal (title, value, stage, expected close date)",
        },
        ApiEndpoint {
            method: "PUT",
            path: "/api/crm/deals/:id",
            summary: "Update a CRM deal (stage moves, value, probability, notes)",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/crm/deals/:id",
            summary: "Delete a CRM deal",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/contacts",
            summary: "List CRM contacts available for deals",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/activities",
            summary: "List recent CRM activities (calls, emails, meetings)",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/pipeline",
            summary: "Pipeline funnel summary grouped by stage with weighted value",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/stats",
            summary: "Weighted sales forecast and funnel stats from CRM deals",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/quotes",
            summary: "List CRM quotes (former /api/sales/quotes)",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/orders",
            summary: "List CRM orders (former /api/sales/orders)",
        },
        // Marketing campaigns, lists and templates
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/campaigns",
            summary: "List marketing campaigns across channels",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/crm/campaigns",
            summary: "Create a marketing campaign (name, channel, content template)",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/crm/campaigns/:id/send",
            summary: "Send a campaign to its recipient list",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/lists",
            summary: "List audience lists with member counts",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/crm/lists",
            summary: "Create an audience list (name, type, segment query)",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/crm/lists/:id",
            summary: "Delete an audience list",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/crm/templates",
            summary: "List channel templates (email, WhatsApp, social, SMS)",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/crm/templates",
            summary: "Create a channel template with variables",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/crm/templates/:id",
            summary: "Delete a channel template",
        },
        // VDI / remote desktop
        ApiEndpoint {
            method: "GET",
            path: "/api/desktop/connections",
            summary: "List active and saved remote desktop (VNC/RDP) connections for the user",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/desktop/connect",
            summary: "Register a remote desktop connection (VNC or RDP, credentials vaulted)",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/desktop/connections/kill",
            summary: "Terminate all active desktop sessions for the current user",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/desktop/connections/:id",
            summary: "Get a single desktop connection by id",
        },
        ApiEndpoint {
            method: "DELETE",
            path: "/api/desktop/connections/:id",
            summary: "Delete a desktop connection and terminate its session",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/desktop/health/rdp",
            summary: "Probe an RDP endpoint (host, port) and confirm an RDP server is listening",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/desktop/health/tcp",
            summary: "Probe TCP reachability of a host and port",
        },
        // Compliance
        ApiEndpoint {
            method: "GET",
            path: "/api/compliance/frameworks",
            summary: "List compliance frameworks with control counts",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/compliance/frameworks",
            summary: "Create a compliance framework (LGPD, GDPR, SOC 2, ISO 27001, PCI-DSS, custom)",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/compliance/frameworks/:framework_id",
            summary: "Framework detail with controls, evidence and coverage",
        },
        ApiEndpoint {
            method: "PUT",
            path: "/api/compliance/frameworks/:framework_id",
            summary: "Update a compliance framework",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/compliance/frameworks/:framework_id/archive",
            summary: "Archive a compliance framework",
        },
        ApiEndpoint {
            method: "GET",
            path: "/api/compliance/frameworks/:framework_id/export.csv",
            summary: "Audit-ready compliance scorecard CSV export",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/compliance/controls",
            summary: "Add a control to a compliance framework",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/compliance/evidence/attach",
            summary: "Attach drive evidence to a control",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/compliance/evidence/:evidence_id/approve",
            summary: "Approve attached evidence",
        },
        // Search / research
        ApiEndpoint {
            method: "GET",
            path: "/api/ui/search",
            summary: "Unified search across business entities",
        },
        ApiEndpoint {
            method: "POST",
            path: "/api/ui/research/web/search",
            summary: "Web search (DuckDuckGo)",
        },
];
