//! Planner stage: prompts the LLM for a strict JSON step graph and falls
//! back to a single-step plan when the model output cannot be parsed.

use super::PlanStep;

pub(crate) const PLANNER_SYSTEM: &str = "You are an automation planner. Given a goal, produce a \
JSON object {\"steps\":[...]} where each step has fields: id (short slug), name (human label), \
kind (one of: tool|llm|fork|merge|deliver), depends_on (array of step ids), tool (tool name for \
kind=tool), input (JSON object). Order steps so dependencies come first. Use kind=fork with \
input.children=[{name,input,tool}] for parallel sub-work, kind=merge to combine outputs of its \
depends_on steps, kind=deliver only when the user explicitly asked to send something now. \
Return JSON only, no prose.";

const MAX_PLAN_STEPS: usize = 32;

/// Parses the planner completion into a step list; malformed shapes are
/// rejected so the caller can fall back.
fn parse_plan(raw: &str) -> Option<Vec<PlanStep>> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    let steps = value.get("steps")?.as_array()?.clone();
    if steps.is_empty() || steps.len() > MAX_PLAN_STEPS {
        return None;
    }
    let mut parsed = Vec::with_capacity(steps.len());
    for step in steps {
        let step: PlanStep = serde_json::from_value(step).ok()?;
        if step.id.is_empty() || step.name.is_empty() || !is_known_kind(&step.kind) {
            return None;
        }
        parsed.push(step);
    }
    Some(parsed)
}

fn is_known_kind(kind: &str) -> bool {
    matches!(kind, "tool" | "llm" | "fork" | "merge" | "deliver")
}

/// Single deterministic fallback executed when the planner fails or the LLM
/// is not wired: one `llm` step carrying the raw goal.
fn fallback_plan(goal: &str) -> Vec<PlanStep> {
    vec![PlanStep {
        id: "step-1".to_string(),
        name: "Execute goal".to_string(),
        kind: "llm".to_string(),
        depends_on: Vec::new(),
        tool: None,
        input: serde_json::json!({ "goal": goal }),
    }]
}

/// Builds the execution plan for a goal. Any planner failure degrades to the
/// single-step fallback so a run never blocks on model formatting issues.
pub(crate) fn build_plan(llm: &crate::state::LlmFn, goal: &str) -> Vec<PlanStep> {
    let user_prompt = format!("Goal:\n{goal}\n\nProduce the steps JSON now.");
    match llm(PLANNER_SYSTEM, &user_prompt, "{}") {
        Ok(raw) => parse_plan(&raw).unwrap_or_else(|| {
            tracing::error!("automation planner: unparseable output, using single-step fallback");
            fallback_plan(goal)
        }),
        Err(e) => {
            tracing::error!("automation planner: llm unavailable ({e}), using single-step fallback");
            fallback_plan(goal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn strict_plan_parses_happy_path() {
        let raw = r#"{"steps":[
            {"id":"a","name":"Collect","kind":"llm","depends_on":[],"input":{"q":"x"}},
            {"id":"b","name":"Report","kind":"merge","depends_on":["a"],"input":{}}
        ]}"#;
        let steps = parse_plan(raw).expect("valid plan");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[1].depends_on, vec!["a".to_string()]);
    }

    #[test]
    fn unknown_kind_or_empty_steps_are_rejected() {
        assert!(parse_plan(r#"{"steps":[]}"#).is_none());
        assert!(
            parse_plan(r#"{"steps":[{"id":"a","name":"n","kind":"explode","depends_on":[]}]}"#)
                .is_none()
        );
        assert!(parse_plan("not json at all").is_none());
    }

    #[test]
    fn fallback_plan_is_single_llm_step_with_goal_input() {
        let steps = fallback_plan("summarize inbox");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].kind, "llm");
        assert_eq!(steps[0].input["goal"], serde_json::json!("summarize inbox"));
    }

    #[test]
    fn build_plan_falls_back_when_llm_errors() {
        let llm: crate::state::LlmFn =
            Arc::new(|_s: &str, _u: &str, _p: &str| Err("no provider".to_string()));
        let steps = build_plan(&llm, "goal");
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn build_plan_uses_valid_llm_output() {
        let llm: crate::state::LlmFn = Arc::new(|_s: &str, _u: &str, _p: &str| {
            Ok(r#"{"steps":[{"id":"only","name":"Do","kind":"llm","depends_on":[]}]}"#.to_string())
        });
        let steps = build_plan(&llm, "goal");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].id, "only");
    }
}
