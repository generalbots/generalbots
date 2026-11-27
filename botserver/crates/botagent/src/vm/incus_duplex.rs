//! `incus exec -T` with stdin, stdout and stderr all held open: the
//! transport for agent CLIs that talk to the server over their own stdio.

use botlib::security::command_guard::{CommandGuardError, SafeCommand};
use botsecurity_core::error_sanitizer::{log_and_sanitize_str, SafeErrorResponse};

use crate::duplex::{DuplexError, DuplexOptions, DuplexSession};
use crate::AgentService;

use super::incus::incus_command;
use super::incus_errors::incus_setup_error;

/// Environment variable that marks every process of one session inside the
/// container (it survives `su <user>` without `-`).
const SESSION_TAG_VAR: &str = "GB_DUPLEX_SESSION";

/// Starts `cmd_args` inside `vm_name` as a two-way line session.
///
/// `cmd_args` follows [`super::exec_in_vm_streaming`]: `["sh", "-c", script]`
/// passes the script through the trusted-script path (use it for
/// `su <user> -s /bin/sh -c '… exec <cli> …'`, not `su -`), anything else is
/// passed as guarded arguments. When the session ends, by any path, every
/// process in the container still carrying the session's tag is killed:
/// killing the Windows-side wrapper alone leaves a peer that ignores
/// end-of-file running in the container. Nothing the bridge needs across
/// calls may live in the container's `/tmp` or `/run`: WSL may restart the
/// distro between sessions.
pub async fn exec_in_vm_duplex(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
    options: DuplexOptions,
) -> Result<DuplexSession, SafeErrorResponse> {
    open_duplex(
        super::incus_ready::ready_command(state).await?,
        incus_command(state)
            .map_err(|(_, message)| log_and_sanitize_str(&message, "incus duplex command", None))?,
        vm_name,
        cmd_args,
        options,
    )
}

/// [`exec_in_vm_duplex`] with the two Incus commands given (`exec` runs the
/// session, `reaper` the end-of-session cleanup).
pub(crate) fn open_duplex(
    exec: SafeCommand,
    reaper: SafeCommand,
    vm_name: &str,
    cmd_args: &[String],
    options: DuplexOptions,
) -> Result<DuplexSession, SafeErrorResponse> {
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let cmd = duplex_exec_command(exec, vm_name, &tag, cmd_args)
        .map_err(|error| log_and_sanitize_str(&format!("incus duplex arg: {error}"), "incus duplex", None))?;
    let cleanup = reap_command(reaper, vm_name, &tag)
        .map_err(|error| log_and_sanitize_str(&format!("incus duplex cleanup: {error}"), "incus duplex", None))?;
    DuplexSession::spawn_with_cleanup(&cmd, options, cleanup).map_err(|error| match error {
        DuplexError::Spawn(detail) => match incus_setup_error(&detail) {
            // The remediation text is fixed and free of host detail, so it is
            // safe to hand to the caller; other spawn failures are logged only.
            Some((_, message)) => SafeErrorResponse::new("service_unavailable", message),
            None => log_and_sanitize_str(&format!("incus duplex spawn: {detail}"), "incus duplex", None),
        },
        other => log_and_sanitize_str(&format!("incus duplex: {other}"), "incus duplex", None),
    })
}

/// `<incus> exec <vm> -T --env GB_DUPLEX_SESSION=<tag> -- <command>`: `-T`
/// keeps the pipes plain (no terminal), so lines pass byte for byte.
fn duplex_exec_command(
    base: SafeCommand,
    vm_name: &str,
    tag: &str,
    cmd_args: &[String],
) -> Result<SafeCommand, CommandGuardError> {
    if cmd_args.is_empty() {
        return Err(CommandGuardError::InvalidArgument(
            "no command to run".to_string(),
        ));
    }
    let mut cmd = base
        .arg("exec")?
        .arg(vm_name)?
        .arg("-T")?
        .arg("--env")?
        .trusted_arg(&format!("{SESSION_TAG_VAR}={tag}"))?
        .arg("--")?;
    if let [shell, flag, script] = cmd_args {
        if shell == "sh" && flag == "-c" {
            return cmd.arg("sh")?.arg("-c")?.trusted_shell_script_arg(script);
        }
    }
    for arg in cmd_args {
        cmd = cmd.arg(arg)?;
    }
    Ok(cmd)
}

/// Kills, as root, every process in the container whose environment holds
/// the session tag. `tag` is a fresh hex UUID, never caller text.
fn reap_command(
    base: SafeCommand,
    vm_name: &str,
    tag: &str,
) -> Result<SafeCommand, CommandGuardError> {
    if !tag.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(CommandGuardError::InvalidArgument(
            "bad session tag".to_string(),
        ));
    }
    let script = format!(
        "for d in /proc/[0-9]*; do \
         if tr '\\000' '\\n' < \"$d/environ\" 2>/dev/null | grep -qx '{SESSION_TAG_VAR}={tag}'; \
         then kill -9 \"${{d#/proc/}}\" 2>/dev/null; fi; done; true"
    );
    base.arg("exec")?
        .arg(vm_name)?
        .arg("--")?
        .arg("sh")?
        .arg("-c")?
        .trusted_shell_script_arg(&script)
}

#[cfg(test)]
#[path = "incus_duplex_live_tests.rs"]
mod live_tests;
