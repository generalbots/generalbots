//! Native tools the model can call without a per-bot `.ast` script.
//!
//! Most LLM tool calls resolve to `{bot}.gbdialog/{tool}.ast`. Workspace
//! execution (#1582) is infrastructure, not bot content: every bot with an
//! agent VM must expose it, so it is handled here before the script lookup.
//!
//! Split by tool family: `vm_exec` (program execution and its repair/render
//! helpers), `terminal` (live PTY runs) and this module (capture buffer,
//! chat send/persist and the dispatch table).

use super::types::SessionIdentity;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "agent-vm")]
mod program_repair;
#[cfg(feature = "agent-vm")]
mod render;
#[cfg(feature = "agent-vm")]
mod terminal;
#[cfg(feature = "agent-vm")]
mod vm_exec;

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
        .unwrap_or_default()
}

/// Returns true when `name` is a native tool and the call was answered.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    if super::surface_tools::try_run(identity, name, args).await {
        return true;
    }
    if name == "terminal_run" {
        #[cfg(feature = "agent-vm")]
        terminal::run_terminal_run(identity, args).await;
        #[cfg(not(feature = "agent-vm"))]
        send(
            identity,
            "Live terminal runs are not available in this build.",
        )
        .await;
        return true;
    }
    if name == "vm_exec" {
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
    if super::native_drive::try_run(identity, name, args).await {
        return true;
    }
    false
}

pub(super) async fn send(identity: &SessionIdentity<'_>, message: &str) {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    if let Ok(mut map) = capture().lock() {
        let entry = map.entry(session_id).or_default();
        if !entry.is_empty() {
            entry.push('\n');
        }
        entry.push_str(message);
    }
    let resp = botlib::models::BotResponse::new(
        bot_uuid.to_string(),
        session_id.to_string(),
        user_id.to_string(),
        message,
        sink.channel_type(),
    );
    // A failed send is already logged by the sink implementation.
    let _ = sink.send_bot_response(&resp).await;
}

/// Save an assistant turn so a result is still there after the user switches
/// conversation, reloads, or closes the tab.
pub(super) async fn persist(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    text: &str,
) {
    let mut sm = state.session_manager.lock().await;
    if let Err(e) = sm.save_message(session_id, user_id, 2, text, 2) {
        botlog::safe_warn!("failed to persist workspace result for {session_id}: {e}");
    }
}
