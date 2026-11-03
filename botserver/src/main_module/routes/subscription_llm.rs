//! Local OpenAI-compatible bridge for subscription LLM accounts (#1654).
//!
//! The Vibe agent loop speaks one protocol (`/chat/completions`). Subscription
//! providers (Claude Code in the VM, ChatGPT plan in Vault) are not HTTP
//! endpoints, so `subscription_llm` issues a short-lived token and points the
//! loop at this route. The token names the user; no bot/global key is used.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use axum::routing::post;
use axum::{Router};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

#[derive(Clone)]
struct Entry {
    user_id: Uuid,
    provider: String,
    model: String,
    bot_id: Uuid,
    session_id: Uuid,
    expires: u64,
}

static TOKENS: LazyLock<Mutex<HashMap<String, Entry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Issues a 10-minute token for one user/provider pair.
pub fn issue_token(
    user_id: Uuid,
    provider: &str,
    model: &str,
    bot_id: Uuid,
    session_id: Uuid,
) -> String {
    let token = format!("sub_{}", Uuid::new_v4().simple());
    if let Ok(mut tokens) = TOKENS.lock() {
        tokens.retain(|_, entry| entry.expires > now());
        tokens.insert(
            token.clone(),
            Entry {
                user_id,
                provider: provider.to_string(),
                model: model.to_string(),
                bot_id,
                session_id,
                expires: now() + 600,
            },
        );
    }
    token
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_string)
}

fn flatten(messages: &Value) -> String {
    let Some(list) = messages.as_array() else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for message in list {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("user");
        let content = message.get("content").and_then(Value::as_str).unwrap_or("");
        if content.trim().is_empty() {
            continue;
        }
        let label = match role {
            "system" => "System",
            "assistant" => "Assistant",
            _ => "User",
        };
        parts.push(format!("{label}: {content}"));
    }
    parts.join("\n\n")
}

/// POST /api/llm/subscription/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = bearer(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": { "message": "Missing subscription token" } })),
        )
    })?;
    let entry = {
        let mut tokens = TOKENS.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": { "message": "Token store unavailable" } })),
            )
        })?;
        tokens.retain(|_, entry| entry.expires > now());
        tokens.get(&token).cloned()
    };
    let Some(entry) = entry else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": { "message": "Subscription token expired; send the message again" } })),
        ));
    };
    if entry.provider == "unlinked" {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": {
                "message": "Connect your Claude or ChatGPT account in Settings \u{2192} AI accounts first."
            } })),
        ));
    }
    let model = body
        .get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&entry.model)
        .to_string();
    let messages = body.get("messages").cloned().unwrap_or(Value::Null);
    let prompt = flatten(&messages);
    if prompt.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": { "message": "No prompt in the request" } })),
        ));
    }
    let resolved = crate::core::ai_accounts::ResolvedLlm {
        provider: entry.provider.clone(),
        model: model.clone(),
        source: "user",
    };
    let provider = crate::llm::subscription::build(
        &state,
        entry.user_id,
        entry.session_id,
        entry.bot_id,
        &resolved,
    )
    .await
    .map_err(|message| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": { "message": message } })),
        )
    })?;
    let text = provider
        .generate(&prompt, &messages, &model, "")
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": { "message": error.to_string() } })),
            )
        })?;
    Ok(Json(json!({
        "id": format!("chatcmpl-{}", Uuid::new_v4().simple()),
        "object": "chat.completion",
        "model": model,
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": text },
            "finish_reason": "stop"
        }],
        "usage": {}
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/llm/subscription/chat/completions", post(chat_completions))
}

#[cfg(test)]
mod tests {
    use super::flatten;

    #[test]
    fn messages_become_one_prompt() {
        let messages = serde_json::json!([
            { "role": "system", "content": "be brief" },
            { "role": "user", "content": "hello" }
        ]);
        let prompt = flatten(&messages);
        assert!(prompt.contains("System: be brief"));
        assert!(prompt.contains("User: hello"));
    }
}
