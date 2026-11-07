//! Step, tool and text events.

use botevents::EventType;
use serde_json::Value;

use crate::core::bot::pipeline::sink::ChannelSink;
use crate::core::bot::runs::observability;

use super::context::RunContext;
use super::emit::emit;

pub async fn step_started(
    sink: &dyn ChannelSink,
    context: &RunContext,
    step_id: &str,
    name: &str,
    index: Option<u32>,
) {
    step_started_with(sink, context, step_id, name, index, Value::Null).await;
}

/// Same as [`step_started`], with extra contract fields merged into `data`.
///
/// Workspace stages (#1807) add `phase`, `step`, `detail`, `started_at` and
/// `progress` without changing the required AG-UI shape every client already
/// understands.
pub async fn step_started_with(
    sink: &dyn ChannelSink,
    context: &RunContext,
    step_id: &str,
    name: &str,
    index: Option<u32>,
    extra: Value,
) {
    let mut data = serde_json::json!({
        "step_id": step_id,
        "name": name,
        "index": index,
    });
    merge_extra(&mut data, &extra);
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
    step_finished_with(
        sink,
        context,
        step_id,
        status,
        duration_ms,
        exit_code,
        Value::Null,
    )
    .await;
}

/// Same as [`step_finished`], with extra contract fields merged into `data`.
pub async fn step_finished_with(
    sink: &dyn ChannelSink,
    context: &RunContext,
    step_id: &str,
    status: &str,
    duration_ms: Option<u64>,
    exit_code: Option<i32>,
    extra: Value,
) {
    let mut data = serde_json::json!({
        "step_id": step_id,
        "status": status,
        "duration_ms": duration_ms,
        "exit_code": exit_code,
    });
    merge_extra(&mut data, &extra);
    observability::finish_span(
        &context.run_id,
        "step",
        step_id,
        vec![("gb.step.status".to_string(), status.to_string())],
    );
    let _ = emit(sink, context, EventType::StepFinished, data).await;
}

fn merge_extra(data: &mut Value, extra: &Value) {
    let (Some(target), Some(source)) = (data.as_object_mut(), extra.as_object()) else {
        return;
    };
    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
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
