//! `incus exec -T` with stdin, stdout and stderr all held open: the
//! transport for agent CLIs that talk to the server over their own stdio.

use axum::http::StatusCode;
use botlib::security::command_guard::{CommandGuardError, SafeCommand};

use crate::duplex::{DuplexError, DuplexOptions, DuplexSession};
use crate::AgentService;

use super::incus::incus_command;
use super::incus_errors::incus_setup_error;
use super::internal;

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
) -> Result<DuplexSession, (StatusCode, String)> {
    open_duplex(
        super::incus_ready::ready_command(state).await?,
        incus_command(state)?,
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
) -> Result<DuplexSession, (StatusCode, String)> {
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let cmd = duplex_exec_command(exec, vm_name, &tag, cmd_args)
        .map_err(|error| internal(format!("incus duplex arg: {error}")))?;
    let cleanup = reap_command(reaper, vm_name, &tag)
        .map_err(|error| internal(format!("incus duplex cleanup: {error}")))?;
    DuplexSession::spawn_with_cleanup(&cmd, options, cleanup).map_err(|error| match error {
        DuplexError::Spawn(detail) => incus_setup_error(&detail)
            .unwrap_or_else(|| internal(format!("incus duplex spawn: {detail}"))),
        other => internal(format!("incus duplex: {other}")),
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
