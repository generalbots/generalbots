//! The system prompt of a chat turn and the one-line instruction that tells
//! the model this turn's execution mode.

use crate::core::bot::ws::stream::routing::Route;

use super::super::types::SessionIdentity;

/// The system prompt: the bot's prompt with the structured tools, the
/// composer's switches, the requested output type and, for a drawing turn,
/// the Canvas block.
pub(super) fn system_prompt(
    identity: &SessionIdentity<'_>,
    parsed: &serde_json::Value,
    active_switchers: &[String],
    ui_automation_enabled: bool,
    media_type: &str,
    user_text: &str,
) -> String {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    // The composer's app-automation/media switches ride with the client
    // capabilities so the prompt only advertises what this turn may use.
    let client_caps_value = {
        let mut caps = parsed
            .get("client_capabilities")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = caps.as_object_mut() {
            obj.insert("ui_automation".to_string(), serde_json::json!(ui_automation_enabled));
            if media_type != "auto" {
                obj.insert("media_type".to_string(), serde_json::json!(media_type));
            }
        }
        caps
    };

    let channel = sink.channel_type();
    let system_prompt = super::super::system_prompt::with_structured_tools(
        super::super::system_prompt::build(
            state,
            bot_uuid,
            session_id,
            user_id,
            bot_name,
            channel,
            active_switchers,
            Some(&client_caps_value),
        ),
        channel,
    );
    // #2238 — bias the answer toward the requested artifact/media type.
    let system_prompt = if media_type != "auto" {
        format!(
            "{system_prompt}\n\n---\nRequested output type: {media_type}. Prefer producing this artifact/format when feasible; if not possible, explain briefly and provide the closest alternative."
        )
    } else {
        system_prompt
    };
    // #1975 — selective Canvas/SVG subsystem prompt for drawing turns.
    let canvas_block = super::super::system_prompt::canvas_drawing_block(user_text);
    if canvas_block.is_empty() {
        system_prompt
    } else {
        format!("{system_prompt}\n\n{canvas_block}")
    }
}

/// This turn's execution mode, for the model.
pub(super) fn turn_instruction(agent_enabled: bool, route: Route) -> String {
    // #1946 — the selected mode is an instruction for this turn, not a
    // session-wide switch: it tells the model whether to plan execution or to
    // answer/patch through the catalog, while the guards stay in place.
    // #2239 — when Agent/VM is disabled, never offer or encourage workspace tools.
    // #2562 — when `agent-vm` is not compiled, the simpler "answer directly"
    //         block replaces the full VM-disabled message so the model never
    //         sees controls or capabilities that do not exist in this build.
    if !agent_enabled {
        if cfg!(feature = "agent-vm") {
            "THIS TURN: Agent/VM is disabled for this conversation. Do not use or mention vm_exec, \
                 terminal_run, or terminal.exec, and do not assume a workspace/VM is available. \
                 Never claim a file was created and never emit a vmfile: link. Answer directly or use \
                 non-VM capabilities only. When the request genuinely needs the VM (running code, \
                 generating or attaching a file), say so in one short sentence in the person's language \
                 (for example: \"Para gerar o PDF, ative o Agente.\"); a card to turn the Agent on is \
                 shown with your answer, so do not explain where the switch is. Then give, as text, \
                 any content you can write without the VM."
                .to_string()
        } else {
            "THIS TURN: Answer directly using your knowledge and the API command catalog. \
             Do not offer to run code, use a sandbox or generate files in this turn."
                .to_string()
        }
    } else {
        let line = match route {
            Route::Agentic => {
                "THIS TURN: agentic. Plan and execute the multi-step work the request needs, using tools/VM where they help, and deliver the requested artifact."
            }
            Route::NonAgentic => {
                "THIS TURN: non-agentic. Answer directly, or handle app-backed work through the app catalog / UI plan. Do not start a vm_exec or terminal_run loop unless the user explicitly asked to execute something or to produce a file."
            }
        };
        format!("{line} {FILES_ARE_REAL}")
    }
}

/// The rule on files for a turn with the Agent on: a file exists only when a
/// `vm_exec` step of this turn wrote it.
const FILES_ARE_REAL: &str = "A file exists only when a vm_exec step of this turn wrote it under \
    output/; never say a file was generated, checked, saved or exported otherwise, and never write \
    code for a file anywhere but vm_exec. After making files, say in two or three short sentences \
    what was produced and that it is shown below the answer.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_switched_off_agent_never_gets_the_agentic_instruction() {
        let text = turn_instruction(false, Route::Agentic);
        assert!(!text.contains("THIS TURN: agentic"));
        assert!(text.starts_with("THIS TURN:"));
    }

    #[test]
    fn a_switched_on_agent_follows_the_route() {
        assert!(turn_instruction(true, Route::Agentic).starts_with("THIS TURN: agentic."));
        assert!(turn_instruction(true, Route::NonAgentic).starts_with("THIS TURN: non-agentic."));
        assert!(turn_instruction(true, Route::Agentic).contains("A file exists only when a vm_exec step"));
    }

    #[test]
    fn a_switched_off_agent_keeps_the_prose_short() {
        let text = turn_instruction(false, Route::Agentic);
        if cfg!(feature = "agent-vm") {
            assert!(text.contains("a card to turn the Agent on"));
        }
    }
}
