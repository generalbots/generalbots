//! Chat system-prompt assembly (#1635/#1637 follow-up).
//!
//! Split out of `exec.rs` so the pipeline entry point stays readable and every
//! pipeline file honors the repository's 450-line limit. Behaviour is
//! unchanged: the channel prompt, active switchers, persisted user memory, the
//! role resolved from RBAC and the channel-specific action surface are
//! composed in the same order as before.

use std::sync::Arc;

use uuid::Uuid;

use botcore::shared::state::AppState;

/// Stored user memory as a bounded prompt block (GB7-16). The memory
/// subsystem keeps its renderer private because it also composes the per-turn
/// note; the prompt needs the same bounded lines without that note. A pool or
/// read failure is non-fatal: an older answer must still be generated.
fn memory_prompt_block(state: &AppState, user_id: Uuid, bot_uuid: Uuid) -> String {
    const MAX_ENTRIES: i64 = 24;
    const MAX_CHARS: usize = 2000;
    let Ok(mut conn) = state.conn.get() else {
        log::warn!("user memory could not be read for the prompt: pool error");
        return String::new();
    };
    if !botsettings::memory_store::is_enabled(&mut conn, user_id) {
        return String::new();
    }
    let rows = match botsettings::memory_store::list(
        &mut conn,
        user_id,
        Some(bot_uuid),
        MAX_ENTRIES,
    ) {
        Ok(rows) => rows,
        Err(error) => {
            log::warn!("user memory could not be read for the prompt: {error}");
            return String::new();
        }
    };
    let mut block = String::new();
    for entry in rows {
        let line = if botsettings::memory_store::is_entry_key(&entry.key) {
            format!("- {}\n", entry.value)
        } else {
            format!("- {}: {}\n", entry.key, entry.value)
        };
        if block.len() + line.len() > MAX_CHARS {
            block.push_str("- (more entries available in Settings › Memory)\n");
            break;
        }
        block.push_str(&line);
    }
    block
}

/// Builds the system prompt for one turn.
///
/// * `channel` is the sink's channel type (`web`, `whatsapp`, …).
/// * `active_switchers` are the composer's active switcher ids; they append
///   their own instructions after the base prompt.
pub(super) fn build(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    channel: &str,
    active_switchers: &[String],
    client_caps: Option<&serde_json::Value>,
) -> String {
    let base_system_prompt =
        crate::core::bot::ws::message::load_system_prompt_for_channel(bot_name, channel);
    let system_prompt = if !active_switchers.is_empty() {
        let switcher_prompts = crate::basic::keywords::switcher::resolve_active_switchers(
            state.cache.as_ref(),
            &bot_uuid.to_string(),
            &session_id.to_string(),
            active_switchers,
        );
        if switcher_prompts.is_empty() {
            base_system_prompt
        } else {
            format!("{base_system_prompt}\n\n{switcher_prompts}")
        }
    } else {
        base_system_prompt
    };
    // GB7-16 — persisted user preferences are part of the system prompt, so a
    // fresh conversation starts with what the user already taught the bot.
    let memory_block = memory_prompt_block(state, user_id, bot_uuid);
    let system_prompt = if memory_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}{memory_block}")
    };
    // #2561 — per-user custom instructions ("Instruções personalizadas").
    let custom_block =
        crate::main_module::routes::custom_instructions::render_for_prompt(&state.conn, user_id);
    let system_prompt = if custom_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}{custom_block}")
    };
    // #1681 — installed Agent Skills packages are advertised as guidance.
    #[cfg(feature = "sources")]
    let skills_block = match state.conn.get() {
        Ok(mut conn) => {
            botsources::handlers::sources_handlers::render_for_prompt(&mut conn, bot_uuid)
        }
        Err(error) => {
            log::warn!("skills prompt skipped: pool error: {error}");
            String::new()
        }
    };
    #[cfg(not(feature = "sources"))]
    let skills_block = String::new();
    let system_prompt = format!("{system_prompt}{skills_block}");
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    // #2033 — the REST catalogue is a capability of the deployment, not of the
    // build flavour. A `desktop-local` binary served over the network (the
    // production bot) still answers REST requests, so suppressing the catalogue
    // there left the model unable to detect any app-data request. The default
    // is on; a single-user install that deliberately drives only the agent VM
    // can opt out with GB_PROMPT_API_CATALOG=0.
    let include_api_catalog =
        api_catalog_enabled(std::env::var("GB_PROMPT_API_CATALOG").ok().as_deref());
    let channel_prompt = if channel == "web" {
        if include_api_catalog {
            format!(
                "{system_prompt}\n\n{}\n\n{}",
                crate::core::bot::api_catalog::api_command_instructions(&role),
                crate::main_module::ui_plan::ui_automation_instructions(client_caps),
            )
        } else {
            system_prompt
        }
    } else if channel == "whatsapp" {
        format!(
            "{system_prompt}\n\n---\nThis conversation is on WhatsApp.\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    } else {
        format!(
            "{system_prompt}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(&role),
        )
    };
    // #1939 — the delivery policy only makes sense where the model can act
    // (web/desktop); a channel without an action surface gets prose only.
    let delivery = if channel == "web" {
        delivery_policy_block()
    } else {
        String::new()
    };
    // #2096 — the response-language rule is a platform property: it is appended
    // here, after every channel-specific block, so a bot whose own `PROMPT.md`
    // omits it still answers in the user's language.
    let language = response_language_block(&client_locale(client_caps));
    // #2562 — inject today's date so authored prompts do not need to hard-code
    // it and the model does not have to guess or leave `[data]` placeholders.
    let date = today_block();
    let blocks = [
        date,
        capability_block(channel, client_caps),
        language,
        delivery,
    ];
    let appended = blocks
        .iter()
        .map(String::as_str)
        .filter(|block| !block.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{channel_prompt}\n\n{appended}")
}

mod blocks;

pub use blocks::{
    canvas_drawing_block, capability_block, client_locale, delivery_policy_block,
    response_language_block,
};
pub(crate) use blocks::{wants_drawing, with_structured_tools};
use blocks::{api_catalog_enabled, today_block};

#[cfg(test)]
mod tests;
