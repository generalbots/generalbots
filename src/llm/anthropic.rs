use async_trait::async_trait;
use reqwest::Client;
use futures_util::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::tools::ToolManager;
use super::LLMProvider;

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicClient {
    async fn generate(
        &self,
        prompt: &str,
        _config: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 1000,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        let content = result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        _config: &Value,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 1000,
                "messages": [{"role": "user", "content": prompt}],
                "stream": true
            }))
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk_bytes = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk_bytes);

            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    if let Ok(data) = serde_json::from_str::<Value>(&line[6..]) {
                        if data["type"] == "content_block_delta" {
                            if let Some(text) = data["delta"]["text"].as_str() {
                                buffer.push_str(text);
                                let _ = tx.send(text.to_string()).await;
                            }
                        }
                    }
                }
            }
        }

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
            format!("\n\nAvailable tools: {}. You can suggest using these tools if they would help answer the user's question.", available_tools.join(", "))
        };

        let enhanced_prompt = format!("{}{}", prompt, tools_info);
        self.generate(&enhanced_prompt, &Value::Null).await
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Anthropic doesn't support job cancellation
        Ok(())
    }
}
