//! HTTP-target adapter. Wraps an OpenAI-compatible chat completions endpoint
//! as an \`LlmTarget\` so the runner can hit a real model.

use crate::runner::{LlmTarget, TaskOutcome};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const COST_PER_CHAR: f64 = 0.000002;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Message>,
    pub temperature: f32,
}

#[derive(Debug, serde::Deserialize)]
struct ChatResponseChoice {
    pub message: Message,
}

#[derive(Debug, serde::Deserialize)]
struct ChatResponse {
    pub choices: Vec<ChatResponseChoice>,
}

pub struct OpenAiCompatibleTarget {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub client: Client,
}

impl OpenAiCompatibleTarget {
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            model: model.into(),
            temperature: 0.0,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmTarget for OpenAiCompatibleTarget {
    async fn complete(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, String> {
        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(Message {
                role: "system".into(),
                content: sys.into(),
            });
        }
        messages.push(Message {
            role: "user".into(),
            content: user_prompt.into(),
        });
        let request = ChatRequest {
            model: &self.model,
            messages,
            temperature: self.temperature,
        };
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("http: {e}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("non-success status {status}: {body}"));
        }
        let body: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("decode: {e}"))?;
        Ok(body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }

    async fn complete_with_usage(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
    ) -> TaskOutcome {
        match self.complete(system_prompt, user_prompt).await {
            Ok(response) => TaskOutcome {
                cost: response.chars().count() as f64 * COST_PER_CHAR,
                response,
                tool_calls: 0,
            },
            Err(_) => TaskOutcome::default(),
        }
    }
}
