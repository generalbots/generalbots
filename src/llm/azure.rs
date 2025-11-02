use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use crate::tools::ToolManager;
use super::LLMProvider;

pub struct AzureOpenAIClient {
    endpoint: String,
    api_key: String,
    api_version: String,
    deployment: String,
    client: Client,
}

impl AzureOpenAIClient {
    pub fn new(endpoint: String, api_key: String, api_version: String, deployment: String) -> Self {
        Self {
            endpoint,
            api_key,
            api_version,
            deployment,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for AzureOpenAIClient {
    async fn generate(
        &self,
        prompt: &str,
        _config: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.deployment, self.api_version
        );

        let body = serde_json::json!({
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7,
            "max_tokens": 1000,
            "top_p": 1.0,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.0
        });

        let response = self.client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let result: Value = response.json().await?;
        if let Some(choice) = result["choices"].get(0) {
            Ok(choice["message"]["content"].as_str().unwrap_or("").to_string())
        } else {
            Err("No response from Azure OpenAI".into())
        }
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        _config: &Value,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = self.generate(prompt, _config).await?;
        let _ = tx.send(content).await;
        Ok(())
    }

    async fn generate_with_tools(
        &self,
        prompt: &str,
        _config: &Value,
        available_tools: &[String],
        _tool_manager: Arc<ToolManager>,
        _session_id: &str,
        _user_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let tools_info = if available_tools.is_empty() {
            String::new()
        } else {
            format!("\n\nAvailable tools: {}.", available_tools.join(", "))
        };
        let enhanced_prompt = format!("{}{}", prompt, tools_info);
        self.generate(&enhanced_prompt, _config).await
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
