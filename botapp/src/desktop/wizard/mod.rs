//! First-run wizard for the desktop edition (issues #1586-#1591).
//!
//! The wizard is a separate static webview that runs *before* the local stack
//! exists. It never depends on botserver, botui, Vault, PostgreSQL, MinIO or the
//! network: detection is a marker file, and provisioning is delegated to
//! `botserver --provision-stack` so the component order and offline rules stay
//! in one place.
//!
//! Split by responsibility: `types` (payloads), `paths` (filesystem locations),
//! `progress` (resume marker and context), `session` (desktop sign-in),
//! `window` (wizard/suite windows) and `handoff` (credentials + completion).

mod commands;
mod handoff;
mod hostcheck;
mod media;
mod paths;
mod progress;
mod provision;
mod session;
mod system;
mod types;
mod window;

pub use commands::*;
pub use handoff::handoff;
pub use hostcheck::HostCheckResult;
pub use paths::{is_desktop_mode, is_first_run};
pub use session::mint_desktop_session;
pub use types::*;
pub use window::{ensure_suite_window, open_window, reveal_suite};

pub(crate) use handoff::write_admin_credentials_draft;
pub(crate) use paths::{
    draft_path, installers_path, log_path, marker_path, progress_path, stack_path, workspace_path,
};
pub(crate) use progress::{context, write_progress, ProgressMarker};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_order_is_dependency_first() {
        let ids: Vec<&str> = COMPONENTS.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(
            ids,
            vec![
                "vault",
                "tables",
                "cache",
                "drive",
                "vector_db",
                "directory",
                "llm",
                "botmodels",
                "botserver"
            ]
        );
    }

    #[test]
    fn progress_marker_round_trips() {
        let marker = ProgressMarker {
            completed: vec!["vault".to_string(), "tables".to_string()],
            step: 5,
        };
        let raw = serde_json::to_string(&marker).unwrap_or_default();
        let back: ProgressMarker = serde_json::from_str(&raw).unwrap_or_default();
        assert_eq!(back.completed.len(), 2);
        assert_eq!(back.step, 5);
    }
}
