//! LLM-facing api command catalog (unified UI/API surface for chat).
//!
//! On web the LLM drives the UI via `__ui_plan__`; on chat/WhatsApp the
//! backend REST surface is otherwise unreachable. This module exposes:
//!
//! - a compact set of **executable commands** (whitelisted, run in-process
//!   against backend state) injected into every channel's system prompt;
//! - the full **API endpoint catalog** (method + path + one-line summary)
//!   searched on demand through `api.find` — the LLM learns the whole
//!   surface without a giant system prompt;
//! - `apps.find` (suite app registry search) and `web.search` (DuckDuckGo),
//!   so the model can decide between UI automation, backend data and the web.
//!
//! The LLM invokes a command by emitting a single
//! `{"__api_call__": {"name": ..., "params": {...}, "compose": true}}`
//! JSON block as the first line of its reply. General-knowledge questions
//! must be answered directly — no api call.

use std::sync::Arc;

use base64::Engine;
use bigdecimal::BigDecimal;
use botcore::shared::state::AppState;
use diesel::RunQueryDsl;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::apps::registry;

pub const API_CALL_TRIGGER: &str = "\"__api_call__\":";

/// Convenience detection for the API-call trigger. Some models emit the JSON
/// key as `__api_call__` (canonical) or `api_call` (common off-by-underscore
/// variant) or `_api_call_`. Tolerate all so the trigger is never missed and
/// never rendered to the user as raw JSON.
pub fn is_api_call_trigger(hay: &str) -> bool {
    ["\"__api_call__\":", "\"api_call\":", "\"_api_call_\":", "\"api-call\":"]
        .iter()
        .any(|t| hay.contains(t))
}

/// Position of the first API-call trigger variant in `hay`, if any.
pub fn find_api_call(hay: &str) -> Option<usize> {
    ["\"__api_call__\":", "\"api_call\":", "\"_api_call_\":", "\"api-call\":"]
        .iter()
        .filter_map(|t| hay.find(t))
        .min()
}

/// The declarative command model lives in `crate::apps::commands`. We reuse
/// that single source of truth here so the chat/WhatsApp prompt, the palette
/// and the discovery commands all agree on the same per-app actions.
pub use crate::apps::commands::{command_by_name, AppCommand};

pub struct ApiEndpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
}

/// The stable command list rendered into the prompt and used by discovery.
/// This now comes from the single source of truth in `crate::apps::commands`.
pub fn commands_list() -> Vec<&'static AppCommand> {
    crate::apps::commands::all_commands().iter().collect()
}

/// Full REST surface (method, path, one-line summary). Not injected — only
/// searched on demand through `api.find`, so the LLM knows the entire scale
/// without a giant system prompt. Keep entries grouped by domain.
pub fn all_endpoints() -> &'static [ApiEndpoint] {
    &[
        // Drive / files
        ApiEndpoint { method: "GET", path: "/api/files/list", summary: "List drive files for the bot" },
        ApiEndpoint { method: "POST", path: "/api/files/write", summary: "Upload a file (base64 content) into a folder" },
        ApiEndpoint { method: "POST", path: "/api/files/download", summary: "Download a file from a folder" },
        ApiEndpoint { method: "DELETE", path: "/api/files/delete", summary: "Delete a file" },
        ApiEndpoint { method: "POST", path: "/api/files/createFolder", summary: "Create a drive folder" },
        ApiEndpoint { method: "POST", path: "/api/files/copy", summary: "Copy a file" },
        ApiEndpoint { method: "POST", path: "/api/files/move", summary: "Move a file" },
        ApiEndpoint { method: "GET", path: "/api/files/search", summary: "Search drive files by name" },
        ApiEndpoint { method: "GET", path: "/api/files/recent", summary: "Most recent files" },
        ApiEndpoint { method: "GET", path: "/api/files/quota", summary: "Storage quota usage" },
        ApiEndpoint { method: "POST", path: "/api/files/upload-binary", summary: "Upload a raw binary file" },
        ApiEndpoint { method: "GET", path: "/api/files/buckets", summary: "List bot buckets" },
        ApiEndpoint { method: "GET", path: "/api/files/shared", summary: "List shared folders" },
        ApiEndpoint { method: "POST", path: "/api/files/share", summary: "Share a folder" },
        ApiEndpoint { method: "GET", path: "/api/files/trash", summary: "List trashed files" },
        ApiEndpoint { method: "POST", path: "/api/files/trash/restore", summary: "Restore a trashed file" },
        // Products / fiscal
        ApiEndpoint { method: "GET", path: "/api/products/items", summary: "List product catalog items" },
        ApiEndpoint { method: "POST", path: "/api/products/items", summary: "Create a product item" },
        ApiEndpoint { method: "GET", path: "/api/products/services", summary: "List registered services" },
        ApiEndpoint { method: "POST", path: "/api/products/services/:id/tax", summary: "Compute Brazilian taxes for a service" },
        ApiEndpoint { method: "GET", path: "/api/products/categories", summary: "List product categories" },
        ApiEndpoint { method: "GET", path: "/api/products/pricelists", summary: "List price lists" },
        ApiEndpoint { method: "GET", path: "/api/products/stats", summary: "Product statistics" },
        ApiEndpoint { method: "GET", path: "/api/products/low-stock", summary: "Low-stock items" },
        // Banking / cash flow
        ApiEndpoint { method: "GET", path: "/api/banking/transactions", summary: "List bank transactions" },
        ApiEndpoint { method: "POST", path: "/api/banking/transactions", summary: "Create a bank transaction" },
        ApiEndpoint { method: "GET", path: "/api/banking/platforms", summary: "List delivery platforms" },
        ApiEndpoint { method: "POST", path: "/api/banking/reconcile", summary: "Run reconciliation" },
        ApiEndpoint { method: "GET", path: "/api/banking/reports", summary: "Banking reports" },
        ApiEndpoint { method: "POST", path: "/api/banking/reconcile/match", summary: "Manual reconciliation match" },
        ApiEndpoint { method: "POST", path: "/api/banking/imports/cashflow", summary: "Import a monthly cash-flow CSV" },
        ApiEndpoint { method: "GET", path: "/api/banking/diagnosis", summary: "Cash-flow diagnosis of the branch" },
        // Billing / payroll
        ApiEndpoint { method: "GET", path: "/api/billing/payroll/months", summary: "Monthly invoice totals as a payroll basis" },
        // Settings: API keys & webhooks
        ApiEndpoint { method: "GET", path: "/api/user/api-keys", summary: "List API keys with scopes and last-used time" },
        ApiEndpoint { method: "POST", path: "/api/user/api-keys", summary: "Create an API key (name, scopes, expiry days)" },
        ApiEndpoint { method: "DELETE", path: "/api/user/api-keys/:key_id", summary: "Revoke an API key" },
        ApiEndpoint { method: "POST", path: "/api/user/api-keys/:key_id/rotate", summary: "Rotate an API key secret" },
        ApiEndpoint { method: "GET", path: "/api/user/webhooks", summary: "List webhook endpoints" },
        ApiEndpoint { method: "POST", path: "/api/user/webhooks", summary: "Register a webhook endpoint (URL, events)" },
        ApiEndpoint { method: "DELETE", path: "/api/user/webhooks/:webhook_id", summary: "Delete a webhook endpoint" },
        ApiEndpoint { method: "POST", path: "/api/user/webhooks/:webhook_id/test", summary: "Send a signed test event to a webhook" },
        ApiEndpoint { method: "GET", path: "/api/user/webhooks/:webhook_id/deliveries", summary: "List webhook delivery history" },
        // Sources / connectors
        ApiEndpoint { method: "GET", path: "/api/integrations/connectors", summary: "List data connectors with type, status and last sync" },
        ApiEndpoint { method: "POST", path: "/api/integrations/connectors", summary: "Create or update a data connector (database, API, SaaS)" },
        ApiEndpoint { method: "POST", path: "/api/integrations/connectors/:id/test", summary: "Test connectivity for a data connector" },
        ApiEndpoint { method: "POST", path: "/api/integrations/connectors/:id/sync", summary: "Trigger an immediate sync for a data connector" },
        ApiEndpoint { method: "DELETE", path: "/api/integrations/connectors/:id/disconnect", summary: "Remove a data connector and its vaulted credentials" },
        ApiEndpoint { method: "GET", path: "/api/integrations/connectors/templates", summary: "Connector type catalog for typed configuration forms" },
        // Integrations: tenant-scoped connection control plane (#939)
        ApiEndpoint { method: "GET", path: "/api/bots/:bot_id/integration-connections", summary: "List integration connections for a bot; credentials stay in Vault" },
        ApiEndpoint { method: "POST", path: "/api/bots/:bot_id/integration-connections", summary: "Create an integration connection (provider_slug, display_name, auth_kind); secret keys are stored in Vault only" },
        ApiEndpoint { method: "GET", path: "/api/bots/:bot_id/integration-connections/:connection_id", summary: "Get one integration connection; never returns vault path or credential material" },
        ApiEndpoint { method: "DELETE", path: "/api/bots/:bot_id/integration-connections/:connection_id", summary: "Delete an integration connection and purge its Vault credentials" },
        ApiEndpoint { method: "POST", path: "/api/bots/:bot_id/integration-connections/:connection_id/test", summary: "Validate stored credentials shape (no outbound call); outcome is recorded as unverified" },
        ApiEndpoint { method: "POST", path: "/api/bots/:bot_id/integration-connections/:connection_id/rotate", summary: "Rotate connection credentials in Vault and increment credential_version" },
        ApiEndpoint { method: "GET", path: "/api/bots/:bot_id/integration-connections/:connection_id/events", summary: "List the sanitized audit event trail of an integration connection" },
        ApiEndpoint { method: "POST", path: "/api/bots/:bot_id/integration-actions/invoke", summary: "Execute an implemented provider action (provider, action, params); credentials load from Vault and never return" },

        // Learning / LMS
        ApiEndpoint { method: "GET", path: "/api/learn/courses", summary: "List published courses with filters (category, difficulty, search)" },
        ApiEndpoint { method: "POST", path: "/api/learn/courses", summary: "Create a course draft" },
        ApiEndpoint { method: "POST", path: "/api/learn/courses/:id/enroll", summary: "Enroll the current user in a course" },
        ApiEndpoint { method: "GET", path: "/api/learn/progress", summary: "Current user learning progress and stats" },
        ApiEndpoint { method: "GET", path: "/api/learn/certifications", summary: "List certifications earned by the current user" },
        ApiEndpoint { method: "GET", path: "/api/learn/leaderboard", summary: "Leaderboard of learners by XP" },
        // Sales / CRM pipeline
        ApiEndpoint { method: "GET", path: "/api/sales/deals", summary: "List sales deals with stage, value, probability and owner" },
        ApiEndpoint { method: "POST", path: "/api/sales/deals", summary: "Create a sales deal (title, value, stage, expected close date)" },
        ApiEndpoint { method: "PATCH", path: "/api/sales/deals/:id", summary: "Update a deal (stage moves, value, probability, notes)" },
        ApiEndpoint { method: "DELETE", path: "/api/sales/deals/:id", summary: "Delete a deal" },
        ApiEndpoint { method: "GET", path: "/api/sales/contacts", summary: "List CRM contacts available for deals" },
        ApiEndpoint { method: "GET", path: "/api/sales/activities", summary: "List recent sales activities (calls, emails, meetings)" },
        ApiEndpoint { method: "GET", path: "/api/sales/funnel", summary: "Pipeline funnel summary grouped by stage with weighted value" },
        ApiEndpoint { method: "GET", path: "/api/sales/forecast", summary: "Weighted sales forecast from stage probabilities" },
        // Marketing campaigns, lists and templates
        ApiEndpoint { method: "GET", path: "/api/crm/campaigns", summary: "List marketing campaigns across channels" },
        ApiEndpoint { method: "POST", path: "/api/crm/campaigns", summary: "Create a marketing campaign (name, channel, content template)" },
        ApiEndpoint { method: "POST", path: "/api/crm/campaigns/:id/send", summary: "Send a campaign to its recipient list" },
        ApiEndpoint { method: "GET", path: "/api/crm/lists", summary: "List audience lists with member counts" },
        ApiEndpoint { method: "POST", path: "/api/crm/lists", summary: "Create an audience list (name, type, segment query)" },
        ApiEndpoint { method: "DELETE", path: "/api/crm/lists/:id", summary: "Delete an audience list" },
        ApiEndpoint { method: "GET", path: "/api/crm/templates", summary: "List channel templates (email, WhatsApp, social, SMS)" },
        ApiEndpoint { method: "POST", path: "/api/crm/templates", summary: "Create a channel template with variables" },
        ApiEndpoint { method: "DELETE", path: "/api/crm/templates/:id", summary: "Delete a channel template" },
        // VDI / remote desktop
        ApiEndpoint { method: "GET", path: "/api/desktop/connections", summary: "List active and saved remote desktop (VNC/RDP) connections for the user" },
        ApiEndpoint { method: "POST", path: "/api/desktop/connect", summary: "Register a remote desktop connection (VNC or RDP, credentials vaulted)" },
        ApiEndpoint { method: "POST", path: "/api/desktop/connections/kill", summary: "Terminate all active desktop sessions for the current user" },
        ApiEndpoint { method: "GET", path: "/api/desktop/connections/:id", summary: "Get a single desktop connection by id" },
        ApiEndpoint { method: "DELETE", path: "/api/desktop/connections/:id", summary: "Delete a desktop connection and terminate its session" },
        ApiEndpoint { method: "POST", path: "/api/desktop/health/rdp", summary: "Probe an RDP endpoint (host, port) and confirm an RDP server is listening" },
        ApiEndpoint { method: "POST", path: "/api/desktop/health/tcp", summary: "Probe TCP reachability of a host and port" },
        // Compliance
        ApiEndpoint { method: "GET", path: "/api/compliance/frameworks", summary: "List compliance frameworks with control counts" },
        ApiEndpoint { method: "POST", path: "/api/compliance/frameworks", summary: "Create a compliance framework (LGPD, GDPR, SOC 2, ISO 27001, PCI-DSS, custom)" },
        ApiEndpoint { method: "GET", path: "/api/compliance/frameworks/:framework_id", summary: "Framework detail with controls, evidence and coverage" },
        ApiEndpoint { method: "PUT", path: "/api/compliance/frameworks/:framework_id", summary: "Update a compliance framework" },
        ApiEndpoint { method: "POST", path: "/api/compliance/frameworks/:framework_id/archive", summary: "Archive a compliance framework" },
        ApiEndpoint { method: "GET", path: "/api/compliance/frameworks/:framework_id/export.csv", summary: "Audit-ready compliance scorecard CSV export" },
        ApiEndpoint { method: "POST", path: "/api/compliance/controls", summary: "Add a control to a compliance framework" },
        ApiEndpoint { method: "POST", path: "/api/compliance/evidence/attach", summary: "Attach drive evidence to a control" },
        ApiEndpoint { method: "POST", path: "/api/compliance/evidence/:evidence_id/approve", summary: "Approve attached evidence" },
        // Search / research
        ApiEndpoint { method: "GET", path: "/api/ui/search", summary: "Unified search across business entities" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/search", summary: "Web search (DuckDuckGo)" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/summarize", summary: "Summarize web search results" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/deep", summary: "Deep research on a topic" },
        ApiEndpoint { method: "GET", path: "/api/ui/research/web/instant", summary: "Instant answer from the web" },
        // Catalog / apps
        ApiEndpoint { method: "GET", path: "/api/apps/catalog", summary: "Suite application catalog" },
        ApiEndpoint { method: "GET", path: "/api/apps/integrations/catalog", summary: "Search the integration provider and action catalog" },
        ApiEndpoint { method: "GET", path: "/api/apps/integrations/catalog/:provider", summary: "Get one integration provider and its action catalog" },
        ApiEndpoint { method: "GET", path: "/api/catalog/products", summary: "Public product catalog" },
        ApiEndpoint { method: "GET", path: "/api/catalog/plans", summary: "Public plans catalog" },
        ApiEndpoint { method: "GET", path: "/api/catalog/prices.json", summary: "Price list (JSON-LD)" },
        // Organization / cloud
        ApiEndpoint { method: "GET", path: "/api/organizations/current", summary: "Current organization profile" },
        ApiEndpoint { method: "PUT", path: "/api/organizations/current", summary: "Update organization profile" },
        ApiEndpoint { method: "GET", path: "/api/organizations/current/stats", summary: "Organization statistics" },
        ApiEndpoint { method: "POST", path: "/api/cloud/auth/signup", summary: "Create a cloud workspace account" },
        ApiEndpoint { method: "POST", path: "/api/cloud/auth/login", summary: "Log into the cloud" },
        ApiEndpoint { method: "GET", path: "/api/cloud/domains", summary: "List bot domain mappings" },
        ApiEndpoint { method: "POST", path: "/api/cloud/domains", summary: "Map a domain to a bot" },
        ApiEndpoint { method: "GET", path: "/api/domains/resolve", summary: "Resolve a hostname to a bot" },
        ApiEndpoint { method: "POST", path: "/api/autotask/classify", summary: "Classify a plain-language intent into an automation plan" },
        ApiEndpoint { method: "POST", path: "/api/autotask/compile", summary: "Compile an automation plan into a BASIC script" },
        ApiEndpoint { method: "POST", path: "/api/autotask/decide", summary: "Decide immediate vs scheduled execution mode" },
        ApiEndpoint { method: "POST", path: "/api/autotask/execute", summary: "Execute an automation plan (compiled BASIC script)" },
        ApiEndpoint { method: "POST", path: "/api/autotask/create-and-execute", summary: "Turn a plain-language request into an automated task and run it" },
        ApiEndpoint { method: "GET", path: "/api/autotask/tasks", summary: "List AutoTask runs and their status" },
        ApiEndpoint { method: "GET", path: "/api/autotask/stats", summary: "AutoTask statistics: runs, success rate, pending approvals" },
        ApiEndpoint { method: "POST", path: "/api/autotask/tasks/:task_id/approve", summary: "Approve a pending automated task" },
        ApiEndpoint { method: "POST", path: "/api/autotask/tasks/:task_id/cancel", summary: "Cancel a queued or running automated task" },
        // Attendant / human handoff
        ApiEndpoint { method: "POST", path: "/api/attendant/attachments", summary: "Upload an attachment to an attendant conversation" },
        ApiEndpoint { method: "GET", path: "/api/attendant/attachments/:id", summary: "Download an attendant conversation attachment" },
        // Meet / recording
        ApiEndpoint { method: "POST", path: "/api/meet/rooms/:id/recording/start", summary: "Start recording a meeting (host/co-host only)" },
        ApiEndpoint { method: "POST", path: "/api/meet/rooms/:id/recording/stop", summary: "Stop the active meeting recording" },
        ApiEndpoint { method: "GET", path: "/api/meet/rooms/:id/recordings", summary: "List recordings for a meeting room" },
        ApiEndpoint { method: "GET", path: "/api/meet/recordings/:recording_id/file", summary: "Stream or download a meeting recording file" },
        // Player / playlists
        ApiEndpoint { method: "GET", path: "/api/player/playlists", summary: "List media playlists for the current user" },
        ApiEndpoint { method: "POST", path: "/api/player/playlists", summary: "Create a media playlist" },
        ApiEndpoint { method: "GET", path: "/api/player/playlists/:id", summary: "Get a playlist with its items" },
        ApiEndpoint { method: "PUT", path: "/api/player/playlists/:id", summary: "Rename a media playlist" },
        ApiEndpoint { method: "DELETE", path: "/api/player/playlists/:id", summary: "Delete a media playlist" },
        ApiEndpoint { method: "POST", path: "/api/player/playlists/:id/items", summary: "Add a media item to a playlist" },
        ApiEndpoint { method: "DELETE", path: "/api/player/playlists/:id/items/:item_id", summary: "Remove a media item from a playlist" },
        ApiEndpoint { method: "PUT", path: "/api/player/playlists/:id/reorder", summary: "Reorder playlist items" },
        ApiEndpoint { method: "GET", path: "/api/player/playlists/:id/analytics", summary: "Playlist analytics: counts, total items, recent activity" },
        ApiEndpoint { method: "POST", path: "/api/player/playbacks", summary: "Record a media playback event (play, pause, complete)" },
        // Dashboards / analytics
        ApiEndpoint { method: "GET", path: "/api/dashboards", summary: "List dashboards with their widgets" },
        ApiEndpoint { method: "POST", path: "/api/dashboards", summary: "Create a dashboard" },
        ApiEndpoint { method: "GET", path: "/api/dashboards/:id", summary: "Get a dashboard with its widgets and filters" },
        ApiEndpoint { method: "PUT", path: "/api/dashboards/:id", summary: "Rename or reconfigure a dashboard" },
        ApiEndpoint { method: "DELETE", path: "/api/dashboards/:id", summary: "Delete a dashboard" },
        ApiEndpoint { method: "POST", path: "/api/dashboards/:id/widgets", summary: "Add a widget to a dashboard" },
        ApiEndpoint { method: "PUT", path: "/api/dashboards/:id/widgets/:widget_id", summary: "Update a widget title, size or chart configuration" },
        ApiEndpoint { method: "DELETE", path: "/api/dashboards/:id/widgets/:widget_id", summary: "Remove a widget from a dashboard" },
        ApiEndpoint { method: "GET", path: "/api/dashboards/sources", summary: "List dashboard data sources" },
    ]
}

/// Compact system-prompt fragment teaching the `__api_call__` contract and
/// the routing rule, shortened to the current user's role so admin-only
/// commands are never proposed to regular users.
/// Compact system-prompt fragment teaching the `__api_call__` contract and the
/// discovery round-trip. The injected surface is intentionally tiny — instead
/// of dumping every command (there are now 1000+ harvested endpoints), the LLM
/// is taught to DISCOVER the exact command/endpoint with `api.find`, then EXECUTE
/// it with `api.exec` (or a named curated command). This keeps the prompt small
/// while unlocking the full action surface on every channel.
pub fn api_command_instructions(role: &str) -> String {
    let mut lines = Vec::with_capacity(24);
    lines.push("---".to_string());
    lines.push("## API Commands".to_string());
    lines.push(
        "You can call backend commands to read or change business data (fiscal, banking, drive files) \
         or search the web. Answer general-knowledge questions (trivia, definitions, opinions) DIRECTLY \
         with no command."
            .to_string(),
    );
    lines.push(format!("Your role: {role}. Only issue commands you are allowed to use."));
    lines.push("To call one, make the FIRST line of your reply exactly a JSON object:".to_string());
    lines.push("{\"__api_call__\": {\"name\": \"<command>\", \"params\": {<named params>}, \"compose\": true}}".to_string());
    lines.push("then add a short user-facing note after it.".to_string());

    // Discovery contract — never list the full surface in the prompt.
    lines.push("Discovery (2-step, prefer this over guessing a command):".to_string());
    lines.push("  1. `api.find` with params {query}: returns the matching curated command OR a harvested endpoint (kind, name/method/path, summary).".to_string());
    lines.push("  2. Run it: `api.exec` with params {method, path, params} executes any registered endpoint; or use the curated command name when returned.".to_string());

    // Only the most common commands are named explicitly to avoid prompt bloat.
    lines.push("Common commands you may call directly:".to_string());
    for name in ["apps.find", "api.find", "api.exec", "integrations.actions.list", "service.tax", "banking.diagnosis", "drive.list"] {
        if let Some(cmd) = command_by_name(name) {
            let params = cmd
                .params
                .iter()
                .map(|(n, d)| format!("{n}: {d}"))
                .collect::<Vec<_>>()
                .join("; ");
            lines.push(format!("- {name}: {summary} (params: {params})", summary = cmd.summary));
        }
    }
    lines.push(
        "Rules: use compose=true when the answer needs prose from the data. Always discover first with \
         api.find unless the command is obvious. Never mention JSON or commands to the user."
            .to_string(),
    );
    lines.join("\n")
}

/// Shared keyword scorer: how many query tokens appear in the haystack.
fn score_tokens(hay: &str, tokens: &[String]) -> i32 {
    tokens.iter().filter(|t| hay.contains(t.as_str())).count() as i32
}

fn tokens_of(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .map(|t| t.to_string())
        .collect()
}

/// App registry search used by the `apps.find` command (covers all apps).
fn find_apps(query: &str) -> Vec<Value> {
    let q = tokens_of(query);
    if q.is_empty() {
        return Vec::new();
    }
    let apps = registry::all_apps();
    let mut scored: Vec<(i32, &registry::AppDefinition)> = apps
        .iter()
        .filter_map(|app| {
            let hay = format!(
                "{} {} {} {}",
                app.title.to_lowercase(),
                app.id.to_lowercase(),
                app.description.to_lowercase(),
                app.keywords.to_lowercase()
            );
            let score = score_tokens(&hay, &q);
            if score > 0 {
                Some((score, app))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored
        .into_iter()
        .take(3)
        .map(|(_, app)| {
            json!({
                "id": app.id,
                "title": app.title,
                "description": app.description,
                "url": app.url,
            })
        })
        .collect()
}

/// Catalogued-command + endpoint search, filtered by the current user's role.
fn find_api_entries(state: &Arc<AppState>, user_id: Uuid, query: &str) -> Vec<Value> {
    let q = tokens_of(query);
    if q.is_empty() {
        return Vec::new();
    }
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    let mut out: Vec<Value> = Vec::new();
    let mut scored_cmds: Vec<(i32, &AppCommand)> = commands_list()
        .iter()
        .filter_map(|cmd| {
            if cmd.admin_only && role != crate::security::user_role::ROLE_ADMIN {
                return None;
            }
            let hay = format!("{} {}", cmd.name.to_lowercase(), cmd.summary.to_lowercase());
            let score = score_tokens(&hay, &q);
            if score > 0 {
                Some((score, *cmd))
            } else {
                None
            }
        })
        .collect();
    scored_cmds.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, cmd) in scored_cmds.into_iter().take(3) {
        out.push(json!({
            "kind": "command",
            "name": cmd.name,
            "summary": cmd.summary,
            "admin_only": cmd.admin_only,
            "params": cmd.params.iter().map(|(n, d)| json!({"name": n, "description": d})).collect::<Vec<_>>(),
        }));
    }
    // Harvested on-the-fly commands derived from the endpoint inventory. These
    // are proposed as `api.exec` candidates (method + path) for regular users.
    let derived = super::commands_derived::derived_commands();
    let mut scored_dc: Vec<(i32, &super::commands_derived::DerivedCommand)> = derived
        .iter()
        .filter(|c| {
            if role != crate::security::user_role::ROLE_ADMIN
                && crate::security::user_role::is_admin_only_endpoint(&state.conn, user_id, c.method, &c.path)
            {
                return false;
            }
            true
        })
        .filter_map(|c| {
            let hay = format!("{} {}", c.name.to_lowercase(), c.summary.to_lowercase());
            let score = score_tokens(&hay, &q);
            if score > 0 { Some((score, c)) } else { None }
        })
        .collect();
    scored_dc.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, c) in scored_dc.into_iter().take(6) {
        out.push(json!({
            "kind": "derived",
            "name": c.name,
            "summary": c.summary,
            "method": c.method,
            "path": c.path,
            "admin_only": crate::security::user_role::is_admin_only_endpoint(&state.conn, user_id, c.method, &c.path),
        }));
    }
    let mut scored_ep: Vec<(i32, &ApiEndpoint)> = all_endpoints()
        .iter()
        .chain(super::endpoint_inventory::ALL_ROUTES)
        .filter_map(|ep| {
            if crate::security::user_role::is_admin_only_endpoint(&state.conn, user_id, ep.method, ep.path) {
                return None;
            }
            let hay = format!("{} {} {}", ep.method.to_lowercase(), ep.path.to_lowercase(), ep.summary.to_lowercase());
            let score = score_tokens(&hay, &q);
            if score > 0 {
                Some((score, ep))
            } else {
                None
            }
        })
        .collect();
    // first occurrence wins (curated list precedes the auto inventory)
    scored_ep.sort_by(|a, b| b.0.cmp(&a.0));
    let mut seen = std::collections::HashSet::new();
    for (_, ep) in scored_ep.into_iter().take(6) {
        if !seen.insert((ep.method, ep.path)) {
            continue;
        }
        out.push(json!({
            "kind": "endpoint",
            "method": ep.method,
            "path": ep.path,
            "summary": ep.summary,
        }));
    }
    out
}

fn normalize_drive_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim().trim_start_matches('/').to_string();
    if trimmed.is_empty() || trimmed.contains("..") {
        return Err("invalid path".to_string());
    }
    Ok(trimmed)
}

/// Executes a catalog command in-process against the backend state. Enforces
/// the RBAC matrix: admin-only commands and admin-only endpoints are denied
/// to non-admin users.
pub async fn execute_command(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    bot_name: &str,
    user_id: Uuid,
    name: &str,
    params: &Value,
) -> Result<Value, String> {
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    if role != crate::security::user_role::ROLE_ADMIN {
        let denied = commands_list()
            .iter()
            .any(|c| c.name == name && c.admin_only);
        if denied {
            return Err(format!("command '{name}' requires the admin role"));
        }
    }
    let obj = match params {
        Value::Object(map) => map,
        _ => return Err("'params' must be a JSON object".to_string()),
    };
    let str_of = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string());
    match name {
        "service.tax" | "tax.calculate" => execute_tax(state, &bot_uuid, str_of("service"), str_of("value")).await,
        "banking.diagnosis" => {
            let period = str_of("period");
            let resp =
                botbanking::cashflow::cashflow_diagnosis_inner(&state.conn, &bot_uuid, None, period.as_deref())
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp).map_err(|e| e.to_string())?)
        }
        "banking.import" => {
            let file_key = str_of("file_key").ok_or_else(|| "params.file_key is required".to_string())?;
            let period = str_of("period");
            let resp = botbanking::cashflow::cashflow_import_inner(
                &state.conn,
                state.drive.as_ref(),
                &bot_uuid,
                None,
                Some(&file_key),
                None,
                period,
            )
            .await
            .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp).map_err(|e| e.to_string())?)
        }
        "drive.write" => {
            let path = str_of("path").ok_or_else(|| "params.path is required".to_string())?;
            let content = str_of("content_base64").ok_or_else(|| "params.content_base64 is required".to_string())?;
            write_drive_file(state, bot_name, &path, &content).await
        }
        "drive.file" => {
            let from = str_of("from").ok_or_else(|| "params.from is required".to_string())?;
            let to = str_of("to").ok_or_else(|| "params.to is required".to_string())?;
            move_drive_file(state, bot_name, &from, &to).await
        }
        "web.search" => {
            let query = str_of("query").ok_or_else(|| "params.query is required".to_string())?;
            let max: usize = obj
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(5)
                .min(25) as usize;
            let results = botresearch::web_search::search_web(&query, max, "wt-wt")
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "query": query,
                "results": results.iter().map(|r| json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                })).collect::<Vec<_>>(),
            }))
        }
        "apps.find" => Ok(json!({ "apps": find_apps(str_of("query").unwrap_or_default().as_str()) })),
        "api.find" => Ok(json!({ "matches": find_api_entries(state, user_id, str_of("query").unwrap_or_default().as_str()) })),
        "integrations.catalog.search" => {
            let providers = crate::apps::integration_catalog::llm_search(
                str_of("q").as_deref(),
                str_of("category").as_deref(),
                str_of("status").as_deref(),
            );
            Ok(json!({ "provider_count": providers.len(), "providers": providers }))
        }
        "integrations.actions.list" => {
            let provider = str_of("provider")
                .ok_or_else(|| "params.provider is required".to_string())?;
            let actions = crate::apps::integration_catalog::llm_actions(&provider)
                .ok_or_else(|| "integration provider not found".to_string())?;
            Ok(json!({
                "provider": provider,
                "action_count": actions.len(),
                "actions": actions,
            }))
        }
        "integrations.invoke" => {
            #[cfg(feature = "integrations")]
            {
                let provider = str_of("provider")
                    .ok_or_else(|| "params.provider is required".to_string())?;
                let action =
                    str_of("action").ok_or_else(|| "params.action is required".to_string())?;
                let mut action_params = serde_json::Map::new();
                if let Some(Value::Object(map)) = obj.get("params") {
                    action_params = map.clone();
                }
                integrations_invoke(
                    state,
                    bot_uuid,
                    user_id,
                    &provider,
                    &action,
                    &Value::Object(action_params),
                )
                .await
            }
            #[cfg(not(feature = "integrations"))]
            {
                Err("integration actions are not available in this build".to_string())
            }
        }
        "api.exec" => {
            let method = str_of("method").ok_or_else(|| "params.method is required".to_string())?;
            let path = str_of("path").ok_or_else(|| "params.path is required".to_string())?;
            let mut params = serde_json::Map::new();
            if let Some(Value::Object(map)) = obj.get("params") {
                params = map.clone();
            }
            crate::core::bot::api_exec::exec_endpoint(state, user_id, Some(bot_uuid), &method, &path, &params).await
        }
        "crm.people.list" | "people.list" => list_people(state, &bot_uuid, None).await,
        "crm.people.search" | "people.search" => {
            let query = str_of("query").unwrap_or_default();
            list_people(state, &bot_uuid, Some(&query)).await
        }
        "billing.invoice.list" => list_invoices(state, &bot_uuid).await,
        "products.items.list" => list_products(state, &bot_uuid, str_of("category").as_deref()).await,
        "tickets.list" => list_tickets(state, &bot_uuid).await,
        "drive.list" => list_drive_files(state, bot_name, str_of("path").as_deref()).await,
        "drive.archive" => {
            let source = str_of("source").unwrap_or_default();
            let dest = str_of("destination").ok_or_else(|| "params.destination is required".to_string())?;
            archive_drive_files(state, bot_name, &source, &dest).await
        }
        "payroll.diagnosis" => {
            let period = str_of("period");
            payroll_diagnosis(state, &bot_uuid, period.as_deref()).await
        }
        "monitoring.health" => Ok(json!({ "health": "ok", "note": "suite services running" })),
        _ => {
            // Unknown/mostly-navigation commands resolve to an in-app deep link
            // when the command declares one, otherwise a clear error.
            if let Some(cmd) = command_by_name(name) {
                if let Some(link) = cmd.deep_link {
                    Ok(json!({ "navigate": true, "app": cmd.app, "label": cmd.label, "deep_link": link }))
                } else {
                    Err(format!("command '{name}' is not executable from chat; open the {app} app instead",
                        app = if cmd.app.is_empty() { "suite" } else { cmd.app }))
                }
            } else {
                Err(format!("unknown command '{name}'"))
            }
        }
    }
}

fn branch_scope(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Uuid, String> {
    botbanking::cashflow::resolve_bot_scope(&state.conn, bot_uuid, None)
        .map(|s| s.branch_id)
        .ok_or_else(|| "bot not found".to_string())
}

/// Executes one registered provider action from the chat command path (#950).
///
/// The chat session is server-side and bot-bound: the tenant scope derives
/// exclusively from the `bots` row via `resolve_scope_parts` (see the
/// trade-off documented there), while the actor stays the authenticated
/// `user_id`. Credentials load strictly from Vault inside
/// `invoke_registered` and are stripped from every response.
#[cfg(feature = "integrations")]
async fn integrations_invoke(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    provider: &str,
    action: &str,
    params: &Value,
) -> Result<Value, String> {
    let scope = botintegrations::scope::resolve_scope_parts(&state.conn, user_id, bot_uuid)
        .map_err(|error| match error {
            botintegrations::error::IntegrationError::NotFound => {
                "integration provider requires an active bot".to_string()
            }
            other => format!("integration scope rejected: {other}"),
        })?;
    let secrets_manager = botcoresecrets::SecretsManager::get_clone()
        .map_err(|_| "credential store unavailable; request rejected".to_string())?;
    let integration_state = botintegrations::IntegrationState::new(
        state.conn.clone(),
        botintegrations::secrets::ConnectionVault::new(secrets_manager),
    );
    let outcome = botintegrations::providers::invoke_registered(
        &integration_state,
        &scope,
        provider,
        action,
        params,
    )
    .await?;
    Ok(json!({
        "provider": provider,
        "action": action,
        "summary": outcome.summary,
        "data": outcome.data,
        "truncated": outcome.truncated,
    }))
}

#[derive(diesel::QueryableByName)]
struct PersonRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    person_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    first_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    last_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    email: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
}

#[derive(diesel::QueryableByName)]
struct InvoiceRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    invoice_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    invoice_number: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    customer_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    total: BigDecimal,
}

#[derive(diesel::QueryableByName)]
struct ProductRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    product_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    sku: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    price: BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Text)]
    product_type: String,
    #[diesel(sql_type = diesel::sql_types::Int4)]
    stock_quantity: i32,
}

#[derive(diesel::QueryableByName)]
struct TicketRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    ticket_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    ticket_number: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    subject: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    priority: String,
}

/// Lists CRM/people contacts (read-only), scoped to the bot's branch.
async fn list_people(state: &Arc<AppState>, bot_uuid: &Uuid, search: Option<&str>) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let term = search.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<PersonRow> = match &term {
        Some(t) => {
            let pattern = format!("%{t}%");
            diesel::sql_query(
                "SELECT id AS person_id, first_name, last_name, email, status FROM crm_contacts \
                 WHERE branch_id = $1 AND (first_name ILIKE $2 OR last_name ILIKE $2 OR email ILIKE $2) \
                 ORDER BY last_name ASC LIMIT 25",
            )
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Text, _>(pattern)
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?
        }
        None => diesel::sql_query(
            "SELECT id AS person_id, first_name, last_name, email, status FROM crm_contacts \
             WHERE branch_id = $1 ORDER BY last_name ASC LIMIT 25",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let name = format!("{} {}", r.first_name.unwrap_or_default(), r.last_name.unwrap_or_default()).trim().to_string();
            json!({
                "person_id": r.person_id,
                "name": if name.is_empty() { "Unnamed".to_string() } else { name },
                "email": r.email.unwrap_or_default(),
                "status": r.status,
                "deep_link": format!("app://people?person_id={}", r.person_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "people": items }))
}

/// Lists invoices (branch-scoped, read-only).
async fn list_invoices(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<InvoiceRow> = diesel::sql_query(
        "SELECT id AS invoice_id, invoice_number, customer_name, status, total FROM billing_invoices \
         WHERE branch_id = $1 ORDER BY issue_date DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| json!({
            "invoice_id": r.invoice_id,
            "invoice_number": r.invoice_number,
            "customer_name": r.customer_name,
            "status": r.status,
            "total": r.total.to_string(),
            "deep_link": format!("app://billing?invoice_id={}", r.invoice_id),
        }))
        .collect();
    Ok(json!({ "count": items.len(), "invoices": items }))
}

/// Lists products/services (branch-scoped, read-only).
async fn list_products(state: &Arc<AppState>, bot_uuid: &Uuid, category: Option<&str>) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<ProductRow> = match category.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(cat) => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE branch_id = $1 AND category = $2 ORDER BY name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Text, _>(cat)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
        None => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE branch_id = $1 ORDER BY name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| json!({
            "product_id": r.product_id,
            "sku": r.sku.unwrap_or_default(),
            "name": r.name,
            "price": r.price.to_string(),
            "product_type": r.product_type,
            "stock_quantity": r.stock_quantity,
            "deep_link": format!("app://products?product_id={}", r.product_id),
        }))
        .collect();
    Ok(json!({ "count": items.len(), "products": items }))
}

/// Lists support tickets (branch-scoped, read-only).
async fn list_tickets(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<TicketRow> = diesel::sql_query(
        "SELECT id AS ticket_id, ticket_number, subject, status, priority FROM support_tickets \
         WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| json!({
            "ticket_id": r.ticket_id,
            "ticket_number": r.ticket_number,
            "subject": r.subject,
            "status": r.status,
            "priority": r.priority,
            "deep_link": format!("app://tickets?ticket_id={}", r.ticket_id),
        }))
        .collect();
    Ok(json!({ "count": items.len(), "tickets": items }))
}

/// Lists drive files under a folder (read-only, via the configured S3 repository).
async fn list_drive_files(state: &Arc<AppState>, bot_name: &str, path: Option<&str>) -> Result<Value, String> {
    let drive = state.drive.clone().ok_or_else(|| "drive unavailable".to_string())?;
    let bucket = format!("{bot_name}.gbai");
    let prefix = normalize_drive_path(path.unwrap_or(""))?;
    let prefix = if prefix.is_empty() { format!("{bot_name}.gbdrive/") } else { format!("{bot_name}.gbdrive/{prefix}") };
    let keys = drive
        .list_objects_with_metadata(&bucket, Some(&prefix))
        .await
        .map_err(|e| format!("drive list error: {e}"))?;
    let items: Vec<Value> = keys
        .into_iter()
        .map(|o| json!({ "path": o.key, "size": o.size, "etag": o.etag }))
        .collect();
    Ok(json!({ "count": items.len(), "files": items }))
}

async fn execute_tax(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    service: Option<String>,
    value: Option<String>,
) -> Result<Value, String> {
    let scope = botbanking::cashflow::resolve_bot_scope(&state.conn, bot_uuid, None)
        .ok_or_else(|| "bot not found".to_string())?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let result = match service {
        Some(service) => {
            botproducts::service_tax::calculate_service_tax_inner(
                &mut conn,
                scope.branch_id,
                &service,
                None,
                value.as_deref(),
            )
            .await
        }
        // No registered service named by the user: compute the breakdown for
        // the raw value with the branch tax rates (issue #722).
        None => calculate_anonymous_service_tax(&mut conn, &scope.branch_id, value.as_deref()),
    };
    match result {
        Ok(v) => Ok(v),
        Err((_code, msg)) => Err(msg),
    }
}

/// Computes the tax breakdown for a service value without a registered
/// service (chat asks like "calcular o imposto de um serviço de R$ 10.000").
/// Rates come from `billing_tax_rates` for the branch; the audit row is
/// persisted with a null service id.
fn calculate_anonymous_service_tax(
    conn: &mut diesel::PgConnection,
    branch_id: &Uuid,
    value: Option<&str>,
) -> Result<serde_json::Value, (axum::http::StatusCode, String)> {
    use std::str::FromStr as _;
    let service_value = value
        .map(Decimal::from_str)
        .transpose()
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Invalid value: {e}")))?
        .ok_or_else(|| (axum::http::StatusCode::BAD_REQUEST, "Provide a 'value' for the service".to_string()))?;
    let loaded = bottax::storage::load_rates_from_billing(conn, branch_id);
    let rates = loaded.unwrap_or_default();
    let source = if loaded.is_some() { "billing_tax_rates" } else { "default" };
    let breakdown = bottax::calculator::calculate_service_tax(service_value, &rates);
    if let Err((_code, msg)) = bottax::handlers::persist_calculation(
        conn,
        branch_id,
        None,
        None,
        &breakdown,
        source,
    ) {
        log::warn!("service.tax audit insert skipped: {msg}");
    }
    Ok(serde_json::json!({
        "service": null,
        "service_value": breakdown.service_value.to_string(),
        "breakdown": breakdown,
        "rates": {
            "irpj_pct": rates.irpj_pct.to_string(),
            "csll_pct": rates.csll_pct.to_string(),
            "pis_cofins_pct": rates.pis_cofins_pct.to_string(),
            "iss_pct": rates.iss_pct.to_string(),
        },
        "rate_source": source,
    }))
}

async fn write_drive_file(state: &Arc<AppState>, bot_name: &str, path: &str, content_b64: &str) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let data = base64::engine::general_purpose::STANDARD
        .decode(content_b64)
        .map_err(|e| format!("invalid base64 content: {e}"))?;
    let key = normalize_drive_path(path)?;
    let bucket = format!("{bot_name}.gbai");
    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.write -> {bucket}/{key}");
    Ok(json!({ "success": true, "bucket": bucket, "path": key }))
}

/// Copies a file from one drive path to another (e.g. an attached invoice
/// from `inbox/` into its `faturas/<month>/` folder).
async fn move_drive_file(state: &Arc<AppState>, bot_name: &str, from: &str, to: &str) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let bucket = format!("{bot_name}.gbai");
    let from_key = format!("{bot_name}.gbdrive/{}", normalize_drive_path(from)?);
    let to_key = format!("{bot_name}.gbdrive/{}", normalize_drive_path(to)?);
    let bytes = drive
        .get_object(&bucket, &from_key)
        .await
        .map_err(|e| format!("not found at {from_key}: {e}"))?;
    drive
        .put_object(&bucket, &to_key, bytes, None)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("api_call drive.file -> {bucket}/{to_key}");
    Ok(json!({ "success": true, "bucket": bucket, "from": from_key, "to": to_key }))
}
/// Archives invoice-like drive files (issue #723): lists files under the
/// `{bot}.gbdrive/{source}` folder, moves each into `{destination}/` so the
/// inbox stays clean, and returns a summary of archived files. Source may be
/// left empty to scan the whole drive; destination is required.
async fn archive_drive_files(
    state: &Arc<AppState>,
    bot_name: &str,
    source: &str,
    destination: &str,
) -> Result<Value, String> {
    let drive = state
        .drive
        .clone()
        .ok_or_else(|| "drive unavailable".to_string())?;
    let bucket = format!("{bot_name}.gbai");
    let src_prefix = if source.is_empty() {
        format!("{bot_name}.gbdrive/")
    } else {
        format!("{bot_name}.gbdrive/{}", normalize_drive_path(source)?)
    };
    let keys = drive
        .list_objects_with_metadata(&bucket, Some(&src_prefix))
        .await
        .map_err(|e| format!("drive list error: {e}"))?;

    let mut archived = 0i32;
    let mut skipped = 0i32;
    for obj in keys {
        let key = obj.key.clone();
        // Do not re-process a file already inside the destination folder.
        if key.starts_with(&format!("{bot_name}.gbdrive/{destination}/")) {
            skipped += 1;
            continue;
        }
        // Resolve the relative file name (the final path segment).
        let file_name = key.rsplit('/').next().unwrap_or(&key);
        let to_key = format!("{bot_name}.gbdrive/{}/{}", normalize_drive_path(destination)?, file_name);
        let bytes = drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| format!("read failed at {key}: {e}"))?;
        let put = drive.put_object(&bucket, &to_key, bytes, None).await;
        match put {
            Ok(()) => {
                archived += 1;
                log::info!("api_call drive.archive -> {to_key}");
            }
            Err(e) => {
                skipped += 1;
                log::error!("drive.archive skip {key}: {e}");
            }
        }
    }

    Ok(json!({
        "success": true,
        "bucket": bucket,
        "archived": archived,
        "skipped": skipped,
        "destination": destination,
    }))
}

/// Payroll diagnosis (issue #724): aggregates outgoing `billing_invoices` of
/// the branch by month and returns a compact financial summary the LLM can
/// present to the user.
async fn payroll_diagnosis(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    period: Option<&str>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;

    #[derive(diesel::QueryableByName)]
    struct PayRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        month_key: String,
        #[diesel(sql_type = diesel::sql_types::Int8)]
        count: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)]
        total: bigdecimal::BigDecimal,
    }

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let filter = "SELECT to_char(issue_date, 'YYYY-MM') AS month_key, \
                count(*)::bigint AS count, \
                coalesce(sum(total), 0) AS total \
         FROM billing_invoices WHERE branch_id = $1 AND to_char(issue_date, 'YYYY-MM') LIKE $2 \
         GROUP BY month_key ORDER BY month_key DESC LIMIT 12";
    let all = "SELECT to_char(issue_date, 'YYYY-MM') AS month_key, \
                count(*)::bigint AS count, \
                coalesce(sum(total), 0) AS total \
         FROM billing_invoices WHERE branch_id = $1 \
         GROUP BY month_key ORDER BY month_key DESC LIMIT 12";

    let rows: Vec<PayRow>; 
    if let Some(p) = period {
        let month_prefix = format!("{p}%");
        rows = diesel::sql_query(filter)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Text, _>(month_prefix)
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;
    } else {
        rows = diesel::sql_query(all)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .load(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;
    }

    let months: Vec<Value> = rows
        .into_iter()
        .map(|r| json!({ "month": r.month_key, "invoices": r.count, "total": r.total }))
        .collect();
    Ok(json!({
        "branch_id": branch,
        "months": months,
        "summary": if months.is_empty() { "no invoices recorded".to_string() } else { "monthly invoice totals above; use them as the payroll ledger basis".to_string() },
    }))
}
