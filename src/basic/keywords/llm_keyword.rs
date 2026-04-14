use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::error;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
pub fn llm_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(["LLM", "$expr$"], false, move |context, inputs| {
            let first_input = inputs.first().ok_or_else(|| {
                Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "LLM requires at least one input".into(),
                    rhai::Position::NONE,
                ))
            })?;
            let text = context
                .eval_expression_tree(first_input)?
                .to_string();
            let state_for_async = Arc::clone(&state_clone);
            let prompt = build_llm_prompt(&text);

            let handle = tokio::runtime::Handle::current();
            let result = handle.block_on(async move {
                tokio::time::timeout(
                    Duration::from_secs(45),
                    execute_llm_generation(state_for_async, prompt)
                ).await
            });

            match result {
                Ok(Ok(output)) => Ok(Dynamic::from(output)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "LLM generation timed out after 45 seconds".into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}
fn build_llm_prompt(user_text: &str) -> String {
    format!(
        "Você é um assistente virtual em português brasileiro. Responda sempre em português do Brasil, de forma clara e amigável.\n\nPedido do usuário: {}",
        user_text.trim()
    )
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
