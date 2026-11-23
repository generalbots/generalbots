//! `GET /api/llm/limits` — usage limits of every engine, from real sources only.
//!
//! - Codex subscription: the CLI's own session logs inside the agent workspace;
//!   `?refresh=1` re-reads it from the service through the CLI's app server.
//! - HTTP providers (own key, free model, ChatGPT plan): the rate-limit headers
//!   of their last response, kept in memory by `botllm::rate_limits`.
//! - Claude Code: the plan windows its stream-json turns reported, kept in
//!   memory by `botagent::plan_usage` (none until a turn ran since start).
//! - Antigravity reports nothing until a turn ends on a spent plan.
//! - The local engine has no quota.
//!
//! Nothing is estimated: an engine without a source has an empty `windows`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Extension, Json, Router};
use botcore::shared::state::AppState;
use botllm::rate_limits::{self, HostLimits, LimitWindow};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::security::auth_api::types::AuthenticatedUser;

const FREE_HOST: &str = "api.kilo.ai";
const PLAN_HOST: &str = "chatgpt.com";
const ACCOUNTS_PATH: &str = "gbo/llm-accounts";

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/llm/limits", get(limits))
}

/// Provider accounts as stored by Settings; only the address and name are
/// read, the key field is never deserialized.
#[derive(Debug, Deserialize)]
struct AccountRef {
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
}

fn rfc3339(at: Option<chrono::DateTime<chrono::Utc>>) -> Value {
    at.map(|t| json!(t.to_rfc3339())).unwrap_or(Value::Null)
}

fn window_json(w: &LimitWindow) -> Value {
    json!({
        "name": w.name,
        "used_percent": w.used_percent,
        "limit": w.limit,
        "remaining": w.remaining,
        "resets_at": rfc3339(w.resets_at),
    })
}

fn engine(kind: &str, provider: &str, label: &str, seen: Option<&HostLimits>) -> Value {
    json!({
        "kind": kind,
        "provider": provider,
        "label": label,
        "unlimited": false,
        "source": if seen.is_some() { json!("response-headers") } else { Value::Null },
        "observed_at": rfc3339(seen.map(|s| s.observed_at)),
        "windows": seen.map(|s| s.windows.iter().map(window_json).collect::<Vec<_>>()).unwrap_or_default(),
    })
}

fn no_source(kind: &str, provider: &str, label: &str) -> Value {
    engine(kind, provider, label, None)
}

fn is_loopback(host: &str) -> bool {
    host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]"
}

/// Own-key and free engines: the main endpoint, every provider account and
/// every other host a response was seen from. One entry per host.
fn http_engines(main_url: &str, accounts: &[AccountRef], seen: &[HostLimits]) -> Vec<Value> {
    let lookup: HashMap<&str, &HostLimits> = seen.iter().map(|h| (h.host.as_str(), h)).collect();
    let mut hosts: Vec<(String, String)> = Vec::new();
    let mut push = |host: String, label: String| {
        if !host.is_empty() && !is_loopback(&host) && host != PLAN_HOST && !hosts.iter().any(|(h, _)| *h == host) {
            hosts.push((host, label));
        }
    };
    let main_host = rate_limits::host_of(main_url);
    push(main_host.clone(), main_host);
    for account in accounts {
        let host = rate_limits::host_of(&account.url);
        let label = if account.name.trim().is_empty() { host.clone() } else { account.name.clone() };
        push(host, label);
    }
    for limits in seen {
        push(limits.host.clone(), limits.host.clone());
    }
    hosts
        .iter()
        .map(|(host, label)| {
            let free = host == FREE_HOST;
            let label = if free { "Free model" } else { label.as_str() };
            engine(if free { "free" } else { "api_key" }, host, label, lookup.get(host.as_str()).copied())
        })
        .collect()
}

async fn provider_accounts() -> Vec<AccountRef> {
    let Ok(secrets) = botcoresecrets::SecretsManager::get_clone() else {
        return Vec::new();
    };
    let map = secrets.get_secret(ACCOUNTS_PATH).await.unwrap_or_default();
    map.get("accounts")
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_default()
}

/// What each subscription engine can report, for the page's wording:
/// `live` is re-read from the service on refresh, `turns` arrives with each
/// answer, `headers` with each HTTP response, `none` only when the plan is spent.
#[cfg(feature = "agent-vm")]
fn reports(provider: &str) -> &'static str {
    use crate::core::ai_accounts::{CHATGPT_PLAN, CLAUDE_CODE, CODEX_CLI};
    match provider {
        CODEX_CLI => "live",
        CLAUDE_CODE => "turns",
        CHATGPT_PLAN => "headers",
        _ => "none",
    }
}

/// Adds what the CLI said during its turns: Claude Code's plan windows and,
/// for any agent, a spent plan that has not come back yet.
#[cfg(feature = "agent-vm")]
fn with_turn_reports(entry: &mut Value, usage: Option<botagent::plan_usage::PlanUsage>) {
    let Some(usage) = usage else { return };
    let now = chrono::Utc::now();
    let mut windows: Vec<Value> = usage
        .windows
        .iter()
        .map(|w| {
            json!({
                "name": w.name,
                "used_percent": w.used_percent,
                "limit": null,
                "remaining": null,
                "status": w.status,
                "resets_at": rfc3339(w.resets_at),
            })
        })
        .collect();
    let spent_now = match (usage.spent_at, usage.spent_until) {
        (Some(_), Some(until)) => until > now,
        (Some(at), None) => now - at < chrono::Duration::hours(1),
        _ => false,
    };
    if spent_now {
        windows.insert(0, json!({ "name": "plan", "used_percent": 100.0, "limit": null, "remaining": null,
            "status": "rejected", "resets_at": rfc3339(usage.spent_until) }));
    }
    if windows.is_empty() {
        return;
    }
    let newer = usage.observed_at().map(|t| t.to_rfc3339());
    let current = entry["observed_at"].as_str().map(str::to_string);
    if entry["windows"].as_array().map_or(true, Vec::is_empty) {
        entry["windows"] = json!(windows);
        entry["source"] = json!("agent-turn");
        entry["observed_at"] = json!(newer);
    } else if spent_now {
        if let Some(list) = entry["windows"].as_array_mut() {
            list.insert(0, windows.remove(0));
        }
        if newer > current {
            entry["observed_at"] = json!(newer);
        }
    }
}

#[cfg(feature = "agent-vm")]
fn workspace(user: &AuthenticatedUser) -> Option<(&'static Arc<botagent::AgentService>, String)> {
    let service = crate::core::bot::agent_vm_hook::service()?;
    match botagent::vm::get_user_vm(service, &user.user_id) {
        Ok(Some(row)) => Some((service, row.vm_name)),
        Ok(None) => None,
        Err((code, message)) => {
            log::warn!("llm limits: workspace lookup failed ({}): {message}", code.as_u16());
            None
        }
    }
}

#[cfg(feature = "agent-vm")]
async fn subscription_engines(state: &Arc<AppState>, user: &AuthenticatedUser, refresh: bool) -> Vec<Value> {
    use crate::core::ai_accounts;
    let linked: Vec<String> = ai_accounts::list_accounts(&state.conn, user.user_id)
        .into_iter()
        .filter(|a| a.get("status").and_then(Value::as_str) == Some("linked"))
        .filter_map(|a| a.get("provider").and_then(Value::as_str).map(str::to_string))
        .collect();
    let vm = if linked.is_empty() { None } else { workspace(user) };
    let mut out = Vec::new();
    for provider in linked {
        let mut entry = match provider.as_str() {
            ai_accounts::CODEX_CLI => codex_engine(vm.as_ref(), refresh).await,
            ai_accounts::CHATGPT_PLAN => {
                let seen = rate_limits::for_host(PLAN_HOST);
                engine("subscription", ai_accounts::CHATGPT_PLAN, "ChatGPT plan", seen.as_ref())
            }
            ai_accounts::CLAUDE_CODE => no_source("subscription", ai_accounts::CLAUDE_CODE, "Claude Code"),
            ai_accounts::ANTIGRAVITY => no_source("subscription", ai_accounts::ANTIGRAVITY, "Antigravity"),
            _ => continue,
        };
        if let Some((_, vm_name)) = vm.as_ref() {
            with_turn_reports(&mut entry, botagent::plan_usage::latest(vm_name, &provider));
        }
        entry["reports"] = json!(reports(&provider));
        out.push(entry);
    }
    out
}

#[cfg(feature = "agent-vm")]
fn codex_json(entry: &mut Value, limits: &botagent::codex::CodexLimits, source: &str) {
    entry["source"] = json!(source);
    entry["observed_at"] = rfc3339(limits.observed_at);
    entry["windows"] = limits
        .windows
        .iter()
        .map(|w| {
            json!({
                "name": w.name,
                "used_percent": w.used_percent,
                "limit": null,
                "remaining": null,
                "status": null,
                "resets_at": rfc3339(w.resets_at),
            })
        })
        .collect();
}

/// Codex: re-read from the service on refresh, else the newest state its
/// session logs recorded.
#[cfg(feature = "agent-vm")]
async fn codex_engine(vm: Option<&(&'static Arc<botagent::AgentService>, String)>, refresh: bool) -> Value {
    let mut entry = no_source("subscription", crate::core::ai_accounts::CODEX_CLI, "Codex");
    let Some((service, vm_name)) = vm else {
        return entry;
    };
    if refresh {
        match botagent::codex::live_limits(service, vm_name).await {
            Ok(Some(limits)) => {
                codex_json(&mut entry, &limits, "codex-service");
                entry["refreshed"] = json!(true);
                return entry;
            }
            Ok(None) => log::info!("llm limits: codex service sent no plan windows"),
            Err((code, message)) => log::warn!("llm limits: codex live read failed ({}): {message}", code.as_u16()),
        }
        entry["refresh_failed"] = json!(true);
    }
    match botagent::codex::limits(service, vm_name).await {
        Ok(Some(limits)) => codex_json(&mut entry, &limits, "codex-session"),
        Ok(None) => {}
        Err((code, message)) => {
            log::warn!("llm limits: codex session logs unreadable ({}): {message}", code.as_u16());
        }
    }
    entry
}

#[cfg(not(feature = "agent-vm"))]
async fn subscription_engines(_: &Arc<AppState>, _: &AuthenticatedUser, _: bool) -> Vec<Value> {
    Vec::new()
}

#[derive(Debug, Default, Deserialize)]
struct LimitsQuery {
    #[serde(default)]
    refresh: Option<String>,
}

async fn limits(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<LimitsQuery>,
) -> Json<Value> {
    let refresh = matches!(query.refresh.as_deref(), Some("1" | "true"));
    let mut engines = subscription_engines(&state, &user, refresh).await;
    let config = botcore::config::ConfigManager::new(state.conn.clone());
    let main_url = config
        .get_config(&uuid::Uuid::nil(), "llm-url", Some(""))
        .unwrap_or_default();
    let accounts = provider_accounts().await;
    engines.extend(http_engines(&main_url, &accounts, &rate_limits::all()));
    let local_configured = is_loopback(&rate_limits::host_of(&main_url));
    if local_configured || botsettings::on_device_ai::runtime_ctl::port_open().await {
        let mut local = no_source("local", "local", "Local model");
        local["unlimited"] = json!(true);
        engines.push(local);
    }
    Json(json!({ "engines": engines, "checked_at": chrono::Utc::now().to_rfc3339(), "refreshed": refresh }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn seen(host: &str) -> HostLimits {
        HostLimits {
            host: host.to_string(),
            observed_at: Utc::now(),
            windows: vec![LimitWindow {
                name: "requests".to_string(),
                used_percent: 40.0,
                limit: Some(10),
                remaining: Some(6),
                resets_at: None,
            }],
        }
    }

    #[test]
    fn hosts_are_listed_once_with_their_last_headers() {
        let accounts = vec![
            AccountRef { name: "OpenAI".into(), url: "https://api.openai.com/v1/chat/completions".into() },
            AccountRef { name: "Again".into(), url: "https://api.openai.com/v1".into() },
        ];
        let out = http_engines(
            "https://api.kilo.ai/api/gateway/chat/completions",
            &accounts,
            &[seen("api.openai.com"), seen("chatgpt.com"), seen("api.groq.com")],
        );
        let providers: Vec<&str> = out.iter().filter_map(|e| e["provider"].as_str()).collect();
        assert_eq!(providers, ["api.kilo.ai", "api.openai.com", "api.groq.com"]);
        assert_eq!(out[0]["kind"], "free");
        assert_eq!(out[0]["source"], Value::Null);
        assert_eq!(out[0]["windows"], json!([]));
        assert_eq!(out[1]["label"], "OpenAI");
        assert_eq!(out[1]["source"], "response-headers");
        assert_eq!(out[1]["windows"][0]["used_percent"], 40.0);
        assert_eq!(out[1]["windows"][0]["resets_at"], Value::Null);
    }

    #[cfg(feature = "agent-vm")]
    #[test]
    fn turn_reports_fill_a_silent_engine_and_a_spent_plan_leads() {
        use botagent::plan_usage::{PlanUsage, PlanWindow};
        let window = PlanWindow { name: "5h".into(), used_percent: Some(82.0), resets_at: None, status: Some("allowed_warning".into()), observed_at: Utc::now() };
        let mut silent = no_source("subscription", "claude-code", "Claude Code");
        with_turn_reports(&mut silent, Some(PlanUsage { windows: vec![window], ..PlanUsage::default() }));
        assert_eq!(silent["source"], "agent-turn");
        assert_eq!(silent["windows"][0]["status"], "allowed_warning");
        let mut spent = no_source("subscription", "antigravity", "Antigravity");
        let until = Utc::now() + chrono::Duration::hours(1);
        with_turn_reports(&mut spent, Some(PlanUsage { windows: Vec::new(), spent_at: Some(Utc::now()), spent_until: Some(until) }));
        assert_eq!(spent["windows"][0]["name"], "plan");
        assert_eq!(spent["windows"][0]["used_percent"], 100.0);
        let mut over = no_source("subscription", "antigravity", "Antigravity");
        let past = Utc::now() - chrono::Duration::hours(1);
        with_turn_reports(&mut over, Some(PlanUsage { windows: Vec::new(), spent_at: Some(past), spent_until: Some(past) }));
        assert_eq!(over["windows"], json!([]));
    }

    #[test]
    fn local_and_empty_addresses_are_not_http_engines() {
        let out = http_engines("http://127.0.0.1:8081/v1", &[AccountRef { name: String::new(), url: String::new() }], &[]);
        assert!(out.is_empty());
    }
}
