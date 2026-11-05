//! Native tools the model can call without a per-bot `.ast` script.
//!
//! Most LLM tool calls resolve to `{bot}.gbdialog/{tool}.ast`. Workspace
//! execution (#1582) is infrastructure, not bot content: every bot with an
//! agent VM must expose it, so it is handled here before the script lookup.

use super::types::SessionIdentity;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// The last message(s) a native tool sent to chat, keyed by session. The LLM
/// pipeline uses it to feed the tool result back for a follow-up answer;
/// native tools write to the sink directly and previously returned no value.
static NATIVE_CAPTURE: OnceLock<Mutex<HashMap<uuid::Uuid, String>>> = OnceLock::new();

fn capture() -> &'static Mutex<HashMap<uuid::Uuid, String>> {
    NATIVE_CAPTURE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn begin_capture(session_id: uuid::Uuid) {
    if let Ok(mut map) = capture().lock() {
        map.remove(&session_id);
    }
}

pub(super) fn take_capture(session_id: uuid::Uuid) -> String {
    capture()
        .lock()
        .ok()
        .and_then(|mut map| map.remove(&session_id))
        .unwrap_or_default()
}

/// Maximum characters of process output echoed back into chat. A runaway
/// program must not paste megabytes into the conversation.
#[cfg(feature = "agent-vm")]
const OUTPUT_LIMIT: usize = 8_000;

/// Returns true when `name` is a native tool and the call was answered.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    if super::surface_tools::try_run(identity, name, args).await {
        return true;
    }
    if name == "terminal_run" {
        #[cfg(feature = "agent-vm")]
        run_terminal_run(identity, args).await;
        #[cfg(not(feature = "agent-vm"))]
        send(
            identity,
            "Live terminal runs are not available in this build.",
        )
        .await;
        return true;
    }
    if name == "vm_exec" {
        #[cfg(feature = "agent-vm")]
        run_vm_exec(identity, args).await;
        #[cfg(not(feature = "agent-vm"))]
        send(
            identity,
            "Workspace execution is not available in this build.",
        )
        .await;
        return true;
    }
    #[cfg(all(feature = "agent-vm", feature = "drive"))]
    if super::native_drive::try_run(identity, name, args).await {
        return true;
    }
    false
}

pub(super) async fn send(identity: &SessionIdentity<'_>, message: &str) {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    if let Ok(mut map) = capture().lock() {
        let entry = map.entry(session_id).or_default();
        if !entry.is_empty() {
            entry.push('\n');
        }
        entry.push_str(message);
    }
    let resp = botlib::models::BotResponse::new(
        bot_uuid.to_string(),
        session_id.to_string(),
        user_id.to_string(),
        message,
        sink.channel_type(),
    );
    // A failed send is already logged by the sink implementation.
    let _ = sink.send_bot_response(&resp).await;
}

/// Save an assistant turn so a result is still there after the user switches
/// conversation, reloads, or closes the tab.
pub(super) async fn persist(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    text: &str,
) {
    let mut sm = state.session_manager.lock().await;
    if let Err(e) = sm.save_message(session_id, user_id, 2, text, 2) {
        botlog::safe_warn!("failed to persist workspace result for {session_id}: {e}");
    }
}

#[cfg(feature = "agent-vm")]
fn truncate(text: &str) -> String {
    let mut out: String = text.chars().take(OUTPUT_LIMIT).collect();
    if text.chars().count() > OUTPUT_LIMIT {
        out.push_str("\n… (output truncated)");
    }
    out
}

/// The truthful replacement for the dead-end "workspace is not available"
/// phrase: show the concrete reason the VM subsystem reports (status is
/// logged only, never shown), then one short pointer forward.
#[cfg(feature = "agent-vm")]
fn workspace_failure_message(reason: &str) -> String {
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
    format!(
        "The workspace could not run this program: {clipped}. Start a new message once the sandbox host is back; on a desktop install, restarting the app brings the workspace up."
    )
}

#[cfg(feature = "agent-vm")]
async fn run_vm_exec(identity: &SessionIdentity<'_>, args: &str) {
    let parsed: serde_json::Value = match super::native_args::parse_args(args) {
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
                    workspace_failure_message(&reason)
                }
                Ok(None) => "The workspace became unavailable and the background job could not finish.".to_string(),
                Ok(Some((ok, stdout, stderr))) => {
                    render_result(user_id, session_id, ok, &stdout, &stderr).await
                }
            };
            persist(&worker_state, session_id, user_id, &message).await;
        });
        let started = "Started in the background. The result will be saved to this conversation.";
        send(identity, started).await;
        persist(&state, session_id, user_id, started).await;
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
            let message = workspace_failure_message(&reason);
            send(identity, &message).await;
            persist(state, session_id, user_id, &message).await;
        }
        Ok(None) => {
            let message = "The workspace is not available right now. Start a new message and try again.";
            send(identity, message).await;
            persist(state, session_id, user_id, message).await;
        }
        Ok(Some((ok, stdout, stderr))) => {
            if !ok && looks_like_syntax_error(&stderr) {
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
                        persist(state, session_id, user_id, &message).await;
                        return;
                    }
                }
            }
            let message = render_result(user_id, session_id, ok, &stdout, &stderr).await;
            send(identity, &message).await;
            persist(state, session_id, user_id, &message).await;
        }
    }
}

/// Asks the model to rewrite a program that failed with a syntax error.
/// Returns only the corrected source, or None when the provider is absent.
#[cfg(feature = "agent-vm")]
async fn repair_program(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    bot_uuid: uuid::Uuid,
    language: &str,
    code: &str,
    error: &str,
) -> Option<String> {
    let provider = match state.llm_provider.as_ref() {
        Some(p) => p,
        None => {
            botlog::safe_warn!("program repair skipped: no LLM provider");
            return None;
        }
    };
    let (model, key) =
        super::tool_exec::bot_llm_credentials(&state.conn, bot_uuid);
    // Keyless gateways (Kilo free tier) legitimately carry an empty key; only
    // a missing model makes the repair call impossible.
    if model.is_empty() {
        botlog::safe_warn!("program repair skipped: no llm-model configured");
        return None;
    }
    let prompt = format!(
        "The following {language} program failed with this error:\n\n{error}\n\n\
         Program:\n\n{code}\n\n\
         Rewrite the whole program so it runs correctly. Preserve the intent and \
         the output path. A newline inside a quoted string is invalid Python: use \
         \\n escapes or triple-quoted strings for multi-line text. Return ONLY the \
         complete corrected program, no markdown fences and no explanation."
    );
    let reply = match provider
        .generate(&prompt, &serde_json::json!({}), &model, &key)
        .await
    {
        Ok(text) => text,
        Err(e) => {
            botlog::safe_warn!("program repair LLM call failed: {e}");
            return None;
        }
    };
    let cleaned = strip_code_fences(&reply);
    if cleaned.trim().is_empty() || cleaned.trim() == code.trim() {
        None
    } else {
        Some(cleaned)
    }
}

#[cfg(feature = "agent-vm")]
fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let mut lines = trimmed.lines();
    let _ = lines.next();
    let body: Vec<&str> = lines.collect();
    let mut out = body.join("\n");
    if let Some(pos) = out.rfind("```") {
        out.truncate(pos);
    }
    out.trim().to_string()
}

/// True when a failed run looks like a malformed program rather than a runtime
/// error (those are reported to the user, not silently rewritten).
#[cfg(feature = "agent-vm")]
fn looks_like_syntax_error(stderr: &str) -> bool {
    stderr.contains("SyntaxError")
        || stderr.contains("IndentationError")
        || stderr.contains("TabError")
        || stderr.contains("Unexpected token")
        || stderr.contains("Unexpected end of input")
}

/// Formats stdout/stderr and the files left in `output/` as plain command
/// output (#1632 item 4): no "program" wording, just what the run produced.
#[cfg(feature = "agent-vm")]
async fn render_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    ok: bool,
    stdout: &str,
    stderr: &str,
) -> String {
    let mut message = String::new();
    let out = stdout.trim();
    if !out.is_empty() {
        message.push_str("```\n");
        message.push_str(&truncate(out));
        message.push_str("\n```\n");
    }
    let err = stderr.trim();
    if !err.is_empty() || !ok {
        message.push_str("\n**Error**\n");
        if err.is_empty() {
            message.push_str("```\nThe run finished without output.\n```\n");
        } else {
            message.push_str("```\n");
            message.push_str(&truncate(err));
            message.push_str("\n```\n");
        }
    }
    let files = crate::core::bot::agent_vm_hook::list_user_vm_output(user_id, session_id)
        .await
        .unwrap_or_default();
    if !files.is_empty() {
        let conversation = session_id.to_string();
        message.push_str("\n**Files**\n");
        for file in &files {
            message.push_str(&format!(
                "- [{file}](vmfile:{conversation}/output/{file})\n"
            ));
        }
    }
    message.trim_end().to_string()
}

#[cfg(all(test, feature = "agent-vm"))]
mod workspace_failure_message_tests {
    use super::workspace_failure_message;

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
    }

    #[test]
    fn a_reason_far_beyond_the_limit_is_clipped_with_an_ellipsis() {
        let long = "x".repeat(600);
        let message = workspace_failure_message(&long);
        assert!(message.contains('…'), "clip marker present");
        assert!(message.chars().count() < 500, "message stays readable");
        assert!(message.ends_with("brings the workspace up."));
    }
}


/// A live terminal run: spawns a real PTY inside the user's agent VM and
/// streams `terminal_output` / `gb.command_output` frames to the chat run so
/// the run card can expose Take over. This is the PTY equivalent of the
/// AutoTask agent executor (which is not wired in this build).
#[cfg(feature = "agent-vm")]
async fn run_terminal_run(identity: &SessionIdentity<'_>, args: &str) {
    use std::io::Read;

    let parsed = match super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            send(identity, &format!("Could not read the terminal request: {error}")).await;
            return;
        }
    };
    let command = parsed
        .get("command")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if command.is_empty() {
        send(identity, "terminal_run requires a `command`.").await;
        return;
    }
    let timeout_secs = parsed
        .get("timeout_ms")
        .and_then(|value| value.as_u64())
        .unwrap_or(60_000)
        .clamp(5_000, 120_000)
        / 1000;

    let SessionIdentity {
        session_id,
        user_id,
        ..
    } = *identity;
    let Some(vm_name) = crate::core::bot::agent_vm_hook::user_vm_name(user_id).await else {
        send(identity, "No running workspace VM. Open the workspace first.").await;
        return;
    };
    let thread_id = session_id.to_string();
    let run_id = crate::core::bot::runs::bus::bus()
        .current_run_id(&thread_id)
        .unwrap_or_default();
    let vm_for_task = vm_name.clone();
    let thread_for_task = thread_id.clone();
    let run_for_task = run_id.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("openpty: {error}"))?;
        let mut builder = portable_pty::CommandBuilder::new("incus");
        builder.args([
            "exec",
            &vm_for_task,
            "--",
            "timeout",
            &format!("{timeout_secs}s"),
            "bash",
            "-lc",
            &command,
        ]);
        let mut child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| format!("spawn: {error}"))?;
        drop(pair.slave);
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("reader: {error}"))?;
        let bus = crate::core::bot::runs::bus::bus();
        let mut output = String::new();
        let mut buffer = [0u8; 2048];
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            let chunk = String::from_utf8_lossy(&buffer[..read]).to_string();
            output.push_str(&chunk);
            for line in chunk.lines() {
                let line = line.trim_end();
                if line.is_empty() {
                    continue;
                }
                if !run_for_task.is_empty() {
                    bus.publish(
                        &thread_for_task,
                        serde_json::json!({
                            "type": "terminal_output",
                            "run_id": run_for_task,
                            "data": { "line": line },
                        }),
                    );
                }
                bus.publish(
                    &thread_for_task,
                    serde_json::json!({
                        "type": "CUSTOM",
                        "run_id": run_for_task,
                        "data": {
                            "name": "gb.command_output",
                            "payload": { "line": line, "stream": "stdout" },
                        },
                    }),
                );
            }
        }
        let _ = child.wait();
        Ok(output)
    })
    .await;

    let text = match result {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => format!("terminal_run failed: {error}"),
        Err(error) => format!("terminal_run task failed: {error}"),
    };
    let trimmed = text.trim();
    let summary: String = if trimmed.is_empty() {
        "The command finished with no output.".to_string()
    } else {
        trimmed.chars().take(8_000).collect()
    };
    send(identity, &summary).await;
}
