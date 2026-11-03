//! Typed model tools for structured interaction (#1670).
//!
//! These are native tools: every bot gets them without shipping a `.bas`
//! script. The model authors data against a validated schema; the runtime
//! emits a `CUSTOM gb.*` frame and persists a plain-text fallback so the
//! conversation stays useful when the client cannot render the card.

use serde_json::{json, Value};

use super::event_sink::RunContext;
use super::native_tools::{persist, send};
use super::types::SessionIdentity;

/// Component kinds the Tier 1 renderer understands. Unknown kinds are rejected
/// at the tool edge instead of silently degrading a whole surface.
const SURFACE_COMPONENTS: [&str; 12] = [
    "text", "callout", "table", "metrics", "bars", "lines", "flow", "timeline",
    "form", "plan", "diff", "tabs",
];

/// Field values already answered in a thread. In-memory is enough for the
/// active conversation; the durable record is the persisted question turn.
fn answers() -> &'static std::sync::Mutex<std::collections::HashMap<String, std::collections::HashMap<String, String>>> {
    static ANSWERS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    > = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Records the answers from a structured reply so the same field is never
/// asked twice in one thread.
pub fn record_answers(thread_id: &str, values: &serde_json::Map<String, Value>) {
    let mut guard = match answers().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let entry = guard.entry(thread_id.to_string()).or_default();
    for (key, value) in values {
        let text = value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string());
        entry.insert(key.clone(), text);
    }
}

#[must_use]
pub fn answered_in_thread(thread_id: &str) -> std::collections::HashMap<String, String> {
    let guard = match answers().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.get(thread_id).cloned().unwrap_or_default()
}

/// Returns true when `name` is one of the structured-interaction tools.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    match name {
        "ask_user" => {
            run_ask_user(identity, args).await;
            true
        }
        "propose_plan" => {
            run_propose_plan(identity, args).await;
            true
        }
        "show_surface" => {
            run_show_surface(identity, args).await;
            true
        }
        _ => false,
    }
}

fn context(identity: &SessionIdentity<'_>) -> RunContext {
    let SessionIdentity {
        state,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    RunContext::new(&session_id.to_string(), "chat")
        .with_scope(bot_uuid, user_id, None)
        .with_state(state.clone())
}

async fn emit_custom(identity: &SessionIdentity<'_>, name: &str, payload: Value) {
    let context = context(identity);
    let sent = super::event_sink::emit_custom(identity.sink, &context, name, payload).await;
    if !sent {
        botlog::safe_warn!("structured tool {name} could not emit its frame");
    }
}

async fn invalid(identity: &SessionIdentity<'_>, tool: &str, issues: &[String]) {
    let message = format!(
        "The `{tool}` call is invalid: {}. Correct the arguments and call the tool again.",
        issues.join("; ")
    );
    send(identity, &message).await;
}

/// Prompt block advertising the native structured tools. Callers gate it on
/// the web channel and the run-event feature.
#[must_use]
pub fn instructions() -> &'static str {
    "STRUCTURED INTERACTION (native tools)\n\
     Use these instead of prose when the interaction is a decision, a plan or a result that is clearer as data.\n\
     - ask_user: {\"title\": \"…\", \"fields\": [{\"id\": \"env\", \"label\": \"Environment\", \"type\": \"select\", \"options\": [\"staging\", \"production\"], \"required\": true}], \"blocking\": true}. Ask only for information you cannot infer. Never repeat a field the user already answered in this thread.\n\
     - propose_plan: {\"title\": \"…\", \"steps\": [{\"title\": \"…\", \"description\": \"…\"}]}. Use for work with three or more stages; the user can reorder or remove steps before approving.\n\
     - show_surface: {\"title\": \"…\", \"components\": [{\"id\": \"m1\", \"type\": \"metrics\", \"items\": [...]}], \"fallback\": \"plain text version\"}. Catalog: text, callout, table, metrics, bars, lines, flow, timeline, form, plan, diff, tabs. Always include a fallback. Never send HTML.\n\
     Rules: never claim a card was shown unless the tool returned; prefer one card per turn; if prose is enough, answer in prose.\n"
}

async fn run_ask_user(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::native_args::parse_args(args) {
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

async fn run_propose_plan(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::native_args::parse_args(args) {
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
    crate::core::bot::runs::bus::bus()
        .thread(&identity.session_id.to_string())
        .set_state(botevents::RunState::WaitingApproval);
}

async fn run_show_surface(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            let message = format!("Could not read the surface: {error}");
            send(identity, &message).await;
            return;
        }
    };
    let surface = match parsed.get("surface") {
        Some(Value::Object(_)) => parsed.get("surface").cloned().unwrap_or(Value::Null),
        _ => parsed.clone(),
    };
    let mut issues = Vec::new();
    let components = match surface.get("components") {
        Some(Value::Array(items)) if !items.is_empty() => items.clone(),
        Some(Value::Array(_)) => {
            issues.push("data.components must contain at least one component".to_string());
            Vec::new()
        }
        Some(_) => {
            issues.push("data.components must be an array".to_string());
            Vec::new()
        }
        None => {
            issues.push("data.components is required".to_string());
            Vec::new()
        }
    };
    for (index, component) in components.iter().enumerate() {
        let Some(object) = component.as_object() else {
            issues.push(format!("components[{index}] must be an object"));
            continue;
        };
        let kind = object.get("type").and_then(Value::as_str).unwrap_or("");
        if kind.is_empty() {
            issues.push(format!("components[{index}].type is required"));
        } else if !SURFACE_COMPONENTS.contains(&kind) {
            issues.push(format!(
                "components[{index}].type '{kind}' is not in the catalog"
            ));
        }
        if object.get("id").and_then(Value::as_str).unwrap_or("").is_empty() {
            issues.push(format!("components[{index}].id is required"));
        }
    }
    if !issues.is_empty() {
        invalid(identity, "show_surface", &issues).await;
        return;
    }
    let fallback = surface
        .get("fallback")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            parsed
                .get("fallback")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "A result is available in the client.".to_string());
    let title = surface
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Result");
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
        "gb.surface",
        json!({
            "surface": {
                "version": "gb.basic.v1",
                "surface_id": surface.get("surface_id").cloned().unwrap_or(Value::Null),
                "title": title,
                "components": components,
                "data": surface.get("data").cloned().unwrap_or(Value::Null),
                "fallback": fallback,
            },
            "fallback": fallback,
        }),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::SURFACE_COMPONENTS;

    #[test]
    fn catalog_matches_the_renderer() {
        assert!(SURFACE_COMPONENTS.contains(&"form"));
        assert!(SURFACE_COMPONENTS.contains(&"plan"));
        assert_eq!(SURFACE_COMPONENTS.len(), 12);
    }
}
