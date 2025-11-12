use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use serde_json::Value;
use tokio::sync::mpsc;
pub mod local;
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn summarize(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!("Summarize the following conversation while preserving key details:\n\n{}", text);
        self.generate(&prompt, &serde_json::json!({"max_tokens": 500}))
            .await
    }
    async fn cancel_job(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}
#[async_trait]
impl LLMProvider for OpenAIClient {
    async fn generate(
        &self,
        prompt: &str,
        messages: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": if messages.is_array() && !messages.as_array().unwrap().is_empty() {
                    messages
                } else {
                    &default_messages
                }
            }))
            .send()
            .await?;
        let result: Value = response.json().await?;
        let raw_content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");
        let end_token = "final<|message|>";
        let content = if let Some(pos) = raw_content.find(end_token) {
            raw_content[(pos + end_token.len())..].to_string()
        } else {
            raw_content.to_string()
        };
        Ok(content)
    }
    async fn generate_stream(
        &self,
        prompt: &str,
        messages: &Value,
        tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": if messages.is_array() && !messages.as_array().unwrap().is_empty() {
                    info!("Using provided messages: {:?}", messages);
                    messages
                } else {
                    &default_messages
                },
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
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    if let Ok(data) = serde_json::from_str::<Value>(&line[6..]) {
                        if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                            buffer.push_str(content);
                            let _ = tx.send(content.to_string()).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap()
        }
    }

    pub fn build_messages(system_prompt: &str, context_data: &str, history: &[(String, String)]) -> Value {
        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt
            }));
        }
        if !context_data.is_empty() {
            messages.push(serde_json::json!({
                "role": "system", 
                "content": context_data
            }));
        }
        for (role, content) in history {
            messages.push(serde_json::json!({
                "role": role,
                "content": content
            }));
        }
        serde_json::Value::Array(messages)
    }
}
