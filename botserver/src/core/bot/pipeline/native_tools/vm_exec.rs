//! `vm_exec`: runs a program in the user's agent VM, including the background
//! path, the one-shot syntax repair and the visible status frames.

use super::super::types::SessionIdentity;
use super::super::workspace_progress::{WorkspaceProgress, PHASE_TOOL};
use super::program_repair::repair_program;
use super::render::render_result;
use super::send;
use super::vm_budget::{
    timeout_cap_ms, workspace_budget_ms, workspace_failure_message,
};
use super::vm_trace::trace_event;

/// #2152 — the one-shot syntax repair shares the turn's provider. Past this
/// budget the run reports the original error instead of leaving the chat on
/// "fixing it automatically…" with no terminal state.
const REPAIR_TIMEOUT_SECS: u64 = 45;

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

    // #2045 — a cold workspace plus a real build needs more than two minutes;
    // the cap is configurable (GB_WORKSPACE_TIMEOUT_MAX_MS, default 600 s).
    let timeout_ms = parsed
        .get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(60_000)
        .clamp(1_000, timeout_cap_ms());
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
                    let stdout = super::vm_trace::strip_trace_lines(&stdout);
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
    // #2045 — the budget is shown in the run card and the chat note, so the
    // user can tell how much time this run has before it is cut off.
    let budget_ms = workspace_budget_ms(timeout_ms);
    let budget_secs = budget_ms / 1_000;
    let progress = WorkspaceProgress::new(identity);
    // #2045 — remember which stage was running when the budget expires.
    let mut last_step = "workspace";
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
        last_step = "execute";
        let detail = format!("{language} · budget {budget_secs}s");
        progress
            .start(PHASE_TOOL, "execute", "Running the program", &detail)
            .await;
    }

    // Every run gets a visible status in the chat: the turn otherwise looks
    // frozen while the program executes, and users cannot tell whether the bot
    // is working or stuck.
    {
        let note = if packages.is_empty() {
            format!("⏳ Executando no workspace ({language}, orçamento {budget_secs}s)…")
        } else {
            format!(
                "⏳ Executando no workspace ({language}, orçamento {budget_secs}s)… instalando {}",
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
    let budget = std::time::Duration::from_millis(budget_ms);
    // #2046 — the program's stdout is streamed so trace markers become table
    // rows while it runs. A run without markers behaves like before.
    let (trace_tx, mut trace_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut trace_rows: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let streamed = async {
        let exec = crate::core::bot::agent_vm_hook::exec_with_wake_events(
            user_id, session_id, bot_uuid, language, code, timeout_ms, &packages, trace_tx,
        );
        tokio::pin!(exec);
        let mut channel_open = true;
        loop {
            if channel_open {
                tokio::select! {
                    event = trace_rx.recv() => match event {
                        Some(event) => {
                            trace_event(progress.as_ref(), &mut trace_rows, event).await;
                        }
                        None => channel_open = false,
                    },
                    outcome = &mut exec => return outcome,
                }
            } else {
                return exec.await;
            }
        }
    };
    let outcome = tokio::time::timeout(budget, streamed).await;
    // Frames that arrived just before the timeout must still reach the card.
    while let Ok(event) = trace_rx.try_recv() {
        trace_event(progress.as_ref(), &mut trace_rows, event).await;
    }
    let result = match outcome {
        Ok(result) => result,
        Err(_) => {
            botlog::safe_warn!(
                "vm_exec exceeded its overall budget of {budget_ms} ms for session {session_id}"
            );
            Err((
                504,
                format!(
                    "the run budget elapsed during '{last_step}' before the workspace answered"
                ),
            ))
        }
    };
    // #2046 — a trace that started but never finished is the in-flight step.
    // Close its row explicitly so a timeout shows the last completed row plus
    // this one, instead of leaving a spinner that never resolves.
    let succeeded = matches!(&result, Ok(Some((true, _, _))));
    if let Some(progress) = progress.as_ref() {
        let mut open: Vec<(String, String)> = trace_rows.drain().collect();
        open.sort_by(|a, b| a.1.cmp(&b.1));
        for (step_name, task) in open {
            // A program that exits cleanly may simply omit the final marker;
            // only a failed or timed-out run means the step was interrupted.
            let (status, detail) = if succeeded {
                ("ok", format!("{task} — finished"))
            } else {
                ("error", format!("{task} — interrupted before it finished"))
            };
            progress
                .finish(PHASE_TOOL, &format!("trace:{step_name}"), status, &detail)
                .await;
        }
    }
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
            let mut repair_failed = false;
            if !ok && super::program_repair::looks_like_syntax_error(&stderr) {
                send(
                    identity,
                    "🔧 The generated program had a syntax error; fixing it automatically…",
                )
                .await;
                // One repair pass: long generated programs occasionally carry a
                // broken string/indent and the user should not have to retype
                // the request. #2152 — the repair call talks to the same provider
                // as the turn, so it must be bounded: an overloaded provider kept
                // the chat on "fixing it automatically…" for minutes with no way
                // out. Past the budget the error is reported instead.
                let repaired = tokio::time::timeout(
                    std::time::Duration::from_secs(REPAIR_TIMEOUT_SECS),
                    repair_program(state, bot_uuid, language, code, &stderr),
                )
                .await;
                let fixed = match repaired {
                    Ok(found) => found,
                    Err(_) => {
                        botlog::safe_warn!(
                            "program repair gave up after {REPAIR_TIMEOUT_SECS}s: session={session_id}"
                        );
                        None
                    }
                };
                if let Some(fixed) = fixed {
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
                // Reaching here means the repair produced nothing usable (no
                // provider, timeout, or a retry that failed again).
                repair_failed = true;
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
            // #2046 — the trace table replaces the raw markers.
            let stdout = super::vm_trace::strip_trace_lines(&stdout);
            let rendered = render_result(user_id, session_id, ok, &stdout, &stderr).await;
            let message = if repair_failed {
                format!(
                    "The automatic fix did not work, so here is the original error.\n\n{rendered}"
                )
            } else {
                rendered
            };
            send(identity, &message).await;
            super::persist(state, session_id, user_id, &message).await;
            if let Some(progress) = progress.as_ref() {
                let status = if ok { "ok" } else { "error" };
                progress.finish(PHASE_TOOL, "save", status, "").await;
            }
        }
    }
}

