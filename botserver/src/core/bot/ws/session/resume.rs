use std::sync::Arc;

use axum::extract::ws::Message;
use futures_util::SinkExt;
use log::warn;
use uuid::Uuid;

use super::WsSender;
use crate::core::bot::runs::store;

/// #1666 — replays only the frames the client has not seen after a reconnect,
/// then acknowledges the resume.
pub(super) async fn replay(
    state: &Arc<botcore::shared::state::AppState>,
    ws_sender: &WsSender,
    bot_uuid: Uuid,
    session_id: Uuid,
    thread_id: &str,
    after_seq: u64,
) {
    match state.conn.get() {
        Ok(mut conn) => match store::load_thread(&mut conn, bot_uuid, thread_id, after_seq) {
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
        },
        Err(error) => warn!("run resume skipped: pool error: {error}"),
    }
    let done = serde_json::json!({
        "type": "run_resume_done",
        "session_id": session_id,
        "last_seq": after_seq,
    });
    let _ = ws_sender
        .lock()
        .await
        .send(Message::Text(done.to_string()))
        .await;
}
