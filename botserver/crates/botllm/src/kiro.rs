//! Kiro (kiro.dev) LLM provider.
//!
//! Kiro API keys (`ksk_…`) authenticate against the Amazon Q Developer /
//! CodeWhisperer backend — NOT an OpenAI-compatible endpoint. The protocol is
//! AWS JSON 1.0 over `X-Amz-Target:
//! AmazonCodeWhispererStreamingService.GenerateAssistantResponse` at
//! `https://q.<region>.amazonaws.com/`, and the response is an AWS Event
//! Stream binary envelope with JSON event objects embedded (`{"content":…}`,
//! `{"toolUseId":…}`, `{"usage":…}`, …).
//!
//! Reference protocol behavior (pi-kiro / kiro-gateway, MIT):
//! - `Authorization: Bearer <ksk key>` + `tokentype: API_KEY` header
//! - `origin: AI_EDITOR` on every user message; `agentTaskType: "vibe"`
//! - alternating `userInputMessage` / `assistantResponseMessage` history
//! - `INSUFFICIENT_MODEL_CAPACITY` errors are retryable with backoff

use crate::llm_models::{get_handler, ModelHandler};
use crate::LLMProvider;
use async_trait::async_trait;
use log::{error, warn};
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const DEFAULT_ENDPOINT: &str = "https://q.us-east-1.amazonaws.com/";
const TARGET_STREAMING: &str = "AmazonCodeWhispererStreamingService.GenerateAssistantResponse";
const ORIGIN: &str = "AI_EDITOR";
const MAX_CAPACITY_RETRIES: u32 = 3;
/// Aligns with the other providers (OpenAI/GLM/Bedrock use 180s).
const REQUEST_TIMEOUT_SECS: u64 = 180;

#[derive(Debug)]
pub struct KiroClient {
    http: reqwest::Client,
    endpoint: String,
    rate_limiter: Arc<crate::rate_limiter::ApiRateLimiter>,
}

impl KiroClient {
    pub fn new(base_url: String) -> Self {
        let endpoint = if base_url.trim().is_empty()
            || base_url.contains("kiro.dev")
            || !base_url.starts_with("http")
        {
            DEFAULT_ENDPOINT.to_string()
        } else {
            base_url.trim_end_matches('/').to_string() + "/"
        };
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| {
                warn!("Kiro: failed to build reqwest client with timeouts, using default");
                reqwest::Client::new()
            });
        Self {
            http,
            endpoint,
            rate_limiter: Arc::new(crate::rate_limiter::ApiRateLimiter::unlimited()),
        }
    }

    /// Normalizes a requested model id to a Kiro model id. Kiro exposes
    /// Claude-family models; dash/dot/versioned names all resolve to the
    /// canonical catalog id.
    fn kiro_model_id(model: &str) -> String {
        let m = model.to_lowercase();
        // Strip version suffixes like "-20250929".
        let base = m
            .split('-')
            .filter(|p| !p.chars().all(|c| c.is_ascii_digit()) || p.len() != 8)
            .collect::<Vec<_>>()
            .join("-");
        let base = base.replace(".5", "-5").replace(".4", "-4");
        if base.contains("opus") {
            "claude-opus-5".to_string()
        } else if base.contains("haiku") {
            "claude-haiku-4-5".to_string()
        } else if base.contains("sonnet-4-5") || base.contains("sonnet-4.5") {
            "claude-sonnet-4-5".to_string()
        } else if base.contains("sonnet") {
            "claude-sonnet-4-6".to_string()
        } else if base.contains("glm") {
            "GLM-5".to_string()
        } else if base.contains("deepseek") {
            "DeepSeek-V3.2".to_string()
        } else if base.contains("qwen") {
            "Qwen3-Coder-Next".to_string()
        } else if base.contains("minimax") {
            "MiniMax-M2.5".to_string()
        } else {
            "auto".to_string()
        }
    }

    fn is_capacity_error(body: &str) -> bool {
        body.contains("INSUFFICIENT_MODEL_CAPACITY")
    }

    /// Extracts the error message text from an AWS JSON error body.
    fn extract_error_text(body: &str) -> String {
        if let Ok(v) = serde_json::from_str::<Value>(body) {
            let msg = v
                .get("message")
                .or_else(|| v.get("Message"))
                .or_else(|| v.get("__type"))
                .and_then(|m| m.as_str())
                .unwrap_or("");
            if !msg.is_empty() {
                return msg.to_string();
            }
        }
        body.chars().take(400).collect()
    }

    /// Single streaming request → parsed events. Capacity errors are retried
    /// with exponential backoff; everything else fails fast.
    async fn request_stream(
        &self,
        request: &Value,
        key: &str,
    ) -> Result<impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>>, Box<dyn Error + Send + Sync>>
    {
        let capacity_retries = Arc::new(std::sync::atomic::AtomicU32::new(0));
        for attempt in 0..=MAX_CAPACITY_RETRIES {
            let ua = format!(
                "aws-sdk-rust/1.0.0 ua/2.1 os/other lang/rust api/codewhispererstreaming#1.28.3 m/E app/AmazonQ-For-CLI md/appVersion-1.28.3-{}",
                uuid::Uuid::new_v4().simple()
            );
            let response = self
                .http
                .post(&self.endpoint)
                .header("Content-Type", "application/x-amz-json-1.0")
                .header("Accept", "application/json")
                .header("Authorization", format!("Bearer {key}"))
                .header("tokentype", "API_KEY")
                .header("X-Amz-Target", TARGET_STREAMING)
                .header("x-amzn-codewhisperer-optout", "true")
                .header("amz-sdk-invocation-id", uuid::Uuid::new_v4().to_string())
                .header("amz-sdk-request", "attempt=1; max=1")
                .header("x-amzn-kiro-agent-mode", "vibe")
                .header("x-amz-user-agent", &ua)
                .header("user-agent", &ua)
                .json(request)
                .send()
                .await;

            match response {
                Ok(r) if r.status().is_success() => {
                    return Ok(r.bytes_stream());
                }
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    if Self::is_capacity_error(&body) {
                        let n = capacity_retries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if n < MAX_CAPACITY_RETRIES {
                            let delay = Duration::from_millis(800 * 2u64.pow(n));
                            warn!("Kiro: model capacity error, retry {}/{} in {:?}", n + 1, MAX_CAPACITY_RETRIES, delay);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    let msg = Self::extract_error_text(&body);
                    error!("Kiro request failed: status {status}: {msg}");
                    if status.as_u16() == 401 || status.as_u16() == 403 {
                        return Err(format!("Kiro API key rejected ({status}) — check the ksk_ key in Vault: {msg}").into());
                    }
                    return Err(format!("Kiro API error ({status}): {msg}").into());
                }
                Err(e) => {
                    if attempt < MAX_CAPACITY_RETRIES {
                        warn!("Kiro connection error ({e}), retrying…");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    return Err(format!("Kiro connection failed: {e}").into());
                }
            }
        }
        Err("Kiro request failed after retries".into())
    }

    /// Builds the Kiro request from OpenAI-style messages.
    fn build_request(messages: &Value, model_id: &str) -> Value {
        let empty = vec![];
        let arr = messages.as_array().unwrap_or(&empty);
        // System content is folded into the first user message (Kiro has no
        // system role).
        let system_text: String = arr
            .iter()
            .filter(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"))
            .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut history: Vec<Value> = Vec::new();
        let mut current_content = String::new();

        for msg in arr {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
            match role {
                "system" => {}
                "user" => {
                    if !current_content.is_empty() || !history.is_empty() {
                        // Flush a pending assistant turn before a new user turn
                        // keeps the strict alternation Kiro expects.
                    }
                    current_content = if history.is_empty() && !system_text.is_empty() {
                        format!("{system_text}\n\n{content}")
                    } else {
                        content.to_string()
                    };
                    history.push(json!({
                        "userInputMessage": {
                            "content": current_content,
                            "modelId": model_id,
                            "origin": ORIGIN,
                        }
                    }));
                }
                "assistant" => {
                    history.push(json!({
                        "assistantResponseMessage": { "content": content }
                    }));
                }
                "tool" => {
                    // Tool results are surfaced as plain context (chat tools
                    // are injected via __api_call__, not native tool_use).
                    let last = history.last_mut();
                    if let Some(last) = last {
                        if let Some(uim) = last.get_mut("userInputMessage") {
                            if let Some(c) = uim.get_mut("content").and_then(|c| c.as_str()) {
                                let merged = format!("{}\n\n[tool result] {content}", c);
                                uim["content"] = json!(merged);
                                continue;
                            }
                        }
                    }
                    history.push(json!({
                        "userInputMessage": {
                            "content": format!("[tool result] {content}"),
                            "modelId": model_id,
                            "origin": ORIGIN,
                        }
                    }));
                }
                _ => {}
            }
        }

        // The last entry is the CURRENT user turn — pop it out of history so
        // it becomes currentMessage (Kiro requires exactly one).
        let current = match history.pop() {
            Some(c) => c,
            None => json!({
                "userInputMessage": {
                    "content": "Hello",
                    "modelId": model_id,
                    "origin": ORIGIN,
                }
            }),
        };
        let current_user = current;

        let mut conversation = json!({
            "chatTriggerType": "MANUAL",
            "agentTaskType": "vibe",
            "currentMessage": { "userInputMessage": current_user["userInputMessage"].clone() },
        });
        if !history.is_empty() {
            conversation["history"] = json!(history);
        }
        json!({
            "conversationState": conversation,
            "agentMode": "vibe",
        })
    }

    async fn emit_stream(
        &self,
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>>,
        tx: &mpsc::Sender<String>,
        handler: &dyn ModelHandler,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use futures_util::StreamExt;
        let mut stream = std::pin::pin!(stream);
        let mut buffer = String::new();
        let mut state_buffer = String::new();
        const EVENT_PATTERNS: [&str; 11] = [
            "{\"content\":", "{\"name\":", "{\"input\":", "{\"stop\":",
            "{\"contextUsagePercentage\":", "{\"followupPrompt\":", "{\"usage\":",
            "{\"toolUseId\":", "{\"unit\":", "{\"error\":", "{\"Error\":",
        ];
        loop {
            let next = stream.next().await;
            if let Some(chunk) = &next {
                if let Ok(bytes) = chunk {
                    buffer.push_str(&String::from_utf8_lossy(bytes));
                }
            }
            // Parse all complete events out of the buffer.
            loop {
                let mut earliest: Option<usize> = None;
                for p in EVENT_PATTERNS.iter() {
                    if let Some(idx) = buffer.find(p) {
                        earliest = Some(match earliest {
                            Some(e) if e <= idx => e,
                            _ => idx,
                        });
                    }
                }
                let Some(start) = earliest else { break };
                let Some(end) = find_json_end(&buffer, start) else { break };
                let raw: String = buffer.drain(..end + 1).collect();
                let raw = raw[start..].to_string();
                if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
                    if let Some(content) = parsed.get("content").and_then(|c| c.as_str()) {
                        let processed = handler.process_content_streaming(content, &mut state_buffer);
                        if !processed.content.is_empty() {
                            let _ = tx.send(processed.content).await;
                        }
                        if !processed.reasoning.is_empty() {
                            let _ = tx
                                .send(serde_json::json!({"__reasoning__": processed.reasoning}).to_string())
                                .await;
                        }
                    } else if let Some(tool) = parsed.get("toolUseId").cloned() {
                        // toolUse events: {"toolUseId":…,"name":…,"input":…}
                        let _ = tool;
                        if let (Some(name), Some(input)) = (
                            parsed.get("name").and_then(|n| n.as_str()),
                            parsed.get("input").cloned(),
                        ) {
                            let args = match input {
                                Value::String(s) => s,
                                v => v.to_string(),
                            };
                            let msg = json!({"__tool_call__": true, "name": name, "arguments": args});
                            let _ = tx.send(msg.to_string()).await;
                        }
                    } else if let Some(err) = parsed.get("error").or_else(|| parsed.get("Error")) {
                        let msg = match err.as_str() {
                            Some(s) => s.to_string(),
                            None => err.to_string(),
                        };
                        let _ = tx
                            .send(format!("__kiro_error__{msg}"))
                            .await;
                    }
                }
            }
            if next.is_none() {
                break;
            }
        }
        Ok(())
    }
}

/// Finds the matching `}` for the `{` at `start` (brace-balanced, string-aware).
fn find_json_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        let c = b as char;
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[async_trait]
impl LLMProvider for KiroClient {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let raw_messages = if config.is_array() && !config.as_array().unwrap_or(&vec![]).is_empty() {
            config.clone()
        } else {
            json!([{"role": "user", "content": prompt}])
        };
        let model_id = Self::kiro_model_id(model);
        let request = Self::build_request(&raw_messages, &model_id);
        let stream = self.request_stream(&request, key).await?;
        let (tx, mut rx) = mpsc::channel::<String>(100_000);
        let handler = get_handler(model);
        let self_ = std::sync::Arc::new(self_clone(self));
        let emit = tokio::spawn(async move {
            let _ = self_.emit_stream(stream, &tx, handler.as_ref()).await;
        });
        let mut full = String::new();
        while let Some(chunk) = rx.recv().await {
            if chunk.starts_with("{\"__tool_call__") || chunk.starts_with("__kiro_error__") || chunk.starts_with("{\"__reasoning__") {
                if let Some(err) = chunk.strip_prefix("__kiro_error__") {
                    return Err(format!("Kiro stream error: {err}").into());
                }
                continue;
            }
            full.push_str(&chunk);
        }
        let _ = emit.await;
        Ok(full)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let raw_messages = if config.is_array() && !config.as_array().unwrap_or(&vec![]).is_empty() {
            config.clone()
        } else {
            json!([{"role": "user", "content": prompt}])
        };
        let model_id = Self::kiro_model_id(model);
        let request = Self::build_request(&raw_messages, &model_id);
        if let Err(e) = self.rate_limiter.acquire(1).await {
            error!("Kiro rate limit: {e}");
            return Err(Box::new(e));
        }
        let stream = self.request_stream(&request, key).await?;
        let handler = get_handler(model);
        self.emit_stream(stream, &tx, handler.as_ref()).await
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

/// KiroClient is used behind Arc by the emit task in `generate`; provide a
/// cheap clone (all fields are cheap handles).
fn self_clone(client: &KiroClient) -> KiroClient {
    KiroClient {
        http: client.http.clone(),
        endpoint: client.endpoint.clone(),
        rate_limiter: client.rate_limiter.clone(),
    }
}

impl Clone for KiroClient {
    fn clone(&self) -> Self {
        self_clone(self)
    }
}
