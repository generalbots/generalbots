//! `ask_user`: asks the user for structured input and blocks the turn until an
//! answer arrives.

use serde_json::{json, Value};

use super::super::native_tools::{persist, send};
use super::super::types::SessionIdentity;
use super::{answered_in_thread, emit_custom, invalid};

pub(super) async fn run_ask_user(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            let message = format!("Could not read the question: {error}");
            send(identity, &message).await;
            return;
        }
    };
    let mut issues = Vec::new();
    let mut fields = match parsed.get("fields") {
        Some(Value::Array(items)) => items.clone(),
        Some(_) => {
            issues.push("fields must be an array".to_string());
            Vec::new()
        }
        None => Vec::new(),
    };
    if fields.is_empty() {
        if let Some(question) = parsed.get("question").and_then(Value::as_str) {
            fields.push(json!({ "id": "answer", "label": question, "type": "text", "required": true }));
        } else {
            issues.push("data.fields: at least one field (or a question) is required".to_string());
        }
    }
    for (index, field) in fields.iter().enumerate() {
        let Some(object) = field.as_object() else {
            issues.push(format!("fields[{index}] must be an object"));
            continue;
        };
        if object.get("id").and_then(Value::as_str).map(str::trim).unwrap_or("").is_empty() {
            issues.push(format!("fields[{index}].id is required"));
        }
        let kind = object.get("type").and_then(Value::as_str).unwrap_or("text");
        if !matches!(kind, "text" | "textarea" | "select" | "number" | "date" | "email") {
            issues.push(format!("fields[{index}].type '{kind}' is not supported"));
        }
    }
    if !issues.is_empty() {
        invalid(identity, "ask_user", &issues).await;
        return;
    }

    // #1670 — never ask again for a field this thread already answered.
    let answered = answered_in_thread(&identity.session_id.to_string());
    if !answered.is_empty() {
        fields.retain(|field| {
            let id = field.get("id").and_then(Value::as_str).unwrap_or("");
            !answered.contains_key(id)
        });
        if fields.is_empty() {
            let known: Vec<String> = answered
                .iter()
                .map(|(key, value)| format!("{key}: {value}"))
                .collect();
            let message = format!("Already answered in this thread: {}", known.join("; "));
            send(identity, &message).await;
            return;
        }
    }

    let question_id = parsed
        .get("question_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let title = parsed
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("A question");
    let blocking = parsed
        .get("blocking")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let fallback = parsed
        .get("fallback")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            let labels: Vec<String> = fields
                .iter()
                .filter_map(|field| field.get("label").and_then(Value::as_str).map(str::to_string))
                .collect();
            format!("{title}: {}", labels.join("; "))
        });

    let payload = json!({
        "question_id": question_id,
        "title": title,
        "fields": fields,
        "blocking": blocking,
        "fallback": fallback,
    });

    // Persist the question as an assistant turn before showing it: the next
    // model call must see what was asked, including after a reconnect.
    {
        let SessionIdentity {
            state,
            session_id,
            user_id,
            ..
        } = *identity;
        let text = format!("{title}\n{fallback}");
        persist(state, session_id, user_id, &text).await;
    }
    emit_custom(identity, "gb.question", payload).await;
    if blocking {
        // The turn ends here; the next user message starts a new run. The bus
        // records the waiting state so a second message is accepted (not
        // rejected as a concurrent run).
        crate::core::bot::runs::bus::bus()
            .thread(&identity.session_id.to_string())
            .set_state(botevents::RunState::WaitingInput);
    } else {
        send(
            identity,
            "I will continue with a stated assumption and adjust if your answer differs.",
        )
        .await;
    }
}
