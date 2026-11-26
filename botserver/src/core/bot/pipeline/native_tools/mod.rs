//! Native tools the model can call without a per-bot `.ast` script.
//!
//! Most LLM tool calls resolve to `{bot}.gbdialog/{tool}.ast`. Workspace
//! execution (#1582) is infrastructure, not bot content: every bot with an
//! agent VM must expose it, so it is handled here before the script lookup.
//!
//! Split by tool family: `vm_exec` (program execution and its repair/render
//! helpers), `terminal` (live PTY runs), `web_fetch` (one public page),
//! `web_search` (a web search), `web_steps` (every search and read as a step),
//! `web_prefetch` (named pages read ahead for engines without tools) and
//! this module (capture buffer, chat send/persist and the dispatch table).

use super::types::SessionIdentity;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

mod model_only;
#[cfg(feature = "agent-vm")]
pub(super) use model_only::ModelOnly;
use model_only::model_only;
#[cfg(feature = "agent-vm")]
mod program_repair;
#[cfg(feature = "agent-vm")]
mod render;
#[cfg(feature = "agent-vm")]
mod terminal;
#[cfg(feature = "agent-vm")]
mod vm_background;
#[cfg(feature = "agent-vm")]
mod vm_budget;
#[cfg(feature = "agent-vm")]
mod vm_exec;
#[cfg(feature = "agent-vm")]
mod vm_trace;
#[cfg(feature = "agent-vm")]
pub mod vm_turn_engine;
pub(super) mod web_fetch;
pub(super) mod web_prefetch;
#[cfg(any(feature = "research", feature = "web-search"))]
pub(super) mod web_search;
pub mod web_steps;
mod web_text;

/// The last message(s) a native tool sent to chat, keyed by session. The LLM
/// pipeline uses it to feed the tool result back for a follow-up answer;
/// native tools write to the sink directly and previously returned no value.
static NATIVE_CAPTURE: OnceLock<Mutex<HashMap<uuid::Uuid, String>>> = OnceLock::new();

fn capture() -> &'static Mutex<HashMap<uuid::Uuid, String>> {
    NATIVE_CAPTURE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn begin_capture(session_id: uuid::Uuid) {
    if let Ok(mut map) = capture().lock() {
        map.remove(&session_id);
    }
}

pub(super) fn take_capture(session_id: uuid::Uuid) -> String {
    capture()
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&session_id))
        .map_or_else(Default::default, |v| v)
}

/// Returns true when `name` is a native tool and the call was answered.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    if super::surface_tools::try_run(identity, name, args).await {
        return true;
    }
    if name == web_fetch::TOOL {
        web_fetch::run(identity, args).await;
        return true;
    }
    #[cfg(any(feature = "research", feature = "web-search"))]
    if name == web_search::TOOL {
        web_search::run(identity, args).await;
        return true;
    }
    if name == "terminal_run" || name == "vm_exec" {
        if !super::agent_turn::workspace_route_allowed(identity.agent_vm_enabled) {
            refuse_vm_off(identity).await;
            return true;
        }
        if name == "terminal_run" {
            #[cfg(feature = "agent-vm")]
            if let Some(ticket) = super::vm_consent::tools::gate_terminal(identity, args).await {
                let receipt = ticket.receipt();
                let scope = ModelOnly::begin(identity.session_id);
                let outcome = terminal::run_terminal_run(identity, args, ticket).await;
                drop(scope);
                super::vm_consent::report(identity, &receipt, &outcome).await;
            }
            #[cfg(not(feature = "agent-vm"))]
            send(
                identity,
                "Live terminal runs are not available in this build.",
            )
            .await;
            return true;
        }
        #[cfg(feature = "agent-vm")]
        vm_exec::run_vm_exec(identity, args).await;
        #[cfg(not(feature = "agent-vm"))]
        send(
            identity,
            "Workspace execution is not available in this build.",
        )
        .await;
        return true;
    }
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if super::vm_consent::tools::run_drive_tool(identity, name, args).await {
        return true;
    }
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if super::native_drive::try_run(identity, name, args).await {
        return true;
    }
    false
}

/// Records `message` as the tool result the model reads, without showing it
/// to the person (an instruction for the model, or a result the person
/// already sees as the outcome of the tool's step).
pub(super) fn tell_model(session_id: uuid::Uuid, message: &str) {
    if let Ok(mut map) = capture().lock() {
        let entry = map.entry(session_id).or_default();
        if !entry.is_empty() {
            entry.push('\n');
        }
        entry.push_str(message);
    }
}

pub(super) async fn send(identity: &SessionIdentity<'_>, message: &str) {
    tell_model(identity.session_id, message);
    show(identity, message).await;
}

/// The Agent (VM) switch is off: the model reads why in English, the person
/// reads it in the edition's language (outside a model-only scope).
pub(super) async fn refuse_vm_off(identity: &SessionIdentity<'_>) {
    send_split(identity, super::agent_turn::DISABLED_MESSAGE, super::agent_turn::DISABLED_FOR_PERSON).await;
    let request = super::turn_events::request_of(identity.session_id);
    let frame = super::turn_events::agent_enable_frame(super::turn_events::AGENT_REASON, &request);
    super::turn_events::emit(identity, super::turn_events::AGENT_ENABLE_REQUEST, frame).await;
}

/// The model reads `for_model`; the person reads `for_person` (outside a
/// model-only scope).
pub(super) async fn send_split(identity: &SessionIdentity<'_>, for_model: &str, for_person: &str) {
    tell_model(identity.session_id, for_model);
    show(identity, for_person).await;
}

/// Shows `message` to the person unless a model-only scope is open.
async fn show(identity: &SessionIdentity<'_>, message: &str) {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    if model_only(session_id) {
        return;
    }
    let resp = botlib::models::BotResponse::new(
        bot_uuid.to_string(),
        session_id.to_string(),
        user_id.to_string(),
        message,
        sink.channel_type(),
    );
    // Best-effort send; log here when the sink rejects the frame.
    if let Err(e) = sink.send_bot_response(&resp).await {
        log::warn!("native tools: response frame not delivered: {e}");
    }
}

/// Save an assistant turn so a result is still there after the user switches
/// conversation, reloads, or closes the tab.
pub(super) async fn persist(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    text: &str,
) {
    if model_only(session_id) {
        return;
    }
    let mut sm = state.session_manager.lock().await;
    if let Err(e) = sm.save_message(session_id, user_id, 2, text, 2) {
        botlog::safe_warn!("failed to persist workspace result for {session_id}: {e}");
    }
}
