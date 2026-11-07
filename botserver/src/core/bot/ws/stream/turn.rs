use std::sync::Arc;

use axum::extract::ws::Message;
use futures_util::SinkExt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::bot::runs::caps::{CapBreach, RunBudget};

use super::{frames, ttft, LlmTurn, WsSink};

/// Everything the streaming turn hands back to the caller.
pub(super) struct TurnResult {
    pub final_content: String,
    pub reasoning: String,
    pub full_response: String,
    pub capped: bool,
}

pub(super) async fn run(
    ws_sender: &mut WsSink,
    turn: &LlmTurn<'_>,
    llm: Arc<dyn botlib::traits::LLMProvider>,
    llm_key: String,
    llm_model: String,
    mut budget: RunBudget,
) -> TurnResult {
    let LlmTurn {
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        messages,
        user_text,
    } = *turn;
    let session_id_s = session_id.to_string();
    let bot_name_clone = bot_name.to_string();

    let style_css = crate::core::bot::ws::message::load_bot_styles_css(&bot_name_clone);

    let session_tools = {
        let sid: Uuid = match session_id_s.parse() {
            Ok(id) => id,
            Err(_) => Uuid::new_v4(),
        };
        let pool = state.conn.clone();
        let name = bot_name_clone.clone();
        let sid_for_block = sid;
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::task::spawn_blocking(move || {
                crate::core::bot::tool_context::get_session_tools(&pool, &name, &sid_for_block)
            }),
        )
        .await
        {
            Ok(Ok(Ok(tools))) => {
                botlog::safe_info!(
                    "Loaded {} tools for LLM session {}",
                    tools.len(),
                    session_id_s
                );
                tools
            }
            Ok(Ok(Err(e))) => {
                botlog::safe_warn!("get_session_tools error for {}: {}", session_id_s, e);
                Vec::new()
            }
            Ok(Err(e)) => {
                botlog::safe_warn!(
                    "get_session_tools spawn_blocking join error for {}: {}",
                    session_id_s,
                    e
                );
                Vec::new()
            }
            Err(_) => {
                botlog::safe_warn!(
                    "get_session_tools TIMEOUT after 10s for session {}",
                    session_id_s
                );
                Vec::new()
            }
        }
    };
    // #1711 — the native structured tools must be callable through the
    // provider's tool API. This path serves the web socket only, which is where
    // the renderers exist.
    let mut session_tools = session_tools;
    if crate::core::bot::pipeline::event_sink::enabled() {
        session_tools.extend(crate::core::bot::pipeline::surface_catalog::llm_tools());
    }

    let steer_llm = llm.clone();
    let steer_key = llm_key.clone();
    let steer_model = llm_model.clone();
    let steer_tools = if session_tools.is_empty() {
        None
    } else {
        Some(session_tools.clone())
    };
    let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
    let prompt_clone = user_text.to_string();
    let messages_clone = messages.clone();
    let llm_key_clone = llm_key.clone();
    let llm_model_clone = llm_model.clone();
    let mut stream_handle = tokio::spawn(async move {
        botlog::safe_info!("LLM spawn task starting: model={}", llm_model_clone);
        let tools_arg = if session_tools.is_empty() {
            None
        } else {
            Some(session_tools)
        };
        if let Err(e) = llm
            .generate_stream(
                &prompt_clone,
                &messages_clone,
                stream_tx,
                &llm_model_clone,
                &llm_key_clone,
                tools_arg.as_ref(),
            )
            .await
        {
            log::error!("LLM stream error: {}", e);
        } else {
            log::info!("LLM spawn task completed successfully");
        }
    });

    // #1700 — first-token latency is recorded once per turn at the first
    // content chunk.
    let llm_started_at = std::time::Instant::now();
    let mut ttft_logged = false;

    {
        let init_msg = frames::init(turn, &style_css);
        if style_css.is_empty() {
            log::info!(
                "CSS NOT included - style_css was empty for bot {}",
                bot_name_clone
            );
        }
        let _ = ws_sender.send(Message::Text(init_msg.to_string())).await;
    }

    let mut full_response = String::new();
    let mut reasoning_accumulated = String::new();
    // #1701 — messages sent while this turn streams are honored inside the run
    // (bounded), not only at the turn boundary.
    let mut steering_rounds: u8 = 0;
    let mut cap_breach: Option<CapBreach> = None;

    let mut keepalive_interval =
        tokio::time::interval(std::time::Duration::from_millis(800));
    keepalive_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            chunk = stream_rx.recv() => {
                match chunk {
                    Some(chunk) => {
                        if chunk.contains("\"__reasoning__\"") {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&chunk) {
                                if let Some(r) = val.get("__reasoning__").and_then(|v| v.as_str()) {
                                    reasoning_accumulated.push_str(r);
                                }
                            }
                            continue;
                        }
                        if chunk.contains("\"__tool_call__\"") {
                            full_response.push_str(&chunk);
                            continue;
                        }
                        full_response.push_str(&chunk);
                        if !ttft_logged {
                            ttft_logged = true;
                            let ttft_ms = i64::try_from(llm_started_at.elapsed().as_millis())
                                .unwrap_or(i64::MAX);
                            ttft::record(state, bot_uuid, session_id, &llm_model, ttft_ms);
                        }
                        // #1696 — stop spending as soon as a limit is reached;
                        // the final frame explains why.
                        budget.observe_chars(full_response.chars().count());
                        if let Some(breach) = budget.check() {
                            botlog::safe_warn!(
                                "run cap: {} (session {})",
                                RunBudget::describe(&breach),
                                session_id_s
                            );
                            cap_breach = Some(breach);
                            break;
                        }
                        let chunk_resp = frames::chunk(turn, chunk);
                        if ws_sender.send(Message::Text(chunk_resp.to_string())).await.is_err() {
                            break;
                        }
                        // #1701 — honor a message sent mid-run before the next
                        // step: abort this provider stream and continue with the
                        // partial answer plus the steering instruction.
                        if steering_rounds < crate::core::bot::runs::steering::MAX_STEERS_PER_RUN {
                            let thread_id = session_id_s.clone();
                            if let Some(steer) =
                                crate::core::bot::runs::bus::bus().take_steering(&thread_id)
                            {
                                steering_rounds += 1;
                                botlog::safe_warn!(
                                    "steering mid-run (session {}, {} chars)",
                                    session_id_s,
                                    steer.chars().count()
                                );
                                stream_handle.abort();
                                let mut steered = messages.clone();
                                if let Some(items) = steered.as_array_mut() {
                                    let partial = full_response.trim();
                                    if !partial.is_empty() {
                                        items.push(serde_json::json!({
                                            "role": "assistant",
                                            "content": partial,
                                        }));
                                    }
                                    items.push(serde_json::json!({
                                        "role": "user",
                                        "content": crate::core::bot::runs::steering::instruction(&steer),
                                    }));
                                }
                                let notice = serde_json::json!({
                                    "type": "gb.steer",
                                    "session_id": session_id_s,
                                    "message": steer,
                                });
                                let _ = ws_sender
                                    .send(Message::Text(notice.to_string()))
                                    .await;
                                crate::core::bot::runs::bus::bus().publish(
                                    &thread_id,
                                    serde_json::json!({
                                        "type": "CUSTOM",
                                        "session_id": session_id_s,
                                        "data": {
                                            "name": "gb.steer",
                                            "payload": { "message": steer },
                                        },
                                    }),
                                );
                                full_response.clear();
                                reasoning_accumulated.clear();
                                let (steer_tx, steer_rx) = mpsc::channel::<String>(100);
                                stream_rx = steer_rx;
                                let llm2 = steer_llm.clone();
                                let key2 = steer_key.clone();
                                let model2 = steer_model.clone();
                                let prompt2 = user_text.to_string();
                                let tools2 = steer_tools.clone();
                                stream_handle = tokio::spawn(async move {
                                    if let Err(e) = llm2
                                        .generate_stream(
                                            &prompt2,
                                            &steered,
                                            steer_tx,
                                            &model2,
                                            &key2,
                                            tools2.as_ref(),
                                        )
                                        .await
                                    {
                                        log::error!("LLM stream error after steering: {}", e);
                                    }
                                });
                                continue;
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = keepalive_interval.tick() => {
                let _ = ws_sender.send(Message::Text(frames::keepalive(turn).to_string())).await;
            }
        }
    }

    let capped = cap_breach.is_some();
    let final_content = if let Some(breach) = cap_breach.take() {
        // Stop the producer too: consuming the loop alone would leave the
        // provider stream running and still spending.
        stream_handle.abort();
        let notice = format!("Run stopped — {}.", RunBudget::describe(&breach));
        // Persist the stopping message so the run reads the same after a
        // reconnect or on another device.
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 2, &notice, 2);
        notice
    } else {
        let mut content = full_response.trim_end().to_string();
        if let Some(pos) = content.rfind("{\"__tool_call__\":") {
            content = content[..pos].trim_end().to_string();
        }
        content
    };

    TurnResult {
        final_content,
        reasoning: reasoning_accumulated,
        full_response,
        capped,
    }
}
