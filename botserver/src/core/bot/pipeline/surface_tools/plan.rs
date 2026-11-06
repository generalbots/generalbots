//! `propose_plan`: shows a step plan for approval, optionally asking for the
//! decisions it needs in the same turn.

use serde_json::{json, Value};

use super::super::native_tools::{persist, send};
use super::super::types::SessionIdentity;
use super::{answered_in_thread, emit_custom, invalid};

pub(super) async fn run_propose_plan(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            let message = format!("Could not read the plan: {error}");
            send(identity, &message).await;
            return;
        }
    };
    let mut issues = Vec::new();
    let steps = match parsed.get("steps") {
        Some(Value::Array(items)) if !items.is_empty() => items.clone(),
        Some(Value::Array(_)) => {
            issues.push("data.steps must contain at least one step".to_string());
            Vec::new()
        }
        Some(_) => {
            issues.push("data.steps must be an array".to_string());
            Vec::new()
        }
        None => {
            issues.push("data.steps is required".to_string());
            Vec::new()
        }
    };
    if steps.len() > 50 {
        issues.push("data.steps must not exceed 50 steps".to_string());
    }
    for (index, step) in steps.iter().enumerate() {
        let title = step
            .get("title")
            .or_else(|| step.get("label"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if title.trim().is_empty() {
            issues.push(format!("steps[{index}].title is required"));
        }
    }
    // Optional decisions: a plan that needs input should ask for it in the
    // same turn instead of ending with a paragraph the reader must answer by
    // hand. Fields use the same schema as ask_user.
    let mut question_fields = match parsed.get("questions") {
        Some(Value::Array(items)) => items.clone(),
        Some(_) => {
            issues.push("questions must be an array".to_string());
            Vec::new()
        }
        None => Vec::new(),
    };
    for (index, field) in question_fields.iter().enumerate() {
        let Some(object) = field.as_object() else {
            issues.push(format!("questions[{index}] must be an object"));
            continue;
        };
        if object.get("id").and_then(Value::as_str).map(str::trim).unwrap_or("").is_empty() {
            issues.push(format!("questions[{index}].id is required"));
        }
        let kind = object.get("type").and_then(Value::as_str).unwrap_or("text");
        if !matches!(kind, "text" | "textarea" | "select" | "number" | "date" | "email") {
            issues.push(format!("questions[{index}].type '{kind}' is not supported"));
        }
    }
    if !question_fields.is_empty() {
        // Never ask again for a field this thread already answered.
        let answered = answered_in_thread(&identity.session_id.to_string());
        if !answered.is_empty() {
            question_fields.retain(|field| {
                let id = field.get("id").and_then(Value::as_str).unwrap_or("");
                !answered.contains_key(id)
            });
        }
    }
    if !issues.is_empty() {
        invalid(identity, "propose_plan", &issues).await;
        return;
    }
    let title = parsed
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Proposed plan");
    let approve_label = parsed
        .get("approve_label")
        .and_then(Value::as_str)
        .unwrap_or("Approve plan");
    let fallback = format!(
        "{title}: {}",
        steps
            .iter()
            .map(|step| {
                step.get("title")
                    .or_else(|| step.get("label"))
                    .and_then(Value::as_str)
                    .unwrap_or("Step")
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(" → ")
    );
    {
        let SessionIdentity {
            state,
            session_id,
            user_id,
            ..
        } = *identity;
        persist(state, session_id, user_id, &fallback).await;
    }
    emit_custom(
        identity,
        "gb.plan",
        json!({
            "title": title,
            "steps": steps,
            "approve_label": approve_label,
            "fallback": fallback,
        }),
    )
    .await;
    if question_fields.is_empty() {
        crate::core::bot::runs::bus::bus()
            .thread(&identity.session_id.to_string())
            .set_state(botevents::RunState::WaitingApproval);
        return;
    }
    let question_title = parsed
        .get("question_title")
        .and_then(Value::as_str)
        .unwrap_or("Before we start");
    let labels: Vec<String> = question_fields
        .iter()
        .filter_map(|field| field.get("label").and_then(Value::as_str).map(str::to_string))
        .collect();
    let question_fallback = format!("{question_title}: {}", labels.join("; "));
    {
        let SessionIdentity {
            state,
            session_id,
            user_id,
            ..
        } = *identity;
        let text = format!("{question_title}\n{question_fallback}");
        persist(state, session_id, user_id, &text).await;
    }
    emit_custom(
        identity,
        "gb.question",
        json!({
            "question_id": uuid::Uuid::new_v4().to_string(),
            "title": question_title,
            "fields": question_fields,
            "blocking": true,
            "fallback": question_fallback,
        }),
    )
    .await;
    crate::core::bot::runs::bus::bus()
        .thread(&identity.session_id.to_string())
        .set_state(botevents::RunState::WaitingInput);
}
