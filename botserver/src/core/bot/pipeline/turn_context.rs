//! Per-turn prompt context for chat (#1545/#1549/#1580 follow-up).
//!
//! Model preference, session context, Vibe project routing, referenced project
//! and the Drive listing are pushed into the message list here, in the same
//! order the pipeline used before the split.

use std::sync::Arc;

use serde_json::Value;

use botcore::shared::state::AppState;

use super::sink::ChannelSink;
use super::types::PipelineResult;

/// Applies the selected model tier as a shaping sentence. The tier already
/// picks a real model in `resolve_bot_llm_provider`; this hint only shapes the
/// answer (`auto` and unknown values add nothing).
pub(super) fn apply_model_preference(messages: &mut Vec<Value>, model_preference: &str) {
    if model_preference == "auto" || model_preference.is_empty() {
        return;
    }
    let directive = match model_preference {
        "fast" => "The user selected Fast mode: answer as concisely as possible and prioritise latency over exhaustive detail.",
        "quality" => "The user selected Quality mode: take extra care, verify assumptions, and provide thorough reasoning and complete answers.",
        "balanced" => "The user selected Balanced mode: balance completeness with brevity.",
        _ => "",
    };
    if !directive.is_empty() {
        messages.push(serde_json::json!({"role": "system", "content": directive}));
    }
}

pub(super) fn apply_session_context(messages: &mut Vec<Value>, session_context: &str) {
    if !session_context.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!("Contexto da conversa:\n{session_context}")
        }));
    }
}

/// Everything the project router needs from the current turn.
pub(super) struct TurnRouting<'a> {
    pub state: &'a Arc<AppState>,
    pub sink: &'a dyn ChannelSink,
    pub bot_uuid: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

/// Routes a turn that names a Vibe project. Returns `true` when the router
/// already replied and the caller must stop.
pub(super) async fn route_project(
    routing: &TurnRouting<'_>,
    project_context: Option<&Value>,
    user_text: &str,
    messages: &mut Vec<Value>,
) -> PipelineResult<bool> {
    if super::vibe_route::route_vibe_project(
        routing.state,
        routing.sink,
        project_context,
        user_text,
        routing.bot_uuid,
        routing.session_id,
        routing.user_id,
    )
    .await?
    {
        return Ok(true);
    }
    if let Some(context) = project_context {
        let project_id = context
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let project_name = context
            .get("project_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !project_id.is_empty() || !project_name.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!(
                    "The user referenced Vibe project '{project_name}' (id: {project_id}).\n\
                     For code changes to this project, use the vibe.project.change command with \
                     project_id='{project_id}', project_name='{project_name}', and the user's requested intent."
                ),
            }));
        }
    }
    Ok(false)
}

/// #1670 — structured answers travel with the user turn. Records them so a
/// field is never asked twice and marks the values authoritative for this turn.
pub(super) fn apply_structured_answers(
    parsed: &Value,
    session_id: uuid::Uuid,
    messages: &mut Vec<Value>,
) {
    if let Some(values) = parsed
        .get("question_answers")
        .and_then(|value| value.as_object())
    {
        super::surface_tools::record_answers(&session_id.to_string(), values);
        let rendered = serde_json::to_string(values).unwrap_or_default();
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!(
                "The user answered the structured question with {rendered}. These values are authoritative; do not ask for the same fields again."
            )
        }));
    }
    if let Some(plan) = parsed.get("plan_approval") {
        let rendered = serde_json::to_string(plan).unwrap_or_default();
        messages.push(serde_json::json!({
            "role": "system",
            "content": format!(
                "The user approved the plan with {rendered}. Execute it; do not propose it again."
            )
        }));
    }
}

/// #1644 — results of approved browser/terminal actions are fed back into the
/// conversation before the model answers, so the reply can cite the page title
/// (or say the action was denied/failed).
pub(super) async fn apply_completed_actions(
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    messages: &mut Vec<Value>,
) {
    let completed =
        crate::core::computer_use::take_completed_for_prompt(&session_id.to_string(), user_id)
            .await;
    if completed.is_empty() {
        return;
    }
    let mut block =
        String::from("Results of computer actions the user approved in this conversation:\n");
    for action in &completed {
        let name = action
            .action
            .get("action")
            .and_then(|value| value.as_str())
            .unwrap_or(&action.kind);
        let result = serde_json::to_string(&action.result).unwrap_or_default();
        block.push_str(&format!("- {name}: {result}\n"));
    }
    block.push_str(
        "Answer from these results; quote the page title/URL when the user asked about a page. If a result says denied or error, say so plainly.",
    );
    messages.push(serde_json::json!({"role": "system", "content": block}));
}

/// #1545 — the client's recent Drive names become prompt context when
/// `index_all_drive` is enabled (the default). The Settings toggle writes
/// `gb-index-all-drive`; `chat-send.js` forwards it as this boolean.
pub(super) fn apply_drive_listing(messages: &mut Vec<Value>, parsed: &Value) {
    let index_all_drive = parsed
        .get("index_all_drive")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !index_all_drive {
        return;
    }
    let Some(files) = parsed
        .get("drive_context")
        .and_then(|v| v.get("files"))
        .and_then(|v| v.as_array())
    else {
        return;
    };

    let names: Vec<String> = files
        .iter()
        .filter_map(|v| v.as_str())
        .take(50)
        .map(str::to_string)
        .collect();
    if names.is_empty() {
        return;
    }
    messages.push(serde_json::json!({
        "role": "system",
        "content": format!(
            "The user's Drive contains these recent files. Use them as \
             context when relevant and cite the file name: {}",
            names.join(", ")
        )
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn drive_payload(index_all_drive: Option<bool>) -> Value {
        let mut payload = json!({
            "drive_context": {
                "source": "user_drive",
                "untrusted": true,
                "files": ["budget.xlsx", "contract.pdf"]
            }
        });
        if let Some(enabled) = index_all_drive {
            payload["index_all_drive"] = json!(enabled);
        }
        payload
    }

    #[test]
    fn drive_listing_is_included_when_key_is_absent() {
        let mut messages = Vec::new();
        apply_drive_listing(&mut messages, &drive_payload(None));
        assert_eq!(messages.len(), 1);
        let content = messages[0]["content"].as_str().unwrap_or_default();
        assert!(content.contains("budget.xlsx"));
        assert!(content.contains("contract.pdf"));
    }

    #[test]
    fn drive_listing_is_included_when_toggle_is_on() {
        let mut messages = Vec::new();
        apply_drive_listing(&mut messages, &drive_payload(Some(true)));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn drive_listing_is_skipped_when_toggle_is_off() {
        let mut messages = Vec::new();
        apply_drive_listing(&mut messages, &drive_payload(Some(false)));
        assert!(messages.is_empty());
    }

    #[test]
    fn drive_listing_ignores_missing_or_empty_context() {
        let mut messages = Vec::new();
        apply_drive_listing(&mut messages, &json!({ "index_all_drive": true }));
        apply_drive_listing(
            &mut messages,
            &json!({ "index_all_drive": true, "drive_context": { "files": [] } }),
        );
        apply_drive_listing(
            &mut messages,
            &json!({ "index_all_drive": true, "drive_context": { "files": "not-an-array" } }),
        );
        assert!(messages.is_empty());
    }
}
