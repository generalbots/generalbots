mod bus_sink;
mod inbound;
mod message_limit;
mod resume;
mod run_task;
mod takeover;

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use botcore::shared::state::AppState;
use futures_util::{SinkExt, StreamExt};
use log::error;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::message::run_start_bas_on_connect;
use crate::core::bot::runs::bus;

/// Sender half of the socket, shared with the background turn handlers.
pub(super) type WsSender =
    Arc<tokio::sync::Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>;

/// #2084 — a finished answer that the socket could not carry is kept for the
/// next connection of this session. The chat client always redials, and the
/// connect path already delivers `pending_stream_responses`, so the user sees
/// the reply instead of an empty bubble stuck on "Waiting for the model…".
async fn keep_final_frame(state: &Arc<AppState>, frame: &serde_json::Value) {
    let is_complete = frame
        .get("is_complete")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !is_complete {
        return;
    }
    let content = match frame.get("content").and_then(serde_json::Value::as_str) {
        Some(content) if !content.is_empty() => content,
        _ => return,
    };
    let session_key = match frame.get("session_id").and_then(serde_json::Value::as_str) {
        Some(key) => key.to_string(),
        None => return,
    };
    let mut pending = state.pending_stream_responses.lock().await;
    pending.insert(session_key, content.to_string());
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
    let ws_sender: WsSender = Arc::new(tokio::sync::Mutex::new(ws_sender));
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
    botlog::safe_info!(
        "WebSocket connected: bot={bot}, session={session_id}",
        bot = bot_name
    );

    let welcome = serde_json::json!({
        "type": "connected", "session_id": session_id, "user_id": user_id,
        "bot_id": bot_uuid, "message": "Connected to bot server", "tools": []
    });
    let _ = ws_sender
        .lock()
        .await
        .send(Message::Text(welcome.to_string()))
        .await;

    {
        let mut pending = state.pending_stream_responses.lock().await;
        if let Some(content) = pending.remove(&session_id.to_string()) {
            botlog::safe_info!(
                "Delivering pending stream response for session {session_id} ({len} bytes)",
                len = content.len()
            );
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
            let _ = ws_sender
                .lock()
                .await
                .send(Message::Text(resp.to_string()))
                .await;
        }
    }

    let mut start_bas_guard = ws_sender.lock().await;
    let mut start_bas_ran = run_start_bas_on_connect(
        &state,
        &mut start_bas_guard,
        &mut rx,
        bot_uuid,
        session_id,
        user_id,
        &bot_name,
    )
    .await;
    drop(start_bas_guard);

    // #1666 — the socket subscribes to the thread, it does not own the run.
    // Frames produced by a background turn (or another tab) arrive here too.
    // #2084 — the thread id comes from `bus::thread_id` so the sink that
    // publishes the LLM answer and this subscription can never disagree.
    let thread_id = bus::thread_id(session_id);
    let (_live_seq, mut bus_rx) = bus::bus().subscribe(&thread_id);

    // IMP-07 — the HTTP limiter stops at the upgrade. Every inbound frame is
    // metered here so one socket cannot drive an unbounded number of turns.
    let message_limiter = message_limit::MessageLimiter::new();

    loop {
        tokio::select! {
            frame = bus_rx.recv() => {
                match frame {
                    Some(frame) => {
                        let payload = frame.to_string();
                        if let Err(error) = ws_sender
                            .lock()
                            .await
                            .send(Message::Text(payload))
                            .await
                        {
                            // #2084 — this used to break silently: the answer
                            // was produced and published, then dropped with no
                            // trace. Keep it for the next connection.
                            botlog::safe_warn!(
                                "WS send failed on thread {}: {error}; reply kept for the next connection",
                                thread_id
                            );
                            keep_final_frame(&state, &frame).await;
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
                        if !message_limiter.allow() {
                            botlog::safe_warn!(
                                "WS message rate limit hit: session={session_id}; frame dropped"
                            );
                            let notice = serde_json::json!({
                                "type": "rate_limited",
                                "session_id": session_id,
                                "message": "Too many messages. Please slow down.",
                            });
                            let _ = ws_sender
                                .lock()
                                .await
                                .send(Message::Text(notice.to_string()))
                                .await;
                            continue;
                        }
                        // #cross-browser-sync — broadcast the user's input to every
                        // subscriber of this session so other browsers see the
                        // turn in real time, not just the bot's reply.
                        let parsed_msg = serde_json::from_str::<serde_json::Value>(&text).ok();
                        let frame_type = parsed_msg
                            .as_ref()
                            .and_then(|v| v.get("type").and_then(|t| t.as_str()))
                            .unwrap_or("");
                        let is_control = matches!(
                            frame_type,
                            "run_resume" | "run_cancel" | "run_takeover"
                        );
                        if !is_control {
                            let message_id = parsed_msg
                                .as_ref()
                                .and_then(|v| v.get("message_id"))
                                .and_then(|m| match m {
                                    serde_json::Value::String(s) if !s.is_empty() => {
                                        Some(s.clone())
                                    }
                                    serde_json::Value::Number(n) => {
                                        let s = n.to_string();
                                        if s.is_empty() {
                                            None
                                        } else {
                                            Some(s)
                                        }
                                    }
                                    _ => None,
                                });
                            if let Some(mid) = message_id {
                                let dedup_key = format!("ws:dedup:{session_id}:{user_id}:{mid}");
                                let mut duplicate = false;
                                if let Some(ref cache) = state.cache {
                                    match cache.get_multiplexed_async_connection().await {
                                        Ok(mut conn) => {
                                            let set_result: Result<Option<String>, _> =
                                                redis::cmd("SET")
                                                    .arg(&dedup_key)
                                                    .arg("1")
                                                    .arg("NX")
                                                    .arg("EX")
                                                    .arg(600_i64)
                                                    .query_async(&mut conn)
                                                    .await;
                                            match set_result {
                                                Ok(None) => duplicate = true,
                                                Ok(Some(_)) => {}
                                                Err(e) => {
                                                    botlog::safe_warn!(
                                                        "WS dedup check failed (continuing): {e}"
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            botlog::safe_warn!(
                                                "WS dedup cache unavailable (continuing): {e}"
                                            );
                                        }
                                    }
                                }
                                if duplicate {
                                    botlog::safe_info!(
                                        "WS duplicate message_id ignored: session={session_id} message_id={mid}"
                                    );
                                    let ack = serde_json::json!({
                                        "type": "message_received",
                                        "session_id": session_id,
                                        "message_id": mid,
                                    });
                                    let _ = ws_sender
                                        .lock()
                                        .await
                                        .send(Message::Text(ack.to_string()))
                                        .await;
                                    continue;
                                }
                            }
                        }
                        let msg_text = parsed_msg
                            .as_ref()
                            .and_then(|v| v.get("content").and_then(|c| c.as_str()).map(String::from))
                            .unwrap_or_else(|| text.clone());
                        let client_id = parsed_msg
                            .as_ref()
                            .and_then(|v| v.get("client_id").and_then(|c| c.as_str()).map(String::from))
                            .unwrap_or_default();
                        let user_frame = serde_json::json!({
                            "content": msg_text,
                            "message_type": 1,
                            "is_complete": true,
                            "session_id": session_id.to_string(),
                            "user_id": user_id.to_string(),
                            "sender_id": user_id.to_string(),
                            "sender_client_id": client_id,
                        });
                        let delivered = bus::bus().publish(&thread_id, user_frame);
                        botlog::safe_info!("Published user message to bus: thread={} delivered={}", thread_id, delivered);

                        inbound::handle(inbound::Inbound {
                            ws_sender: &ws_sender,
                            state: &state,
                            bot_uuid,
                            session_id,
                            user_id,
                            bot_name: &bot_name,
                            thread_id: &thread_id,
                            start_bas_ran: &mut start_bas_ran,
                            text,
                        })
                        .await;
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
    botlog::safe_info!("WebSocket disconnected: session={session_id}");
}
