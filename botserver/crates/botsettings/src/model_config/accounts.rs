//! Provider accounts: one API key per provider, the models it offers, and the
//! ones the person switched on for the chat's model menu.
//!
//! Stored in Vault at `gbo/llm-accounts` (one JSON list under `accounts`);
//! keys never leave the server. A chat turn whose model preference is
//! `acct:<provider>/<model>` is answered by that account (see
//! [`resolve_choice`]); every other preference keeps the bot's own model.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use botcoresecrets::SecretsManager;
use botsecurity::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;

use super::{api_error, require_admin, ApiResult};

const PATH: &str = "gbo/llm-accounts";
const PREFIX: &str = "acct:";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
    /// Provider id from the interface (`anthropic`, `openai`, `org`, ...).
    pub id: String,
    pub name: String,
    /// Chat address, as the provider list fills it in.
    pub url: String,
    #[serde(default)]
    pub key: String,
    /// Models switched on for the chat's model menu.
    #[serde(default)]
    pub enabled: Vec<String>,
}

/// What a chat turn uses when the person picked an account's model.
#[derive(Debug, Clone)]
pub struct Choice {
    pub url: String,
    pub key: String,
    pub model: String,
}

async fn load() -> Vec<Account> {
    let Ok(secrets) = SecretsManager::get_clone() else { return Vec::new() };
    let map: HashMap<String, String> = secrets.get_secret(PATH).await.unwrap_or_default();
    map.get("accounts")
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_default()
}

async fn save(accounts: &[Account]) -> Result<(), (StatusCode, Json<Value>)> {
    let secrets = SecretsManager::get_clone()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_unavailable"))?;
    let mut map = HashMap::new();
    map.insert(
        "accounts".to_string(),
        serde_json::to_string(accounts).unwrap_or_else(|_| "[]".into()),
    );
    secrets.put_secret(PATH, map).await.map_err(|e| {
        log::error!("[llm_accounts] vault write failed: {e}");
        api_error(StatusCode::INTERNAL_SERVER_ERROR, "vault_write_failed")
    })
}

/// Pure selection: given accounts, return the key of the first whose host
/// or id matches.  Testable without Vault.
fn select_key(accounts: &[Account], endpoint_url: &str, provider_id: Option<&str>) -> Option<String> {
    let target_host = host(endpoint_url);
    for acct in accounts {
        if acct.key.trim().is_empty() {
            continue;
        }
        if !target_host.is_empty() && host(&acct.url) == target_host {
            return Some(acct.key.clone());
        }
        if let Some(pid) = provider_id {
            if acct.id == pid {
                return Some(acct.key.clone());
            }
        }
    }
    None
}

/// Returns the API key of a saved account whose chat address matches
/// `endpoint_host`, or whose id is `provider_id`.
pub async fn key_for_host_or_provider(
    endpoint_url: &str,
    provider_id: Option<&str>,
) -> Option<String> {
    select_key(&load().await, endpoint_url, provider_id)
}

/// Whether any saved account has a non-empty key for the given provider id
/// or for a chat address with the same host.
pub async fn has_key_for(endpoint_url: &str, provider_id: Option<&str>) -> bool {
    key_for_host_or_provider(endpoint_url, provider_id).await.is_some()
}

/// Resolves the API key for an `own_key` mode switch.  Typed key wins;
/// same-host preserves existing; otherwise falls back to a saved account.
pub async fn resolve_own_key_fallback(
    typed_key: Option<&str>,
    endpoint: &str,
    provider_id: Option<&str>,
    secrets: &botcoresecrets::SecretsManager,
    central_path: &str,
) -> Option<String> {
    if typed_key.is_some_and(|k| !k.trim().is_empty()) {
        return typed_key.map(|k| k.trim().to_string());
    }
    let old_url = secrets
        .get_secret(central_path)
        .await
        .unwrap_or_default()
        .get("url")
        .cloned()
        .unwrap_or_default();
    if super::write_sets::same_host(&old_url, endpoint) {
        return None; // write_sets::compute preserves the existing key
    }
    key_for_host_or_provider(endpoint, provider_id).await
}

/// `acct:<provider>/<model>` → provider id and model (the model may itself
/// contain `/` and `:`).
pub fn parse_preference(pref: &str) -> Option<(&str, &str)> {
    let rest = pref.trim().strip_prefix(PREFIX)?;
    let (id, model) = rest.split_once('/')?;
    (!id.is_empty() && !model.is_empty()).then_some((id, model))
}

/// The account and model a chat turn asked for, when it is one of the
/// switched-on models of a stored account.
pub async fn resolve_choice(pref: &str) -> Option<Choice> {
    let (id, model) = parse_preference(pref)?;
    let account = load().await.into_iter().find(|a| a.id == id)?;
    if !account.enabled.iter().any(|m| m == model) {
        log::warn!("[llm_accounts] model {model} is not switched on for {id}");
        return None;
    }
    Some(Choice { url: account.url, key: account.key, model: model.to_string() })
}

/// The display name of a stored account, or the provider id as fallback.
pub async fn account_label(provider_id: &str) -> String {
    load()
        .await
        .into_iter()
        .find(|a| a.id == provider_id)
        .map(|a| if a.name.is_empty() { a.id } else { a.name })
        .unwrap_or_else(|| provider_id.to_string())
}

/// Accounts and their switched-on models, for the chat's model menu.
pub async fn chat_groups() -> Vec<Value> {
    load()
        .await
        .into_iter()
        .filter(|a| !a.enabled.is_empty())
        .map(|a| {
            json!({
                "id": a.id,
                "name": a.name,
                "models": a.enabled.iter().map(|m| json!({
                    "model": m,
                    "value": format!("{PREFIX}{}/{m}", a.id),
                })).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default()
}

fn is_host(h: &str, domain: &str) -> bool {
    h == domain || h.ends_with(&format!(".{domain}"))
}

/// Where a provider lists its models, derived from its chat address.
pub fn models_url(chat_url: &str) -> Option<String> {
    let url = reqwest::Url::parse(chat_url).ok()?;
    let h = host(chat_url);
    if is_host(&h, "anthropic.com") {
        return Some(format!("{}://{h}/v1/models?limit=1000", url.scheme()));
    }
    if is_host(&h, "googleapis.com") {
        return Some(format!("{}://{h}/v1beta/openai/models", url.scheme()));
    }
    let full = chat_url.trim_end_matches('/');
    let base = full.strip_suffix("/chat/completions").unwrap_or(full);
    Some(format!("{base}/models"))
}

// ── HTTP ─────────────────────────────────────────────────────────────────

pub(super) async fn handle_list(Extension(user): Extension<AuthenticatedUser>) -> ApiResult {
    require_admin(&user)?;
    let accounts: Vec<Value> = load()
        .await
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id, "name": a.name, "host": host(&a.url),
                "has_key": !a.key.trim().is_empty(), "enabled": a.enabled,
            })
        })
        .collect();
    Ok(Json(json!({ "accounts": accounts })))
}

#[derive(Deserialize)]
pub(super) struct ModelsRequest {
    id: String,
    url: String,
    api_key: Option<String>,
}

/// Asks the provider which models this key can use.
pub(super) async fn handle_models(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ModelsRequest>,
) -> ApiResult {
    require_admin(&user)?;
    let mut key = req.api_key.unwrap_or_default().trim().to_string();
    if key.is_empty() {
        key = load()
            .await
            .into_iter()
            .find(|a| a.id == req.id && a.url == req.url)
            .map(|a| a.key)
            .unwrap_or_default();
    }
    if key.is_empty() {
        key = configured_key(&state, &req.url);
    }
    let Some(list_url) = models_url(&req.url) else {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_endpoint_url"));
    };
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "http_client_failed"))?;
    let mut request = client.get(&list_url);
    if is_host(&host(&req.url), "anthropic.com") {
        request = request.header("x-api-key", &key).header("anthropic-version", "2023-06-01");
    } else if !key.is_empty() {
        request = request.bearer_auth(&key);
    }
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => {
            let code = if e.is_timeout() { "timeout" } else { "connection_refused" };
            return Ok(Json(json!({ "models": [], "error": code })));
        }
    };
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Ok(Json(json!({
            "models": [],
            "error": format!("http_status_{}", status.as_u16()),
            "detail": super::test_endpoint::provider_message(&body),
        })));
    }
    Ok(Json(json!({ "models": model_ids(&body) })))
}

/// The key of the main "own key" model when it points at the same provider
/// host, so a key typed once in Modelo also lists that provider's models.
pub(super) fn configured_key(state: &Arc<AppState>, url: &str) -> String {
    let cfg = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();
    let main_url = cfg.get_config(&nil, "llm-url", Some("")).unwrap_or_default();
    if main_url.is_empty() || !super::write_sets::same_host(&main_url, url) {
        return String::new();
    }
    cfg.get_config(&nil, "llm-key", Some("")).unwrap_or_default().trim().to_string()
}


/// Model ids from an OpenAI-shaped (`data[].id`) or Anthropic/Gemini list,
/// newest first when the provider dates them (`created` / `created_at`),
/// otherwise in the provider's own order.
pub fn model_ids(body: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let list = v.get("data").or_else(|| v.get("models")).and_then(Value::as_array);
    let mut rows: Vec<(String, String)> = list
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id").or_else(|| m.get("name")).and_then(Value::as_str)?;
                    Some((id.trim_start_matches("models/").to_string(), release_key(m)))
                })
                .collect()
        })
        .unwrap_or_default();
    // Stable sort: undated models keep the provider's order after dated ones.
    rows.sort_by(|a, b| b.1.cmp(&a.1));
    let mut seen = std::collections::HashSet::new();
    rows.into_iter().map(|(id, _)| id).filter(|id| seen.insert(id.clone())).collect()
}

/// A sortable release stamp: unix seconds (OpenAI-style `created`) or an
/// RFC 3339 date (Anthropic `created_at`), both as a zero-padded string.
fn release_key(m: &Value) -> String {
    if let Some(n) = m.get("created").and_then(Value::as_i64) {
        return chrono::DateTime::from_timestamp(n, 0)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default();
    }
    m.get("created_at").and_then(Value::as_str).unwrap_or("").to_string()
}

#[derive(Deserialize)]
pub(super) struct SaveRequest {
    id: String,
    name: String,
    url: String,
    api_key: Option<String>,
    enabled: Vec<String>,
}

/// Stores the account and its switched-on models. A key left empty keeps
/// the one already stored for this provider.
pub(super) async fn handle_save(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<SaveRequest>,
) -> ApiResult {
    require_admin(&user)?;
    if req.id.trim().is_empty() || req.id.contains('/') {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_provider"));
    }
    if reqwest::Url::parse(req.url.trim()).is_err() {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_endpoint_url"));
    }
    let mut accounts = load().await;
    let old_key = accounts
        .iter()
        .find(|a| a.id == req.id)
        .map(|a| a.key.clone())
        .unwrap_or_default();
    let mut key = req.api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).unwrap_or(old_key);
    if key.is_empty() {
        key = configured_key(&state, &req.url);
    }
    accounts.retain(|a| a.id != req.id);
    accounts.push(Account {
        id: req.id.trim().to_string(),
        name: req.name.trim().to_string(),
        url: req.url.trim().to_string(),
        key,
        enabled: req.enabled,
    });
    save(&accounts).await?;
    Ok(Json(json!({ "ok": true })))
}

pub(super) async fn handle_delete(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> ApiResult {
    require_admin(&user)?;
    let mut accounts = load().await;
    accounts.retain(|a| a.id != id);
    save(&accounts).await?;
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_name_an_account_and_a_model() {
        assert_eq!(parse_preference("acct:anthropic/claude-opus-5-5"), Some(("anthropic", "claude-opus-5-5")));
        assert_eq!(
            parse_preference("acct:openrouter/meta-llama/llama-4:free"),
            Some(("openrouter", "meta-llama/llama-4:free"))
        );
        assert_eq!(parse_preference("auto"), None);
        assert_eq!(parse_preference("acct:anthropic/"), None);
    }

    #[test]
    fn model_lists_live_next_to_the_chat_address() {
        assert_eq!(models_url("https://api.openai.com/v1/chat/completions").unwrap(), "https://api.openai.com/v1/models");
        assert_eq!(models_url("https://api.deepseek.com/chat/completions").unwrap(), "https://api.deepseek.com/models");
        assert_eq!(models_url("https://api.anthropic.com").unwrap(), "https://api.anthropic.com/v1/models?limit=1000");
        assert_eq!(
            models_url("https://generativelanguage.googleapis.com").unwrap(),
            "https://generativelanguage.googleapis.com/v1beta/openai/models"
        );
    }

    #[test]
    fn model_ids_are_read_from_both_shapes() {
        let openai = r#"{"object":"list","data":[{"id":"gpt-b"},{"id":"gpt-a"}]}"#;
        assert_eq!(model_ids(openai), ["gpt-b", "gpt-a"], "undated: provider order");
        let anthropic = r#"{"data":[{"type":"model","id":"claude-opus-5-5"}],"has_more":false}"#;
        assert_eq!(model_ids(anthropic), ["claude-opus-5-5"]);
        let gemini = r#"{"data":[{"id":"models/gemini-3.5-flash"}]}"#;
        assert_eq!(model_ids(gemini), ["gemini-3.5-flash"]);
    }

    #[test]
    fn dated_models_come_newest_first() {
        let openai = r#"{"data":[{"id":"old","created":1600000000},{"id":"new","created":1760000000},{"id":"mid","created":1700000000}]}"#;
        assert_eq!(model_ids(openai), ["new", "mid", "old"]);
        let anthropic = r#"{"data":[{"id":"claude-3","created_at":"2024-03-04T00:00:00Z"},{"id":"claude-5","created_at":"2026-05-01T00:00:00Z"}]}"#;
        assert_eq!(model_ids(anthropic), ["claude-5", "claude-3"]);
    }

    fn acct(id: &str, url: &str, key: &str) -> Account {
        Account { id: id.into(), name: id.into(), url: url.into(), key: key.into(), enabled: vec![] }
    }

    #[test]
    fn select_key_by_host_and_provider() {
        let list = vec![
            acct("anthropic", "https://api.anthropic.com/v1/messages", "sk-ant"),
            acct("openai", "https://api.openai.com/v1/chat/completions", "sk-oai"),
        ];
        // Host match
        assert_eq!(super::select_key(&list, "https://api.anthropic.com/v1/chat/completions", None), Some("sk-ant".into()));
        // No host match, falls through to provider id match
        assert_eq!(super::select_key(&list, "https://other.example.com/v1/chat", Some("openai")), Some("sk-oai".into()));
        // No match at all
        assert_eq!(super::select_key(&list, "https://other.example.com/v1/chat", None), None);
        // Empty key is skipped
        let empty = vec![acct("x", "https://api.anthropic.com/v1/chat/completions", "  ")];
        assert_eq!(super::select_key(&empty, "https://api.anthropic.com/v1/chat/completions", None), None);
    }
}
