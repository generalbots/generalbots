//! Subscription-backed LLM providers (#1652/#1653).
//!
//! Both providers implement `botlib::traits::LLMProvider`, so the chat
//! pipeline can use them exactly like an HTTP provider. The credentials stay
//! where the account lives: Claude inside the user's VM, ChatGPT in Vault.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;

use botagent::AgentService;

/// Runs prompts through the `claude` CLI inside the user's agent VM (#1652).
pub struct ClaudeCodeProvider {
    service: Arc<AgentService>,
    vm_name: String,
    model: String,
}

impl std::fmt::Debug for ClaudeCodeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeCodeProvider")
            .field("vm_name", &self.vm_name)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl ClaudeCodeProvider {
    pub fn new(service: Arc<AgentService>, vm_name: String, model: String) -> Self {
        Self { service, vm_name, model }
    }
}

fn boxed(message: String) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::other(message))
}

async fn send_in_chunks(tx: &mpsc::Sender<String>, text: &str) {
    // The CLI is non-streaming here; chunking keeps the chat renderer alive
    // instead of freezing one large frame.
    let mut buffer = String::new();
    for character in text.chars() {
        buffer.push(character);
        if buffer.len() >= 120 {
            if tx.send(std::mem::take(&mut buffer)).await.is_err() {
                return;
            }
        }
    }
    if !buffer.is_empty() {
        let _ = tx.send(buffer).await;
    }
}

#[async_trait]
impl botlib::traits::LLMProvider for ClaudeCodeProvider {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        _key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = flatten_prompt(prompt, config);
        let selected = if model.trim().is_empty() { self.model.as_str() } else { model };
        botagent::claude::run_prompt(&self.service, &self.vm_name, &prompt, selected)
            .await
            .map_err(|(_, error)| boxed(error))
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = self.generate(prompt, config, model, key).await?;
        send_in_chunks(&tx, &text).await;
        Ok(())
    }
}

/// Uses a ChatGPT plan token against the Responses backend (#1653).
pub struct ChatGptPlanProvider {
    client: reqwest::Client,
    base_url: String,
    token: String,
    account_id: Option<String>,
    model: String,
}

impl std::fmt::Debug for ChatGptPlanProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatGptPlanProvider")
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl ChatGptPlanProvider {
    pub fn new(
        base_url: String,
        token: String,
        account_id: Option<String>,
        model: String,
    ) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|error| format!("could not build the ChatGPT client: {error}"))?;
        Ok(Self { client, base_url, token, account_id, model })
    }
}

#[async_trait]
impl botlib::traits::LLMProvider for ChatGptPlanProvider {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        _key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let selected = if model.trim().is_empty() { self.model.as_str() } else { model };
        botllm::openai_responses::generate(
            &self.client,
            &self.base_url,
            &self.token,
            self.account_id.as_deref(),
            selected,
            prompt,
            config,
        )
        .await
        .map_err(boxed)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = self.generate(prompt, config, model, key).await?;
        send_in_chunks(&tx, &text).await;
        Ok(())
    }
}

/// Constructs the provider selected by `ai_accounts::choose_for_request`.
///
/// Claude needs the user's VM (provisioned on demand); ChatGPT reads its token
/// from Vault. Neither falls back to a bot/global API key.
pub async fn build(
    state: &Arc<botcore::shared::state::AppState>,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    resolved: &crate::core::ai_accounts::ResolvedLlm,
) -> Result<Arc<dyn botlib::traits::LLMProvider>, String> {
    match resolved.provider.as_str() {
        crate::core::ai_accounts::CLAUDE_CODE => {
            let service = crate::core::bot::agent_vm_hook::service()
                .ok_or_else(|| "The agent workspace is not available in this build.".to_string())?;
            let row = crate::core::bot::agent_vm_hook::ensure_agent_vm(
                session_id,
                user_id,
                bot_uuid,
            )
            .await
            .ok_or_else(|| {
                "Sign in to use your Claude Code subscription in the agent workspace.".to_string()
            })?;
            Ok(Arc::new(ClaudeCodeProvider::new(
                service.clone(),
                row.vm_name,
                resolved.model.clone(),
            )))
        }
        crate::core::ai_accounts::CHATGPT_PLAN => {
            let manager = botcoresecrets::SecretsManager::get()
                .map_err(|error| format!("Secrets are unavailable: {error}"))?;
            let path = chatgpt_vault_path(user_id);
            let secret = manager
                .get_secret(&path)
                .await
                .map_err(|error| format!("Could not read the ChatGPT plan token: {error}"))?;
            let token = secret
                .get("access_token")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    "Connect your ChatGPT plan in Settings \u{2192} AI accounts first.".to_string()
                })?;
            let base_url = secret
                .get("base_url")
                .filter(|value| !value.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| botllm::openai_responses::DEFAULT_BASE_URL.to_string());
            let account_id = secret.get("account_id").cloned();
            ChatGptPlanProvider::new(
                base_url,
                token.clone(),
                account_id,
                resolved.model.clone(),
            )
            .map(|provider| Arc::new(provider) as Arc<dyn botlib::traits::LLMProvider>)
        }
        other => {
            let _ = state;
            Err(format!("Unsupported subscription provider: {other}"))
        }
    }
}

/// Vault path holding the ChatGPT plan token for one user.
pub fn chatgpt_vault_path(user_id: uuid::Uuid) -> String {
    format!("gbo/users/{user_id}/chatgpt-plan")
}

/// Flattens the chat transcript into the single prompt the CLI accepts.
pub fn flatten_prompt(prompt: &str, config: &Value) -> String {
    let Some(messages) = config.as_array() else {
        return prompt.to_string();
    };
    let mut parts: Vec<String> = Vec::new();
    for message in messages {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("user");
        let content = message.get("content").and_then(Value::as_str).unwrap_or("");
        if content.trim().is_empty() {
            continue;
        }
        let label = match role {
            "system" => "System",
            "assistant" => "Assistant",
            _ => "User",
        };
        parts.push(format!("{label}: {content}"));
    }
    if parts.is_empty() {
        prompt.to_string()
    } else {
        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::flatten_prompt;

    #[test]
    fn transcript_is_flattened_with_roles() {
        let config = serde_json::json!([
            { "role": "system", "content": "be brief" },
            { "role": "user", "content": "hello" }
        ]);
        let prompt = flatten_prompt("fallback", &config);
        assert!(prompt.contains("System: be brief"));
        assert!(prompt.contains("User: hello"));
    }

    #[test]
    fn empty_transcript_uses_the_prompt() {
        assert_eq!(flatten_prompt("fallback", &serde_json::Value::Null), "fallback");
    }
}
