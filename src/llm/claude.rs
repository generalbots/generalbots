use async_trait::async_trait;
use futures_util::StreamExt;
use log::{error, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

use super::{llm_models::get_handler, LLMProvider};

// Configuration matching Node.js proxy exactly
const MAX_RETRIES: u32 = 5;
const INITIAL_DELAY_MS: u64 = 1000;
const MAX_DELAY_MS: u64 = 30000;
const BACKOFF_MULTIPLIER: f64 = 2.0;
const TIMEOUT_MS: u64 = 60000;
const STREAMING_TIMEOUT_MS: u64 = 180000; // 3 minutes for streaming
const ACTIVITY_TIMEOUT_MS: u64 = 30000; // 30 seconds no data = timeout

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ClaudeContentBlock>,
    pub model: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
}

// SSE event structures - Anthropic format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStreamDelta {
    #[serde(rename = "type", default)]
    pub delta_type: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub delta: Option<ClaudeStreamDelta>,
    #[serde(default)]
    pub index: Option<u32>,
}

// Azure/OpenAI style streaming structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStreamChoice {
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub delta: Option<AzureStreamDelta>,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStreamDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStreamChunk {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub choices: Vec<AzureStreamChoice>,
}

pub struct ClaudeClient {
    base_url: String,
    deployment_name: String,
}

impl ClaudeClient {
    pub fn new(base_url: String, deployment_name: Option<String>) -> Self {
        Self {
            base_url,
            deployment_name: deployment_name.unwrap_or_else(|| "claude-opus-4-5".to_string()),
        }
    }

    pub fn azure(endpoint: String, deployment_name: String) -> Self {
        Self {
            base_url: endpoint,
            deployment_name,
        }
    }

    fn get_retry_delay(attempt: u32) -> Duration {
        let delay = (INITIAL_DELAY_MS as f64 * BACKOFF_MULTIPLIER.powi(attempt as i32))
            .min(MAX_DELAY_MS as f64);
        // Add jitter like Node.js: delay * 0.25 * (Math.random() * 2 - 1)
        let jitter = delay * 0.25 * (rand::random::<f64>() * 2.0 - 1.0);
        Duration::from_millis((delay + jitter) as u64)
    }

    fn is_retryable_error(err_msg: &str, status_code: Option<u16>) -> bool {
        // Retryable status codes matching Node.js: [408, 429, 500, 502, 503, 504]
        if let Some(code) = status_code {
            if [408, 429, 500, 502, 503, 504].contains(&code) {
                return true;
            }
        }

        // Retryable error patterns matching Node.js
        let msg = err_msg.to_lowercase();
        msg.contains("timeout")
            || msg.contains("econnreset")
            || msg.contains("etimedout")
            || msg.contains("enotfound")
            || msg.contains("econnrefused")
            || msg.contains("epipe")
            || msg.contains("ehostunreach")
            || msg.contains("aborted")
            || msg.contains("socket hang up")
            || msg.contains("network")
            || msg.contains("connection reset")
            || msg.contains("broken pipe")
            || msg.contains("connection closed")
    }

    fn build_messages_from_value(
        &self,
        prompt: &str,
        messages: &Value,
    ) -> (Option<String>, Vec<ClaudeMessage>) {
        let empty_vec = vec![];

        let mut claude_messages: Vec<ClaudeMessage> = if messages.is_array() {
            let arr = messages.as_array().unwrap_or(&empty_vec);
            if arr.is_empty() {
                vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }]
            } else {
                arr.iter()
                    .filter_map(|m| {
                        let role = m["role"].as_str().unwrap_or("user");
                        let content = m["content"].as_str().unwrap_or("");
                        if role == "system"
                            || role == "episodic"
                            || role == "compact"
                            || content.is_empty()
                        {
                            None
                        } else {
                            let normalized_role = match role {
                                "user" | "assistant" => role.to_string(),
                                _ => "user".to_string(),
                            };
                            Some(ClaudeMessage {
                                role: normalized_role,
                                content: content.to_string(),
                            })
                        }
                    })
                    .collect()
            }
        } else {
            vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]
        };

        if claude_messages.is_empty() && !prompt.is_empty() {
            claude_messages.push(ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            });
        }

        // Extract system messages
        let system_prompt: Option<String> = if messages.is_array() {
            let system_text: String = messages
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter(|m| m["role"].as_str() == Some("system"))
                .map(|m| m["content"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
                .join("\n\n");
            if system_text.is_empty() {
                None
            } else {
                Some(system_text)
            }
        } else {
            None
        };

        (system_prompt, claude_messages)
    }

    /// Sanitizes a string by removing invalid UTF-8 surrogate characters
    fn sanitize_utf8(input: &str) -> String {
        input.chars()
            .filter(|c| {
                let cp = *c as u32;
                !(0xD800..=0xDBFF).contains(&cp) && !(0xDC00..=0xDFFF).contains(&cp)
            })
            .collect()
    }

    pub fn build_messages(
        system_prompt: &str,
        context_data: &str,
        history: &[(String, String)],
    ) -> (Option<String>, Vec<ClaudeMessage>) {
        let mut messages = Vec::new();
        let mut system_parts = Vec::new();

        if !system_prompt.is_empty() {
            system_parts.push(Self::sanitize_utf8(system_prompt));
        }
        if !context_data.is_empty() {
            system_parts.push(Self::sanitize_utf8(context_data));
        }

        for (role, content) in history {
            if role == "episodic" || role == "compact" {
                system_parts.push(format!("[Previous conversation summary]: {}", Self::sanitize_utf8(content)));
            }
        }

        let system = if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n\n"))
        };

        let mut last_role: Option<String> = None;

        for (role, content) in history {
            let normalized_role = match role.as_str() {
                "user" => Some("user".to_string()),
                "assistant" => Some("assistant".to_string()),
                "system" | "episodic" | "compact" => None,
                _ => Some("user".to_string()),
            };

            if let Some(norm_role) = normalized_role {
                let sanitized_content = Self::sanitize_utf8(content);
                if sanitized_content.is_empty() {
                    continue;
                }

                if Some(&norm_role) == last_role.as_ref() {
                    if let Some(last_msg) = messages.last_mut() {
                        let last_msg: &mut ClaudeMessage = last_msg;
                        last_msg.content.push_str("\n\n");
                        last_msg.content.push_str(&sanitized_content);
                        continue;
                    }
                }

                messages.push(ClaudeMessage {
                    role: norm_role.clone(),
                    content: sanitized_content,
                });
                last_role = Some(norm_role);
            }
        }

        (system, messages)
    }

    fn extract_text_from_response(&self, response: &ClaudeResponse) -> String {
        response
            .content
            .iter()
            .filter(|block| block.content_type == "text")
            .map(|block| block.text.clone())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Process SSE data and extract text - handles both Anthropic and Azure formats
    fn process_sse_data(&self, data: &str, model_name: &str) -> Option<String> {
        if data == "[DONE]" {
            return None;
        }

        let handler = get_handler(model_name);

        // Try Azure/OpenAI format first (chat.completion.chunk)
        if let Ok(chunk) = serde_json::from_str::<AzureStreamChunk>(data) {
            if chunk.object == "chat.completion.chunk" || !chunk.choices.is_empty() {
                for choice in &chunk.choices {
                    if let Some(delta) = &choice.delta {
                        // Get content (prefer content over reasoning_content)
                        let text = delta
                            .content
                            .as_deref()
                            .or(delta.reasoning_content.as_deref())
                            .unwrap_or("");

                        if !text.is_empty() {
                            let processed = handler.process_content(text);
                            if !processed.is_empty() {
                                return Some(processed);
                            }
                        }
                    }
                }
                return None;
            }
        }

        // Try standard Anthropic SSE format
        if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(data) {
            match event.event_type.as_str() {
                "content_block_delta" => {
                    if let Some(delta) = event.delta {
                        if delta.delta_type == "text_delta" && !delta.text.is_empty() {
                            let processed = handler.process_content(&delta.text);
                            if !processed.is_empty() {
                                return Some(processed);
                            }
                        }
                    }
                }
                "message_start" => trace!("CLAUDE message_start"),
                "content_block_start" => trace!("CLAUDE content_block_start"),
                "content_block_stop" => trace!("CLAUDE content_block_stop"),
                "message_stop" => trace!("CLAUDE message_stop"),
                "message_delta" => trace!("CLAUDE message_delta"),
                "error" => {
                    error!("CLAUDE Error event: {}", data);
                }
                _ => trace!("CLAUDE Event: {}", event.event_type),
            }
        }

        None
    }

    /// Streaming implementation using reqwest - mimics Node.js https.request with res.on('data')
    async fn stream_with_reqwest(
        &self,
        request_body: String,
        api_key: &str,
        tx: mpsc::Sender<String>,
        model_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let start_time = Instant::now();

        trace!(
            "CLAUDE Streaming request to {} body_len={}",
            url,
            request_body.len()
        );

        // Build client matching Node.js httpsAgent configuration
        // IMPORTANT: NO timeout() here - it causes premature stream closure!
        // Node.js doesn't set a timeout on the response body, only on connect/headers
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(TIMEOUT_MS))
            .pool_idle_timeout(Duration::from_secs(30)) // keepAliveMsecs: 30000
            .pool_max_idle_per_host(10) // maxFreeSockets: 10
            .tcp_nodelay(true)
            .tcp_keepalive(Duration::from_secs(60))
            .http1_only() // Force HTTP/1.1 like Node.js
            .build()?;

        // Send request with headers matching Node.js exactly
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Accept", "text/event-stream")
            .header("Connection", "keep-alive")
            .body(request_body)
            .send()
            .await?;

        let status = response.status();
        trace!("CLAUDE Response status: {} in {:?}", status, start_time.elapsed());

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("[CLAUDE] Error response: {}", error_text);
            return Err(format!("HTTP {}: {}", status, error_text).into());
        }

        // Stream response body - this is like Node.js res.on('data')
        let mut stream = response.bytes_stream();
        let mut sse_buffer = String::new();
        let mut text_chunks_sent = 0usize;
        let mut total_bytes = 0usize;
        let mut chunk_count = 0usize;
        let mut last_activity = Instant::now();
        let mut done_received = false;

        trace!("CLAUDE Starting stream read");

        loop {
            // Check overall timeout
            if start_time.elapsed() > Duration::from_millis(STREAMING_TIMEOUT_MS) {
                error!(
                    "CLAUDE Overall streaming timeout {}ms exceeded",
                    STREAMING_TIMEOUT_MS
                );
                break;
            }

            // Check activity timeout
            if last_activity.elapsed() > Duration::from_millis(ACTIVITY_TIMEOUT_MS) {
                if done_received || text_chunks_sent > 0 {
                    trace!(
                        "CLAUDE Activity timeout but {} chunks sent, treating as complete",
                        text_chunks_sent
                    );
                    break;
                }
                error!(
                    "CLAUDE Activity timeout - no data for {}ms",
                    ACTIVITY_TIMEOUT_MS
                );
                return Err(format!(
                    "Stream activity timeout - no data for {}ms",
                    ACTIVITY_TIMEOUT_MS
                )
                .into());
            }

            // Read next chunk with timeout - like Node.js res.on('data')
            let chunk_result = tokio::time::timeout(
                Duration::from_millis(ACTIVITY_TIMEOUT_MS),
                stream.next(),
            )
            .await;

            let chunk = match chunk_result {
                Ok(Some(Ok(bytes))) => {
                    last_activity = Instant::now();
                    bytes
                }
                Ok(Some(Err(e))) => {
                    error!("CLAUDE Stream read error: {}", e);
                    if text_chunks_sent > 0 {
                        warn!(
                            "CLAUDE Had {} chunks before error, treating as partial success",
                            text_chunks_sent
                        );
                        break;
                    }
                    return Err(format!("Stream read error: {}", e).into());
                }
                Ok(None) => {
                    trace!(
                        "CLAUDE Stream ended after {} chunks, {} bytes, {} text chunks",
                        chunk_count, total_bytes, text_chunks_sent
                    );
                    break;
                }
                Err(_) => {
                    if text_chunks_sent > 0 || done_received {
                        trace!(
                            "CLAUDE Timeout but {} chunks sent, treating as complete",
                            text_chunks_sent
                        );
                        break;
                    }
                    error!("CLAUDE Timeout waiting for stream data");
                    return Err("Timeout waiting for stream data".into());
                }
            };

            chunk_count += 1;
            total_bytes += chunk.len();
            let chunk_str = String::from_utf8_lossy(&chunk);



            sse_buffer.push_str(&chunk_str);

            // Process complete SSE events in buffer
            while let Some(event_end) = sse_buffer.find("\n\n") {
                let event_block = sse_buffer[..event_end].to_string();
                sse_buffer = sse_buffer[event_end + 2..].to_string();

                // Parse SSE event lines
                for line in event_block.lines() {
                    let line = line.trim();

                    if line.is_empty() || line.starts_with("event:") {
                        continue;
                    }

                    if !line.starts_with("data: ") {
                        continue;
                    }

                    let data = &line[6..];

                    if data == "[DONE]" {
                        trace!(
                            "CLAUDE Received DONE - {} chunks, {} bytes in {:?}",
                            text_chunks_sent,
                            total_bytes,
                            start_time.elapsed()
                        );
                        done_received = true;
                        continue;
                    }

                    // Process SSE data and send text chunks
                    if let Some(text) = self.process_sse_data(data, model_name) {
                        if tx.send(text.clone()).await.is_err() {
                            warn!("CLAUDE Receiver dropped, stopping stream");
                            return Ok(());
                        }
                        text_chunks_sent += 1;
                    }
                }
            }
        }

        // Process any remaining data in buffer
        if !sse_buffer.is_empty() {
            for line in sse_buffer.lines() {
                let line = line.trim();
                if let Some(data) = line.strip_prefix("data: ") {
                    if data != "[DONE]" {
                        if let Some(text) = self.process_sse_data(data, model_name) {
                            let _ = tx.send(text).await;
                            text_chunks_sent += 1;
                        }
                    }
                }
            }
        }

        trace!(
            "CLAUDE Stream complete: {} chunks, {} bytes, {} text in {:?}",
            chunk_count, total_bytes, text_chunks_sent, start_time.elapsed()
        );

        if text_chunks_sent == 0 && !done_received {
            warn!("CLAUDE No text chunks sent and no DONE received");
        }

        Ok(())
    }

    /// Single streaming attempt with full error handling
    async fn stream_single_attempt(
        &self,
        prompt: &str,
        messages: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let model_name = if model.is_empty() {
            &self.deployment_name
        } else {
            model
        };
        let (system, claude_messages) = self.build_messages_from_value(prompt, messages);

        if claude_messages.is_empty() {
            return Err("No messages to send".into());
        }

        let request = ClaudeRequest {
            model: model_name.to_string(),
            max_tokens: 16000,
            messages: claude_messages,
            system,
            stream: Some(true),
        };

        let request_body = serde_json::to_string(&request)?;
        trace!(
            "CLAUDE Streaming request: model={}, messages={}, body_len={}",
            model_name,
            request.messages.len(),
            request_body.len()
        );

        self.stream_with_reqwest(request_body, key, tx, model_name)
            .await
    }
}

#[async_trait]
impl LLMProvider for ClaudeClient {
    async fn generate(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let model_name = if model.is_empty() {
            &self.deployment_name
        } else {
            model
        };
        let (system, claude_messages) = self.build_messages_from_value(prompt, messages);

        if claude_messages.is_empty() {
            return Err("No messages to send".into());
        }

        let request = ClaudeRequest {
            model: model_name.to_string(),
            max_tokens: 4096,
            messages: claude_messages,
            system,
            stream: None,
        };

        let body = serde_json::to_string(&request)?;
        trace!("CLAUDE request: model={}, body_len={}", model_name, body.len());

        let start = Instant::now();
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));

        // Use reqwest for non-streaming
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .build()?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Claude API error ({}): {}", status, error_text).into());
        }

        trace!("CLAUDE response in {:?}, status={}", start.elapsed(), status);

        let result: ClaudeResponse = response.json().await?;
        let content = self.extract_text_from_response(&result);
        let handler = get_handler(model_name);

        Ok(handler.process_content(&content))
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        messages: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let delay = Self::get_retry_delay(attempt - 1);
                trace!(
                    "CLAUDE Retry {}/{} in {:?}",
                    attempt, MAX_RETRIES, delay
                );
                tokio::time::sleep(delay).await;
            }

            match self
                .stream_single_attempt(prompt, messages, tx.clone(), model, key, _tools)
                .await
            {
                Ok(()) => {
                    if attempt > 0 {
                        trace!("CLAUDE Success after {} attempts", attempt + 1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    error!("CLAUDE Attempt {} failed: {}", attempt + 1, err_msg);

                    if attempt < MAX_RETRIES && Self::is_retryable_error(&err_msg, None) {
                        last_error = Some(e);
                        continue;
                    }

                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "Max retries exceeded".into()))
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_client_new() {
        let client = ClaudeClient::new(
            "https://api.anthropic.com".to_string(),
            Some("claude-3-opus".to_string()),
        );
        assert_eq!(client.deployment_name, "claude-3-opus");
    }

    #[test]
    fn test_claude_client_azure() {
        let client = ClaudeClient::azure(
            "https://myendpoint.openai.azure.com/anthropic".to_string(),
            "claude-opus-4-5".to_string(),
        );
        assert_eq!(client.deployment_name, "claude-opus-4-5");
    }

    #[test]
    fn test_build_messages_empty() {
        let (system, messages) = ClaudeClient::build_messages("", "", &[]);
        assert!(system.is_none());
        assert!(messages.is_empty());
    }

    #[test]
    fn test_build_messages_with_system() {
        let (system, messages) = ClaudeClient::build_messages("Be helpful", "", &[]);
        assert_eq!(system, Some("Be helpful".to_string()));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_build_messages_with_history() {
        let history = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there".to_string()),
        ];
        let (system, messages) = ClaudeClient::build_messages("", "", &history);
        assert!(system.is_none());
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
    }
}
