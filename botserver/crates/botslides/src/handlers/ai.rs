//! AI assistant for the slide editor.
//!
//! #2923 — this endpoint never answers from a keyword table. It sends the
//! request to the configured LLM (the same `LLM_URL`/`LLM_KEY`/`LLM_MODEL`
//! contract the Docs editor uses) and returns either the LLM's answer or an
//! explicit `503 ai_unavailable`. The caller receives a typed verb in
//! `action`/`data` when the model requests a deck mutation; the editor's verb
//! endpoints are the executors, this endpoint does not silently mutate a deck.

use crate::storage::DriveOps;
use crate::types::{SlidesAiRequest, SlidesAiResponse};
use crate::SlidesState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

/// Typed verbs bound in `botserver/src/apps/commands/verb_endpoints/table/office_endpoints.rs`.
/// Only these may be returned as an executable `action`.
const ALLOWED_ACTIONS: &[&str] = &[
    "slides.slide.add",
    "slides.slide.delete",
    "slides.slide.duplicate",
    "slides.slide.notes",
    "slides.element.add",
    "slides.element.delete",
    "slides.media.add",
    "slides.theme.set",
    "slides.transition.set",
];

#[derive(Debug, Deserialize)]
struct LlmPayload {
    #[serde(default)]
    response: String,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unavailable {
    NotConfigured,
    Provider,
}

impl Unavailable {
    fn reason(self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::Provider => "provider_error",
        }
    }
}

/// Env config is optional by design: an unset key means "not configured", not
/// an error. Log the reason instead of using unwrap_or_default so the ratchet
/// stays clean.
fn env_or_empty(name: &str) -> String {
    match std::env::var(name) {
        Ok(value) => value,
        Err(error) => {
            log::debug!("{name} is unset or invalid: {error}");
            String::new()
        }
    }
}

fn llm_config() -> Option<(String, String)> {
    let url = env_or_empty("LLM_URL");
    let key = env_or_empty("LLM_KEY");
    if url.trim().is_empty() || key.trim().is_empty() {
        None
    } else {
        Some((url, key))
    }
}

/// Mirrors `botdocs/src/handlers_api/ai.rs::llm_complete`: OpenAI-compatible
/// chat completions with a system + user prompt and a Bearer key. Uses the
/// shared botlib HTTP client because this crate deliberately owns no HTTP dep.
async fn llm_complete(system_prompt: &str, user_prompt: &str) -> Result<String, Unavailable> {
    let (url, key) = llm_config().ok_or(Unavailable::NotConfigured)?;
    let model = env_or_empty("LLM_MODEL");
    let model = if model.trim().is_empty() {
        "default".to_string()
    } else {
        model
    };
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "temperature": 0.3,
        "response_format": { "type": "json_object" }
    });
    let client =
        botlib::http_client::BotServerClient::with_timeout(Some(url), Duration::from_secs(60));
    let reply: serde_json::Value = client.post_authorized("", &body, &key).await.map_err(|e| {
        log::error!("Slides AI: LLM request failed: {e}");
        Unavailable::Provider
    })?;
    reply["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            log::error!("Slides AI: LLM response has no message content");
            Unavailable::Provider
        })
}

fn system_prompt() -> String {
    format!(
        "You are an assistant embedded in a slide editor. Return ONLY a JSON object with a \
         string key \"response\" describing what you did or found. If the user asks for a deck \
         change, also set \"action\" to exactly one of these typed verbs: {}. Put the verb's \
         arguments in the object \"data\". Never claim an edit you did not request.",
        ALLOWED_ACTIONS.join(", ")
    )
}

fn user_prompt(req: &SlidesAiRequest) -> String {
    format!(
        "Command: {}\nPresentation id: {}\nSlide index: {}",
        req.command,
        req.presentation_id.as_deref().unwrap_or(""),
        req.slide_index
            .map(|index| index.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    )
}

/// Parses an LLM answer into the typed response DTO. A malformed envelope is
/// kept as a plain answer; an action outside the allowlist is stripped so it
/// can never be executed as a verb.
fn parse_llm_answer(content: &str, req: &SlidesAiRequest) -> SlidesAiResponse {
    let payload: LlmPayload = match serde_json::from_str(content) {
        Ok(payload) => payload,
        Err(_) => {
            return SlidesAiResponse {
                response: content.trim().to_string(),
                action: None,
                data: None,
            }
        }
    };
    let mut data = match payload.data {
        Some(value @ serde_json::Value::Object(_)) => Some(value),
        _ => None,
    };
    let action = match payload.action {
        Some(action) if ALLOWED_ACTIONS.contains(&action.as_str()) => Some(action),
        _ => None,
    };
    if let (Some(action), Some(serde_json::Value::Object(fields))) = (&action, &mut data) {
        if let Some(id) = req.presentation_id.as_deref() {
            fields
                .entry("presentation_id".to_string())
                .or_insert_with(|| serde_json::Value::String(id.to_string()));
        }
        if let (Some(index), Some(value)) = (req.slide_index, fields.get("slide_index")) {
            if value.is_null() {
                fields.insert("slide_index".to_string(), serde_json::json!(index));
            }
        }
        log::debug!("Slides AI typed action: {action}");
    } else {
        data = None;
    }
    SlidesAiResponse {
        response: payload.response,
        action,
        data,
    }
}

fn unavailable_body(reason: Unavailable) -> serde_json::Value {
    serde_json::json!({
        "error": "ai_unavailable",
        "reason": reason.reason()
    })
}

fn unavailable_response(reason: Unavailable) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(unavailable_body(reason)),
    )
        .into_response()
}

pub async fn handle_slides_ai<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<SlidesAiRequest>,
) -> Response {
    log::trace!(
        "Slides AI request (drive configured: {})",
        state.drive.is_some()
    );
    match llm_complete(&system_prompt(), &user_prompt(&req)).await {
        Ok(content) => Json(parse_llm_answer(&content, &req)).into_response(),
        Err(reason) => unavailable_response(reason),
    }
}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod ai_tests;
