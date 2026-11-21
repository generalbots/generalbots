//! `GET /api/llm/limits` — usage limits of every engine, from real sources only.
//!
//! - Codex subscription: the CLI's own session logs inside the agent workspace.
//! - HTTP providers (own key, free model, ChatGPT plan): the rate-limit headers
//!   of their last response, kept in memory by `botllm::rate_limits`.
//! - Claude Code and Antigravity expose no usage outside their interactive
//!   sessions, so they are listed with no windows and no source.
//! - The local engine has no quota.
//!
//! Nothing is estimated: an engine without a source has an empty `windows`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
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

#[cfg(feature = "agent-vm")]
async fn subscription_engines(state: &Arc<AppState>, user: &AuthenticatedUser) -> Vec<Value> {
    use crate::core::ai_accounts;
    let linked: Vec<String> = ai_accounts::list_accounts(&state.conn, user.user_id)
        .into_iter()
        .filter(|a| a.get("status").and_then(Value::as_str) == Some("linked"))
        .filter_map(|a| a.get("provider").and_then(Value::as_str).map(str::to_string))
        .collect();
    let mut out = Vec::new();
    for provider in linked {
        let entry = match provider.as_str() {
            ai_accounts::CODEX_CLI => codex_engine(user).await,
            ai_accounts::CHATGPT_PLAN => {
                let seen = rate_limits::for_host(PLAN_HOST);
                engine("subscription", ai_accounts::CHATGPT_PLAN, "ChatGPT plan", seen.as_ref())
            }
            ai_accounts::CLAUDE_CODE => no_source("subscription", ai_accounts::CLAUDE_CODE, "Claude Code"),
            ai_accounts::ANTIGRAVITY => no_source("subscription", ai_accounts::ANTIGRAVITY, "Antigravity"),
            _ => continue,
        };
        out.push(entry);
    }
    out
}

#[cfg(feature = "agent-vm")]
async fn codex_engine(user: &AuthenticatedUser) -> Value {
    let mut entry = no_source("subscription", crate::core::ai_accounts::CODEX_CLI, "Codex");
    let Some(service) = crate::core::bot::agent_vm_hook::service() else {
        return entry;
    };
    let vm = match botagent::vm::get_user_vm(service, &user.user_id) {
        Ok(Some(row)) => row,
        Ok(None) => return entry,
        Err((code, message)) => {
            log::warn!("llm limits: workspace lookup failed ({}): {message}", code.as_u16());
            return entry;
        }
    };
    match botagent::codex::limits(service, &vm.vm_name).await {
        Ok(Some(limits)) => {
            entry["source"] = json!("codex-session");
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
                        "resets_at": rfc3339(w.resets_at),
                    })
                })
                .collect();
        }
        Ok(None) => {}
        Err((code, message)) => {
            log::warn!("llm limits: codex session logs unreadable ({}): {message}", code.as_u16());
        }
    }
    entry
}

#[cfg(not(feature = "agent-vm"))]
async fn subscription_engines(_: &Arc<AppState>, _: &AuthenticatedUser) -> Vec<Value> {
    Vec::new()
}

async fn limits(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Json<Value> {
    let mut engines = subscription_engines(&state, &user).await;
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
    Json(json!({ "engines": engines }))
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

    #[test]
    fn local_and_empty_addresses_are_not_http_engines() {
        let out = http_engines("http://127.0.0.1:8081/v1", &[AccountRef { name: String::new(), url: String::new() }], &[]);
        assert!(out.is_empty());
    }
}
