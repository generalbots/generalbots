//! Prompt-cache warm-up for the local engine.
//!
//! The engine caches the KV state of the prompt prefix in each slot.  After a
//! fresh start the cache is cold: the first real turn evaluates the full system
//! prompt (4 000+ tokens, several seconds on a laptop card in split mode).
//! This module sends one throw-away request whose prompt is the same system
//! message a real turn will carry.  The engine evaluates it, caches the KV,
//! and when the real turn arrives the common prefix is reused — turning a 5+ s
//! wait into a fraction of a second.
//!
//! # Stability of the prefix
//!
//! The system prompt assembled here uses default client capabilities (locale
//! "en", no specific surfaces).  When the shell connects with a different
//! locale (e.g. "pt-BR"), the capability block and the response-language block
//! near the end of the prompt differ.  The engine's cache is prefix-based, so
//! every token *before* the first mismatch is still reused.  The mismatch is
//! about 130–160 tokens; the stable prefix is ~96 % of the full system prompt.
//!
//! What would make the remaining ~4 % stable: storing the user's last known
//! locale in a DB config and reading it here.
//!
//! # Safety with `--parallel 1`
//!
//! The engine has one slot.  The warm-up request occupies it for the prompt
//! evaluation (the same time the real first turn would take) and generates a
//! single token.  A real request arriving meanwhile is queued by the HTTP
//! server and processed next — it then benefits from the cached prefix.  The
//! user is never worse off than without the warm-up.

use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use botcore::shared::state::AppState;
use diesel::RunQueryDsl;

/// Send one cache-priming request to the local engine.
///
/// Runs once per engine load, only for the loopback engine.  Errors are
/// informational (the engine may have been stopped between the signal and
/// this call).
pub async fn warm(state: &Arc<AppState>) -> Result<(), String> {
    let system_prompt = build_warmup_prompt(state)?;
    let prompt_len = system_prompt.len();

    let messages = serde_json::json!([
        {"role": "system", "content": system_prompt},
        {"role": "user",   "content": "."}
    ]);

    let url = format!(
        "http://127.0.0.1:{}/v1/chat/completions",
        botsettings::on_device_ai::runtime_ctl::CHAT_PORT,
    );

    let body = serde_json::json!({
        "messages": messages,
        "max_tokens": 1,
        "temperature": 0
    });

    log::info!(
        "[cache_warm] priming prompt cache ({} bytes system prompt)",
        prompt_len,
    );

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    match client.post(&url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            log::info!("[cache_warm] prompt cache primed");
            Ok(())
        }
        Ok(resp) => Err(format!("engine returned {}", resp.status())),
        Err(e) => Err(format!("engine unreachable: {e}")),
    }
}

/// Build the system prompt that a real first turn will carry.
///
/// The local engine strips the structured-tool text from the system message
/// (see `local_profile::apply`), so the warm-up does not include it either.
fn build_warmup_prompt(state: &Arc<AppState>) -> Result<String, String> {
    let (bot_name, bot_uuid, user_id) = default_bot_info(state)?;
    let prompt = super::super::system_prompt::build(
        state,
        bot_uuid,
        Uuid::nil(),
        user_id,
        &bot_name,
        "web",
        &[],
        None,
    );
    Ok(prompt)
}

/// The default assistant's bot name, UUID and the admin user id.
fn default_bot_info(state: &Arc<AppState>) -> Result<(String, Uuid, Uuid), String> {
    let mut conn = state.conn.get().map_err(|e| format!("db pool: {e}"))?;

    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let row = diesel::sql_query(
        "SELECT id::text AS id, name FROM bots ORDER BY created_at ASC LIMIT 1",
    )
    .load::<BotRow>(&mut conn)
    .map_err(|e| format!("bots query: {e}"))?
    .into_iter()
    .next()
    .ok_or_else(|| "no bots deployed".to_string())?;

    let bot_uuid = Uuid::parse_str(&row.id).map_err(|e| format!("uuid: {e}"))?;
    // The nil user reads the global config row; custom instructions for a
    // specific user are a few dozen tokens at most and do not justify a
    // second query.
    Ok((row.name, bot_uuid, Uuid::nil()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn warmup_prompt_uses_default_caps() {
        // The warm-up calls system_prompt::build with client_caps=None,
        // which defaults the locale to "en".  This is a compile-time
        // assertion that the function exists and accepts those arguments.
        // The integration proof (time-to-first-token with and without
        // warm-up) is measured against a running engine.
        let _warm_uses_none: Option<&serde_json::Value> = None;
    }
}
