//! `vm_exec`: runs a program in the user's agent VM, including the background
//! path, the one-shot syntax repair and the visible status frames.

use super::program_repair::repair_program;
use super::render::render_result;
use super::send;
use super::super::types::SessionIdentity;

/// User-facing message when the per-run time budget is exceeded.  The sandbox
/// itself is fine — only this particular run took too long (#1803).
fn budget_elapsed_message() -> String {
    "The workspace run exceeded its time budget. Retry with a simpler request, or split the task into smaller steps.".to_string()
}

/// Returns `true` when the reason text describes a budget / timeout exhaustion
/// rather than a workspace infrastructure failure.
fn is_budget_elapsed(reason: &str) -> bool {
    reason.contains("budget elapsed")
}

/// The truthful replacement for the dead-end "workspace is not available"
/// phrase: show the concrete reason the VM subsystem reports (status is
/// logged only, never shown), then one short pointer forward.
///
/// **Budget-elapsed errors must be caught by the caller before reaching this
/// function** — see [`budget_elapsed_message`].  The guard here is a safety
/// net so the misleading "restart the app" suffix never leaks into a budget
/// error (#1803).
fn workspace_failure_message(reason: &str) -> String {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return "The workspace could not run this program. Start a new message and try again."
            .to_string();
    }
    // Safety net: a budget-elapsed reason must never get the sandbox/restart
    // suffix — even if a caller forgets to check `is_budget_elapsed` first.
    if is_budget_elapsed(trimmed) {
        return budget_elapsed_message();
    }
    let clipped: String = trimmed.chars().take(320).collect();
    let clipped = if trimmed.chars().count() > 320 {
        format!("{clipped}…")
    } else {
        clipped
    };
    format!(
        "The workspace could not run this program: {clipped}. Start a new message once the sandbox host is back; on a desktop install, restarting the app brings the workspace up."
    )
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
                    if is_budget_elapsed(&reason) {
                        budget_elapsed_message()
                    } else {
                        workspace_failure_message(&reason)
                    }
                }
                Ok(None) => "Workspace is starting \u{2014} retry in a moment.".to_string(),
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
            let message = if is_budget_elapsed(&reason) {
                budget_elapsed_message()
            } else {
                workspace_failure_message(&reason)
            };
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
        }
        Ok(None) => {
            let message = "Workspace is starting \u{2014} retry in a moment.";
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
    use super::{budget_elapsed_message, is_budget_elapsed, workspace_failure_message};

    #[test]
    fn empty_reason_keeps_a_short_call_to_action() {
        let message = workspace_failure_message("   ");
        assert_eq!(
            message,
            "The workspace could not run this program. Start a new message and try again."
        );
    }

    #[test]
    fn the_reason_is_shown_so_the_cause_stays_actionable() {
        let message = workspace_failure_message(
            "container ag-uf4b70eb7-7e97 did not reach RUNNING within 60s; instance log: bpf_program_load_kernel: Operation not permitted",
        );
        assert!(message.contains("container ag-uf4b70eb7-7e97"));
        assert!(message.contains("Operation not permitted"));
        assert!(message.starts_with("The workspace could not run this program: "));
        // Workspace infrastructure failures keep the sandbox/restart guidance.
        assert!(message.contains("sandbox host"));
        assert!(message.contains("restarting the app"));
    }

    #[test]
    fn a_reason_far_beyond_the_limit_is_clipped_with_an_ellipsis() {
        let long = "x".repeat(600);
        let message = workspace_failure_message(&long);
        assert!(message.contains('…'), "clip marker present");
        assert!(message.chars().count() < 500, "message stays readable");
        assert!(message.ends_with("brings the workspace up."));
    }

    // #1803 — budget-elapsed must never mention "sandbox host" or "restarting
    // the app"; the workspace is fine, only the run took too long.
    #[test]
    fn budget_elapsed_detected_by_reason_string() {
        assert!(is_budget_elapsed(
            "the run budget elapsed before the workspace answered"
        ));
        assert!(!is_budget_elapsed("container timed out"));
        assert!(!is_budget_elapsed(""));
    }

    #[test]
    fn budget_elapsed_message_has_no_sandbox_or_restart_text() {
        let msg = budget_elapsed_message();
        assert!(
            !msg.contains("sandbox host"),
            "budget message must not mention sandbox host"
        );
        assert!(
            !msg.contains("restarting the app"),
            "budget message must not mention restarting the app"
        );
        assert!(msg.contains("time budget"), "must say what happened");
        assert!(msg.contains("simpler request"), "must suggest a remedy");
    }

    #[test]
    fn workspace_failure_message_safety_net_for_budget_reason() {
        // Even if a caller forgets to check is_budget_elapsed and passes the
        // reason through workspace_failure_message, the safety net catches it.
        let msg = workspace_failure_message(
            "the run budget elapsed before the workspace answered",
        );
        assert!(
            !msg.contains("sandbox host"),
            "safety net must strip sandbox text"
        );
        assert!(
            !msg.contains("restarting the app"),
            "safety net must strip restart text"
        );
        assert!(msg.contains("time budget"));
    }
}
