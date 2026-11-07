use std::sync::Arc;

use axum::extract::ws::Message;
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use serde_json::Value;
use uuid::Uuid;

use super::{resume, run_task, takeover, WsSender};
use crate::core::bot::runs::bus;

/// One inbound text frame plus everything the handlers need from the socket.
pub(super) struct Inbound<'a> {
    pub(super) ws_sender: &'a WsSender,
    pub(super) state: &'a Arc<AppState>,
    pub(super) bot_uuid: Uuid,
    pub(super) session_id: Uuid,
    pub(super) user_id: Uuid,
    pub(super) bot_name: &'a str,
    pub(super) thread_id: &'a str,
    pub(super) start_bas_ran: &'a mut bool,
    pub(super) text: String,
}

pub(super) async fn handle(frame: Inbound<'_>) {
    let Inbound {
        ws_sender,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
        thread_id,
        start_bas_ran,
        text,
    } = frame;
    botlog::safe_info!(
        "WS frame received: session={session_id} bytes={}",
        text.len()
    );
    let parsed: Option<Value> = serde_json::from_str(&text).ok();
    let frame_type = parsed
        .as_ref()
        .and_then(|value| value.get("type"))
        .and_then(Value::as_str)
        .unwrap_or("");

    // #1666 — resume after a reconnect: replay only the frames the client has
    // not seen.
    if frame_type == "run_resume" {
        let after_seq = parsed
            .as_ref()
            .and_then(|value| value.get("last_seq"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        resume::replay(state, ws_sender, bot_uuid, session_id, thread_id, after_seq).await;
        return;
    }

    if frame_type == "run_cancel" {
        bus::bus().cancel(thread_id);
        let ack = serde_json::json!({
            "type": "run_cancelled",
            "session_id": session_id,
        });
        let _ = ws_sender
            .lock()
            .await
            .send(Message::Text(ack.to_string()))
            .await;
        return;
    }

    if frame_type == "run_takeover" {
        takeover::handle(
            ws_sender,
            state,
            bot_uuid,
            user_id,
            session_id,
            thread_id,
            parsed.as_ref(),
        )
        .await;
        return;
    }

    // Receipt: the client clears its retry timer on this frame only, so a turn
    // accepted here is never resent and one dropped by a dying socket is.
    let ack = serde_json::json!({
        "type": "message_received",
        "session_id": session_id,
    });
    let _ = ws_sender
        .lock()
        .await
        .send(Message::Text(ack.to_string()))
        .await;

    // #1677 — a message sent during a run is steering: queue it, acknowledge
    // and deliver at the next turn boundary instead of interleaving two agents.
    if bus::bus().is_running(thread_id) {
        bus::bus().push_steering(thread_id, &text);
        let queued = serde_json::json!({
            "type": "gb.steering_queued",
            "session_id": session_id,
            "message": "Queued. The agent will pick this up at the next step.",
        });
        let _ = ws_sender
            .lock()
            .await
            .send(Message::Text(queued.to_string()))
            .await;
        return;
    }

    let run_id = Uuid::new_v4().to_string();
    bus::bus().start(thread_id, &run_id);
    let task_start_bas_ran = *start_bas_ran;
    *start_bas_ran = true;
    run_task::spawn(run_task::RunSpec {
        state: state.clone(),
        bot_uuid,
        session_id,
        user_id,
        bot_name: bot_name.to_string(),
        thread_id: thread_id.to_string(),
        text,
        start_bas_ran: task_start_bas_ran,
    });
}
