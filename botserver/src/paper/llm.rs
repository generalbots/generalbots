use std::sync::Arc;

use crate::core::shared::state::AppState;

#[cfg(feature = "llm")]
use crate::llm::OpenAIClient;

pub async fn call_llm(
    state: &Arc<AppState>,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    #[cfg(feature = "llm")]
    {
        let llm = &state.llm_provider;

        let messages = OpenAIClient::build_messages(
            system_prompt,
            "",
            &[("user".to_string(), user_content.to_string())],
        );

        let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
        let model = config_manager
            .get_config(&uuid::Uuid::nil(), "llm-model", None)
            .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
        let key = config_manager
            .get_config(&uuid::Uuid::nil(), "llm-key", None)
            .unwrap_or_else(|_| String::new());

        llm.generate(user_content, &messages, &model, &key)
            .await
            .map_err(|e| format!("LLM error: {}", e))
    }

    #[cfg(not(feature = "llm"))]
    {
        let _ = (state, system_prompt);
        Ok(format!(
            "[LLM not available] Processing: {}...",
            &user_content[..50.min(user_content.len())]
        ))
    }
}
