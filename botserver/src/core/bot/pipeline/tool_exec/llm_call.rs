//! LLM tool-call execution: parse the model's call, gate it, then hand the
//! compiled script to `script::run_script_tool`.

use super::is_placeholder_value::all_args_are_placeholder;
use super::parse::{
    extract_tool_call, parse_dsml_tool_call, parse_xml_tool_use, recover_tool_name,
    salvage_tool_call,
};
use super::script::run_script_tool;
use super::super::types::SessionIdentity;

pub async fn run_llm_tool_call(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    full_response: &str,
    user_text: &str,
) -> String {
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
            return Box::pin(run_llm_tool_call(identity, rx, &text, user_text)).await;
        }
    }
    let tool_call_trigger = "\"__tool_call__\":";
    let tc_start = match full_response.find(tool_call_trigger) {
        Some(pos) => full_response[..pos].rfind('{').unwrap_or(pos),
        None => return String::new(),
    };
    let tc_json = &full_response[tc_start..];
    let tool_call = match extract_tool_call(tc_json).or_else(|| salvage_tool_call(tc_json)) {
        Some(value) => value,
        None => {
            botlog::safe_warn!(
                "tool_call: could not parse the model's JSON call; reply starts with: {}",
                tc_json.chars().take(200).collect::<String>()
            );
            return String::new();
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
        if let Some(staged) = crate::core::bot::pipeline::attachments::find_staged_attachment(user_text) {
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
            return String::new();
        }

        if all_args_are_placeholder(tool_args) {
            log::info!("All tool args are placeholder - asking user for real data instead of executing");
            let msg = "I need the actual values before I can run that tool. Please provide the requested details.".to_string();
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(), &msg, sink.channel_type(),
            );
            let _ = sink.send_bot_response(&resp).await;
            return String::new();
        }

        if crate::core::bot::pipeline::tool_exec::is_generic_greeting(user_text) {
            log::info!("Blocking tool '{raw_tool_name}' on short generic user message: '{user_text}'");
            return String::new();
        }

        // Native infrastructure tools (#1582) have no per-bot `.ast`: the
        // workspace executor is available to every bot with an agent VM.
        // Their output goes straight to the sink, so capture it for the
        // model follow-up before running.
        crate::core::bot::pipeline::native_tools::begin_capture(session_id);
        if crate::core::bot::pipeline::native_tools::try_run(identity, &raw_tool_name, tool_args).await {
            return crate::core::bot::pipeline::native_tools::take_capture(session_id);
        }

        let tool_name = match validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("LLM tool_call: invalid tool name '{raw_tool_name}': {e}");
                return String::new();
            }
        };

        let work_path = get_work_path();
        let rel_tool_path = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{tool_name}.ast");
        if !verify_path_within_workdir(&rel_tool_path) {
            log::error!("Path traversal detected in LLM tool_call for tool: {tool_name}");
            return String::new();
        }

        // The org directory is named after the org id (`<uuid>.gborg`), not the
        // bot: assuming `{bot}.gborg` meant every tool call resolved to a
        // missing file and was dropped as "no compiled script".
        let ast_path = crate::core::bot::pipeline::tool_paths::resolve_bot_file(&work_path, &bot_name, &format!("{tool_name}.ast"));
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
                let (model, key) = super::is_placeholder_value::bot_llm_credentials(&state.conn, bot_uuid);
                // Boxed: the command path can reroute a session tool back
                // into this executor, so the two futures are mutually
                // recursive.
                let llm_target = crate::core::bot::pipeline::types::LlmTarget {
                    provider,
                    model: &model,
                    key: &key,
                };
                Box::pin(crate::core::bot::pipeline::api_call::handle_api_call(
                    identity, &llm_target, &api_call, user_text, rx,
                ))
                .await;
                return String::new();
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
            return String::new();
        }

        run_script_tool(
            identity,
            rx,
            &tool_name,
            &ast_content,
            tool_args,
            &work_path,
        )
        .await
    }
}
