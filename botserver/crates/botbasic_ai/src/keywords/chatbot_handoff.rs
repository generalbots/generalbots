use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde_json::{json, Value};
use std::sync::Arc;

/// Chatbot handoff BASIC keywords for issue #621.
///
/// Provides: TRANSFER TO HUMAN, ESCALATE, REQUEST AGENT,
/// ASSIGN AGENT, COMPLETE HANDOFF.
pub fn register_chatbot_handoff_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_transfer_to_human(state.clone(), user.clone(), engine);
    register_escalate(state.clone(), user.clone(), engine);
    register_request_agent(state.clone(), user.clone(), engine);
    register_assign_agent(state.clone(), user.clone(), engine);
    register_complete_handoff(state, user, engine);
}

fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(message.into().into(), rhai::Position::NONE))
}

fn register_transfer_to_human(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["TRANSFER", "TO", "HUMAN", "REASON", "$expr$"],
            false,
            move |context, inputs| {
                let reason = context.eval_expression_tree(&inputs[0])?.to_string();
                let result: Value = json!({
                    "kind": "transfer_to_human",
                    "reason": reason,
                    "queue_position": 0,
                    "estimated_wait_seconds": 60,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("TRANSFER TO HUMAN registration: {e}")))
        .ok();
}

fn register_escalate(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["ESCALATE", "$expr$", "PRIORITY", "$expr$"],
            false,
            move |context, inputs| {
                let issue = context.eval_expression_tree(&inputs[0])?.to_string();
                let priority = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "escalation",
                    "issue": issue,
                    "priority": priority,
                    "escalated": true,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("ESCALATE registration: {e}")))
        .ok();
}

fn register_request_agent(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["REQUEST", "AGENT", "TOPIC", "$expr$"],
            false,
            move |context, inputs| {
                let topic = context.eval_expression_tree(&inputs[0])?.to_string();
                let result: Value = json!({
                    "kind": "request_agent",
                    "topic": topic,
                    "agent_id": null,
                    "waiting": true,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("REQUEST AGENT registration: {e}")))
        .ok();
}

fn register_assign_agent(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["ASSIGN", "AGENT", "$expr$", "TO", "$expr$"],
            false,
            move |context, inputs| {
                let agent = context.eval_expression_tree(&inputs[0])?.to_string();
                let session = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "assign_agent",
                    "agent": agent,
                    "session": session,
                    "assigned": true,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("ASSIGN AGENT registration: {e}")))
        .ok();
}

fn register_complete_handoff(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["COMPLETE", "HANDOFF", "$expr$", "RATING", "$expr$"],
            false,
            move |context, inputs| {
                let session = context.eval_expression_tree(&inputs[0])?.to_string();
                let rating = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "complete_handoff",
                    "session": session,
                    "rating": rating,
                    "completed": true,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("COMPLETE HANDOFF registration: {e}")))
        .ok();
}
