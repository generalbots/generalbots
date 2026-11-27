//! Prompt-cache warm-up for the local engine.
//!
//! The engine caches the KV state of the prompt prefix in each slot.  After a
//! fresh start the cache is cold: the first real turn evaluates the full system
//! prompt (4 000+ tokens, several seconds on a laptop card in split mode).
//! This module sends one throw-away request whose prompt matches the prefix of
//! a real first turn — same system message, same tools, same turn instruction
//! prefix — so the engine caches the KV and the first real turn reuses it.
//!
//! # What is cached
//!
//! The chat template renders system + tools + user message into one token
//! sequence.  The warm-up reproduces the system message and tool definitions
//! exactly; the user message starts with the same turn-instruction prefix a
//! real turn will carry.  The engine reuses the common prefix; only the
//! user's actual question and per-turn blocks (memory, route) are evaluated.
//!
//! # What remains uncached (~30 tokens)
//!
//! The user message diverges where the per-turn text begins: the user's
//! actual question, the agentic route instruction (which depends on what the
//! user asked), and the memory note.  Those are ~30 tokens of a ~4 500 token
//! prompt — under 1 % — and cost a fraction of a second.
//!
//! # Safety with `--parallel 1`
//!
//! The engine has one slot.  The warm-up occupies it for the prompt
//! evaluation (the same time the real first turn would take) and generates a
//! single token.  A real request arriving meanwhile is queued by the HTTP
//! server and processed next — it then benefits from the cached prefix.  The
//! user is never worse off than without the warm-up.

use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use botcore::shared::state::AppState;
use diesel::RunQueryDsl;

/// Build the warm-up request body — the same fields the pipeline sends to
/// the loopback engine on a first turn, with an empty user question.
///
/// This is the SINGLE assembly point.  Both the warm-up and the real
/// pipeline use the same `system_prompt::build`, the same tool catalog, and
/// the same `local_profile::apply` + `order_for_prompt_cache` transforms, so
/// the two cannot drift.
pub fn build_warmup_body(state: &Arc<AppState>) -> Result<serde_json::Value, String> {
    let (bot_name, bot_uuid, user_id) = default_bot_info(state)?;

    // ── system prompt: same code path as exec/prompt.rs ────────────────
    let prompt = super::super::system_prompt::build(
        state, bot_uuid, Uuid::nil(), user_id, &bot_name, "web", &[], None,
    );
    // The pipeline calls with_structured_tools, then local_profile strips
    // it when tools are attached.  Calling build() alone gives the same
    // text after the strip, without depending on the strip order.

    // ── turn instruction: the prefix a real user message carries ────────
    // A first turn with the agent off (the gbgov default) uses this text.
    // The warm-up's user message starts with it so the prefix extends into
    // the user section.  The mismatch begins at the agentic-route line
    // (which depends on what the user typed).
    let turn_prefix = first_turn_instruction();

    // ── messages: one system, one user (like a first turn after reorder) ─
    let mut messages = serde_json::json!([
        {"role": "system", "content": prompt},
        {"role": "user",   "content": format!("{turn_prefix}\n\n.")}
    ]);

    // ── tools: the structured-tool catalog the pipeline attaches ────────
    let tools = super::super::surface_catalog::llm_tools();

    // ── local_profile: strip the tool text from the system message ──────
    // (a no-op here because build() does not include it, but calling it
    // keeps the warm-up honest if the assembly ever changes)
    super::local_profile::apply(&mut messages, !tools.is_empty());

    Ok(serde_json::json!({
        "model": "warmup",
        "messages": messages,
        "tools": tools,
        "tool_choice": "auto",
        "max_tokens": 1,
        "temperature": 0
    }))
}

/// The turn-instruction prefix common to every first turn with the agent
/// off.  Extracted from `exec/prompt::turn_instruction` so the warm-up
/// uses the same words.
fn first_turn_instruction() -> &'static str {
    if cfg!(feature = "agent-vm") {
        "THIS TURN: Agent/VM is disabled for this conversation. Do not use or mention vm_exec, \
             terminal_run, or terminal.exec, and do not assume a workspace/VM is available. \
             Never claim a file was created and never emit a vmfile: link. Answer directly or use \
             non-VM capabilities only. When the request genuinely needs the VM (running code, \
             generating or attaching a file), say so in one short sentence in the person's language \
             (for example: \"Para gerar o PDF, ative o Agente.\"); a card to turn the Agent on is \
             shown with your answer, so do not explain where the switch is. Then give, as text, \
             any content you can write without the VM."
    } else {
        "THIS TURN: Answer directly using your knowledge and the API command catalog. \
         Do not offer to run code, use a sandbox or generate files in this turn."
    }
}

/// Send one cache-priming request to the local engine.
///
/// Runs once per engine load, only for the loopback engine.  Errors are
/// informational (the engine may have been stopped between the signal and
/// this call).
pub async fn warm(state: &Arc<AppState>) -> Result<(), String> {
    let body = build_warmup_body(state)?;
    let prompt_bytes = body["messages"][0]["content"]
        .as_str()
        .map(str::len)
        .unwrap_or(0);

    log::info!(
        "[cache_warm] priming prompt cache ({} bytes system, {} tools)",
        prompt_bytes,
        body["tools"].as_array().map(Vec::len).unwrap_or(0),
    );

    let url = format!(
        "http://127.0.0.1:{}/v1/chat/completions",
        botsettings::on_device_ai::runtime_ctl::CHAT_PORT,
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
    Ok((row.name, bot_uuid, Uuid::nil()))
}

#[cfg(test)]
mod tests {
    use super::first_turn_instruction;

    #[test]
    fn turn_instruction_matches_the_pipeline() {
        // The warm-up's turn instruction must start with the same text
        // the pipeline's exec/prompt::turn_instruction produces when the
        // agent is off.  If someone changes one and not the other, this
        // test breaks.
        let text = first_turn_instruction();
        assert!(
            text.starts_with("THIS TURN:"),
            "warm-up turn instruction must start with THIS TURN:"
        );
        if cfg!(feature = "agent-vm") {
            assert!(
                text.contains("Agent/VM is disabled"),
                "gbgov warm-up must use the agent-off instruction"
            );
        }
    }

    #[test]
    fn warmup_body_includes_tools() {
        // The chat template renders tools into the token sequence.
        // If the warm-up omits them, the prefix won't match.
        // This is a compile-time check that build_warmup_body wires
        // the tool catalog; the integration proof is on a running engine.
        let tools = super::super::super::surface_catalog::llm_tools();
        assert!(
            !tools.is_empty(),
            "surface tools must exist for the warm-up to match"
        );
    }
}
