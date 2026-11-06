//! Prepares, redacts, validates and sends frames; appends them to the run log
//! when the context carries state.

use botevents::{EventSource, EventType, PendingEvent};
use serde_json::Value;

use crate::core::bot::pipeline::sink::ChannelSink;

use super::context::RunContext;
use super::redactor::{enabled, pipeline};

fn persist(envelope: &botevents::Envelope, context: &RunContext) {
    let (Some(state), Some(bot_id)) = (context.state.as_ref(), context.bot_id) else {
        return;
    };
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("run event persistence skipped: pool error: {error}");
            return;
        }
    };
    if let Err(error) = crate::core::bot::runs::store::append(
        &mut conn,
        envelope,
        bot_id,
        context.user_id,
        context.org_id,
    ) {
        log::warn!("run event persistence failed: {error}");
    }
}

/// Prepares, redacts, validates and sends one event.
///
/// Returns `false` when the feature is off, the channel cannot carry raw
/// frames, or the payload was rejected. Rejections are logged with schema
/// paths only — never with payload values.
pub async fn emit(
    sink: &dyn ChannelSink,
    context: &RunContext,
    event_type: EventType,
    data: Value,
) -> bool {
    if !enabled() || !sink.supports_raw_frames() {
        return false;
    }
    let pending = PendingEvent::new(
        context.thread_id.clone(),
        context.run_id.clone(),
        EventSource::Runtime,
        event_type,
        data,
    )
    .with_parent(context.parent_run_id.clone());
    let envelope = match pipeline().prepare(pending) {
        Ok(envelope) => envelope,
        Err(error) => {
            log::warn!("run event rejected: {error}");
            return false;
        }
    };
    persist(&envelope, context);
    let mut frame = match envelope.to_json() {
        Ok(frame) => frame,
        Err(error) => {
            log::warn!("run event serialization failed: {error}");
            return false;
        }
    };
    if let Some(map) = frame.as_object_mut() {
        map.insert("gb_event".to_string(), Value::Bool(true));
    }
    sink.send_raw_json(&frame).await.is_ok()
}

/// Sends a product-profile event (`gb.*`) as a `CUSTOM` frame.
pub async fn emit_custom(
    sink: &dyn ChannelSink,
    context: &RunContext,
    name: &str,
    payload: Value,
) -> bool {
    if !enabled() || !sink.supports_raw_frames() {
        return false;
    }
    let envelope = match pipeline().prepare_custom(
        context.thread_id.clone(),
        context.run_id.clone(),
        EventSource::Model,
        name,
        payload,
    ) {
        Ok(envelope) => envelope,
        Err(error) => {
            log::warn!("run custom event rejected: {error}");
            return false;
        }
    };
    persist(&envelope, context);
    let mut frame = match envelope.to_json() {
        Ok(frame) => frame,
        Err(error) => {
            log::warn!("run custom event serialization failed: {error}");
            return false;
        }
    };
    if let Some(map) = frame.as_object_mut() {
        map.insert("gb_event".to_string(), Value::Bool(true));
    }
    sink.send_raw_json(&frame).await.is_ok()
}

/// Returns true when the feature is on and the channel can carry frames.
#[must_use]
pub fn can_emit(sink: &dyn ChannelSink) -> bool {
    enabled() && sink.supports_raw_frames()
}
