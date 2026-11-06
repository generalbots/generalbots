//! Interactive terminal sessions for the web terminal (#1588).
//!
//! `session` owns the PTY-backed shell, `manager` the per-process registry and
//! `sanitize` the cwd validation shared by the routes. `TERMINAL_BUFFER_LINES`
//! and the public types are re-exported here so `crate::api::terminal::*`
//! keeps working unchanged.

pub const TERMINAL_BUFFER_LINES: usize = 512;

pub mod routes;

mod manager;
mod sanitize;
mod session;

pub use manager::TerminalManager;
pub use sanitize::{default_shell, sanitize_container_cwd, sanitize_cwd};
pub use session::{TerminalLine, TerminalSession};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_cwd_falls_back_when_missing() {
        let cwd = sanitize_cwd("/definitely/not/a/dir/xyz");
        assert!(std::path::Path::new(&cwd).is_dir());
        let tmp = std::env::temp_dir();
        assert_eq!(
            sanitize_cwd(tmp.to_str().unwrap()),
            tmp.to_str().unwrap().to_string()
        );
    }

    #[test]
    fn default_shell_is_non_empty() {
        assert!(!default_shell().is_empty());
    }

    #[test]
    fn history_ring_bounds_lines() {
        let session = TerminalSession::new("t1".into(), "/bin/sh".into(), "/tmp".into());
        for i in 0..(TERMINAL_BUFFER_LINES + 50) {
            session.append_output(format!("line {i}\n"));
        }
        let history = session.history();
        assert_eq!(history.len(), TERMINAL_BUFFER_LINES);
        assert_eq!(history.first().map(|l| l.data.as_str()), Some("line 50\n"));
    }

    #[test]
    fn not_running_when_no_exit_code_set() {
        let session = TerminalSession::new("t2".into(), "/bin/sh".into(), "/tmp".into());
        assert!(session.is_running());
    }
}
