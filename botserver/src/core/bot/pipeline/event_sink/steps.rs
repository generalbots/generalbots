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
