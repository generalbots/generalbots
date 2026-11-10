//! Per-user agent container lifecycle (#1580). One running container per
//! logged-in user; each conversation gets its own folder inside it at
//! `/opt/vibe/conversations/{conversation_id}/`. Switching conversations only
//! changes which folder is active — running processes are never signalled, so
//! background work continues while the user moves between conversations.
//! Isolation is per container (per user); folders are namespaces inside that
//! container. Every invocation goes through
//! `botlib::security::command_guard::SafeCommand` and runs blocking work
//! inside `tokio::task::spawn_blocking`.

mod bootstrap;
mod db;
mod exec;
mod identity;
mod incus;
mod incus_errors;
mod launch;
mod lifecycle;
mod profile;
mod retry;
mod workspace;

use axum::http::StatusCode;

pub use exec::{exec_in_conversation, exec_in_conversation_streamed, run_root_shell};
pub(crate) use identity::allow_unregistered_identities;
pub use identity::sanitize_id;
pub use incus::{exec_in_vm, exec_in_vm_streaming, ExecOutcome};
pub(crate) use incus::{is_running, wait_for_running};
pub use lifecycle::{
    delete_vm, ensure_vm, ensure_vm_by_session, expiry_from_now, get_user_vm, stop_vm,
    stop_vm_for_user,
};
pub use workspace::{
    delete_conversation_folder, ensure_conversation_folder, list_conversation_output,
    switch_conversation,
};

pub const STATUS_PROVISIONING: &str = "provisioning";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_ERROR: &str = "error";

const CONVERSATIONS_ROOT: &str = "/opt/vibe/conversations";

/// Sanitized failure text returned to API/chat callers while the concrete Incus
/// detail stays in the log. The previous "Agent VM operation failed" made a
/// transient cold-start race look like a permanently broken host (issue #1798).
const INTERNAL_MESSAGE: &str =
    "the agent workspace operation failed; the Incus error detail is in the botserver log";

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        INTERNAL_MESSAGE.to_string(),
    )
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

#[deprecated(note = "use sanitize_id instead")]
pub fn sanitize_session_id(session_id: &str) -> Result<String, (StatusCode, String)> {
    sanitize_id(session_id)
}
