use std::sync::Arc;

use axum::extract::ws::Message;
use botevents::{Envelope, EventSource, EventType};
use botcore::shared::state::AppState;
use futures_util::SinkExt;
use log::warn;
use serde_json::Value;
use uuid::Uuid;

use super::WsSender;
use crate::core::bot::runs::{bus, store};

/// #1677 — takeover/hand-back: the user takes over the live terminal or
/// browser surface. The segment is persisted as a CUSTOM run event (seq'd) so a
/// reload or a second device restores it, then broadcast to every subscriber of
/// this thread.
pub(super) async fn handle(
    ws_sender: &WsSender,
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    thread_id: &str,
    parsed: Option<&Value>,
) {
    let state_label = parsed
        .and_then(|value| value.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("taken_over")
        .to_string();
    let surface = parsed
        .and_then(|value| value.get("surface"))
        .and_then(Value::as_str)
        .unwrap_or("terminal")
        .to_string();
    let run_id = parsed
        .and_then(|value| value.get("run_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| bus::bus().current_run_id(thread_id))
        .unwrap_or_default();
    if run_id.is_empty() {
        let denied = serde_json::json!({
            "type": "run_takeover_ack",
            "accepted": false,
            "message": "No active run to take over",
            "session_id": session_id,
        });
        let _ = ws_sender
            .lock()
            .await
            .send(Message::Text(denied.to_string()))
            .await;
        return;
    }
    let seq = persist(
        state,
        bot_uuid,
        user_id,
        thread_id,
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
    bus::bus().publish(thread_id, frame);
    let ack = serde_json::json!({
        "type": "run_takeover_ack",
        "accepted": true,
        "state": state_label,
        "seq": seq,
    });
    let _ = ws_sender
        .lock()
        .await
        .send(Message::Text(ack.to_string()))
        .await;
}

/// Persists one takeover segment as a CUSTOM run event (#1677). Returns the
/// allocated sequence, or 0 when persistence is unavailable — the live frame
/// still reaches subscribers in that case.
fn persist(
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
    // Keep the in-process sequencer ahead of this out-of-band row; otherwise
    // the next emitted event reuses this seq and the unique constraint drops
    // it (#1701 durability).
    crate::core::bot::pipeline::event_sink::observe_persisted_seq(thread_id, seq);
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
