use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::error;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
pub fn llm_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(["LLM", "$expr$"], false, move |context, inputs| {
            let text = context
                .eval_expression_tree(inputs.first().unwrap())?
                .to_string();
            let state_for_thread = Arc::clone(&state_clone);
            let prompt = build_llm_prompt(&text);
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();
                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_llm_generation(state_for_thread, prompt).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("failed to build tokio runtime".into())).err()
                };
                if send_err.is_some() {
                    error!("Failed to send LLM thread result");
                }
            });
            match rx.recv_timeout(Duration::from_secs(500)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
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
        })
        .unwrap();
}
fn build_llm_prompt(user_text: &str) -> String {
    user_text.trim().to_string()
}
pub async fn execute_llm_generation(
    state: Arc<AppState>,
    prompt: String,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let model = config_manager
        .get_config(&Uuid::nil(), "llm-model", None)
        .unwrap_or_default();
    let key = config_manager
        .get_config(&Uuid::nil(), "llm-key", None)
        .unwrap_or_default();

    let handler = crate::llm::llm_models::get_handler(&model);
    let raw_response = state
        .llm_provider
        .generate(&prompt, &serde_json::Value::Null, &model, &key)
        .await?;
    let processed = handler.process_content(&raw_response);
    Ok(processed)
}
