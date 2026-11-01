//! Native tools the model can call without a per-bot `.ast` script.
//!
//! Most LLM tool calls resolve to `{bot}.gbdialog/{tool}.ast`. Workspace
//! execution (#1582) is infrastructure, not bot content: every bot with an
//! agent VM must expose it, so it is handled here before the script lookup.

use super::types::SessionIdentity;

/// Maximum characters of process output echoed back into chat. A runaway
/// program must not paste megabytes into the conversation.
#[cfg(feature = "agent-vm")]
const OUTPUT_LIMIT: usize = 8_000;

/// Returns true when `name` is a native tool and the call was answered.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
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
    let code = parsed.get("code").and_then(|v| v.as_str()).unwrap_or("");
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
            let result = crate::core::bot::agent_vm_hook::exec_with_wake(
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
                None => "The workspace became unavailable and the background job could not finish.".to_string(),
                Some((ok, stdout, stderr)) => {
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
        crate::core::bot::agent_vm_hook::exec_with_wake(
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
            None
        }
    };
    botlog::safe_info!(
        "vm_exec done: session={session_id} ok={:?}",
        result.as_ref().map(|r| r.0)
    );

    match result {
        None => {
            let message = "The workspace is not available right now. Start a new message and try again.";
            send(identity, message).await;
            persist(state, session_id, user_id, message).await;
        }
        Some((ok, stdout, stderr)) => {
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
    if model.is_empty() || key.is_empty() {
        botlog::safe_warn!("program repair skipped: model/key empty");
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

/// Formats stdout/stderr and the files the program left in `output/`.
#[cfg(feature = "agent-vm")]
async fn render_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    ok: bool,
    stdout: &str,
    stderr: &str,
) -> String {
    let mut message = String::new();
    if ok {
        message.push_str("**Program finished.**\n");
    } else {
        message.push_str("**Program failed.**\n");
    }
    let out = stdout.trim();
    if !out.is_empty() {
        message.push_str("\n```\n");
        message.push_str(&truncate(out));
        message.push_str("\n```\n");
    }
    let err = stderr.trim();
    if !err.is_empty() {
        message.push_str("\n**stderr**\n```\n");
        message.push_str(&truncate(err));
        message.push_str("\n```\n");
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
