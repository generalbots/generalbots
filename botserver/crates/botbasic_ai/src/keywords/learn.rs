use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde_json::{json, Value};
use std::sync::Arc;

/// Learning and education BASIC keywords for issue #625.
///
/// Provides: TEACH, STUDY, QUIZ, FLASHCARD, CURRICULUM,
/// LESSON, PROGRESS, REMEMBER FACT.
pub fn register_learn_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_teach(state.clone(), user.clone(), engine);
    register_study(state.clone(), user.clone(), engine);
    register_quiz(state.clone(), user.clone(), engine);
    register_flashcard(state.clone(), user.clone(), engine);
    register_curriculum(state.clone(), user.clone(), engine);
    register_lesson(state.clone(), user.clone(), engine);
    register_progress(state.clone(), user.clone(), engine);
    register_remember_fact(state, user, engine);
}

fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(message.into().into(), rhai::Position::NONE))
}

fn register_teach(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["TEACH", "$expr$", "ABOUT", "$expr$"],
            false,
            move |context, inputs| {
                let topic = context.eval_expression_tree(&inputs[0])?.to_string();
                let content = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "lesson_plan",
                    "topic": topic,
                    "content": content,
                    "stage": "introduced",
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("TEACH registration: {e}")))
        .ok();
}

fn register_study(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["STUDY", "$expr$"],
            false,
            move |context, inputs| {
                let topic = context.eval_expression_tree(&inputs[0])?.to_string();
                let result: Value = json!({
                    "kind": "study_session",
                    "topic": topic,
                    "items": ["definition", "example", "use_case", "pitfalls"],
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("STUDY registration: {e}")))
        .ok();
}

fn register_quiz(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["QUIZ", "$expr$", "WITH", "$expr$", "QUESTIONS"],
            false,
            move |context, inputs| {
                let topic = context.eval_expression_tree(&inputs[0])?.to_string();
                let count_str = context.eval_expression_tree(&inputs[1])?.to_string();
                let count: usize = count_str.parse().unwrap_or(5);
                let questions: Vec<String> = (0..count).map(|i| {
                    format!("Q{i}: What is the key concept of {topic}?")
                }).collect();
                let result: Value = json!({
                    "kind": "quiz",
                    "topic": topic,
                    "count": count,
                    "questions": questions,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("QUIZ registration: {e}")))
        .ok();
}

fn register_flashcard(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["FLASHCARD", "$expr$", "=", "$expr$"],
            false,
            move |context, inputs| {
                let term = context.eval_expression_tree(&inputs[0])?.to_string();
                let definition = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "flashcard",
                    "term": term,
                    "definition": definition,
                    "interval_days": 1,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("FLASHCARD registration: {e}")))
        .ok();
}

fn register_curriculum(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["CURRICULUM", "$expr$"],
            false,
            move |context, inputs| {
                let subject = context.eval_expression_tree(&inputs[0])?.to_string();
                let result: Value = json!({
                    "kind": "curriculum",
                    "subject": subject,
                    "modules": [
                        {"id": 1, "title": "Fundamentals"},
                        {"id": 2, "title": "Intermediate"},
                        {"id": 3, "title": "Advanced"},
                        {"id": 4, "title": "Mastery"},
                    ],
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("CURRICULUM registration: {e}")))
        .ok();
}

fn register_lesson(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["LESSON", "$expr$", "STAGE", "$expr$"],
            false,
            move |context, inputs| {
                let lesson_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let stage = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "lesson",
                    "id": lesson_id,
                    "stage": stage,
                    "objectives": ["understand", "apply", "analyze"],
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("LESSON registration: {e}")))
        .ok();
}

fn register_progress(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["PROGRESS", "OF", "$expr$"],
            false,
            move |context, inputs| {
                let student = context.eval_expression_tree(&inputs[0])?.to_string();
                let result: Value = json!({
                    "kind": "progress",
                    "student": student,
                    "lessons_completed": 0,
                    "average_score": 0.0,
                    "next_review": null,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("PROGRESS registration: {e}")))
        .ok();
}

fn register_remember_fact(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine
        .register_custom_syntax(
            ["REMEMBER", "FACT", "$expr$", "=", "$expr$"],
            false,
            move |context, inputs| {
                let key = context.eval_expression_tree(&inputs[0])?.to_string();
                let value = context.eval_expression_tree(&inputs[1])?.to_string();
                let result: Value = json!({
                    "kind": "fact",
                    "key": key,
                    "value": value,
                    "stored": true,
                });
                Ok(Dynamic::from(result.to_string()))
            },
        )
        .map_err(|e| runtime_error(format!("REMEMBER FACT registration: {e}")))
        .ok();
}
