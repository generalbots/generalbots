//! HTTP-target adapter. Wraps an OpenAI-compatible chat completions endpoint
//! as an \`LlmTarget\` so the runner can hit a real model.

use crate::runner::{LlmTarget, TaskOutcome};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const COST_PER_CHAR: f64 = 0.000002;
const MAX_RETRIES: u32 = 2;
const RETRY_DELAYS_SECS: [u64; 2] = [1, 3];

#[derive(Debug, Clone, Serialize)]
struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponseMessage {
    // Reasoning models behind free gateways return `content: null` and put the
    // text in `reasoning` when the budget runs out. Rejecting the whole
    // response over that turned every such task into a transport error.
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Message>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
struct ChatResponseChoice {
    pub message: ResponseMessage,
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
            // A stalled provider must fail fast, not block the whole gate
            // (same lesson as the S3 boot hang: reqwest defaults to no
            // timeout).
            client: Client::builder()
                .timeout(Duration::from_secs(90))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|e| {
                    log::error!("boteval client build failed: {e}");
                    Client::new()
                }),
        }
    }
}

#[async_trait]
impl LlmTarget for OpenAiCompatibleTarget {
    async fn complete(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, String> {
        let mut attempt = 0;
        loop {
            match self.complete_once(system_prompt, user_prompt).await {
                // An empty completion is a provider hiccup, not a wrong
                // answer. Retry it like a transport error so a blank response
                // never counts as a failed contract.
                Ok(response) if !response.trim().is_empty() => return Ok(response),
                Ok(_) => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        return Ok(String::new());
                    }
                    log::warn!(
                        "boteval attempt {attempt} returned an empty completion; retrying in {}s",
                        RETRY_DELAYS_SECS[(attempt - 1) as usize]
                    );
                    tokio::time::sleep(Duration::from_secs(
                        RETRY_DELAYS_SECS[(attempt - 1) as usize],
                    ))
                    .await;
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        log::warn!("boteval task failed after {attempt} attempts: {e}");
                        return Err(e);
                    }
                    log::warn!("boteval attempt {attempt} failed ({e}); retrying in {}s", RETRY_DELAYS_SECS[(attempt - 1) as usize]);
                    tokio::time::sleep(Duration::from_secs(RETRY_DELAYS_SECS[(attempt - 1) as usize])).await;
                }
            }
        }
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
            Err(e) => {
                log::error!("boteval task errored: {e}");
                TaskOutcome::default()
            }
        }
    }
}

impl OpenAiCompatibleTarget {
    async fn complete_once(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
    ) -> Result<String, String> {
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
            max_tokens: Some(1024),
        };
        let mut request_builder = self.client.post(&self.endpoint);
        // An empty key means anonymous access: free gateway models accept it,
        // but a literal `Authorization: Bearer ` header is rejected as an
        // invalid token, so omit the header entirely.
        if !self.api_key.is_empty() {
            request_builder = request_builder.bearer_auth(&self.api_key);
        }
        let response = request_builder
            // Some OpenAI-compatible gateways route by a session id and reject
            // requests without it; other providers ignore the header.
            .header("x-opencode-session", uuid::Uuid::new_v4().to_string())
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
            .and_then(|c| c.message.content)
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::ChatResponse;

    #[test]
    fn parses_content_when_present() {
        let raw = r#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(raw).expect("valid response");
        assert_eq!(
            parsed.choices[0].message.content.as_deref(),
            Some("ok")
        );
    }

    #[test]
    fn parses_null_content_from_reasoning_models() {
        let raw = r#"{"choices":[{"message":{"role":"assistant","content":null,"reasoning":"thinking"}}]}"#;
        let parsed: ChatResponse = serde_json::from_str(raw).expect("null content must parse");
        assert_eq!(parsed.choices[0].message.content, None);
    }
}
