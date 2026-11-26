//! VM entry points that take a consent ticket (`vm_consent::VmConsent`):
//! running a program and writing a file in the conversation folder.
//! Split from `agent_vm_hook.rs` so that file stays under the size limit.

use super::*;

/// Execute code in the user's VM within the conversation folder. Returns
/// (success, stdout, stderr) or None if the service is absent.
pub async fn exec_in_user_vm(
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
) -> Option<(bool, String, String)> {
    exec_in_user_vm_with_timeout(ticket, user_id, session_id, language, code, 60_000, &[]).await
}

/// Same as [`exec_in_user_vm`] with an explicit timeout (bounded by the
/// native tool to 120s) and optional third-party packages to install first.
pub async fn exec_in_user_vm_with_timeout(
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Option<(bool, String, String)> {
    exec_in_user_vm_result(&ticket, user_id, session_id, language, code, timeout_ms, packages)
        .await
        .ok()
        .flatten()
}

/// Writes a file into the conversation folder (#1584 Drive → VM).
#[cfg(feature = "agent-vm")]
pub async fn write_session_file(
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    rel_path: &str,
    bytes: &[u8],
) -> Result<usize, String> {
    if !ticket.covers(session_id) {
        return Err(not_allowed_here().1);
    }
    if workspace_disabled(session_id) {
        return Err(crate::core::bot::pipeline::agent_turn::DISABLED_MESSAGE.to_string());
    }
    if rig_fake::enabled() {
        return Ok(bytes.len());
    }
    let service = SERVICE
        .get()
        .ok_or_else(|| "workspace unavailable".to_string())?;
    let row = botagent::vm::get_user_vm(service, &user_id)
        .map_err(|(_, e)| e)?
        .ok_or_else(|| "no workspace".to_string())?;
    if row.status != botagent::vm::STATUS_RUNNING {
        return Err("workspace is not running".to_string());
    }
    botagent::transfer::write_file(
        service,
        &row.vm_name,
        &session_id.to_string(),
        rel_path,
        bytes,
    )
    .await
    .map_err(|(_, e)| e)
}

/// What a live terminal run produced: its whole output (stdout, then
/// stderr) and the program's exit code (`timeout` reports 124 when the limit
/// was reached).
#[cfg(feature = "agent-vm")]
pub(crate) struct TerminalRun {
    pub output: String,
    pub exit_code: Option<i32>,
}

/// Runs `command` in the person's VM, streaming each output line to the
/// conversation's run channel while it runs. Takes the consent ticket like
/// the other VM entry points, so a terminal run cannot start without a
/// decision for this conversation. It goes through the same guarded Incus
/// command as every other VM exec (`AGENT_INCUS_BIN`, so the WSL wrapper on
/// Windows).
#[cfg(feature = "agent-vm")]
pub(crate) async fn run_in_terminal(
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    command: String,
    timeout_secs: u64,
) -> Result<TerminalRun, String> {
    use botagent::stream::ExecEvent;

    if !ticket.covers(session_id) {
        return Err(not_allowed_here().1);
    }
    let vm_name = user_vm_name(user_id)
        .await
        .ok_or_else(|| "No running workspace VM. Open the workspace first.".to_string())?;
    let service = SERVICE
        .get()
        .ok_or_else(|| "the agent workspace subsystem is unavailable".to_string())?;
    let thread_id = crate::core::bot::runs::bus::thread_id(session_id);
    let run_id = crate::core::bot::runs::bus::bus()
        .current_run_id(&thread_id)
        .unwrap_or_else(|| { log::debug!("using default for current run id"); Default::default() });
    let (events, mut lines) = tokio::sync::mpsc::unbounded_channel();
    let publisher = tokio::spawn(async move {
        while let Some(event) = lines.recv().await {
            match event {
                ExecEvent::Stdout { line } => publish_line(&thread_id, &run_id, &line, "stdout"),
                ExecEvent::Stderr { line } => publish_line(&thread_id, &run_id, &line, "stderr"),
                _ => {}
            }
        }
    });
    let args = ["sh".to_string(), "-c".to_string(), terminal_script(&command, timeout_secs)];
    let limit = std::time::Duration::from_secs(timeout_secs.saturating_add(30));
    let outcome = botagent::vm::exec_in_vm_streaming(service, &vm_name, &args, Some(limit), None, Some(events)).await;
    if let Err(join_err) = publisher.await
    {
        log::debug!("publisher task failed: {join_err}");
    }
    let outcome = outcome.map_err(|(_, error)| error)?;
    Ok(TerminalRun {
        output: format!("{}{}", outcome.stdout, outcome.stderr),
        exit_code: outcome.exit_code,
    })
}

/// The in-VM script: the command reaches `bash -l` as a 0600 file decoded
/// from base64, so no quoting of the person's command crosses the host
/// shell or the WSL wrapper.
#[cfg(feature = "agent-vm")]
fn terminal_script(command: &str, timeout_secs: u64) -> String {
    use base64::Engine;

    let encoded = base64::engine::general_purpose::STANDARD.encode(command.as_bytes());
    let file = format!("/tmp/gb-term-{}.sh", uuid::Uuid::new_v4().simple());
    format!(
        "umask 077; printf '%s' {encoded} | base64 -d > {file} && timeout {timeout_secs}s bash -l {file}; rc=$?; rm -f {file}; exit $rc"
    )
}

#[cfg(feature = "agent-vm")]
fn publish_line(thread_id: &str, run_id: &str, line: &str, stream: &str) {
    let line = line.trim_end();
    if line.is_empty() {
        return;
    }
    let bus = crate::core::bot::runs::bus::bus();
    if !run_id.is_empty() {
        bus.publish(
            thread_id,
            serde_json::json!({ "type": "terminal_output", "run_id": run_id, "data": { "line": line } }),
        );
    }
    bus.publish(
        thread_id,
        serde_json::json!({
            "type": "CUSTOM",
            "run_id": run_id,
            "data": { "name": "gb.command_output", "payload": { "line": line, "stream": stream } },
        }),
    );
}

#[cfg(all(test, feature = "agent-vm"))]
mod terminal_tests {
    use super::terminal_script;

    #[test]
    fn the_command_crosses_as_base64_only() {
        let script = terminal_script("echo \"it's\" | grep -v $HOME", 60);
        assert!(!script.contains("it's") && !script.contains("$HOME"));
        assert!(script.contains("timeout 60s bash -l /tmp/gb-term-"));
        assert!(script.ends_with("exit $rc"));
    }
}
