use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use super::LLMProvider;

// GLM / z.ai API Client
// Similar to OpenAI but with different endpoint structure
// For z.ai, base URL already contains version (e.g., /v4), endpoint is just /chat/completions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMRequest {
    pub model: String,
    pub messages: Vec<GLMMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMResponseChoice {
    #[serde(default)]
    pub index: u32,
    pub message: GLMMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<GLMResponseChoice>,
    #[serde(default)]
    pub usage: Option<Value>,
}

// Streaming structures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GLMStreamDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMStreamChoice {
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub delta: GLMStreamDelta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLMStreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<GLMStreamChoice>,
    #[serde(default)]
    pub usage: Option<Value>,
}

#[derive(Debug)]
pub struct GLMClient {
    client: reqwest::Client,
    base_url: String,
}

impl GLMClient {
    pub fn new(base_url: String) -> Self {
        // For z.ai GLM API:
        // - Base URL typically is: https://api.z.ai/api/coding/paas/v4
        // - Endpoint path is: /chat/completions
        // - Full URL becomes: https://api.z.ai/api/coding/paas/v4/chat/completions

        // Remove trailing slash from base_url if present
        let base = base_url.trim_end_matches('/').to_string();

        Self {
            client: reqwest::Client::new(),
            base_url: base,
        }
    }

    fn build_url(&self) -> String {
        // GLM/z.ai uses /chat/completions (not /v1/chat/completions)
        format!("{}/chat/completions", self.base_url)
    }
}

#[async_trait]
impl LLMProvider for GLMClient {
    async fn generate(
        &self,
        prompt: &str,
        _config: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let messages = vec![GLMMessage {
            role: "user".to_string(),
            content: Some(prompt.to_string()),
            tool_calls: None,
        }];

        // Use glm-4.7 instead of glm-4 for z.ai API
        let model_name = if model == "glm-4" { "glm-4.7" } else { model };

        let request = GLMRequest {
            model: model_name.to_string(),
            messages,
            stream: Some(false),
            max_tokens: None,
            temperature: None,
            tools: None,
            tool_choice: None,
        };

        let url = self.build_url();
        info!("GLM non-streaming request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("GLM API error: {}", error_text);
            return Err(format!("GLM API error: {}", error_text).into());
        }

        let glm_response: GLMResponse = response.json().await?;
        let content = glm_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // DEBUG: Log what we received
        info!("[GLM_DEBUG] config type: {}", config);
        info!("[GLM_DEBUG] prompt: '{}'", prompt);
        info!("[GLM_DEBUG] config as JSON: {}", serde_json::to_string_pretty(config).unwrap_or_default());

        // config IS the messages array directly, not nested
        let messages = if let Some(msgs) = config.as_array() {
            // Convert messages from config format to GLM format
            msgs.iter()
                .filter_map(|m| {
                    let role = m.get("role")?.as_str()?;
                    let content = m.get("content")?.as_str()?;
                    info!("[GLM_DEBUG] Processing message - role: {}, content: '{}'", role, content);
                    if !content.is_empty() {
                        Some(GLMMessage {
                            role: role.to_string(),
                            content: Some(content.to_string()),
                            tool_calls: None,
                        })
                    } else {
                        info!("[GLM_DEBUG] Skipping empty content message");
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            // Fallback to building from prompt
            info!("[GLM_DEBUG] No array found, using prompt: '{}'", prompt);
            vec![GLMMessage {
                role: "user".to_string(),
                content: Some(prompt.to_string()),
                tool_calls: None,
            }]
        };

        // If no messages or all empty, return error
        if messages.is_empty() {
            return Err("No valid messages in request".into());
        }

        info!("[GLM_DEBUG] Final GLM messages count: {}", messages.len());

        // Use glm-4.7 for tool calling support
        // GLM-4.7 supports standard OpenAI-compatible function calling
        let model_name = if model == "glm-4" { "glm-4.7" } else { model };

        // Set tool_choice to "auto" when tools are present - this tells GLM to automatically decide when to call a tool
        let tool_choice = if tools.is_some() {
            Some(serde_json::json!("auto"))
        } else {
            None
        };

        let request = GLMRequest {
            model: model_name.to_string(),
            messages,
            stream: Some(true),
            max_tokens: None,
            temperature: None,
            tools: tools.map(|t| t.clone()),
            tool_choice,
        };

        let url = self.build_url();
        info!("GLM streaming request to: {}", url);

        // Log the exact request being sent
        let request_json = serde_json::to_string_pretty(&request).unwrap_or_default();
        info!("GLM request body: {}", request_json);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("GLM streaming error: {}", error_text);
            return Err(format!("GLM streaming error: {}", error_text).into());
        }

        let mut stream = response.bytes_stream();

        let mut buffer = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;

            buffer.extend_from_slice(&chunk);
            let data = String::from_utf8_lossy(&buffer);

            // Process SSE lines
            for line in data.lines() {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                if line == "data: [DONE]" {
                    let _ = tx.send(String::new()); // Signal end
                    return Ok(());
                }

                if line.starts_with("data: ") {
                    let json_str = line[6..].trim();
                    info!("[GLM_SSE] Received SSE line ({} chars): {}", json_str.len(), json_str);
                    if let Ok(chunk_data) = serde_json::from_str::<Value>(json_str) {
                        if let Some(choices) = chunk_data.get("choices").and_then(|c| c.as_array()) {
                            for choice in choices {
                                info!("[GLM_SSE] Processing choice");
                                if let Some(delta) = choice.get("delta") {
                                    info!("[GLM_SSE] Delta: {}", serde_json::to_string(delta).unwrap_or_default());

                                    // Handle tool_calls (GLM-4.7 standard function calling)
                                    if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                        for tool_call in tool_calls {
                                            info!("[GLM_SSE] Tool call detected: {}", serde_json::to_string(tool_call).unwrap_or_default());
                                            // Send tool_calls as JSON for the calling code to process
                                            let tool_call_json = serde_json::json!({
                                                "type": "tool_call",
                                                "content": tool_call
                                            }).to_string();
                                            match tx.send(tool_call_json).await {
                                                Ok(_) => {},
                                                Err(e) => {
                                                    error!("[GLM_TX] Failed to send tool_call to channel: {}", e);
                                                }
                                            }
                                        }
                                    }

                                    // GLM/z.ai returns both reasoning_content (thinking) and content (response)
                                    // We only send the actual content, ignoring reasoning_content
                                    // This makes GLM behave like OpenAI-compatible APIs
                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                        if !content.is_empty() {
                                            info!("[GLM_TX] Sending to channel: '{}'", content);
                                            match tx.send(content.to_string()).await {
                                                Ok(_) => {},
                                                Err(e) => {
                                                    error!("[GLM_TX] Failed to send to channel: {}", e);
                                                }
                                            }
                                        }
                                    } else {
                                        info!("[GLM_SSE] No content field in delta");
                                    }
                                } else {
                                    info!("[GLM_SSE] No delta in choice");
                                }
                                if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                                    if !reason.is_empty() {
                                        info!("GLM stream finished: {}", reason);
                                        let _ = tx.send(String::new());
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Keep unprocessed data in buffer
            if let Some(last_newline) = data.rfind('\n') {
                buffer = buffer[last_newline + 1..].to_vec();
            }
        }

        let _ = tx.send(String::new()); // Signal completion
        Ok(())
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // GLM doesn't have job cancellation
        info!("GLM cancel requested for session {} (no-op)", _session_id);
        Ok(())
    }
}
