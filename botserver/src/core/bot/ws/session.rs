use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use botevents::{Envelope, EventSource, EventType};
use botcore::shared::state::AppState;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::message::run_start_bas_on_connect;
use crate::core::bot::pipeline::{self, ChannelSink, PipelineError, SessionIdentity};
use crate::core::bot::runs::{bus, store};

/// Sink used by a background turn: frames are published to the thread channel
/// so every subscribed socket receives them, and the turn keeps running after
/// the socket that started it is gone.
struct RunBusSink {
    thread_id: String,
}

#[async_trait::async_trait]
impl ChannelSink for RunBusSink {
    async fn send_bot_response(
        &self,
        response: &botlib::models::BotResponse,
    ) -> Result<(), PipelineError> {
        let frame = serde_json::to_value(response)
            .map_err(|error| PipelineError::Transport(format!("run frame encode: {error}")))?;
        bus::bus().publish(&self.thread_id, frame);
        Ok(())
    }

    async fn send_raw_json(&self, json: &serde_json::Value) -> Result<(), PipelineError> {
        bus::bus().publish(&self.thread_id, json.clone());
        Ok(())
    }

    async fn send_error(&self, session_id: &str, message: &str) -> Result<(), PipelineError> {
        let response = botlib::models::BotResponse::new("", session_id, "", message, "web");
        self.send_bot_response(&response).await
    }

    fn channel_type(&self) -> &str {
        "web"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_suggestions(&self) -> bool {
        true
    }

    fn supports_raw_frames(&self) -> bool {
        true
    }
}

pub async fn handle_ws(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: String,
) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));
    let (tx, mut rx) = mpsc::channel::<botlib::models::BotResponse>(100);
    let key = session_id.to_string();
    // Keep a handle so disconnect can prove ownership of the map entry.
    let tx_handle = tx.clone();
    {
        let mut channels = state.response_channels.lock().await;
        // #1288 — parallel chat tabs mean MULTIPLE sockets can carry the same
        // session_id. Overwriting the previous sender dropped it, ending that
        // socket's recv loop and closing it (stock tab vs parallel tab
        // flapping accept/close forever). First socket wins the entry: each
        // socket still receives replies to its OWN messages via its local rx;
        // the map only routes async/pushed frames.
        channels.entry(key.clone()).or_insert(tx);
    }
    info!("WebSocket connected: bot={bot}, session={session_id}", bot = bot_name);

    let welcome = serde_json::json!({
        "type": "connected", "session_id": session_id, "user_id": user_id,
        "bot_id": bot_uuid, "message": "Connected to bot server", "tools": []
    });
    let _ = ws_sender.lock().await.send(Message::Text(welcome.to_string())).await;

    {
        let mut pending = state.pending_stream_responses.lock().await;
        if let Some(content) = pending.remove(&session_id.to_string()) {
            info!("Delivering pending stream response for session {session_id} ({len} bytes)", len = content.len());
            let resp = serde_json::json!({
                "bot_id": bot_uuid.to_string(),
                "user_id": user_id.to_string(),
                "session_id": session_id.to_string(),
                "channel": "web",
                "content": content,
                "message_type": 2,
                "is_complete": true,
                "suggestions": [],
                "switchers": [],
                "context_length": 0,
                "context_max_length": 0,
            });
            let _ = ws_sender.lock().await.send(Message::Text(resp.to_string())).await;
        }
    }

    let mut start_bas_guard = ws_sender.lock().await;
    let mut start_bas_ran = run_start_bas_on_connect(
        &state, &mut start_bas_guard, &mut rx,
        bot_uuid, session_id, user_id, &bot_name,
    ).await;
    drop(start_bas_guard);

    // #1666 — the socket subscribes to the thread, it does not own the run.
    // Frames produced by a background turn (or another tab) arrive here too.
    let thread_id = session_id.to_string();
    let (_live_seq, mut bus_rx) = bus::bus().subscribe(&thread_id);

    loop {
        tokio::select! {
            frame = bus_rx.recv() => {
                match frame {
                    Some(frame) => {
                        if ws_sender.lock().await.send(Message::Text(frame.to_string())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            response = rx.recv() => {
                if let Some(response) = response {
                    if let Ok(json) = serde_json::to_string(&response) {
                        if ws_sender.lock().await.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        info!(
                            "WS frame received: session={session_id} bytes={} head={}",
                            text.len(),
                            text.chars().take(70).collect::<String>()
                        );
                        let parsed: Option<serde_json::Value> = serde_json::from_str(&text).ok();
                        let frame_type = parsed
                            .as_ref()
                            .and_then(|value| value.get("type"))
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");

                        // #1666 — resume after a reconnect: replay only the
                        // frames the client has not seen.
                        if frame_type == "run_resume" {
                            let after_seq = parsed
                                .as_ref()
                                .and_then(|value| value.get("last_seq"))
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0);
                            match state.conn.get() {
                                Ok(mut conn) => {
                                    match store::load_thread(&mut conn, bot_uuid, &thread_id, after_seq) {
                                        Ok(events) => {
                                            for envelope in events {
                                                match envelope.to_json() {
                                                    Ok(mut frame) => {
                                                        if let Some(map) = frame.as_object_mut() {
                                                            map.insert(
                                                                "gb_event".to_string(),
                                                                serde_json::Value::Bool(true),
                                                            );
                                                        }
                                                        let _ = ws_sender
                                                            .lock()
                                                            .await
                                                            .send(Message::Text(frame.to_string()))
                                                            .await;
                                                    }
                                                    Err(error) => warn!("resume frame encode failed: {error}"),
                                                }
                                            }
                                        }
                                        Err(error) => warn!("run resume load failed: {error}"),
                                    }
                                }
                                Err(error) => warn!("run resume skipped: pool error: {error}"),
                            }
                            let done = serde_json::json!({
                                "type": "run_resume_done",
                                "session_id": session_id,
                                "last_seq": after_seq,
                            });
                            let _ = ws_sender.lock().await.send(Message::Text(done.to_string())).await;
                            continue;
                        }

                        if frame_type == "run_cancel" {
                            bus::bus().cancel(&thread_id);
                            let ack = serde_json::json!({
                                "type": "run_cancelled",
                                "session_id": session_id,
                            });
                            let _ = ws_sender.lock().await.send(Message::Text(ack.to_string())).await;
                            continue;
                        }

                        // #1677 — takeover/hand-back: the user takes over the
                        // live terminal or browser surface. The segment is
                        // persisted as a CUSTOM run event (seq'd) so a reload
                        // or a second device restores it, then broadcast to
                        // every subscriber of this thread.
                        if frame_type == "run_takeover" {
                            let state_label = parsed
                                .as_ref()
                                .and_then(|value| value.get("state"))
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("taken_over")
                                .to_string();
                            let surface = parsed
                                .as_ref()
                                .and_then(|value| value.get("surface"))
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("terminal")
                                .to_string();
                            let run_id = parsed
                                .as_ref()
                                .and_then(|value| value.get("run_id"))
                                .and_then(serde_json::Value::as_str)
                                .filter(|value| !value.trim().is_empty())
                                .map(str::to_string)
                                .or_else(|| bus::bus().current_run_id(&thread_id))
                                .unwrap_or_default();
                            if run_id.is_empty() {
                                let denied = serde_json::json!({
                                    "type": "run_takeover_ack",
                                    "accepted": false,
                                    "message": "No active run to take over",
                                    "session_id": session_id,
                                });
                                let _ = ws_sender.lock().await.send(Message::Text(denied.to_string())).await;
                                continue;
                            }
                            let seq = persist_takeover(
                                &state,
                                bot_uuid,
                                user_id,
                                &thread_id,
                                &run_id,
                                &state_label,
                                &surface,
                            );
                            #[cfg(feature = "consent")]
                            if let Ok(mut conn) = state.conn.get() {
                                botconsent::store::audit(
                                    &mut conn,
                                    None,
                                    Some(user_id),
                                    &serde_json::json!({
                                        "action": "run_takeover",
                                        "thread_id": thread_id,
                                        "run_id": run_id,
                                        "state": state_label,
                                        "surface": surface,
                                    }),
                                    "allowed",
                                    None,
                                );
                            }
                            let frame = serde_json::json!({
                                "type": "CUSTOM",
                                "run_id": run_id,
                                "seq": seq,
                                "data": {
                                    "name": "gb.takeover",
                                    "payload": { "state": state_label, "surface": surface },
                                },
                            });
                            bus::bus().publish(&thread_id, frame);
                            let ack = serde_json::json!({
                                "type": "run_takeover_ack",
                                "accepted": true,
                                "state": state_label,
                                "seq": seq,
                            });
                            let _ = ws_sender.lock().await.send(Message::Text(ack.to_string())).await;
                            continue;
                        }

                        // Receipt: the client clears its retry timer on this
                        // frame only, so a turn accepted here is never resent
                        // and one dropped by a dying socket is.
                        let ack = serde_json::json!({
                            "type": "message_received",
                            "session_id": session_id,
                        });
                        let _ = ws_sender
                            .lock()
                            .await
                            .send(Message::Text(ack.to_string()))
                            .await;

                        // #1677 — a message sent during a run is steering:
                        // queue it, acknowledge and deliver at the next turn
                        // boundary instead of interleaving two agents.
                        if bus::bus().is_running(&thread_id) {
                            bus::bus().push_steering(&thread_id, &text);
                            let queued = serde_json::json!({
                                "type": "gb.steering_queued",
                                "session_id": session_id,
                                "message": "Queued. The agent will pick this up at the next step.",
                            });
                            let _ = ws_sender.lock().await.send(Message::Text(queued.to_string())).await;
                            continue;
                        }

                        let run_id = Uuid::new_v4().to_string();
                        bus::bus().start(&thread_id, &run_id);
                        let task_thread = thread_id.clone();
                        let task_bot_name = bot_name.clone();
                        let task_state = state.clone();
                        let task_text = text;
                        let mut task_start_bas_ran = start_bas_ran;
                        start_bas_ran = true;
                        tokio::spawn(async move {
                            let sink = RunBusSink {
                                thread_id: task_thread.clone(),
                            };
                            let identity = SessionIdentity {
                                sink: &sink,
                                state: &task_state,
                                bot_uuid,
                                session_id,
                                user_id,
                                bot_name: &task_bot_name,
                            };
                            let mut next_text = Some(task_text);
                            let mut first_turn = true;
                            while let Some(turn_text) = next_text {
                                if !first_turn {
                                    let next_run = Uuid::new_v4().to_string();
                                    bus::bus().start(&task_thread, &next_run);
                                }
                                let (closed_tx, mut task_rx) =
                                    mpsc::channel::<botlib::models::BotResponse>(1);
                                drop(closed_tx);
                                let result = pipeline::process_message_internal(
                                    &identity,
                                    &mut task_rx,
                                    &mut task_start_bas_ran,
                                    &turn_text,
                                )
                                .await;
                                match result {
                                    Ok(()) => {
                                        bus::bus().finish(&task_thread, botevents::RunState::Succeeded)
                                    }
                                    Err(error) => {
                                        error!("pipeline task failed: {error}");
                                        bus::bus().finish(&task_thread, botevents::RunState::Failed);
                                        break;
                                    }
                                }
                                // Deliver steering at the turn boundary. The
                                // queued text is processed exactly once.
                                next_text = bus::bus().take_steering(&task_thread);
                                first_turn = false;
                            }
                        });
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.lock().await.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { error!("WS err: {e}"); break; }
                    _ => {}
                }
            }
        }
    }

    {
        let mut channels = state.response_channels.lock().await;
        // #1288 — remove only OUR registration: with parallel tabs another
        // socket may own this session key, and a blind remove would leave it
        // without an entry for pushed frames.
        if channels
            .get(&key)
            .is_some_and(|stored| stored.same_channel(&tx_handle))
        {
            channels.remove(&key);
        }
    }
    if let Ok(mut hear_map) = state.hear_channels.lock() {
        hear_map.remove(&session_id);
    }
    info!("WebSocket disconnected: session={session_id}");
}
/// Persists one takeover segment as a CUSTOM run event (#1677). Returns the
/// allocated sequence, or 0 when persistence is unavailable — the live frame
/// still reaches subscribers in that case.
fn persist_takeover(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    thread_id: &str,
    run_id: &str,
    state_label: &str,
    surface: &str,
) -> u64 {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            warn!("takeover event skipped: pool error: {error}");
            return 0;
        }
    };
    let seq = match store::last_seq(&mut conn, bot_uuid, thread_id) {
        Ok(last) => last.saturating_add(1),
        Err(error) => {
            warn!("takeover seq lookup failed: {error}");
            return 0;
        }
    };
    let envelope = Envelope {
        v: 1,
        event_type: EventType::Custom,
        thread_id: thread_id.to_string(),
        run_id: run_id.to_string(),
        parent_run_id: None,
        seq,
        event_id: Uuid::new_v4().to_string(),
        ts: chrono::Utc::now(),
        source: EventSource::User,
        data: serde_json::json!({
            "name": "gb.takeover",
            "payload": { "state": state_label, "surface": surface },
        }),
    };
    if let Err(error) = store::append(&mut conn, &envelope, bot_uuid, Some(user_id), None) {
        warn!("takeover event persist failed: {error}");
        return 0;
    }
    seq
}
