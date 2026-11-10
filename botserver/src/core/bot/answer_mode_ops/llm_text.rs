//! LLM call and response parsing shared by the answer modes.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

pub(super) async fn call_llm_for_text(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    prompt: &str,
) -> Result<String, String> {
    use botcore::config::ConfigManager;

    let cfg = ConfigManager::new(state.conn.clone());
    let llm_url = cfg
        .get_config(&bot_uuid, "llm-url", Some(""))
        .unwrap_or_default();
    let llm_key = cfg
        .get_config(&bot_uuid, "llm-key", Some(""))
        .unwrap_or_default();
    let llm_model = cfg
        .get_config(&bot_uuid, "llm-model", Some(""))
        .unwrap_or_default();

    let provider: Arc<dyn botlib::traits::LLMProvider> = if !llm_url.is_empty() {
        let inner = crate::llm::provider_factory::create_llm_provider_from_url(
            &llm_url,
            if llm_model.is_empty() {
                None
            } else {
                Some(llm_model.clone())
            },
            None,
            None,
        );
        Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
            inner, llm_model, llm_key,
        )) as Arc<dyn botlib::traits::LLMProvider>
    } else if let Some(ref global) = state.llm_provider {
        global.clone()
    } else {
        return Err("No LLM provider available".to_string());
    };

    provider.generate_simple(prompt).await
}

pub(super) fn extract_sql_from_response(response: &str) -> String {
    let trimmed = response.trim();
    if trimmed.starts_with("```") {
        let without_fence = trimmed
            .trim_start_matches("```sql")
            .trim_start_matches("```SQL")
            .trim_start_matches("```")
            .trim();
        if let Some(end) = without_fence.rfind("```") {
            without_fence[..end].trim().to_string()
        } else {
            without_fence.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

pub(super) fn extract_json_from_response(response: &str) -> Result<serde_json::Value, String> {
    let trimmed = response.trim();
    let json_str = if trimmed.starts_with("```") {
        let without_fence = trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_start_matches("```")
            .trim();
        if let Some(end) = without_fence.rfind("```") {
            without_fence[..end].trim().to_string()
        } else {
            without_fence.to_string()
        }
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {e}"))
}
