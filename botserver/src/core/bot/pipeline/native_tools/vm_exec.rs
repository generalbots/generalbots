//! `vm_exec`: runs a program in the user's agent VM, including the background
//! path, the one-shot syntax repair and the visible status frames.

use super::super::types::SessionIdentity;
use super::super::workspace_progress::{WorkspaceProgress, PHASE_TOOL};
use super::program_repair::repair_program;
use super::render::render_result;
use super::send;

/// Time allowed beyond the requested program timeout to wake or provision the
/// workspace and return its result.
const WORKSPACE_OVERHEAD_SECS: u64 = 120;

/// Overall budget for one `vm_exec` call, in milliseconds.
fn workspace_budget_ms(timeout_ms: u64) -> u64 {
    timeout_ms.saturating_add(WORKSPACE_OVERHEAD_SECS * 1_000)
}

/// A run timeout means the program took too long; it says nothing about the
/// sandbox host being down, so it must not advise a restart or a wait.
fn budget_exhausted_message(timeout_ms: u64) -> String {
    let budget_secs = workspace_budget_ms(timeout_ms) / 1_000;
    format!(
        "The workspace run exceeded its {budget_secs}-second budget ({timeout_ms} ms program timeout plus {WORKSPACE_OVERHEAD_SECS} s of workspace overhead). This is a run timeout, not a host outage. Retry, reduce the task scope or split it into smaller steps, and check the budget: raise `timeout_ms` (max 120000 ms) if the work legitimately needs longer."
    )
}

fn is_budget_exhausted(reason: &str) -> bool {
    reason.to_ascii_lowercase().contains("run budget elapsed")
}

/// The truthful replacement for the dead-end "workspace is not available"
/// phrase: show the concrete reason the VM subsystem reports (status is
/// logged only, never shown), then one short pointer forward. A run-budget
/// timeout is a distinct failure and gets its own guidance.
fn workspace_failure_message(reason: &str, timeout_ms: u64) -> String {
    if is_budget_exhausted(reason) {
        return budget_exhausted_message(timeout_ms);
    }
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return "The workspace could not run this program. Start a new message and try again."
            .to_string();
    }
    let clipped: String = trimmed.chars().take(320).collect();
    let clipped = if trimmed.chars().count() > 320 {
        format!("{clipped}…")
    } else {
        clipped
    };
    // The reason is already actionable (a cold start says it is still pulling
    // the image; a host problem names the missing Incus setup). Do not append
    // the old "restart the app" advice: it made a transient first-run race look
    // like a permanently dead sandbox host (issue #1798).
    format!("The workspace could not run this program: {clipped}. Start a new message to retry.")
}

pub(super) async fn run_vm_exec(identity: &SessionIdentity<'_>, args: &str) {
    let parsed: serde_json::Value = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(e) => {
            send(
                identity,
                &format!("Could not read the execution request: {e}"),
            )
            .await;
            return;
        }
    };

    let language = parsed
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("python");
    let raw_code = parsed.get("code").and_then(|v| v.as_str()).unwrap_or("");
    // Models sometimes leave the JSON envelope glued to the program: the code
    // string arrives ending in `"} or \"} (the closing quote/brace of the
    // request), which turns a correct program into a SyntaxError. Strip that
    // artifact; a real program never ends with those bytes.
    let trimmed = raw_code.trim_end();
    let code = trimmed
        .strip_suffix("\"}")
        .or_else(|| trimmed.strip_suffix("\"}"))
        .unwrap_or(trimmed)
        .trim_end();
    if code.trim().is_empty() {
        send(
            identity,
            "The execution request had no code. Send the program to run in `code`.",
        )
        .await;
        return;
    }
    if !matches!(language, "python" | "node" | "shell") {
        send(
            identity,
            "Unsupported language. Use `python`, `node` or `shell`.",
        )
        .await;
        return;
    }

    let timeout_ms = parsed
        .get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(60_000)
        .clamp(1_000, 120_000);
    // Declared third-party packages: installed before the program runs, so the
    // model never shells out to pip/npm inside generated code.
    let packages: Vec<String> = parsed
        .get("packages")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    let background = parsed
        .get("background")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if background {
        // #1580: a long job must outlive the tab. Run it in its own task and
        // save the outcome into the conversation it belongs to.
        let worker_state = state.clone();
        let code = code.to_string();
        let language = language.to_string();
        tokio::spawn(async move {
            let result = crate::core::bot::agent_vm_hook::exec_with_wake_result(
                user_id, session_id, bot_uuid, &language, &code, timeout_ms, &packages,
            )
            .await;
            let message = match result {
                Err((status, reason)) => {
                    botlog::safe_warn!("background job workspace failure ({status}): {reason}");
                    workspace_failure_message(&reason, timeout_ms)
                }
                Ok(None) => {
                    "The workspace became unavailable and the background job could not finish."
                        .to_string()
                }
                Ok(Some((ok, stdout, stderr))) => {
                    render_result(user_id, session_id, ok, &stdout, &stderr).await
                }
            };
            super::persist(&worker_state, session_id, user_id, &message).await;
        });
        let started = "Started in the background. The result will be saved to this conversation.";
        send(identity, started).await;
        super::persist(&state, session_id, user_id, started).await;
        return;
    }

    // Mirror the workspace activity into the run card (#1693): the summary
    // says what the VM is doing while it runs, so a long generation never
    // looks stuck even before the first output line.
    let progress_thread = session_id.to_string();
    let progress_run = crate::core::bot::runs::bus::bus()
        .current_run_id(&progress_thread)
        .unwrap_or_default();
    if !progress_run.is_empty() {
        crate::core::bot::runs::bus::bus().publish(
            &progress_thread,
            serde_json::json!({
                "type": "CUSTOM",
                "run_id": progress_run,
                "data": {
                    "name": "gb.tool_progress",
                    "payload": { "tool": "vm_exec", "phase": "start", "detail": language },
                },
            }),
        );
    }

    // #1807 — split the long workspace call into visible stages. The pure
    // program execution is one step; declared dependency installation is its
    // own step so a slow pip/apt run is distinguishable from slow code.
    let progress = WorkspaceProgress::new(identity);
    if let Some(progress) = progress.as_ref() {
        if !packages.is_empty() {
            progress
                .start(
                    PHASE_TOOL,
                    "package_install",
                    "Installing packages",
                    &packages.join(", "),
                )
                .await;
        }
        progress
            .start(PHASE_TOOL, "execute", "Running the program", language)
            .await;
    }

    // Every run gets a visible status in the chat: the turn otherwise looks
    // frozen while the program executes, and users cannot tell whether the bot
    // is working or stuck.
    {
        let note = if packages.is_empty() {
            format!("⏳ Executando no workspace ({language})…")
        } else {
            format!(
                "⏳ Executando no workspace ({language})… instalando {}",
                packages.join(", ")
            )
        };
        send(identity, &note).await;
    }

    botlog::safe_info!(
        "vm_exec start: session={session_id} language={language} bytes={} packages={:?}",
        code.len(),
        packages
    );
    let budget_ms = workspace_budget_ms(timeout_ms);
    let budget = std::time::Duration::from_millis(budget_ms);
    let result = match tokio::time::timeout(
        budget,
        crate::core::bot::agent_vm_hook::exec_with_wake_result(
            user_id, session_id, bot_uuid, language, code, timeout_ms, &packages,
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            botlog::safe_warn!(
                "vm_exec exceeded its overall budget of {budget_ms} ms for session {session_id}"
            );
            Err((
                504,
                "the run budget elapsed before the workspace answered".to_string(),
            ))
        }
    };
    let succeeded = matches!(&result, Ok(Some((true, _, _))));
    if let Some(progress) = progress.as_ref() {
        let status = if succeeded { "ok" } else { "error" };
        if !packages.is_empty() {
            progress
                .finish(
                    PHASE_TOOL,
                    "package_install",
                    status,
                    if succeeded {
                        "Packages ready"
                    } else {
                        "Package install failed"
                    },
                )
                .await;
        }
        progress
            .finish(
                PHASE_TOOL,
                "execute",
                status,
                if succeeded {
                    "Program finished"
                } else {
                    "Program failed"
                },
            )
            .await;
    }
    if !progress_run.is_empty() {
        let ok = succeeded;
        crate::core::bot::runs::bus::bus().publish(
            &progress_thread,
            serde_json::json!({
                "type": "CUSTOM",
                "run_id": progress_run,
                "data": {
                    "name": "gb.tool_progress",
                    "payload": { "tool": "vm_exec", "phase": "done", "ok": ok },
                },
            }),
        );
    }
    botlog::safe_info!(
        "vm_exec done: session={session_id} ok={:?}",
        result
            .as_ref()
            .ok()
            .and_then(|outcome| outcome.as_ref().map(|r| r.0))
    );

    match result {
        Err((status, reason)) => {
            botlog::safe_warn!(
                "vm_exec workspace failure for session {session_id}: status={status} reason={reason}"
            );
            let message = workspace_failure_message(&reason, timeout_ms);
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
        }
        Ok(None) => {
            let message =
                "The workspace is not available right now. Start a new message and try again.";
            send(identity, message).await;
            super::persist(state, session_id, user_id, message).await;
        }
        Ok(Some((ok, stdout, stderr))) => {
            if !ok && super::program_repair::looks_like_syntax_error(&stderr) {
                send(
                    identity,
                    "🔧 The generated program had a syntax error; fixing it automatically…",
                )
                .await;
                // One repair pass: long generated programs occasionally carry a
                // broken string/indent and the user should not have to retype
                // the request.
                if let Some(fixed) = repair_program(state, bot_uuid, language, code, &stderr).await
                {
                    let retry = crate::core::bot::agent_vm_hook::exec_in_user_vm_with_timeout(
                        user_id, session_id, language, &fixed, timeout_ms, &packages,
                    )
                    .await;
                    if let Some((true, fixed_out, _)) = retry {
                        if let Some(progress) = progress.as_ref() {
                            progress
                                .start(
                                    PHASE_TOOL,
                                    "save",
                                    "Saving the output",
                                    "Collecting the generated files",
                                )
                                .await;
                        }
                        let message =
                            render_result(user_id, session_id, true, &fixed_out, "").await;
                        send(identity, &message).await;
                        super::persist(state, session_id, user_id, &message).await;
                        if let Some(progress) = progress.as_ref() {
                            progress.finish(PHASE_TOOL, "save", "ok", "").await;
                        }
                        return;
                    }
                }
            }
            if let Some(progress) = progress.as_ref() {
                progress
                    .start(
                        PHASE_TOOL,
                        "save",
                        "Saving the output",
                        "Collecting the generated files",
                    )
                    .await;
            }
            let message = render_result(user_id, session_id, ok, &stdout, &stderr).await;
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
            if let Some(progress) = progress.as_ref() {
                let status = if ok { "ok" } else { "error" };
                progress.finish(PHASE_TOOL, "save", status, "").await;
            }
        }
    }
}

#[cfg(test)]
mod workspace_failure_message_tests {
    use super::workspace_failure_message;

    #[test]
    fn empty_reason_keeps_a_short_call_to_action() {
        let message = workspace_failure_message("   ", 60_000);
        assert_eq!(
            message,
            "The workspace could not run this program. Start a new message and try again."
        );
    }

    #[test]
    fn the_reason_is_shown_so_the_cause_stays_actionable() {
        let message = workspace_failure_message(
            "container ag-uf4b70eb7-7e97 did not reach RUNNING within 60s; instance log: bpf_program_load_kernel: Operation not permitted",
            60_000,
        );
        assert!(message.contains("container ag-uf4b70eb7-7e97"));
        assert!(message.contains("Operation not permitted"));
        assert!(message.starts_with("The workspace could not run this program: "));
    }

    #[test]
    fn a_cold_start_reason_is_not_framed_as_a_dead_host() {
        let message = workspace_failure_message(
            "the workspace container ag-u58d054e0-619e is still starting after 900s; the first run downloads and unpacks the Incus image. Retry in a moment",
            60_000,
        );
        assert!(message.contains("still starting"));
        assert!(!message.contains("restarting the app"));
        assert!(message.ends_with("Start a new message to retry."));
    }

    #[test]
    fn a_budget_timeout_gets_timeout_guidance_not_host_guidance() {
        let message = workspace_failure_message(
            "the run budget elapsed before the workspace answered",
            60_000,
        );
        assert!(message.contains("180-second budget"));
        assert!(message.contains("60000 ms program timeout"));
        assert!(message.contains("120 s of workspace overhead"));
        assert!(message.contains("Retry"));
        assert!(message.contains("split it into smaller steps"));
        assert!(message.contains("check the budget"));
        assert!(message.contains("`timeout_ms`"));
        assert!(!message.contains("restart"));
        assert!(!message.contains("Start a new message once"));
        assert!(!message.contains("wait for"));
    }

    #[test]
    fn a_reason_far_beyond_the_limit_is_clipped_with_an_ellipsis() {
        let long = "x".repeat(600);
        let message = workspace_failure_message(&long, 60_000);
        assert!(message.contains('…'), "clip marker present");
        assert!(message.chars().count() < 500, "message stays readable");
        assert!(message.ends_with("Start a new message to retry."));
    }
}
