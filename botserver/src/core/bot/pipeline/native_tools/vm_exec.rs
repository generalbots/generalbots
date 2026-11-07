//! `vm_exec`: runs a program in the user's agent VM, including the background
//! path, the one-shot syntax repair and the visible status frames.

use super::program_repair::repair_program;
use super::render::render_result;
use super::send;
use super::super::types::SessionIdentity;

#[derive(Debug, PartialEq, Eq)]
enum WorkspaceFailureKind {
    BudgetExceeded,
    WorkspaceUnavailable,
}

fn classify_workspace_failure(reason: &str) -> WorkspaceFailureKind {
    if reason.to_ascii_lowercase().contains("run budget elapsed") {
        WorkspaceFailureKind::BudgetExceeded
    } else {
        WorkspaceFailureKind::WorkspaceUnavailable
    }
}

fn workspace_state_label(running: Option<bool>) -> &'static str {
    match running {
        Some(true) => "RUNNING",
        Some(false) => "not RUNNING",
        None => "unknown",
    }
}

/// Formats a user-facing failure without conflating a run timeout with a host outage.
fn workspace_failure_message(
    status: u16,
    reason: &str,
    timeout_ms: u64,
    workspace_running: Option<bool>,
) -> String {
    if classify_workspace_failure(reason) == WorkspaceFailureKind::BudgetExceeded {
        let overall_budget_ms = timeout_ms.saturating_add(120_000);
        return format!(
            "The workspace run exceeded its {overall_budget_ms} ms time budget. Retry, or split the task into smaller steps. Workspace status: {}.",
            workspace_state_label(workspace_running),
        );
    }

    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return format!(
            "The workspace could not run this program (status {status}). Start a new message once the sandbox host is back; on a desktop install, restarting the app brings the workspace up. Workspace status: {}.",
            workspace_state_label(workspace_running),
        );
    }
    let clipped: String = trimmed.chars().take(320).collect();
    let clipped = if trimmed.chars().count() > 320 {
        format!("{clipped}…")
    } else {
        clipped
    };
    format!(
        "The workspace could not run this program (status {status}): {clipped}. Start a new message once the sandbox host is back; on a desktop install, restarting the app brings the workspace up. Workspace status: {}.",
        workspace_state_label(workspace_running),
    )
}

fn workspace_starting_message(timeout_ms: u64) -> String {
    format!(
        "The workspace is starting and did not answer within the {timeout_ms} ms startup window. Retry in a moment. Workspace status: not RUNNING."
    )
}

fn workspace_none_message(timeout_ms: u64, workspace_running: Option<bool>) -> String {
    match workspace_running {
        Some(false) => workspace_starting_message(timeout_ms),
        _ => "The workspace is unavailable right now. Start a new message and try again."
            .to_string(),
    }
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
            let workspace_running =
                crate::core::bot::agent_vm_hook::workspace_is_running(user_id).await;
            let result = crate::core::bot::agent_vm_hook::exec_with_wake_result(
                user_id,
                session_id,
                bot_uuid,
                &language,
                &code,
                timeout_ms,
                &packages,
            )
            .await;
            let message = match result {
                Err((status, reason)) => {
                    botlog::safe_warn!("background job workspace failure ({status}): {reason}");
                    workspace_failure_message(status, &reason, timeout_ms, workspace_running)
                }
                Ok(None) => workspace_none_message(timeout_ms, workspace_running),
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
    let workspace_running =
        crate::core::bot::agent_vm_hook::workspace_is_running(user_id).await;
    if !progress_run.is_empty() {
        crate::core::bot::runs::bus::bus().publish(
            &progress_thread,
            serde_json::json!({
                "type": "CUSTOM",
                "run_id": progress_run,
                "data": {
                    "name": "gb.tool_progress",
                    "payload": {
                        "tool": "vm_exec",
                        "phase": "start",
                        "detail": language,
                        "timeout_ms": timeout_ms,
                        "budget_ms": timeout_ms.saturating_add(120_000),
                        "workspace_running": workspace_running,
                    },
                },
            }),
        );
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
    let budget = std::time::Duration::from_millis(timeout_ms.saturating_add(120_000));
    let result = match tokio::time::timeout(
        budget,
        crate::core::bot::agent_vm_hook::exec_with_wake_result(
            user_id,
            session_id,
            bot_uuid,
            language,
            code,
            timeout_ms,
            &packages,
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            botlog::safe_warn!("vm_exec exceeded its overall budget for session {session_id}");
            Err((
                503,
                "the run budget elapsed before the workspace answered".to_string(),
            ))
        }
    };
    if !progress_run.is_empty() {
        let ok = matches!(&result, Ok(Some((true, _, _))));
        let failure_kind = result.as_ref().err().map(|(_, reason)| {
            if classify_workspace_failure(reason) == WorkspaceFailureKind::BudgetExceeded {
                "budget_exceeded"
            } else {
                "workspace_unavailable"
            }
        });
        crate::core::bot::runs::bus::bus().publish(
            &progress_thread,
            serde_json::json!({
                "type": "CUSTOM",
                "run_id": progress_run,
                "data": {
                    "name": "gb.tool_progress",
                    "payload": {
                        "tool": "vm_exec",
                        "phase": "done",
                        "ok": ok,
                        "timeout_ms": timeout_ms,
                        "budget_ms": timeout_ms.saturating_add(120_000),
                        "workspace_running": workspace_running,
                        "failure_kind": failure_kind,
                    },
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
            let message = workspace_failure_message(status, &reason, timeout_ms, workspace_running);
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
        }
        Ok(None) => {
            let message = workspace_none_message(timeout_ms, workspace_running);
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
                if let Some(fixed) = repair_program(
                    state, bot_uuid, language, code, &stderr,
                )
                .await
                {
                    let retry = crate::core::bot::agent_vm_hook::exec_in_user_vm_with_timeout(
                        user_id, session_id, language, &fixed, timeout_ms, &packages,
                    )
                    .await;
                    if let Some((true, fixed_out, _)) = retry {
                        let message = render_result(
                            user_id, session_id, true, &fixed_out, "",
                        )
                        .await;
                        send(identity, &message).await;
                        super::persist(state, session_id, user_id, &message).await;
                        return;
                    }
                }
            }
            let message = render_result(user_id, session_id, ok, &stdout, &stderr).await;
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
        }
    }
}

#[cfg(test)]
mod workspace_failure_message_tests {
    use super::{classify_workspace_failure, workspace_failure_message, WorkspaceFailureKind};

    #[test]
    fn empty_reason_keeps_a_short_call_to_action() {
        let message = workspace_failure_message(503, "   ", 180_000, Some(false));
        assert_eq!(
            message,
            "The workspace could not run this program (status 503). Start a new message once the sandbox host is back; on a desktop install, restarting the app brings the workspace up. Workspace status: not RUNNING."
        );
    }

    #[test]
    fn the_reason_is_shown_so_the_cause_stays_actionable() {
        let message = workspace_failure_message(
            500,
            "container ag-uf4b70eb7-7e97 did not reach RUNNING within 60s; instance log: bpf_program_load_kernel: Operation not permitted",
            180_000,
            Some(false),
        );
        assert!(message.contains("container ag-uf4b70eb7-7e97"));
        assert!(message.contains("Operation not permitted"));
        assert!(message.starts_with("The workspace could not run this program (status 500): "));
    }

    #[test]
    fn a_reason_far_beyond_the_limit_is_clipped_with_an_ellipsis() {
        let long = "x".repeat(600);
        let message = workspace_failure_message(500, &long, 180_000, None);
        assert!(message.contains('…'), "clip marker present");
        assert!(message.chars().count() < 500, "message stays readable");
        assert!(message.contains("brings the workspace up."));
    }

    #[test]
    fn budget_failure_does_not_suggest_restarting_the_app() {
        let message = workspace_failure_message(
            503,
            "the run budget elapsed before the workspace answered",
            60_000,
            Some(true),
        );
        assert!(message.contains("180000 ms"));
        assert!(message.contains("Workspace status: RUNNING"));
        assert!(message.contains("split the task"));
        assert!(!message.contains("restart the app"));
    }

    #[test]
    fn budget_reason_is_classified_before_generic_status() {
        assert_eq!(
            classify_workspace_failure("the run budget elapsed before the workspace answered"),
            WorkspaceFailureKind::BudgetExceeded
        );
        assert_eq!(
            classify_workspace_failure("container did not reach RUNNING"),
            WorkspaceFailureKind::WorkspaceUnavailable
        );
    }
}
