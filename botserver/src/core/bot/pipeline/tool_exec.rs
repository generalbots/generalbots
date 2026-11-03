use uuid::Uuid;

use super::attachments::find_staged_attachment;
use super::types::SessionIdentity;

mod is_placeholder_value;
use is_placeholder_value::all_args_are_placeholder;
pub(crate) use is_placeholder_value::bot_llm_credentials;
pub use is_placeholder_value::{is_generic_greeting, is_placeholder_value};

pub async fn run_tool_exec(identity: &SessionIdentity<'_>, tool_name: &str) {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
    } = *identity;
    let channel = sink.channel_type();
    let work_path = botcore::shared::utils::get_work_path();

    // Resolve through the shared work-tree resolver: the org directory is
    // `<org-id>.gborg`, so the old `{bot}.gborg` assumption matched nothing
    // and the tool was skipped without any log line (the button did nothing).
    // Tools reach the work tree as `.bas` (compiled to `.ast` only after the
    // compiler pass), so both spellings must be attempted: resolving only the
    // `.ast` returned None on a fresh install and the button did nothing.
    let ast_content = ["ast", "bas"]
        .iter()
        .find_map(|ext| {
            super::tool_paths::resolve_bot_file(
                &work_path,
                bot_name,
                &format!("{tool_name}.{ext}"),
            )
            .and_then(|path| std::fs::read_to_string(path).ok())
            .filter(|c| !c.is_empty())
        })
        .unwrap_or_default();

    if ast_content.is_empty() {
        botlog::safe_info!(
            "TOOL_EXEC: no compiled script for tool '{tool_name}' of bot '{bot_name}'"
        );
    }

    if !ast_content.is_empty() {
        // Declarative gate: only execute tools the bot script associated
        // with this session via USE TOOL (e.g. inside IF role = "admin").
        if !crate::core::bot::tool_context::is_tool_associated_with_session(
            &state.conn, &session_id, tool_name,
        ) {
            botlog::safe_info!(
                "TOOL_EXEC: tool '{tool_name}' not associated with session {session_id}, skipping"
            );
            // The user pressed a button; silence would look like a broken bot.
            let message = format!(
                "A ação '{tool_name}' não está disponível para esta conversa."
            );
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &message,
                channel,
            );
            let _ = sink.send_bot_response(&resp).await;
            return;
        }
        let state_for_tool = state.clone();
        let tool_name_clone = tool_name.to_string();
        let channel_name = channel.to_string();
        let session_for_tool = botlib::models::UserSession {
            id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_uuid,
            title: String::new(),
            context_data: serde_json::json!({"channel": channel_name}),
            current_tool: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let tool_run = tokio::task::spawn_blocking(move || {
            let mut svc = crate::basic::ScriptService::new(
                state_for_tool.clone(), session_for_tool,
            );
            svc.load_bot_config_params(&state_for_tool, bot_uuid);
            svc.run(&ast_content).map_err(|e| e.to_string())
        });
        // The script runs on a blocking task, so the report goes out after it
        // completes: a failure must reach the chat, not only the log.
        // Awaited, like the LLM tool-call path already does, so a failure can
        // be reported to the user instead of only reaching the log. The tool's
        // own TALK output reaches the sink while the script runs.
        if let Ok(Err(detail)) = tool_run.await {
            log::warn!("Tool '{tool_name_clone}' execution error: {detail}");
            let message = format!(
                "Não consegui executar '{tool_name_clone}'. Tente novamente em instantes."
            );
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &message,
                channel,
            );
            let _ = sink.send_bot_response(&resp).await;
        }
    }
}

pub async fn run_llm_tool_call(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    full_response: &str,
    user_text: &str,
) {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
    } = *identity;
    use crate::core::bot::ws::handler::validate_bot_name;
    use crate::core::bot::ws::handler::verify_path_within_workdir;
    use botcore::shared::utils::get_work_path;
    // Some providers wrap function calls in an XML dialect (DSML) instead of
    // the JSON protocol. Convert it to the wrapper this function understands.
    if !full_response.contains("\"__tool_call__\":") {
        if let Some(synthetic) = parse_dsml_tool_call(full_response)
            .or_else(|| parse_xml_tool_use(full_response))
        {
            let text = synthetic.to_string();
            Box::pin(run_llm_tool_call(identity, rx, &text, user_text)).await;
            return;
        }
    }
    let tool_call_trigger = "\"__tool_call__\":";
    let tc_start = match full_response.find(tool_call_trigger) {
        Some(pos) => full_response[..pos].rfind('{').unwrap_or(pos),
        None => return,
    };
    let tc_json = &full_response[tc_start..];
    let tool_call = match extract_tool_call(tc_json).or_else(|| salvage_tool_call(tc_json)) {
        Some(value) => value,
        None => {
            botlog::safe_warn!(
                "tool_call: could not parse the model's JSON call; reply starts with: {}",
                tc_json.chars().take(200).collect::<String>()
            );
            return;
        }
    };
    {
        // A tool call without a usable name used to return in silence: the
        // user saw an empty bubble and a stuck "Thinking" indicator while the
        // log only said `executing tool ''`. Recover the name when the model
        // nests it (`tool`, `function.name`) or when the marker is followed by
        // a bare known tool name, and always answer the user when recovery
        // fails. `arguments` may arrive as an object instead of a string.
        let mut raw_tool_name = tool_call
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let mut tool_args_owned = match tool_call.get("arguments") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Object(_)) => tool_call
                .get("arguments")
                .map(serde_json::Value::to_string)
                .unwrap_or_default(),
            _ => String::new(),
        };
        if raw_tool_name.is_empty() {
            raw_tool_name = recover_tool_name(&tool_call, full_response);
            if !raw_tool_name.is_empty() {
                log::info!("tool_call: recovered missing tool name '{raw_tool_name}'");
            }
        }

        // Chat attachments are staged under the bot's Drive `inbox/` before
        // the LLM call. Web attachments (pipeline::exec) append
        // "[User attached a file stored at inbox/…; …]" after the typed text;
        // channel adapters (bottelegram/botwhatsapp media) send
        // "[image] inbox/…" with the caption on the following line. Closed-
        // contract filing tools such as classify_media take `path`/`caption`;
        // when the model returns the tool call without carrying the staged
        // path, inject it from the user message so execution does not depend
        // on the model copying it verbatim.
        //
        // The staged path from the CURRENT message is authoritative: when the
        // model copies a path from an EARLIER turn's marker (history echo), the
        // tool operates on an already-filed object and fails with a confusing
        // 404. Only when the current message carries no attachment marker do
        // the model's arguments stand.
        if let Some(staged) = find_staged_attachment(user_text) {
            let mut parsed: serde_json::Value =
                serde_json::from_str(&tool_args_owned).unwrap_or_else(|_| serde_json::json!({}));
            match parsed.as_object_mut() {
                Some(obj) => {
                    let model_path = obj
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let stale = !model_path.is_empty() && model_path != staged.path;
                    obj.insert("path".to_string(), serde_json::Value::String(staged.path.clone()));
                    obj.insert("caption".to_string(), serde_json::Value::String(staged.caption));
                    if stale {
                        log::info!(
                            "Tool '{raw_tool_name}': model echoed path '{model_path}' from history; overridden with current message attachment '{}'",
                            staged.path
                        );
                    }
                }
                None => {
                    log::warn!(
                        "Tool '{raw_tool_name}': arguments must be an object; staged attachment not injected"
                    );
                }
            }
            match serde_json::to_string(&parsed) {
                Ok(s) => tool_args_owned = s,
                Err(e) => log::warn!("Failed to serialize injected tool args: {e}"),
            }
        }
        let tool_args: &str = tool_args_owned.as_str();

        log::info!("LLM tool_call: executing tool '{raw_tool_name}' with args: {tool_args}");
        if raw_tool_name.is_empty() {
            botlog::safe_warn!(
                "tool_call: the model sent a call without a tool name; raw: {}",
                tc_json.chars().take(200).collect::<String>()
            );
            let msg = "Não consegui identificar qual ferramenta executar para esse pedido. \
                       Pode repetir o que você precisa?";
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                msg,
                sink.channel_type(),
            );
            let _ = sink.send_bot_response(&resp).await;
            return;
        }

        if all_args_are_placeholder(tool_args) {
            log::info!("All tool args are placeholder - asking user for real data instead of executing");
            let msg = "Para agendar o servico, preciso de algumas informacoes. Por favor, me diga os dados solicitados um de cada vez.".to_string();
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(), &msg, sink.channel_type(),
            );
            let _ = sink.send_bot_response(&resp).await;
            return;
        }

        if is_generic_greeting(user_text) {
            log::info!("Blocking tool '{raw_tool_name}' on short generic user message: '{user_text}'");
            return;
        }

        // Native infrastructure tools (#1582) have no per-bot `.ast`: the
        // workspace executor is available to every bot with an agent VM.
        if super::native_tools::try_run(identity, &raw_tool_name, tool_args).await {
            return;
        }

        let tool_name = match validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("LLM tool_call: invalid tool name '{raw_tool_name}': {e}");
                return;
            }
        };

        let work_path = get_work_path();
        let rel_tool_path = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
        if !verify_path_within_workdir(&rel_tool_path) {
            log::error!("Path traversal detected in LLM tool_call for tool: {tool_name}");
            return;
        }

        // The org directory is named after the org id (`<uuid>.gborg`), not the
        // bot: assuming `{bot}.gborg` meant every tool call resolved to a
        // missing file and was dropped as "no compiled script".
        let ast_path = super::tool_paths::resolve_bot_file(&work_path, &bot_name, &format!("{tool_name}.ast"));
        let ast_content = match ast_path.as_ref() {
            Some(path) => match tokio::fs::read_to_string(path).await {
                Ok(c) if !c.is_empty() => c,
                _ => {
                    let bas_path = path.with_extension("bas");
                    tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
                }
            },
            None => String::new(),
        };

        if ast_content.is_empty() {
            // A tool call naming a catalog command (`api.find`,
            // `tasks.autotask.create`, …) is the model reaching for the
            // declarative surface through the native tool channel. It has no
            // `.ast`, so this path used to return in silence and the user got
            // no answer at all — observed on Telegram: the model asked for
            // `api.find`, nothing came back. Reroute to the command executor.
            if let Some(command) = crate::apps::commands::command_by_name(&tool_name) {
                log::info!(
                    "tool_call '{tool_name}' is catalog command '{}' (app {}); routing to the command executor",
                    command.name, command.app
                );
                let api_call = format!(
                    "{{\"__api_call__\": {{\"name\": \"{tool_name}\", \"params\": {tool_args}, \"compose\": true}}}}"
                );
                // Composing the answer needs a provider AND the bot's model and
                // key: the reroute used to pass empty strings, so every composed
                // answer failed and the user got "Dados obtidos, mas falhei ao
                // redigir a resposta" instead of the command's result.
                let provider = state.llm_provider.clone();
                let provider = provider.as_ref();
                let (model, key) = bot_llm_credentials(&state.conn, bot_uuid);
                // Boxed: the command path can reroute a session tool back
                // into this executor, so the two futures are mutually
                // recursive.
                let llm_target = super::types::LlmTarget {
                    provider,
                    model: &model,
                    key: &key,
                };
                Box::pin(super::api_call::handle_api_call(
                    identity, &llm_target, &api_call, user_text, rx,
                ))
                .await;
                return;
            }
            // Unknown name with no script: say so instead of going silent.
            log::warn!(
                "tool_call '{tool_name}' has no compiled script and is not a catalog command; ignored"
            );
            let message = format!(
                "Não consegui executar '{tool_name}': essa ação não está disponível para mim agora."
            );
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &message,
                sink.channel_type(),
            );
            let _ = sink.send_bot_response(&resp).await;
            return;
        }

        {
            // Declarative gate: only execute tools the bot script associated
            // with this session via USE TOOL (e.g. inside IF role = "admin").
            if !crate::core::bot::tool_context::is_tool_associated_with_session(
                &state.conn, &session_id, &tool_name,
            ) {
                log::info!(
                    "LLM tool_call: tool '{tool_name}' not associated with session {session_id}, skipping"
                );
                return;
            }
            let state_for_tool = state.clone();
            let work_path_for_mcp = work_path.clone();
            let bot_name_for_mcp = bot_name.to_string();
            let tool_name_for_mcp = tool_name.clone();

            let parsed_args: serde_json::Value = serde_json::from_str(tool_args).unwrap_or(serde_json::Value::Null);
            let injected_args = parsed_args.clone();
            let channel_name = sink.channel_type();
            let mut context_data = if parsed_args.is_object() {
                parsed_args
            } else {
                serde_json::json!({})
            };
            if let Some(obj) = context_data.as_object_mut() {
                obj.insert("channel".to_string(), serde_json::Value::String(channel_name.to_string()));
            }

            let session_for_tool = botlib::models::UserSession {
                id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_uuid,
                title: String::new(),
                context_data,
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let tool_name_err = tool_name.clone();
            let tool_run = tokio::task::spawn_blocking(move || {
                let mut svc = crate::basic::ScriptService::new(
                    state_for_tool.clone(), session_for_tool,
                );
                svc.load_bot_config_params(&state_for_tool, bot_uuid);
                let mut injected_param_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                if let Some(obj) = injected_args.as_object() {
                    for (key, value) in obj {
                        injected_param_names.insert(key.clone());
                        match value {
                            serde_json::Value::String(s) => {
                                let clean_val = if is_placeholder_value(s) { String::new() } else { s.clone() };
                                let _ = svc.set_variable(key, &clean_val);
                            }
                            serde_json::Value::Number(n) => {
                                let _ = svc.set_variable(key, &n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                let _ = svc.set_variable(key, if *b { "true" } else { "false" });
                            }
                            _ => {}
                        }
                    }
                }
                let mcp_path = format!("{bot_name_for_mcp}.gborg/{bot_name_for_mcp}.gbai/{bot_name_for_mcp}.gbdialog/{tool_name_for_mcp}.mcp.json");
                let mcp_full = std::path::Path::new(&work_path_for_mcp).join(&mcp_path);
                if mcp_full.exists() {
                    if let Ok(mcp_content) = std::fs::read_to_string(&mcp_full) {
                        if let Ok(mcp_val) = serde_json::from_str::<serde_json::Value>(&mcp_content) {
                            if let Some(props) = mcp_val.get("input_schema").and_then(|s| s.get("properties")).and_then(|p| p.as_object()) {
                                for (param_name, _) in props {
                                    let clean_name = param_name.trim_end_matches(":string").to_string();
                                    if !injected_param_names.contains(&clean_name) {
                                        let _ = svc.set_variable(&clean_name, "");
                                    }
                                }
                            }
                        }
                    }
                }
                svc.run(&ast_content).map_err(|e| e.to_string())
            }).await;

            // A failed tool used to reach only the log, so the user watched the
            // turn stop with no answer at all (observed on the web tool
            // button: "Variable not found: path"). The detail stays in the
            // log; the user is told which action failed.
            if let Ok(Err(detail)) = tool_run {
                log::warn!("Tool '{tool_name_err}' execution error: {detail}");
                let message = format!(
                    "Não consegui executar '{tool_name_err}'. Tente novamente em instantes."
                );
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    &message,
                    sink.channel_type(),
                );
                let _ = sink.send_bot_response(&resp).await;
            }

            // Drain rx to forward tool responses to the sink
            for _ in 0..50 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                match rx.try_recv() {
                    Ok(response) => {
                        let _ = sink.send_bot_response(&response).await;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

/// Extracts the first balanced JSON object from a model reply.
///
/// Models routinely embed real newlines/tabs inside the `arguments` string
/// (generated code), which `serde_json` rejects as control characters. The
/// first attempt parses the object as-is; the second escapes raw control
/// characters inside string literals and retries.
/// Native tools every bot with an agent VM can call; used to recover a tool
/// name the model left out of its JSON call.
const RECOVERABLE_TOOL_NAMES: [&str; 9] = [
    "vm_exec",
    "drive_save",
    "drive_fetch",
    "vm_file",
    "vm_files",
    "classify_media",
    "ask_user",
    "propose_plan",
    "show_surface",
];

/// Recover a missing tool name from a parsed call. Providers occasionally emit
/// `{"__tool_call__": true, "arguments": …}` with no `name`, or nest the name
/// under `tool`/`tool_name`/`function`. Falling back to a known native tool
/// name found in the call text keeps the request executable instead of
/// dropping it.
fn recover_tool_name(call: &serde_json::Value, raw: &str) -> String {
    for key in ["tool", "tool_name", "function"] {
        if let Some(value) = call.get(key) {
            if let Some(name) = value.as_str() {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    RECOVERABLE_TOOL_NAMES
        .iter()
        .find(|name| raw.contains(&format!("\"{name}\"")))
        .map(|name| (*name).to_string())
        .unwrap_or_default()
}

fn extract_tool_call(raw: &str) -> Option<serde_json::Value> {
    let start = raw.find('{')?;
    let slice = &raw[start..];
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = None;
    for (index, ch) in slice.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(index + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let object = &slice[..end?];
    if let Ok(value) = serde_json::from_str(object) {
        return Some(normalize_tool_call(value));
    }

    let mut repaired = String::with_capacity(object.len());
    let mut in_string = false;
    let mut escaped = false;
    for ch in object.chars() {
        if in_string {
            if escaped {
                repaired.push(ch);
                escaped = false;
                continue;
            }
            match ch {
                '\\' => {
                    repaired.push(ch);
                    escaped = true;
                }
                '"' => {
                    repaired.push(ch);
                    in_string = false;
                }
                '\n' => repaired.push_str("\\n"),
                '\r' => repaired.push_str("\\r"),
                '\t' => repaired.push_str("\\t"),
                other => repaired.push(other),
            }
        } else {
            if ch == '"' {
                in_string = true;
            }
            repaired.push(ch);
        }
    }
    serde_json::from_str(&repaired).ok().map(normalize_tool_call)
}

/// Providers do not always follow the documented envelope
/// `{"__tool_call__": true, "name": …, "arguments": …}`. Some wrap the call
/// inside the marker (`{"__tool_call__": {"name": …, "arguments": …}}`), which
/// previously failed extraction and left the reply stuck on "Thinking" with no
/// message. Unwrap that shape so the call still runs.
fn normalize_tool_call(value: serde_json::Value) -> serde_json::Value {
    for key in ["__tool_call__", "tool_call"] {
        if let Some(inner) = value.get(key) {
            if inner.is_object() {
                return inner.clone();
            }
        }
    }
    value
}

/// Lenient recovery for the malformed JSON providers emit when the program
/// inside `arguments` contains unescaped quotes (common with Python
/// `print("x")`): pull the sibling `name` field and the raw `arguments`
/// payload, decoding the escapes the model did apply. Without this the reply
/// stops at "Thinking" and the user never sees the generated files.
fn salvage_tool_call(raw: &str) -> Option<serde_json::Value> {
    let name = {
        let at = raw.find("\"name\"")?;
        let rest = &raw[at + 6..];
        let colon = rest.find(':')?;
        let after = &rest[colon + 1..];
        let start = after.find('"')? + 1;
        let end = after[start..].find('"')? + start;
        after[start..end].trim().to_string()
    };
    if name.is_empty() {
        return None;
    }
    let args = {
        let at = raw.find("\"arguments\"")?;
        let rest = &raw[at + 11..];
        let colon = rest.find(':')?;
        let after = &rest[colon + 1..];
        let start = after.find('"')? + 1;
        let tail = &after[start..];
        let end = tail.rfind('"')?;
        let payload = &tail[..end];
        let mut out = String::with_capacity(payload.len());
        let mut chars = payload.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        }
        out
    };
    Some(serde_json::json!({ "name": name, "arguments": args }))
}

/// Parses the DSML/XML tool-call dialect some providers emit:
/// `invoke name="..."` with `parameter name="arguments"` containing the JSON
/// arguments. Returns a wrapper object in the native protocol shape.
fn parse_dsml_tool_call(raw: &str) -> Option<serde_json::Value> {
    let invoke = raw.find("invoke name=")?;
    let rest = &raw[invoke..];
    let name_start = rest.find("name=\"")? + 6;
    let name_end = dispatch_quote_end(rest, name_start)?;
    let mut name = rest[name_start..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }
    // The envelope is sometimes named instead of the tool.
    if name == "__tool_call__" || name == "tool_call" {
        name = "vm_exec".to_string();
    }
    let args_at = rest.find("parameter name=\"arguments\"")?;
    let after = &rest[args_at..];
    let brace = after.find('{')?;
    let value = extract_tool_call(&after[brace..])?;
    Some(serde_json::json!({
        "__tool_call__": true,
        "name": name,
        "arguments": value.to_string(),
    }))
}

fn dispatch_quote_end(text: &str, start: usize) -> Option<usize> {
    text[start..].find('"').map(|pos| start + pos)
}

/// Parses the `<tool_use>{"name": …, "arguments": …}</tool_use>` dialect
/// emitted by some Kilo-hosted models. Returns the native wrapper shape.
fn parse_xml_tool_use(raw: &str) -> Option<serde_json::Value> {
    // Two shapes reach us: `<tool_use>{"name": …}</tool_use>` and the
    // attribute dialect `<tool_use name="vm_exec">{"language": …}</tool_use>`.
    // The latter used to leak into the chat as raw text ("vm_exec">…") because
    // only the JSON-named form was recognised.
    let tag = raw.find("<tool_use")?;
    let after_tag = tag + "<tool_use".len();
    let tag_end = raw[after_tag..].find('>')? + after_tag;
    let header = &raw[after_tag..tag_end];
    let attribute_name = header
        .split_once("name=")
        .map(|(_, rest)| rest.trim_matches(|c| c == '"' || c == '\'' || c == ' ').to_string())
        .filter(|n| !n.is_empty());
    let body_start = tag_end + 1;
    let close = raw[body_start..].find("</tool_use>")?;
    let payload = raw[body_start..body_start + close].trim();

    if let Some(value) = extract_tool_call(payload).or_else(|| salvage_tool_call(payload)) {
        if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
            let name = name.trim();
            if !name.is_empty() {
                let args = value.get("arguments")?.clone();
                let args = match args {
                    serde_json::Value::String(text) => text,
                    other => other.to_string(),
                };
                return Some(serde_json::json!({
                    "__tool_call__": true,
                    "name": name,
                    "arguments": args,
                }));
            }
        }
    }
    // Attribute dialect: the body IS the arguments object.
    let name = attribute_name?;
    Some(serde_json::json!({
        "__tool_call__": true,
        "name": name,
        "arguments": payload,
    }))
}
