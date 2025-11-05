use async_trait::async_trait;
use futures::StreamExt;
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

impl OpenAIClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIClient {
    async fn generate(
        &self,
        prompt: &str,
        _config: &Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 1000
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        let raw_content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");
        // Define the end token we want to skip up to. Adjust the token string if needed.
        let end_token = "final<|message|>";
        let content = if let Some(pos) = raw_content.find(end_token) {
            // Skip everything up to and including the end token.
            raw_content[(pos + end_token.len())..].to_string()
        } else {
            // If the token is not found, return the full content.
            raw_content.to_string()
        };

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
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 1000,
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
        // OpenAI doesn't support job cancellation
        Ok(())
    }
}
