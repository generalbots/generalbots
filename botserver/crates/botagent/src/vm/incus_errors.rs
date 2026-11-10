//! Incus capability errors: missing binary/daemon and socket permissions.
//!
//! Split out of `incus.rs` to keep both files under the 450-line limit.
//! #2086 — the Windows phrases matter: without them a host with no Incus
//! retried `incus list` five times (≈30 s) before every LLM call.

use axum::http::StatusCode;

/// Remediation returned when the host has no usable Incus daemon. The text is
/// fixed and free of host paths/identities, so it is safe to send to API
/// callers (the raw command output stays in the log).
pub(super) const INCUS_MISSING_MESSAGE: &str = "Incus is not available on this host: install the incus package (Debian/Ubuntu: apt install incus), run `incus admin init --minimal`, add the botserver user to the incus-admin group, then restart the botserver service";

/// Remediation for the socket-permission failure: `/var/lib/incus/unix.socket`
/// is `root:incus-admin 0660` and the daemon checks the peer credentials, so
/// the group must be present in the running process, not only in `/etc/group`.
const INCUS_PERMISSION_MESSAGE: &str = "The botserver service cannot access the Incus daemon: add the botserver user to the incus-admin group (sudo usermod -aG incus-admin botserver) and restart the botserver service";

/// Maps an Incus failure to an actionable capability error when it is caused
/// by host setup (missing binary/daemon or socket permission). Returns `None`
/// for everything else so callers keep the generic sanitized message and its
/// detail stays in the log.
pub(super) fn incus_setup_error(detail: &str) -> Option<(StatusCode, String)> {
    let lower = detail.to_ascii_lowercase();
    let permission_denied = lower.contains("permission denied")
        || lower.contains("needed permissions")
        || lower.contains("access denied")
        || lower.contains("os error 13");
    if permission_denied {
        tracing::error!("incus socket permission denied: {detail}");
        return Some((
            StatusCode::SERVICE_UNAVAILABLE,
            INCUS_PERMISSION_MESSAGE.to_string(),
        ));
    }
    let daemon_missing = lower.contains("no such file or directory")
        || lower.contains("os error 2")
        // #2086 — Windows reports a missing binary differently. Without these
        // phrases the chat pipeline retried `incus list` five times (≈30 s)
        // before every LLM call on a host that has no Incus at all.
        || lower.contains("cannot find the file specified")
        || lower.contains("not recognized as an internal or external command")
        || lower.contains("os error 3")
        || lower.contains("failed to connect to local daemon")
        || lower.contains("daemon is not running")
        || lower.contains("cannot connect to the incus daemon");
    if daemon_missing {
        tracing::error!("incus unavailable: {detail}");
        return Some((
            StatusCode::SERVICE_UNAVAILABLE,
            INCUS_MISSING_MESSAGE.to_string(),
        ));
    }
    None
}

#[cfg(test)]
mod incus_error_tests {
    use super::{incus_setup_error, INCUS_MISSING_MESSAGE, INCUS_PERMISSION_MESSAGE};
    use axum::http::StatusCode;

    #[test]
    fn missing_binary_is_actionable() {
        let (code, message) =
            incus_setup_error("Command execution failed: No such file or directory (os error 2)")
                .expect("missing binary must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_MISSING_MESSAGE);
        assert!(message.contains("incus-admin"));
    }

    #[test]
    fn socket_permission_is_actionable() {
        let (code, message) = incus_setup_error(
            "Error: You don't have the needed permissions to talk with the incus daemon",
        )
        .expect("socket permission must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_PERMISSION_MESSAGE);
        assert!(message.contains("usermod -aG incus-admin"));
    }

    #[test]
    fn unreachable_daemon_is_actionable() {
        let (code, message) = incus_setup_error(
            "Error: Failed to connect to local daemon: dial unix socket: no such file or directory",
        )
        .expect("missing daemon must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_MISSING_MESSAGE);
    }

    #[test]
    fn unrelated_failures_keep_the_generic_error() {
        assert!(incus_setup_error("Error: Instance ag-u1234 is not running").is_none());
        assert!(incus_setup_error("Error: Image not found").is_none());
    }

    #[test]
    fn windows_missing_binary_is_a_capability_error() {
        // #2086 — Windows phrases that used to fall through and be retried five
        // times per chat message.
        for detail in [
            "Command execution failed: The system cannot find the file specified. (os error 3)",
            "'incus' is not recognized as an internal or external command, operable program or batch file.",
        ] {
            let (code, message) =
                incus_setup_error(detail).expect("windows missing binary must map to setup error");
            assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(message, INCUS_MISSING_MESSAGE);
        }
    }
}
