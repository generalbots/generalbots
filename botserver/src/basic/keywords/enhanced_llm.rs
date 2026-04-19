use crate::basic::UserSession;
use crate::core::shared::state::AppState;
#[cfg(feature = "llm")]
use crate::llm::smart_router::{OptimizationGoal, SmartLLMRouter};
#[cfg(not(feature = "llm"))]
use rhai::Engine;
#[cfg(feature = "llm")]
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "llm")]
pub fn register_enhanced_llm_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    if let Err(e) = engine.register_custom_syntax(
        ["LLM", "$string$"],
        false,
        move |context, inputs| {
            let prompt = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_thread = Arc::clone(&state_clone);
            let (tx, rx) = std::sync::mpsc::channel();
            
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();
                    
                if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        let router = SmartLLMRouter::new(Arc::clone(&state_for_thread));
                        crate::llm::smart_router::enhanced_llm_call(
                            &state_for_thread, &router, &prompt, OptimizationGoal::Balanced, None, None,
                        )
                        .await
                    });
                    let _ = tx.send(result);
                }
            });
            
            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(Ok(response)) => Ok(Dynamic::from(response)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "LLM generation timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("LLM thread failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    ) {
        log::warn!("Failed to register simple LLM syntax: {e}");
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
