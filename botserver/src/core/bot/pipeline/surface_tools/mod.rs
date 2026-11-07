//! Typed model tools for structured interaction (#1670).
//!
//! These are native tools: every bot gets them without shipping a `.bas`
//! script. The model authors data against a validated schema; the runtime
//! emits a `CUSTOM gb.*` frame and persists a plain-text fallback so the
//! conversation stays useful when the client cannot render the card.
//!
//! Split by tool: `ask_user`, `plan`, `surface` and `app`; this module holds
//! the dispatch table, the shared answer memory and the validation helpers.

use serde_json::Value;

use super::event_sink::RunContext;
use super::types::SessionIdentity;

mod app;
mod ask_user;
mod plan;
mod surface;

/// Component kinds the Tier 1 renderer understands. Unknown kinds are rejected
/// at the tool edge instead of silently degrading a whole surface.
const SURFACE_COMPONENTS: [&str; 12] = [
    "text", "callout", "table", "metrics", "bars", "lines", "flow", "timeline", "form", "plan",
    "diff", "tabs",
];

/// Field values already answered in a thread. In-memory is enough for the
/// active conversation; the durable record is the persisted question turn.
fn answers() -> &'static std::sync::Mutex<
    std::collections::HashMap<String, std::collections::HashMap<String, String>>,
> {
    static ANSWERS: std::sync::OnceLock<
        std::sync::Mutex<
            std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        >,
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
            ask_user::run_ask_user(identity, args).await;
            true
        }
        "propose_plan" => {
            plan::run_propose_plan(identity, args).await;
            true
        }
        "show_surface" => {
            surface::run_show_surface(identity, args).await;
            true
        }
        "show_app" => {
            app::run_show_app(identity, args).await;
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
    super::native_tools::send(identity, &message).await;
}

/// Prompt block advertising the native structured tools, generated from the
/// declarative seed (`surface_catalog`, #1678). Callers gate it on
/// the web channel and the run-event feature.
#[must_use]
pub fn instructions() -> &'static str {
    // The seed's prompt block is deterministic text; leak-once borrows keep
    // the existing `&'static str` callers untouched.
    static BLOCK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BLOCK.get_or_init(super::surface_catalog::prompt_block)
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

    #[test]
    fn surface_catalog_matches_dispatch_table() {
        // Every declarative entry must be runnable at the tool edge and
        // every runnable tool must be advertised (#1678).
        for entry in super::super::surface_catalog::manifests()
            .as_array()
            .expect("manifests array")
        {
            let name = entry.get("name").and_then(serde_json::Value::as_str);
            if let Some(name) = name {
                assert!(
                    matches!(
                        name,
                        "ask_user" | "propose_plan" | "show_surface" | "show_app"
                    ),
                    "catalog entry {name} is not dispatched by surface_tools::try_run"
                );
            }
        }
        let block = super::instructions();
        for name in ["ask_user", "propose_plan", "show_surface", "show_app"] {
            assert!(
                block.contains(&format!("- {name}: ")),
                "the prompt block must advertise {name}"
            );
        }
    }
}
