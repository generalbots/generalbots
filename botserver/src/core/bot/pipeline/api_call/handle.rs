//! Executes a command call and answers the user: composed text or a one-line
//! description of the result.

use super::super::types::{LlmTarget, SessionIdentity};
use super::args::{extract_deep_links, inject_tax_value_from_user};
use super::humanize::humanize_result;
use super::parse::{extract_api_call_payload, strip_api_call_block};

/// Executes an `{"__api_call__": {"name", "params", "compose"}}` block found
/// in the LLM reply. Returns true when the call was handled (even on error),
/// so the caller skips the regular rendering path.
pub(crate) async fn handle_api_call(
    identity: &SessionIdentity<'_>,
    target: &LlmTarget<'_>,
    full_response: &str,
    user_text: &str,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
) -> bool {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        ..
    } = *identity;
    let LlmTarget {
        provider: llm,
        model,
        key,
    } = *target;
    use crate::core::bot::api_catalog;
    let payload = match extract_api_call_payload(full_response) {
        Some(p) => p,
        None => return false,
    };
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return false;
    }

    let params = payload
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    // #2264/#2239 — honor the per-turn Agent/VM switch. When Agent is off,
    // commands whose work happens inside the workspace VM (terminal runs,
    // Vibe agent runs, autotasks, workspace endpoints reached through
    // `api.exec`) must not run, even if the model emits them through the
    // api-catalog. The toggle is per-turn and can flip mid-conversation, so
    // resolve from the already-resolved identity.
    if super::super::agent_turn::command_runs_in_workspace(&name, &params)
        && !super::super::agent_turn::workspace_route_allowed(identity.agent_vm_enabled)
    {
        let msg = super::super::agent_turn::DISABLED_MESSAGE;
        botlog::safe_info!(
            "api_call '{name}' refused for session {session_id}: Agent/VM is off"
        );
        let channel_name = sink.channel_type().to_string();
        let resp = botlib::models::BotResponse::new(
            bot_uuid.to_string(),
            session_id.to_string(),
            user_id.to_string(),
            msg,
            &channel_name,
        );
        let _ = sink.send_bot_response(&resp).await;
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 2, msg, 2);
        return true;
    }
    let compose = payload
        .get("compose")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let channel = sink.channel_type().to_string();

    // Session tools (USE TOOL / AutoTask, e.g. classify_media) are NOT part
    // of the declarative api-catalog; models sometimes emit them wrapped in
    // an __api_call__ block, which used to fail with "unknown command".
    // When the requested name belongs to the session's tool set, reroute to
    // the regular MCP tool-call executor with equivalent arguments.
    if crate::core::bot::tool_context::is_tool_associated_with_session(
        &state.conn,
        &session_id,
        &name,
    ) {
        log::info!("api_call '{name}' is a session tool; rerouting to tool executor");
        // run_llm_tool_call parses a FLAT payload: name + arguments-as-string
        // on the same level as the __tool_call__ marker.
        let args_str = serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string());
        let args_escaped =
            serde_json::to_string(&args_str).unwrap_or_else(|_| "\"{}\"".to_string());
        let synthetic = format!(
            "{{\"__tool_call__\": true, \"name\": \"{name}\", \"arguments\": {args_escaped}}}"
        );
        let _ = crate::core::bot::pipeline::tool_exec::run_llm_tool_call(
            identity, rx, &synthetic, user_text,
        )
        .await;
        return true;
    }

    // The LLM may have spoken a short "working on it..." line before the
    // __api_call__ block. Surface it to the user so the tool call feels
    // natural, then run the command. The JSON block itself is always
    // stripped out of the leading text (never shown to the user).
    let stripped = strip_api_call_block(full_response);
    let leading = stripped.trim();
    if !leading.is_empty() {
        let progress = botlib::models::BotResponse::new(
            bot_uuid.to_string(),
            session_id.to_string(),
            user_id.to_string(),
            leading,
            &channel,
        );
        let _ = sink.send_bot_response(&progress).await;
        // Persist the leading line so history replay shows it instead of the
        // raw __api_call__ envelope (#consent text-instead-of-buttons).
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 2, leading, 2);
    }

    // Tax commands: models frequently omit the `value` param even when the
    // user stated an amount. Recover it from the user's message (issue #722).
    let params = if matches!(name.as_str(), "service.tax" | "tax.calculate") {
        inject_tax_value_from_user(&params, user_text)
    } else {
        params
    };

    #[cfg(feature = "consent")]
    {
        let app_id = crate::apps::commands::command_by_name(&name)
            .map(|c| c.app.to_string())
            .unwrap_or_else(|| name.clone());
        let gate = crate::core::bot::consent_gate::check_with_context(
            user_id,
            &app_id,
            "update",
            serde_json::json!({ "command": name, "params": params }),
            Some(session_id.to_string()),
            crate::core::bot::runs::bus::bus().current_run_id(&session_id.to_string()),
        )
        .await;
        if !gate.allowed {
            let message = match gate.pending_request {
                Some(ref request) => crate::core::bot::consent_gate::marker_content(request),
                None => crate::core::bot::consent_gate::deny_message(&app_id),
            };
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &message,
                &channel,
            );
            let _ = sink.send_bot_response(&resp).await;
            // Persist the consent marker (or denial) so a page reload still
            // renders the interactive card instead of raw JSON.
            {
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &message, 2);
            }
            return true;
        }
    }

    match api_catalog::execute_command(state, bot_uuid, user_id, &name, &params).await {
        Ok(result) => {
            // #1835 — when the model itself runs web.search, surface its hits
            // as citation chips too, so informational answers show references.
            if name == "web.search" {
                if let Some(results) = result.get("results").and_then(|v| v.as_array()) {
                    let mut citations = Vec::new();
                    for r in results {
                        let title = r
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let url = r
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if url.is_empty() {
                            continue;
                        }
                        let host = url
                            .split("://")
                            .nth(1)
                            .unwrap_or(&url)
                            .split('/')
                            .next()
                            .unwrap_or(&url)
                            .trim_start_matches("www.")
                            .to_string();
                        citations.push(serde_json::json!({
                            "title": title,
                            "url": url,
                            "host": host,
                        }));
                    }
                    if !citations.is_empty() {
                        let mut pending = state.pending_citations.lock().await;
                        pending.insert(session_id.to_string(), citations);
                    }
                }
            }
            if compose {
                let deep_links = extract_deep_links(&result);
                // Grounding: the model once answered "I created the automation"
                // after a discovery-only command returned nothing but a match
                // list, and nothing was ever created. Spell out what the data
                // does and does not support.
                let created_something = result
                    .get("created_resources")
                    .and_then(|c| c.as_array())
                    .is_some_and(|a| !a.is_empty());
                let performed_action = result
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false)
                    || created_something;
                let grounding = if performed_action {
                    "The data confirms the action was carried out: report it as done.\n"
                } else {
                    "The data does NOT show that any action was carried out — it is a lookup, a search \
                     or an empty result. Never tell the user that something was created, saved, sent or \
                     scheduled. If they asked for an action, say you could not perform it yet and what \
                     you would need.\n"
                };
                let prompt = format!(
                    "You are the assistant of bot '{bot_name}'. The user asked: \"{user_text}\".\n\
                     You ran the command '{name}' and received this data:\n{json}\n\n\
                     {grounding}\
                     Write a concise, friendly answer for the user in the language of the user's message, \
                     using the data. Never mention JSON, commands or internal details.\n\
                     {deep_link_instruction}",
                    json = serde_json::to_string_pretty(&result).unwrap_or_default(),
                    grounding = grounding,
                    deep_link_instruction = if name == "web.search" {
                        // #1547 — answers built from search results must show
                        // where each claim came from, and must never invent a
                        // source when the results do not cover the question.
                        "The results are live web search hits. Cite the source of every factual or \
                         time-sensitive claim as a markdown link using the exact title and url from \
                         the results, e.g. [title](url). Never invent a url or a source. If the \
                         results do not answer the question, say what is missing instead of guessing."
                            .to_string()
                    } else if deep_links.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "The result references specific records. End your answer with a clickable \
                             link to the most relevant record, in the exact form provided (it starts \
                             with app://), e.g. '{first}'.",
                            first = deep_links[0],
                        )
                    },
                );
                // A rerouted tool call may arrive before a provider is resolved
                // for this bot; the command result is then reported raw.
                let composed = match llm {
                    Some(llm) => llm
                        .generate(&prompt, &serde_json::json!({}), model, key)
                        .await
                        .ok(),
                    None => None,
                };
                match composed {
                    Some(text) => {
                        let resp = botlib::models::BotResponse::new(
                            bot_uuid.to_string(),
                            session_id.to_string(),
                            user_id.to_string(),
                            &text,
                            &channel,
                        );
                        let _ = sink.send_bot_response(&resp).await;
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &text, 2);
                    }
                    None => {
                        // Never lose the result to a composition failure: the
                        // command's own message is the most useful thing we
                        // have, and a bare apology told the user nothing.
                        let command_message = result
                            .get("message")
                            .and_then(|m| m.as_str())
                            .map(str::trim)
                            .filter(|m| !m.is_empty())
                            .map(str::to_string);
                        let fallback = match (command_message, deep_links.first()) {
                            (Some(message), _) => message,
                            (None, Some(link)) => {
                                format!("Here is the record you asked about: {link}")
                            }
                            (None, None) => format!(
                                "I ran `{name}` but could not write the answer from its result. \
                                 Ask me again and I will retry."
                            ),
                        };
                        let resp = botlib::models::BotResponse::new(
                            bot_uuid.to_string(),
                            session_id.to_string(),
                            user_id.to_string(),
                            &fallback,
                            &channel,
                        );
                        let _ = sink.send_bot_response(&resp).await;
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &fallback, 2);
                    }
                }
            } else {
                // Never paste the command envelope into the chat: a discovery
                // call (api.find) used to render its whole JSON catalog on the
                // page. Speak about the result instead.
                let text = humanize_result(&name, &result);
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(),
                    session_id.to_string(),
                    user_id.to_string(),
                    &text,
                    &channel,
                );
                let _ = sink.send_bot_response(&resp).await;
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &text, 2);
            }
            true
        }
        Err(e) => {
            log::warn!("api_call '{name}' failed: {e}");
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                format!("Falha ao executar {name}: {e}"),
                &channel,
            );
            let _ = sink.send_bot_response(&resp).await;
            true
        }
    }
}
