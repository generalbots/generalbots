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
use botcore::shared::state::AppState;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::apps::registry;

pub const API_CALL_TRIGGER: &str = "\"__api_call__\":";

pub struct ApiCommand {
    pub name: &'static str,
    pub summary: &'static str,
    pub params: &'static [(&'static str, &'static str)],
    /// When true only users with the admin role may invoke this command.
    pub admin_only: bool,
}

pub struct ApiEndpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub summary: &'static str,
}

const TAX_CMD: ApiCommand = ApiCommand {
    name: "service.tax",
    summary: "Compute Brazilian service taxes for a registered service (IRPJ, CSLL, PIS/COFINS, ISS).",
    params: &[("service", "service name or id"), ("value", "optional amount; defaults to the service price")],
    admin_only: false,
};

const DIAGNOSIS_CMD: ApiCommand = ApiCommand {
    name: "banking.diagnosis",
    summary: "Cash-flow health of the account: revenue, expenses, net, pending reconciliation and tax rates.",
    params: &[("period", "optional YYYY-MM month filter")],
    admin_only: false,
};

const IMPORT_CMD: ApiCommand = ApiCommand {
    name: "banking.import",
    summary: "Import a month's cash-flow sheet (CSV stored in the bot drive) into the financial model.",
    params: &[
        ("file_key", "drive path of the CSV, e.g. financeiro/fluxo-caixa-2026-08.csv"),
        ("period", "optional YYYY-MM month filter"),
    ],
    admin_only: false,
};

const DRIVE_WRITE_CMD: ApiCommand = ApiCommand {
    name: "drive.write",
    summary: "Store a file (e.g. an invoice) in the bot drive under the given folder path.",
    params: &[
        ("path", "folder/file name, e.g. faturas/2026-08/fatura.pdf"),
        ("content_base64", "the file bytes in base64"),
    ],
    admin_only: false,
};

const DRIVE_FILE_CMD: ApiCommand = ApiCommand {
    name: "drive.file",
    summary: "Organize a stored drive file (e.g. an attached invoice from inbox) into its folder.",
    params: &[
        ("from", "current drive path, e.g. inbox/fatura.pdf"),
        ("to", "destination folder path, e.g. faturas/2026-08/fatura.pdf"),
    ],
    admin_only: false,
};

const WEB_SEARCH_CMD: ApiCommand = ApiCommand {
    name: "web.search",
    summary: "Search the web (DuckDuckGo) for current facts, news or prices.",
    params: &[("query", "the search terms"), ("max_results", "optional 1-25")],
    admin_only: false,
};

const APPS_FIND_CMD: ApiCommand = ApiCommand {
    name: "apps.find",
    summary: "Find a suite application by a description of what the user wants to do.",
    params: &[("query", "what the user wants to accomplish")],
    admin_only: false,
};

const API_FIND_CMD: ApiCommand = ApiCommand {
    name: "api.find",
    summary: "Discover which backend command or endpoint matches a described need.",
    params: &[("query", "the described need")],
    admin_only: false,
};

/// The stable command list rendered into the prompt and used by discovery.
pub fn commands_list() -> Vec<&'static ApiCommand> {
    vec![
        &TAX_CMD,
        &DIAGNOSIS_CMD,
        &IMPORT_CMD,
        &DRIVE_WRITE_CMD,
        &DRIVE_FILE_CMD,
        &WEB_SEARCH_CMD,
        &APPS_FIND_CMD,
        &API_FIND_CMD,
    ]
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
        // Search / research
        ApiEndpoint { method: "GET", path: "/api/ui/search", summary: "Unified search across business entities" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/search", summary: "Web search (DuckDuckGo)" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/summarize", summary: "Summarize web search results" },
        ApiEndpoint { method: "POST", path: "/api/ui/research/web/deep", summary: "Deep research on a topic" },
        ApiEndpoint { method: "GET", path: "/api/ui/research/web/instant", summary: "Instant answer from the web" },
        // Catalog / apps
        ApiEndpoint { method: "GET", path: "/api/apps/catalog", summary: "Suite application catalog" },
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
    ]
}

/// Compact system-prompt fragment teaching the `__api_call__` contract and
/// the routing rule, shortened to the current user's role so admin-only
/// commands are never proposed to regular users.
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
    lines.push("then add a short user-facing note after it. Available commands:".to_string());
    for cmd in commands_list() {
        if cmd.admin_only && role != crate::security::user_role::ROLE_ADMIN {
            continue;
        }
        let params = cmd
            .params
            .iter()
            .map(|(n, d)| format!("{n}: {d}"))
            .collect::<Vec<_>>()
            .join("; ");
        lines.push(format!(
            "- {name}: {summary} (params: {params}){admin}",
            name = cmd.name,
            summary = cmd.summary,
            admin = if cmd.admin_only { " [admin only]" } else { "" },
        ));
    }
    lines.push(
        "Rules: use compose=true when the answer needs prose from the data. If unsure which command fits, \
         emit api.find or apps.find first. Never mention JSON or commands to the user."
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
    let mut scored_cmds: Vec<(i32, &ApiCommand)> = commands_list()
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
        "service.tax" => execute_tax(state, &bot_uuid, str_of("service"), str_of("value")).await,
        "banking.diagnosis" => {
            let period = str_of("period");
            let resp =
                botbanking::cashflow::cashflow_diagnosis_inner(&state.conn, &bot_uuid, period.as_deref())
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
        _ => Err(format!("unknown command '{name}'")),
    }
}

async fn execute_tax(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    service: Option<String>,
    value: Option<String>,
) -> Result<Value, String> {
    let service = service.ok_or_else(|| "params.service is required (name or id)".to_string())?;
    let scope = botbanking::cashflow::resolve_bot_scope(&state.conn, bot_uuid)
        .ok_or_else(|| "bot not found".to_string())?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let result = botproducts::service_tax::calculate_service_tax_inner(
        &mut conn,
        scope.branch_id,
        &service,
        None,
        value.as_deref(),
    )
    .await;
    match result {
        Ok(v) => Ok(v),
        Err((_code, msg)) => Err(msg),
    }
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