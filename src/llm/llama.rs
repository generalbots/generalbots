use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use crate::tools::ToolManager;
use super::LLMProvider;

pub struct LlamaClient {
    client: Client,
    base_url: String,
}

impl LlamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl LLMProvider for LlamaClient {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&format!("{}/completion", self.base_url))
            .json(&serde_json::json!({
                "prompt": prompt,
                "n_predict": config.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(512),
                "temperature": config.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7),
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        Ok(result["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&format!("{}/completion", self.base_url))
            .json(&serde_json::json!({
                "prompt": prompt,
                "n_predict": config.get("max_tokens").and_then(|v| v.as_i64()).unwrap_or(512),
                "temperature": config.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7),
                "stream": true
            }))
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                if let Ok(data) = serde_json::from_str::<Value>(&line) {
                    if let Some(content) = data["content"].as_str() {
                        buffer.push_str(content);
                        let _ = tx.send(content.to_string()).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn generate_with_tools(
        &self,
        prompt: &str,
        config: &Value,
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
        self.generate(&enhanced_prompt, config).await
    }

    async fn cancel_job(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // llama.cpp cancellation endpoint
        let response = self.client
            .post(&format!("{}/cancel", self.base_url))
            .json(&serde_json::json!({
                "session_id": session_id
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to cancel job for session {}", session_id).into())
        }
    }
}
