//! #1580 — a `vm_exec` job that must outlive the tab. It runs in its own task
//! and saves its result into the conversation, where the person reads it, so
//! its words are the person's (pt-BR), not the model's.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::render::render_result_for_person;
use super::vm_budget::{background_failure_for_person, BACKGROUND_UNAVAILABLE_FOR_PERSON};
use crate::core::bot::pipeline::vm_consent::VmConsent;

/// Everything the background task needs once the turn has moved on.
pub(super) struct BackgroundJob {
    pub state: Arc<AppState>,
    pub ticket: VmConsent,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub bot_uuid: Uuid,
    pub language: String,
    pub code: String,
    pub timeout_ms: u64,
    pub packages: Vec<String>,
}

pub(super) fn spawn(job: BackgroundJob) {
    tokio::spawn(async move {
        let BackgroundJob {
            state,
            ticket,
            user_id,
            session_id,
            bot_uuid,
            language,
            code,
            timeout_ms,
            packages,
        } = job;
        let result = crate::core::bot::agent_vm_hook::exec_with_wake_result(
            ticket, user_id, session_id, bot_uuid, &language, &code, timeout_ms, &packages,
        )
        .await;
        let message = match result {
            Err((status, reason)) => {
                botlog::safe_warn!("background job workspace failure ({status}): {reason}");
                background_failure_for_person(&reason, timeout_ms)
            }
            Ok(None) => BACKGROUND_UNAVAILABLE_FOR_PERSON.to_string(),
            Ok(Some((ok, stdout, stderr))) => {
                let stdout = super::vm_trace::strip_trace_lines(&stdout);
                render_result_for_person(user_id, session_id, ok, &stdout, &stderr).await
            }
        };
        // Saved directly, not through `persist`: another tool's model-only
        // scope in this session at this moment must not swallow the result.
        let mut sessions = state.session_manager.lock().await;
        if let Err(e) = sessions.save_message(session_id, user_id, 2, &message, 2) {
            botlog::safe_warn!("failed to save background result for {session_id}: {e}");
        }
    });
}
