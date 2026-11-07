//! Run lifecycle events: started, finished, failed.

use botevents::{EventType, RunState};

use crate::core::bot::pipeline::sink::ChannelSink;
use crate::core::bot::runs::observability;

use super::context::RunContext;
use super::emit::emit;

pub async fn run_started(sink: &dyn ChannelSink, context: &RunContext, title: Option<&str>) {
    crate::core::bot::runs::bus::bus()
        .thread(&context.thread_id)
        .set_state(RunState::Running);
    if let Some(state) = context.state.as_ref() {
        observability::begin_run_row(&state.conn, &context.run_id, context.bot_id, &context.agent);
    }
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
    if let Some(app) = context.state.as_ref() {
        observability::finish_run_row(&app.conn, &context.run_id, &state.to_string());
    }
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
    if let Some(app) = context.state.as_ref() {
        observability::finish_run_row(&app.conn, &context.run_id, "failed");
    }
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
