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
    // GB6-16 — persisted user preferences are part of the system prompt, so a
    // fresh conversation starts with what the user already taught the bot.
    let memory_block =
        crate::core::user_memory::render_for_prompt(&state.conn, user_id, Some(bot_uuid));
    let system_prompt = if memory_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}{memory_block}")
    };
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    // The desktop product's action surface is the agent VM (`vm_exec`), not the
    // SaaS REST catalogue. Teaching `api.find` there made the free model look
    // for a PDF endpoint instead of writing the program, and the call then
    // leaked into the chat as raw text.
    let include_api_catalog = !cfg!(feature = "desktop-local");
    if channel == "web" {
        if include_api_catalog {
            format!(
                "{system_prompt}\n\n{}\n\n{}",
                crate::core::bot::api_catalog::api_command_instructions(&role),
                crate::main_module::ui_plan::ui_automation_instructions(),
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
    }
}

/// The structured-interaction surface is advertised only when the client can
/// receive run events; otherwise the model would offer cards that this build
/// cannot render.
pub(crate) fn with_structured_tools(prompt: String, channel: &str) -> String {
    if channel != "web" || !super::event_sink::enabled() {
        return prompt;
    }
    format!("{prompt}\n\n{}", super::surface_tools::instructions())
}
