//! Tool call execution and follow-up loop for the LLM stream pipeline.
//!
//! Extracted from `stream.rs` to keep files under 450 lines.

use super::super::types::{PipelineResult, SessionIdentity};

/// Executes tool calls from the LLM response and optionally returns a follow-up
/// messages array for a recursive `stream_llm_response_inner` call.
///
/// Returns `Some(messages)` when a follow-up is needed (the caller should
/// recurse), `None` when the tool execution is done (the caller should break).
pub(super) async fn execute_and_prepare_followup(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    full_response: &str,
    user_text: &str,
    messages: &serde_json::Value,
    tool_depth: usize,
    max_tool_followups: usize,
) -> PipelineResult<Option<serde_json::Value>> {
    let SessionIdentity {
        sink,
        session_id, ..
    } = *identity;
    let session_id_s = session_id.to_string();

    let call_chunks: Vec<&str> = full_response
        .split('\n')
        .filter(|c| c.contains("\"__tool_call__\":"))
        .collect();
    let tool_output = if call_chunks.len() > 1 {
        log::info!("Executing {} parallel tool calls", call_chunks.len());
        let mut parts = Vec::new();
        for call_chunk in call_chunks {
            parts.push(
                super::super::tool_exec::run_llm_tool_call(identity, rx, call_chunk, user_text)
                    .await,
            );
        }
        parts.join("\n\n")
    } else {
        super::super::tool_exec::run_llm_tool_call(identity, rx, full_response, user_text).await
    };

    // A stopped or replaced turn makes no follow-up model call: it would only
    // answer "Stopped." between the person's new question and its answer.
    if tool_depth < max_tool_followups
        && !tool_output.trim().is_empty()
        && !crate::core::bot::runs::bus::bus().is_cancelled(&session_id_s)
    {
        let follow_output: String = tool_output.chars().take(12_000).collect();
        let mut follow_messages = messages.clone();

        if let Some(steer) = crate::core::bot::runs::bus::bus().take_steering(&session_id_s) {
            botlog::safe_warn!(
                "steering applied before follow-up (session {}, {} chars)",
                session_id_s,
                steer.chars().count()
            );
            let notice = serde_json::json!({
                "type": "gb.steer",
                "session_id": session_id_s,
                "message": steer,
            });
            let _ = sink.send_raw_json(&notice).await;
            if let Some(items) = follow_messages.as_array_mut() {
                items.push(serde_json::json!({
                    "role": "user",
                    "content": crate::core::bot::runs::steering::instruction(&steer),
                }));
            }
        }
        if let Some(items) = follow_messages.as_array_mut() {
            items.push(serde_json::json!({
                "role": "assistant",
                "content": full_response,
            }));
            items.push(serde_json::json!({
                "role": "user",
                "content": format!(
                    "The tool finished with this result:\n\n{follow_output}\n\nAnswer the user's original request now using this result. Do not call another tool unless it is strictly necessary."
                ),
            }));
        }
        log::info!(
            "tool follow-up: depth {}/{} after {} bytes of tool output",
            tool_depth + 1,
            max_tool_followups,
            tool_output.len()
        );
        return Ok(Some(follow_messages));
    }
    Ok(None)
}
