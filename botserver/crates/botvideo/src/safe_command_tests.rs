//! Offline unit tests for the botvideo command guard (#2067).
//!
//! `SafeCommand` is the only way this crate spawns a process. Every rule it
//! enforces — the executable allowlist, the forbidden shell characters, the
//! `trusted_arg` pattern list, the `-c` precondition for shell scripts and the
//! path allowlist — is a security boundary, and every one of them is a pure
//! decision. `execute()` is never called here: the tests assert that a hostile
//! input is rejected *before* a process could start.

use super::{validate_argument, CommandGuardError, SafeCommand};
use std::path::Path;

/// A fresh, allowed command. `SafeCommand` is not `Clone`, so each case builds
/// its own value — exactly like the production call sites.
fn ffmpeg() -> SafeCommand {
    SafeCommand::new("ffmpeg").expect("ffmpeg is allowlisted")
}

fn bash_with_c() -> SafeCommand {
    SafeCommand::new("bash")
        .expect("bash is allowlisted")
        .arg("-c")
        .expect("-c is a valid argument")
}

#[test]
fn the_allowlist_accepts_the_media_toolchain() {
    for command in ["ffmpeg", "ffprobe", "convert", "bash", "sh", "python3"] {
        assert!(SafeCommand::new(command).is_ok(), "{command} must be allowed");
    }
}

#[test]
fn the_allowlist_accepts_an_absolute_path_to_an_allowed_binary() {
    // The allowlist matches the file name, not the whole path.
    assert!(SafeCommand::new("/usr/bin/ffmpeg").is_ok());
    assert!(SafeCommand::new("/usr/local/bin/ffprobe").is_ok());
}

#[test]
fn the_allowlist_rejects_everything_else() {
    for command in ["rm", "/bin/rm", "sudo", "shutdown", "", "ffmpeg; rm"] {
        let err = SafeCommand::new(command)
            .err()
            .expect("command must be rejected");
        assert_eq!(
            err,
            CommandGuardError::CommandNotAllowed(command.to_string())
        );
    }
}

#[test]
fn empty_and_overlong_arguments_are_rejected() {
    assert_eq!(
        validate_argument("").err().expect("empty"),
        CommandGuardError::InvalidArgument("Empty argument".to_string())
    );
    let long = "a".repeat(4097);
    assert_eq!(
        validate_argument(&long).err().expect("too long"),
        CommandGuardError::InvalidArgument("Argument too long".to_string())
    );
    assert!(validate_argument(&"a".repeat(4096)).is_ok());
}

#[test]
fn shell_metacharacters_are_rejected_in_arguments() {
    for hostile in [
        "a;b", "a|b", "a&b", "$HOME", "`id`", "a<b", "a>b", "a\nb", "a\rb", "a\0b",
    ] {
        assert!(
            matches!(
                validate_argument(hostile),
                Err(CommandGuardError::ShellInjectionAttempt(_))
            ),
            "{hostile:?} must be rejected as shell injection"
        );
    }
}

#[test]
fn ordinary_arguments_and_paths_are_accepted() {
    for safe in ["-y", "-i", "/tmp/in.mp4", "scale=1280:-2", "a b", "50%", "x=y"] {
        assert!(validate_argument(safe).is_ok(), "{safe:?} must be accepted");
    }
}

#[test]
fn trusted_args_reject_the_documented_dangerous_patterns() {
    for hostile in [
        "$(id)", "`id`", "a && b", "a || b", "a >> out", "a << in", "../etc/passwd",
    ] {
        assert!(
            matches!(
                ffmpeg().trusted_arg(hostile),
                Err(CommandGuardError::ShellInjectionAttempt(_))
            ),
            "{hostile:?} must be rejected"
        );
    }
}

#[test]
fn trusted_args_accept_plain_values() {
    for safe in ["-c:v", "libx264", "/tmp/a b.mp4", "1", "-preset"] {
        assert!(
            ffmpeg().trusted_arg(safe).is_ok(),
            "{safe:?} must be accepted"
        );
    }
    // Empty and overlong values are still refused.
    assert!(ffmpeg().trusted_arg("").is_err());
    assert!(ffmpeg().trusted_arg(&"a".repeat(4097)).is_err());
}

#[test]
fn shell_scripts_require_a_shell_command_and_the_dash_c_flag() {
    assert_eq!(
        ffmpeg().shell_script_arg("echo hi").err().expect("not a shell"),
        CommandGuardError::InvalidArgument(
            "shell_script_arg only allowed for bash/sh commands".to_string()
        )
    );

    let bash = SafeCommand::new("bash").expect("bash is allowlisted");
    assert_eq!(
        bash.shell_script_arg("echo hi").err().expect("no -c"),
        CommandGuardError::InvalidArgument(
            "shell_script_arg requires -c flag to be set first".to_string()
        )
    );

    assert!(bash_with_c().shell_script_arg("echo hi").is_ok());
    assert!(bash_with_c().shell_script_arg("").is_err());
    assert!(bash_with_c().shell_script_arg(&"a".repeat(8193)).is_err());
    for hostile in ["$(id)", "`id`", "cd .."] {
        assert!(
            matches!(
                bash_with_c().shell_script_arg(hostile),
                Err(CommandGuardError::ShellInjectionAttempt(_))
            ),
            "{hostile:?} must be rejected inside a shell script"
        );
    }
}

#[test]
fn paths_outside_the_allowlist_are_traversal_errors() {
    assert!(ffmpeg().path_arg(Path::new("/tmp/in.mp4")).is_ok());
    assert!(ffmpeg().path_arg(Path::new("/var/tmp/in.mp4")).is_ok());

    let err = ffmpeg()
        .path_arg(Path::new("/etc/passwd"))
        .err()
        .expect("/etc/passwd must be rejected");
    assert_eq!(err, CommandGuardError::PathTraversal("/etc/passwd".to_string()));
}

#[test]
fn working_directories_are_validated_like_paths() {
    assert!(ffmpeg().working_dir(Path::new("/tmp")).is_ok());
    assert!(matches!(
        ffmpeg().working_dir(Path::new("/etc")),
        Err(CommandGuardError::PathTraversal(_))
    ));
}

#[test]
fn environment_pairs_are_validated_with_the_argument_rules() {
    assert!(ffmpeg().env("GB_MODE", "fast").is_ok());
    assert!(matches!(
        ffmpeg().env("GB;MODE", "fast"),
        Err(CommandGuardError::ShellInjectionAttempt(_))
    ));
    assert!(matches!(
        ffmpeg().env("GB_MODE", "$(id)"),
        Err(CommandGuardError::ShellInjectionAttempt(_))
    ));
}

#[test]
fn guard_errors_render_a_stable_message() {
    assert_eq!(
        CommandGuardError::CommandNotAllowed("rm".to_string()).to_string(),
        "Command not in allowlist: rm"
    );
    assert_eq!(
        CommandGuardError::PathTraversal("/etc/passwd".to_string()).to_string(),
        "Path traversal detected: /etc/passwd"
    );
    assert_eq!(
        CommandGuardError::InvalidArgument("Empty argument".to_string()).to_string(),
        "Invalid argument: Empty argument"
    );
    assert_eq!(
        CommandGuardError::ExecutionFailed("boom".to_string()).to_string(),
        "Command execution failed: boom"
    );
    assert_eq!(
        CommandGuardError::ShellInjectionAttempt("x".to_string()).to_string(),
        "Shell injection attempt detected: x"
    );
    // `From<CommandGuardError> for String` keeps the message when converted.
    let message: String = CommandGuardError::CommandNotAllowed("rm".to_string()).into();
    assert_eq!(message, "Command not in allowlist: rm");
}
