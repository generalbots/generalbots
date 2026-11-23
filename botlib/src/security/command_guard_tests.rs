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

/// #2621 — the untrusted shell-script path must reject the full adversarial
/// metacharacter corpus, including a bare `&` and the quoting/globbing chars
/// the original fix missed.
#[test]
fn shell_script_arg_rejects_the_adversarial_metacharacter_corpus() {
    let bash = || SafeCommand::new("bash").expect("bash is allowlisted").arg("-c").expect("arg");
    for hostile in [
        "echo a; rm -rf /",
        "echo a | sh",
        "echo a > /etc/passwd",
        "echo a < /etc/shadow",
        "$(curl evil)",
        "`id`",
        "echo a\necho b",
        "echo a\recho b",
        "echo a\0echo b",
        "$HOME",
        "${IFS}cat /etc/passwd",
        "../secret",
        "echo a &",
        "echo a &&& b",
        "echo *",
        "echo ?",
        "echo ~",
        "echo # note",
        "echo 'quoted'",
        "echo \"quoted\"",
        "echo {a,b}",
        "echo [ab]",
        "echo !!",
        "echo (a)",
    ] {
        assert!(
            bash().shell_script_arg(hostile).is_err(),
            "must be rejected: {hostile:?}"
        );
    }
    // The server-constructed installer idiom stays usable on this path.
    assert!(bash().shell_script_arg("cd /tmp/x && ./install.sh").is_ok());
    assert!(bash().shell_script_arg("echo hi").is_ok());
}

/// #2621 — hostile arguments must be refused at the `SafeCommand` boundary,
/// not only by the free function, and environment values are equally guarded.
#[test]
fn hostile_arguments_are_refused_by_safe_command() {
    let corpus = [
        "a;b", "a|b", "a&b", "a$b", "a`b", "a>b", "a<b", "a\nb", "a\rb",
        "a\0b", "a(b", "a)b", "a{b", "a}b", "a*b", "a?b", "a[b", "a]b",
        "a~b", "a!b", "a#b", "a'b", "a\"b", "../../etc/passwd",
    ];
    for hostile in corpus {
        assert!(
            SafeCommand::new("pdftotext")
                .and_then(|c| c.arg(hostile))
                .is_err(),
            "SafeCommand::arg must refuse {hostile:?}"
        );
        assert!(
            SafeCommand::new("pdftotext")
                .and_then(|c| c.args(&[hostile]))
                .is_err(),
            "SafeCommand::args must refuse {hostile:?}"
        );
        assert!(
            SafeCommand::new("pdftotext")
                .and_then(|c| c.env("GB_VALUE", hostile))
                .is_err(),
            "SafeCommand::env must refuse {hostile:?}"
        );
    }
    for safe in ["report.pdf", "My Document 2026.docx", "/tmp/out.png", "a-b_c.d"] {
        assert!(
            SafeCommand::new("pdftotext")
                .and_then(|c| c.arg(safe))
                .is_ok(),
            "ordinary argument must be accepted: {safe:?}"
        );
    }
}

/// A local, constant curl write-out token is not shell syntax. It is the only
/// kind of brace-bearing value allowed through `trusted_arg` (#2621).
#[test]
fn trusted_arg_accepts_local_curl_write_out_token() {
    let cmd = SafeCommand::new("curl")
        .expect("curl is allowlisted")
        .trusted_arg("%{http_code}")
        .expect("local constant token");
    assert!(cmd.arg("https://example.com/health").is_ok());
}

#[test]
fn sudo_and_visudo_are_not_in_the_general_allowlist() {
    assert!(SafeCommand::new("sudo").is_err());
    assert!(SafeCommand::new("visudo").is_err());
}
