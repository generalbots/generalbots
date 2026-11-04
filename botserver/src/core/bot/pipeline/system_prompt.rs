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
    // GB6-16 — persisted user preferences are part of the system prompt, so a
    // fresh conversation starts with what the user already taught the bot.
    let memory_block =
        crate::core::user_memory::render_for_prompt(&state.conn, user_id, Some(bot_uuid));
    let system_prompt = if memory_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}{memory_block}")
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
    // The desktop product's action surface is the agent VM (`vm_exec`), not the
    // SaaS REST catalogue. Teaching `api.find` there made the free model look
    // for a PDF endpoint instead of writing the program, and the call then
    // leaked into the chat as raw text.
    let include_api_catalog = !cfg!(feature = "desktop-local");
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
    format!("{channel_prompt}\n\n{}", capability_block(channel, client_caps))
}

/// Per-session capability block (#1678). The model must not offer a surface
/// this client did not announce.
#[must_use]
pub fn capability_block(channel: &str, client_caps: Option<&serde_json::Value>) -> String {
    let caps = client_caps.unwrap_or(&serde_json::Value::Null);
    let surfaces = caps
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            if super::event_sink::enabled() {
                "text, gb.basic.v1".to_string()
            } else {
                "text".to_string()
            }
        });
    let motion = caps
        .get("motion")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("full");
    let locale = caps
        .get("locale")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let input = caps
        .get("input")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("text,file");
    let max_live = caps
        .get("max_live_apps")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(2);
    format!(
        "CLIENT CAPABILITIES\nchannel={channel}; surfaces={surfaces}; max_live_apps={max_live}; motion={motion}; locale={locale}; input={input}.\nOnly use a surface listed above. If the client lists only `text`, answer in prose and never claim a card was shown. Respect reduced motion (no decorative animation) and the locale."
    )
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
