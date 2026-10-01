//! LLM resolution for the BASIC runtime (`llm_generate`, issue #1465).
//!
//! Tool scripts reach the model through [`botbasic_types::BasicRuntime::llm_generate`],
//! a synchronous trait method, while every provider in the platform is async. This
//! module owns three concerns so `basic/mod.rs` stays a thin adapter:
//!
//! 1. **Per-bot configuration.** `ConfigManager::get_config` is called with the
//!    default bot's UUID, not `Uuid::nil()`, so a bot's own Vault entry at
//!    `secret/gbo/{org}/{branch}/{bot}` (`llm-model`, `llm-key`) wins over the
//!    tenant-wide `secret/gbo/llm` fallback. A tool therefore classifies with the
//!    same model its chat pipeline uses.
//! 2. **The sync/async bridge.** A dedicated OS thread owns a current-thread Tokio
//!    runtime and blocks on the provider future, so the caller's runtime is never
//!    starved — the deadlock pattern documented in the keyword layer.
//! 3. **Model-handler post-processing.** The raw provider string still carries
//!    reasoning markers for several open-weight models (`<think>`, `" response"`).
//!    Routing it through `botllm::llm_models::get_handler` strips them exactly as
//!    the `SUMMARIZE` keyword does; without this a classified caption can come back
//!    with the model's private reasoning prepended.

use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use log::warn;
use std::time::Duration;
use uuid::Uuid;

/// Budget for a single tool-side LLM call. Tool scripts run inside a web request
/// or a channel turn, so a stuck provider must fail the tool rather than the
/// session. Matches the keyword layer's 120 s ceiling.
const GENERATION_TIMEOUT: Duration = Duration::from_secs(120);

/// Model used when neither the bot configuration nor the caller supplies one.
/// Matches the self-hosted llama.cpp default used across the runtime.
const FALLBACK_MODEL: &str = "llama3";

/// Resolves the UUID of the default bot, which owns the LLM settings a tool
/// script inherits. `get_default_bot` reports a name, so the `bots` row is the
/// only place a real UUID can come from.
///
/// Returns `Uuid::nil()` when the row is absent, which keeps resolution on the
/// tenant-wide `secret/gbo/llm` fallback instead of failing the call.
#[must_use]
pub fn default_bot_uuid(state: &AppState) -> Uuid {
    use botcore::shared::models::schema::bots;

    let (name, _) = crate::core::bot::get_default_bot();
    let Ok(mut conn) = state.conn.get() else {
        warn!("[basic_llm] database unavailable while resolving the default bot UUID");
        return Uuid::nil();
    };
    let bot_name = name.clone();
    bots::table
        .filter(bots::name.eq(name))
        .select(bots::id)
        .first::<Uuid>(&mut conn)
        .unwrap_or_else(|e| {
            warn!("[basic_llm] default bot '{bot_name}' not resolvable: {e}");
            Uuid::nil()
        })
}

/// Reads one LLM setting for `bot_id`, preferring configuration over the value the
/// caller passed in and falling back to the process environment.
fn config_or_argument(state: &AppState, bot_id: &Uuid, key: &str, argument: &str) -> String {
    let manager = ConfigManager::new(state.conn.clone());
    manager
        .get_config(bot_id, key, None)
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| (!argument.is_empty()).then(|| argument.to_string()))
        .unwrap_or_default()
}

/// Generates a completion for `prompt` using the bot's configured provider.
///
/// `model` and `api_key` are hints from the keyword layer; configuration wins
/// when it defines a value. Returns a sanitized message — never a raw provider
/// error string — so the reason is safe to surface in chat.
pub fn generate(state: &AppState, prompt: &str, model: &str, api_key: &str) -> Result<String, String> {
    let bot_id = default_bot_uuid(state);
    let resolved_model = {
        let value = config_or_argument(state, &bot_id, "llm-model", model);
        if value.is_empty() { FALLBACK_MODEL.to_string() } else { value }
    };
    let resolved_key = config_or_argument(state, &bot_id, "llm-key", api_key);

    let provider = state
        .llm_provider
        .clone()
        .ok_or_else(|| "LLM provider not configured".to_string())?;

    let (tx, rx) = std::sync::mpsc::channel();
    let prompt = prompt.to_string();
    let handler_model = resolved_model.clone();
    // A fresh thread per call is deliberate: the caller is a synchronous trait
    // method that may already be executing on a runtime worker, where blocking
    // on an async future would deadlock.
    if let Err(e) = std::thread::Builder::new()
        .name("basic-llm-worker".into())
        .spawn(move || {
            let outcome = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on(async move {
                    provider
                        .generate(&prompt, &serde_json::Value::Null, &resolved_model, &resolved_key)
                        .await
                        .map(|raw| {
                            botllm::llm_models::get_handler(&handler_model).process_content(&raw)
                        })
                        .map_err(|e| e.to_string())
                }),
                Err(e) => Err(format!("LLM runtime unavailable: {e}")),
            };
            let _ = tx.send(outcome);
        })
    {
        return Err(format!("LLM worker could not start: {e}"));
    }

    match rx.recv_timeout(GENERATION_TIMEOUT) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "LLM generation timed out after {} seconds",
            GENERATION_TIMEOUT.as_secs()
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("LLM worker stopped before returning a result".to_string())
        }
    }
}

#[cfg(test)]
#[path = "runtime_llm_tests.rs"]
mod tests;
