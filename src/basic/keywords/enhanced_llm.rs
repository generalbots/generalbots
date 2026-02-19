use crate::basic::UserSession;
use crate::core::shared::state::AppState;
#[cfg(feature = "llm")]
use crate::llm::smart_router::{OptimizationGoal, SmartLLMRouter};
#[cfg(not(feature = "llm"))]
use rhai::Engine;
#[cfg(feature = "llm")]
use rhai::{Dynamic, Engine};
use std::sync::Arc;

#[cfg(feature = "llm")]
pub fn register_enhanced_llm_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone1 = Arc::clone(&state);
    let state_clone2 = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["LLM", "$string$", "WITH", "OPTIMIZE", "FOR", "$string$"],
        false,
        move |context, inputs| {
            let prompt = context.eval_expression_tree(&inputs[0])?.to_string();
            let optimization = context.eval_expression_tree(&inputs[1])?.to_string();

            let state_for_spawn = Arc::clone(&state_clone1);
            let _user_clone_spawn = user_clone.clone();

            tokio::spawn(async move {
                let router = SmartLLMRouter::new(state_for_spawn);
                let goal = OptimizationGoal::from_str_name(&optimization);

                match crate::llm::smart_router::enhanced_llm_call(
                    &router, &prompt, goal, None, None,
                )
                .await
                {
                    Ok(_response) => {
                        log::info!("LLM response generated with {} optimization", optimization);
                    }
                    Err(e) => {
                        log::error!("Enhanced LLM call failed: {}", e);
                    }
                }
            });

            Ok(Dynamic::from("LLM response"))
        },
    ) {
        log::warn!("Failed to register enhanced LLM syntax: {e}");
    }

    if let Err(e) = engine.register_custom_syntax(
        [
            "LLM",
            "$string$",
            "WITH",
            "MAX_COST",
            "$float$",
            "MAX_LATENCY",
            "$int$",
        ],
        false,
        move |context, inputs| {
            let prompt = context.eval_expression_tree(&inputs[0])?.to_string();
            let max_cost = context.eval_expression_tree(&inputs[1])?.as_float()?;
            let max_latency = context.eval_expression_tree(&inputs[2])?.as_int()? as u64;

            let state_for_spawn = Arc::clone(&state_clone2);

            tokio::spawn(async move {
                let router = SmartLLMRouter::new(state_for_spawn);

                match crate::llm::smart_router::enhanced_llm_call(
                    &router,
                    &prompt,
                    OptimizationGoal::Balanced,
                    Some(max_cost),
                    Some(max_latency),
                )
                .await
                {
                    Ok(_response) => {
                        log::info!(
                            "LLM response with constraints: cost<={}, latency<={}",
                            max_cost,
                            max_latency
                        );
                    }
                    Err(e) => {
                        log::error!("Constrained LLM call failed: {}", e);
                    }
                }
            });

            Ok(Dynamic::from("LLM response"))
        },
    ) {
        log::warn!("Failed to register constrained LLM syntax: {e}");
    }
}

#[cfg(not(feature = "llm"))]
pub fn register_enhanced_llm_keyword(
    _state: Arc<AppState>,
    _user: UserSession,
    _engine: &mut Engine,
) {
    // No-op when LLM feature is disabled
}
