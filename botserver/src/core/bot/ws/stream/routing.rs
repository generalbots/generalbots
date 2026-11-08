//! #1946 — per-request routing: agentic or non-agentic.
//!
//! The chat used to treat every request the same way: the model was free to
//! reach for the VM/tool loop even when the user just asked a question, and
//! app-backed requests could wander into execution paths. This module makes
//! the decision explicit for the current request and gives it a reason, so the
//! choice is visible in the logs instead of being implicit.
//!
//! The decision is advisory: it never widens a surface. Command, path, budget
//! and permission guards still apply to whatever the model asks to run.

use serde_json::Value;

/// How one request should be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Multi-step work that needs tools, files or the agent VM.
    Agentic,
    /// Direct answer, catalog/UI action or a single deterministic call.
    NonAgentic,
}

impl Route {
    /// Stable label for logs and telemetry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agentic => "agentic",
            Self::NonAgentic => "non-agentic",
        }
    }
}

/// One routing decision, with the rule that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decision {
    pub route: Route,
    pub reason: &'static str,
}

impl Decision {
    const fn non_agentic(reason: &'static str) -> Self {
        Self {
            route: Route::NonAgentic,
            reason,
        }
    }

    const fn agentic(reason: &'static str) -> Self {
        Self {
            route: Route::Agentic,
            reason,
        }
    }
}

/// Work that only makes sense with an execution surface.
const EXECUTION_MARKERS: &[&str] = &[
    "build a",
    "compile",
    "create a pdf",
    "create a pptx",
    "create an xlsx",
    "convert",
    "crawl",
    "generate a",
    "install ",
    "multi-step",
    "node script",
    "pipeline",
    "python",
    "refactor",
    "render ",
    "run the tests",
    "run this script",
    "scrape",
    "shell command",
    "write a script",
    "write the code",
];

/// A request that opens like a question is answered, not executed.
const QUESTION_OPENERS: &[&str] = &[
    "are ", "can ", "do ", "does ", "how ", "is ", "should ", "what ", "when ", "where ",
    "which ", "who ", "why ",
];

/// True when the client can receive actions at all. An unknown set of
/// capabilities keeps today's behaviour.
fn client_can_act(client_caps: Option<&Value>) -> bool {
    let Some(caps) = client_caps else {
        return true;
    };
    let Some(surfaces) = caps.get("surfaces").and_then(Value::as_array) else {
        return true;
    };
    !surfaces.is_empty()
        && surfaces.iter().any(|surface| {
            surface.as_str().is_some_and(|value| {
                value.eq_ignore_ascii_case("gb.basic.v1")
                    || value.eq_ignore_ascii_case("actions")
                    || value.eq_ignore_ascii_case("tools")
            })
        })
}

/// True when the request names a suite app ("open drive", "in the database
/// app"), which the catalog/`__ui_plan__` path answers without the VM.
fn mentions_app(text: &str) -> bool {
    let tokens: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|token| !token.is_empty())
        .collect();
    crate::apps::registry::all_apps()
        .iter()
        .any(|app| tokens.iter().any(|token| *token == app.id))
}

/// Decides how the current request should be handled.
///
/// `attachments` is the number of files the turn carries; an attachment is
/// ingested by the pipeline itself, so it does not need the agent loop.
#[must_use]
pub fn decide(user_text: &str, attachments: usize, client_caps: Option<&Value>) -> Decision {
    let text = user_text.trim().to_lowercase();
    if text.is_empty() {
        return Decision::non_agentic("empty-request");
    }
    if !client_can_act(client_caps) {
        return Decision::non_agentic("client-has-no-action-surface");
    }
    if attachments > 0 {
        return Decision::non_agentic("attachment-ingest");
    }
    if mentions_app(&text) {
        return Decision::non_agentic("app-backed-ui");
    }
    if EXECUTION_MARKERS
        .iter()
        .any(|marker| text.contains(marker))
    {
        return Decision::agentic("artifact-or-execution");
    }
    if QUESTION_OPENERS
        .iter()
        .any(|opener| text.trim_start().starts_with(opener))
    {
        return Decision::non_agentic("question");
    }
    Decision::non_agentic("conversational-default")
}

#[cfg(test)]
mod tests {
    use super::{decide, Route};
    use serde_json::json;

    #[test]
    fn simple_question_stays_non_agentic() {
        let decision = decide("What is the capital of Brazil?", 0, None);
        assert_eq!(decision.route, Route::NonAgentic);
        assert_eq!(decision.reason, "question");
    }

    #[test]
    fn artifact_request_is_agentic() {
        let decision = decide("Create a PDF report with last month totals", 0, None);
        assert_eq!(decision.route, Route::Agentic);
        assert_eq!(decision.reason, "artifact-or-execution");
    }

    #[test]
    fn app_backed_request_never_reaches_the_vm() {
        let decision = decide("open the database app and create a table", 0, None);
        assert_eq!(decision.route, Route::NonAgentic);
        assert_eq!(decision.reason, "app-backed-ui");
    }

    #[test]
    fn attachment_turn_is_handled_by_the_pipeline() {
        let decision = decide("summarise this", 1, None);
        assert_eq!(decision.reason, "attachment-ingest");
    }

    #[test]
    fn a_text_only_client_cannot_be_routed_to_actions() {
        let caps = json!({ "surfaces": ["text"] });
        let decision = decide("build a script that renames files", 0, Some(&caps));
        assert_eq!(decision.route, Route::NonAgentic);
        assert_eq!(decision.reason, "client-has-no-action-surface");
    }

    #[test]
    fn empty_text_is_not_executed() {
        assert_eq!(decide("   ", 0, None).reason, "empty-request");
    }
}
