//! Run event emission for chat turns (#1663).
//!
//! This is the botserver adapter around `botevents`: it owns the process-wide
//! pipeline (one sequencer per thread), applies the shared security redactor
//! and writes validated frames through the existing `ChannelSink`.
//!
//! The transport stays `ChannelSink`/`raw_json`; the event contract, ordering,
//! validation and redaction live in `botevents`. Features that build runs
//! (#1666, #1667, #1668) call the helpers below instead of hand-building
//! frames.

use std::sync::{Arc, OnceLock};

use botevents::{
    DefaultRedactor, EventPipeline, EventSource, EventType, FrameRedactor, PendingEvent, RunState,
};
use serde_json::Value;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::sink::ChannelSink;
use super::super::runs::observability;

/// Security-backed redactor: known credential values from the environment,
/// then the security crate's masking rules, applied to every string in the
/// payload.
struct SecurityRedactor {
    known: DefaultRedactor,
}

impl SecurityRedactor {
    fn new() -> Self {
        Self {
            known: DefaultRedactor::from_env(),
        }
    }
}

impl FrameRedactor for SecurityRedactor {
    fn redact(&self, value: &mut Value) {
        self.known.redact(value);
        scrub_security(value);
    }
}

fn scrub_security(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = botsecurity_crypto::secrets::redact_sensitive_data(text);
        }
        Value::Array(items) => {
            for item in items {
                scrub_security(item);
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                scrub_security(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn pipeline() -> &'static Arc<EventPipeline> {
    static PIPELINE: OnceLock<Arc<EventPipeline>> = OnceLock::new();
    PIPELINE.get_or_init(|| {
        Arc::new(EventPipeline::new(Arc::new(SecurityRedactor::new())))
    })
}

/// Runtime flag. Compiling the contract is unconditional; emitting is opt-in
/// so an existing deployment is unchanged until `GB_RUN_EVENTS` is set.
#[must_use]
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        matches!(
            std::env::var("GB_RUN_EVENTS").as_deref(),
            Ok("1") | Ok("true") | Ok("yes")
        )
    })
}

/// Identity of the turn whose events are being emitted.
#[derive(Debug, Clone)]
pub struct RunContext {
    pub thread_id: String,
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub agent: String,
    pub bot_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    /// When present, every accepted frame is appended to `run_events` before
    /// it is sent. Chat turns set it; tests and transports that only stream
    /// leave it `None`.
    pub state: Option<Arc<AppState>>,
}

impl RunContext {
    #[must_use]
    pub fn new(thread_id: &str, agent: &str) -> Self {
        let run_id = crate::core::bot::runs::bus::bus()
            .current_run_id(thread_id)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        Self {
            thread_id: thread_id.to_string(),
            run_id,
            parent_run_id: None,
            agent: agent.to_string(),
            bot_id: None,
            user_id: None,
            org_id: None,
            state: None,
        }
    }

    /// Rebuilds a context from identifiers already persisted (resume path).
    #[must_use]
    pub fn resumed(thread_id: &str, run_id: &str, agent: &str) -> Self {
        Self {
            thread_id: thread_id.to_string(),
            run_id: run_id.to_string(),
            parent_run_id: None,
            agent: agent.to_string(),
            bot_id: None,
            user_id: None,
            org_id: None,
            state: None,
        }
    }

    #[must_use]
    pub fn with_scope(mut self, bot_id: Uuid, user_id: Uuid, org_id: Option<Uuid>) -> Self {
        self.bot_id = Some(bot_id);
        self.user_id = Some(user_id);
        self.org_id = org_id;
        self
    }

    #[must_use]
    pub fn with_state(mut self, state: Arc<AppState>) -> Self {
        self.state = Some(state);
        self
    }

    #[must_use]
    pub fn with_parent(mut self, parent_run_id: Option<String>) -> Self {
        self.parent_run_id = parent_run_id;
        self
    }
}

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
    if let Err(error) = super::super::runs::store::append(
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

pub async fn run_started(sink: &dyn ChannelSink, context: &RunContext, title: Option<&str>) {
    crate::core::bot::runs::bus::bus().thread(&context.thread_id).set_state(RunState::Running);
    observability::start_run(&context.thread_id, &context.run_id, Some(&context.agent));
    let data = serde_json::json!({
        "agent": context.agent,
        "title": title,
    });
    let _ = emit(sink, context, EventType::RunStarted, data).await;
}

pub async fn run_finished(
    sink: &dyn ChannelSink,
    context: &RunContext,
    state: RunState,
    duration_ms: u64,
    exit_code: Option<i32>,
) {
    crate::core::bot::runs::bus::bus().finish(&context.thread_id, state);
    observability::finish_run(
        &context.run_id,
        &state.to_string(),
        context.state.as_ref().map(|app| &app.conn),
    );
    let data = serde_json::json!({
        "state": state,
        "duration_ms": duration_ms,
        "exit_code": exit_code,
    });
    let _ = emit(sink, context, EventType::RunFinished, data).await;
}

pub async fn run_error(
    sink: &dyn ChannelSink,
    context: &RunContext,
    message: &str,
    code: Option<&str>,
) {
    crate::core::bot::runs::bus::bus().finish(&context.thread_id, RunState::Failed);
    observability::finish_run(
        &context.run_id,
        "failed",
        context.state.as_ref().map(|app| &app.conn),
    );
    let data = serde_json::json!({
        "message": message,
        "code": code,
    });
    let _ = emit(sink, context, EventType::RunError, data).await;
}

pub async fn step_started(
    sink: &dyn ChannelSink,
    context: &RunContext,
    step_id: &str,
    name: &str,
    index: Option<u32>,
) {
    let data = serde_json::json!({
        "step_id": step_id,
        "name": name,
        "index": index,
    });
    observability::start_span(
        &context.run_id,
        "step",
        step_id,
        vec![("gb.step.name".to_string(), name.to_string())],
    );
    let _ = emit(sink, context, EventType::StepStarted, data).await;
}

pub async fn step_finished(
    sink: &dyn ChannelSink,
    context: &RunContext,
    step_id: &str,
    status: &str,
    duration_ms: Option<u64>,
    exit_code: Option<i32>,
) {
    let data = serde_json::json!({
        "step_id": step_id,
        "status": status,
        "duration_ms": duration_ms,
        "exit_code": exit_code,
    });
    observability::finish_span(
        &context.run_id,
        "step",
        step_id,
        vec![(
            "gb.step.status".to_string(),
            status.to_string(),
        )],
    );
    let _ = emit(sink, context, EventType::StepFinished, data).await;
}

pub async fn tool_started(
    sink: &dyn ChannelSink,
    context: &RunContext,
    call_id: &str,
    name: &str,
    risk: Option<&str>,
) {
    let data = serde_json::json!({
        "call_id": call_id,
        "name": name,
        "risk": risk,
    });
    observability::start_span(
        &context.run_id,
        "tool",
        call_id,
        vec![("gen_ai.tool.name".to_string(), name.to_string())],
    );
    let _ = emit(sink, context, EventType::ToolCallStart, data).await;
}

pub async fn tool_result(
    sink: &dyn ChannelSink,
    context: &RunContext,
    call_id: &str,
    status: &str,
    result: Value,
    duration_ms: Option<u64>,
) {
    let data = serde_json::json!({
        "call_id": call_id,
        "status": status,
        "result": result,
        "duration_ms": duration_ms,
    });
    observability::finish_span(
        &context.run_id,
        "tool",
        call_id,
        vec![("gb.tool.status".to_string(), status.to_string())],
    );
    let _ = emit(sink, context, EventType::ToolCallResult, data).await;
}

pub async fn text_delta(
    sink: &dyn ChannelSink,
    context: &RunContext,
    message_id: &str,
    delta: &str,
) {
    let data = serde_json::json!({
        "message_id": message_id,
        "delta": delta,
    });
    let _ = emit(sink, context, EventType::TextMessageContent, data).await;
}

pub async fn text_end(
    sink: &dyn ChannelSink,
    context: &RunContext,
    message_id: &str,
    content: Option<&str>,
) {
    let data = serde_json::json!({
        "message_id": message_id,
        "content": content,
    });
    let _ = emit(sink, context, EventType::TextMessageEnd, data).await;
}

/// Returns true when the feature is on and the channel can carry frames.
#[must_use]
pub fn can_emit(sink: &dyn ChannelSink) -> bool {
    enabled() && sink.supports_raw_frames()
}
