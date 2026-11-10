//! Unit tests for the `SafeCommand` allowlist.
//!
//! Split out of `command_guard.rs` to keep that file inside the project size
//! limit; the allowlist itself stays beside the guard implementation.

use super::*;
use crate::security::{sanitize_filename, validate_argument};

#[test]
fn test_safe_command_allowed() {
    assert!(SafeCommand::new("pdftotext").is_ok());
    assert!(SafeCommand::new("pandoc").is_ok());
    assert!(SafeCommand::new("nvidia-smi").is_ok());
}

#[test]
fn test_safe_command_disallowed() {
    assert!(SafeCommand::new("wget").is_err());
    assert!(SafeCommand::new("nc").is_err());
    assert!(SafeCommand::new("netcat").is_err());
    assert!(SafeCommand::new("dd").is_err());
    assert!(SafeCommand::new("mkfs").is_err());
}

/// Binaries that must never reach the allowlist: they either open a raw socket
/// to a remote listener or let a caller rewrite the filesystem. Every one of
/// them was removed from `ALLOWED_COMMANDS`; these assertions pin that down so
/// a future entry does not silently reintroduce host command execution.
#[test]
fn dangerous_binaries_stay_out_of_the_allowlist() {
    for binary in [
        "nc",
        "netcat",
        "wget",
        "dd",
        "mkfs",
        "rm",
        "openssl",
        "initdb",
        "pg_ctl",
        "createdb",
        "systemctl",
    ] {
        assert!(
            SafeCommand::new(binary).is_err(),
            "{binary} must not be executable through SafeCommand"
        );
    }
}

/// `rclone`, `osascript` and `rm` are reachable from the desktop shell, but
/// only through the guard in `botapp::desktop::safe_command`. The server-side
/// guard has no caller for them, so it must keep refusing them.
#[test]
fn desktop_only_binaries_are_not_server_allowed() {
    for binary in ["rclone", "osascript", "rm"] {
        assert!(SafeCommand::new(binary).is_err());
    }
}

#[test]
fn shell_metacharacters_are_refused_in_arguments() {
    for hostile in [
        "file;rm -rf /",
        "file|cat /etc/passwd",
        "file&&whoami",
        "file`id`",
        "file$(whoami)",
        "file>out.txt",
        "file<in.txt",
    ] {
        assert!(
            validate_argument(hostile).is_err(),
            "{hostile} must be refused"
        );
    }
}

#[test]
fn traversal_patterns_are_refused_in_arguments() {
    assert!(validate_argument("../../etc/passwd").is_err());
    assert!(validate_argument("a//b").is_err());
    assert!(validate_argument("a\\\\b").is_err());
}

#[test]
fn ordinary_arguments_pass() {
    assert!(validate_argument("report.pdf").is_ok());
    assert!(validate_argument("My Document 2026.docx").is_ok());
    assert!(validate_argument("/tmp/output.png").is_ok());
}

#[test]
fn empty_and_overlong_arguments_are_refused() {
    assert!(validate_argument("").is_err());
    assert!(validate_argument(&"a".repeat(4097)).is_err());
    assert!(validate_argument(&"a".repeat(4096)).is_ok());
}

#[test]
fn urls_may_carry_query_separators() {
    assert!(validate_argument("https://example.com/path?a=1&b=2").is_ok());
    assert!(validate_argument("http://example.com/x=y").is_ok());
    // A shell metacharacter that is not part of a URL is still refused.
    assert!(validate_argument("https://example.com/;rm -rf /").is_err());
}

#[test]
fn sanitize_filename_keeps_only_safe_characters() {
    assert_eq!(sanitize_filename("report.pdf"), "report.pdf");
    assert!(!sanitize_filename("../etc/passwd").contains('/'));
    assert!(!sanitize_filename("a;b.txt").contains(';'));
    assert!(!sanitize_filename("a\u{0}b.txt").contains('\u{0}'));
}
