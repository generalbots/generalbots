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

/// Binaries that must never reach the cross-platform allowlist: they either
/// open a raw socket to a remote listener or let a caller rewrite the
/// filesystem. `initdb` and `pg_ctl` are deliberately allowed on Windows
/// only (via `WINDOWS_ONLY_COMMANDS`).
#[test]
fn dangerous_binaries_stay_out_of_the_allowlist() {
    for binary in [
        "nc", "netcat", "wget", "dd", "mkfs", "rm", "openssl", "createdb", "systemctl",
    ] {
        assert!(
            SafeCommand::new(binary).is_err(),
            "{binary} must not be executable through SafeCommand"
        );
    }
}

/// `initdb` and `pg_ctl` are in `WINDOWS_ONLY_COMMANDS` — refused on Linux.
#[cfg(not(windows))]
#[test]
fn pg_bootstrap_binaries_refused_on_linux() {
    assert!(SafeCommand::new("initdb").is_err());
    assert!(SafeCommand::new("pg_ctl").is_err());
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

#[test]
fn normalise_command_name_extracts_file_stem() {
    use crate::security::command_validation::normalise_command_name;
    let n = normalise_command_name("/usr/bin/vault");
    assert_eq!(n, "vault");

    let n2 = normalise_command_name("vault");
    assert_eq!(n2, "vault");
}

#[cfg(windows)]
#[test]
fn normalise_command_name_strips_exe_on_windows() {
    use crate::security::command_validation::normalise_command_name;
    assert_eq!(normalise_command_name("vault.exe"), "vault");
    assert_eq!(normalise_command_name("Vault.EXE"), "vault");
    assert_eq!(
        normalise_command_name("C:\\stack\\bin\\vault\\vault.exe"),
        "vault"
    );
    assert_eq!(normalise_command_name("bad.exe.exe"), "bad.exe");
}

/// Every `.exe` entry that existed in the old allowlist must still be
/// accepted after normalisation was introduced.
#[cfg(windows)]
#[test]
fn previous_exe_entries_still_accepted_on_windows() {
    for exe in [
        "zitadel.exe",
        "llama-server.exe",
        "pg_ctl.exe",
        "initdb.exe",
        "psql.exe",
        "tar.exe",
    ] {
        assert!(
            SafeCommand::new(exe).is_ok(),
            "{exe} must be accepted on Windows"
        );
    }
}

#[cfg(windows)]
#[test]
fn windows_full_paths_resolve_against_allowlist() {
    assert!(SafeCommand::new("C:\\stack\\bin\\vault\\vault.exe").is_ok());
    assert!(SafeCommand::new("taskkill").is_ok());
    assert!(SafeCommand::new("taskkill.exe").is_ok());
    assert!(SafeCommand::new("tasklist.exe").is_ok());
    assert!(SafeCommand::new("TASKKILL.EXE").is_ok());
    assert!(SafeCommand::new("initdb.exe").is_ok());
    assert!(SafeCommand::new("pg_ctl.exe").is_ok());
    assert!(SafeCommand::new("initdb").is_ok());
}

#[test]
fn taskkill_is_in_allowlist() {
    assert!(SafeCommand::new("taskkill").is_ok());
    assert!(SafeCommand::new("tasklist").is_ok());
}

/// Image names used by taskkill /IM are arguments, not commands.
/// They must NOT be in the cross-platform allowlist.
#[test]
fn component_image_names_are_not_commands() {
    for name in ["qdrant", "caddy", "coredns", "livekit-server", "stalwart-mail"] {
        assert!(
            SafeCommand::new(name).is_err(),
            "{name} should not be in the allowlist (it is a taskkill argument)"
        );
    }
}

/// `zitadel` (bare) is only in `WINDOWS_ONLY_COMMANDS`, not in the
/// cross-platform list, so it must be rejected on non-Windows.
#[cfg(not(windows))]
#[test]
fn zitadel_bare_rejected_on_linux() {
    assert!(SafeCommand::new("zitadel").is_err());
}
