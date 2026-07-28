pub mod bedrock;
pub mod claude;
pub mod glm;
pub mod hallucination_detector;
pub mod kimi;
pub mod llm_models;
pub mod rate_limiter;
pub mod vertex;
pub mod cache;
pub mod ci_gate;
pub mod episodic_memory;
pub mod evaluation;
pub mod local;
pub mod smart_router;
pub mod observability;
pub mod pipeline;

pub use ci_gate::{CiGateConfig, CiGateReport, CiGateRunner, RegressionSummary};
pub use evaluation::{
    EvaluationCriterion, EvaluationGate, EvaluationResult, Evaluator, RegressionReport,
};
pub use rate_limiter::{ApiRateLimiter, RateLimits};
pub use hallucination_detector::HallucinationDetector;
pub use llm_models::{get_handler, ProcessedChunk, ModelHandler};
pub use claude::ClaudeClient;
pub use glm::GLMClient;
pub use vertex::VertexTokenManager;
pub use bedrock::BedrockClient;
pub use pipeline::{PipelineConfig, LlmPipeline, MessageBuilder, KbContextManager, PromptManager};

use async_trait::async_trait;
use log::{info, trace, warn};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[async_trait]
pub trait LLMProvider: Send + Sync + std::fmt::Debug {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn cancel_job(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug)]
pub struct OpenAIClient {
    client: reqwest::Client,
    base_url: String,
    endpoint_path: String,
    rate_limiter: Arc<ApiRateLimiter>,
}

#[derive(Debug)]
pub struct AzureGPT5Client {
    client: reqwest::Client,
    base_url: String,
    api_version: String,
    rate_limiter: Arc<ApiRateLimiter>,
}

impl AzureGPT5Client {
    pub fn new(base_url: String, api_version: Option<String>) -> Self {
        let api_version = api_version.unwrap_or_else(|| "2025-04-01-preview".to_string());
        let rate_limiter = Arc::new(ApiRateLimiter::unlimited());
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            base_url,
            api_version,
            rate_limiter,
        }
    }

    fn sanitize_utf8(input: &str) -> String {
        input.chars()
            .filter(|c| {
                let cp = *c as u32;
                !(0xD800..=0xDBFF).contains(&cp) && !(0xDC00..=0xDFFF).contains(&cp)
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for AzureGPT5Client {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let raw_messages = if config.is_array() && !config.as_array().unwrap_or(&vec![]).is_empty() {
            config
        } else {
            &serde_json::json!([{"role": "user", "content": prompt}])
        };

        if let Err(e) = self.rate_limiter.acquire(4096).await {
            log::error!("Rate limit exceeded: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }

        let full_url = format!(
            "{}/openai/responses?api-version={}",
            self.base_url, self.api_version
        );
        let auth_header = format!("Bearer {}", key);

        let input_array: Vec<Value> = raw_messages
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.get("role").and_then(|r| r.as_str()).unwrap_or("user"),
                    "content": Self::sanitize_utf8(msg.get("content").and_then(|c| c.as_str()).unwrap_or(""))
                })
            })
            .collect();

        let response = self
            .client
            .post(&full_url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model,
                "input": input_array,
                "max_output_tokens": 16384
            }))
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("AzureGPT5 generate error: {}", error_text);
            return Err(format!("AzureGPT5 request failed with status: {}", status).into());
        }

        let result: Value = response.json().await?;
        let content = result["output"][0]["content"][0]["text"]
            .as_str()
            .unwrap_or("");

        Ok(content.to_string())
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = self.generate(prompt, config, model, key).await?;
        tx.send(content).await?;
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
    pub fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4)
    }

    pub fn estimate_messages_tokens(messages: &Value) -> usize {
        if let Some(msg_array) = messages.as_array() {
            msg_array
                .iter()
                .map(|msg| {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        Self::estimate_tokens(content)
                    } else {
                        0
                    }
                })
                .sum()
        } else {
            0
        }
    }

    pub fn truncate_messages(messages: &Value, max_tokens: usize) -> Value {
        let mut result = Vec::new();
        let mut token_count = 0;

        if let Some(msg_array) = messages.as_array() {
            for msg in msg_array {
                if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                    if role == "system" {
                        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                            let msg_tokens = Self::estimate_tokens(content);
                            if token_count + msg_tokens <= max_tokens {
                                result.push(msg.clone());
                                token_count += msg_tokens;
                            }
                        }
                    }
                }
            }

            let mut recent_messages: Vec<&Value> = msg_array
                .iter()
                .filter(|msg| msg.get("role").and_then(|r| r.as_str()) != Some("system"))
                .collect();

            recent_messages.reverse();

            for msg in recent_messages {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    let msg_tokens = Self::estimate_tokens(content);
                    if token_count + msg_tokens <= max_tokens {
                        result.push(msg.clone());
                        token_count += msg_tokens;
                    } else {
                        break;
                    }
                }
            }

            let system_count = result.len()
                - result
                    .iter()
                    .filter(|m| m.get("role").and_then(|r| r.as_str()) != Some("system"))
                    .count();
            let mut user_messages: Vec<Value> = result.drain(system_count..).collect();
            user_messages.reverse();
            result.extend(user_messages);
        }

        serde_json::Value::Array(result)
    }

    pub fn ensure_token_limit(messages: &Value, model_context_limit: usize) -> Value {
        let estimated_tokens = Self::estimate_messages_tokens(messages);
        let safe_limit = (model_context_limit as f64 * 0.9) as usize;

        if estimated_tokens > safe_limit {
            log::warn!(
                "Messages exceed token limit ({} > {}), truncating...",
                estimated_tokens,
                safe_limit
            );
            Self::truncate_messages(messages, safe_limit)
        } else {
            messages.clone()
        }
    }

    pub fn new(_api_key: String, base_url: Option<String>, endpoint_path: Option<String>) -> Self {
        let base = base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
        let trimmed_base = base.trim_end_matches('/').to_string();

        let has_v1_path = trimmed_base.contains("/v1/chat/completions");
        let has_chat_path = !has_v1_path && trimmed_base.contains("/chat/completions");

        let endpoint = if let Some(path) = endpoint_path {
            path
        } else if has_v1_path || (has_chat_path && !trimmed_base.contains("z.ai")) {
            "".to_string()
        } else if trimmed_base.contains("z.ai") || trimmed_base.contains("/v4") {
            "/chat/completions".to_string()
        } else {
            "/v1/chat/completions".to_string()
        };

        let (final_base, final_endpoint) = if !endpoint.is_empty() && trimmed_base.ends_with(&endpoint) {
            (trimmed_base, "".to_string())
        } else {
            (trimmed_base, endpoint)
        };

        let rate_limiter = if base.contains("groq.com") {
            ApiRateLimiter::new(RateLimits::groq_free_tier())
        } else if base.contains("openai.com") {
            ApiRateLimiter::new(RateLimits::openai_free_tier())
        } else {
            ApiRateLimiter::unlimited()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| {
                log::warn!("Failed to build reqwest client with timeouts, falling back to default");
                reqwest::Client::new()
            });

        Self {
            client,
            base_url: final_base,
            endpoint_path: final_endpoint,
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    pub fn sanitize_utf8(input: &str) -> String {
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
    ) -> Value {
        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": Self::sanitize_utf8(system_prompt)
            }));
        }
        if !context_data.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": Self::sanitize_utf8(context_data)
            }));
        }
        for (role, content) in history {
            let api_role = match role.as_str() {
                "user" | "assistant" | "system" | "developer" | "tool" => role.as_str(),
                "episodic" | "compact" => "system",
                _ => "system",
            };
            messages.push(serde_json::json!({
                "role": api_role,
                "content": Self::sanitize_utf8(content)
            }));
        }
        serde_json::Value::Array(messages)
    }
}

#[async_trait]
impl LLMProvider for OpenAIClient {
    async fn generate(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);

        let raw_messages =
            if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
                messages
            } else {
                &default_messages
            };

        let context_limit = if model.contains("glm-4") || model.contains("GLM-4") {
            202750
        } else if model.contains("gemini") {
            1000000
        } else if model.contains("gpt-oss") || model.contains("gpt-4") {
            128000
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768
        } else {
            32768
        };

        let messages = OpenAIClient::ensure_token_limit(raw_messages, context_limit);

        let full_url = format!("{}{}", self.base_url, self.endpoint_path);
        let auth_header = format!("Bearer {}", key);

        trace!("LLM Request Details:");
        trace!(" URL: {}", full_url);
        trace!(" Authorization: Bearer <{} chars>", key.len());
        trace!(" Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            trace!(" Messages: {} messages", msg_array.len());
        }
        trace!(" API Key First 8 chars: '{}...'", &key.chars().take(8).collect::<String>());
        trace!(" API Key Last 8 chars: '...{}'", &key.chars().rev().take(8).collect::<String>());

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false
        });

        let max_retries = 2;
        let mut result = None;
        for attempt in 0..=max_retries {
            let outcome = async {
                let resp = self.client
                    .post(&full_url)
                    .header("Authorization", &auth_header)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("HTTP error: {}", e))?;
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Ok::<(u16, String), String>((status.as_u16(), text))
            }.await;

            match outcome {
                Ok((status, text)) if status == 200 => {
                    result = Some(text);
                    break;
                }
                Ok((status, error_text)) => {
                    if attempt < max_retries {
                        warn!("LLM generate attempt {} failed (status {}): {}, retrying...", attempt + 1, status, error_text);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    } else {
                        log::error!("LLM generate error after {} retries: {}", max_retries, error_text);
                        return Err(format!("LLM request failed with status: {}: {}", status, error_text).into());
                    }
                }
                Err(e) => {
                    let err_detail = format!("{:#}", e);
                    if attempt < max_retries {
                        warn!("LLM generate attempt {} failed (connection): {}, retrying...", attempt + 1, err_detail);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    } else {
                        log::error!("LLM generate connection error after {} retries: {}", max_retries, err_detail);
                        return Err(format!("LLM error: {}", err_detail).into());
                    }
                }
            }
        }

        let response_text = result.ok_or_else(|| {
            Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "LLM request failed after {} retries", max_retries
            ))
        })?;

        let result_value: Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        let msg = &result_value["choices"][0]["message"];
        let raw_content = msg["content"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| msg["reasoning_content"].as_str())
            .or_else(|| msg["reasoning"].as_str())
            .unwrap_or("");

        let handler = get_handler(model);
        let content = handler.process_content(raw_content);

        Ok(content)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        messages: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
        tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("OpenAIClient::generate_stream ENTERED: model={}, key_len={}, url={}{}",
            model, key.len(), self.base_url, self.endpoint_path);
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);

        let raw_messages =
            if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
                info!("Using provided messages: {:?}", messages);
                messages
            } else {
                &default_messages
            };

        let context_limit = if model.contains("glm-4") || model.contains("GLM-4") {
            202750
        } else if model.contains("gemini") {
            1000000
        } else if model.contains("gpt-oss") || model.contains("gpt-4") {
            128000
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768
        } else {
            32768
        };

        let messages = OpenAIClient::ensure_token_limit(raw_messages, context_limit);

        let estimated_tokens = OpenAIClient::estimate_messages_tokens(&messages);
        if let Err(e) = self.rate_limiter.acquire(estimated_tokens).await {
            log::error!("Rate limit exceeded: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }

        let full_url = format!("{}{}", self.base_url, self.endpoint_path);
        let auth_header = format!("Bearer {}", key);

        trace!("LLM Request Details:");
        trace!(" URL: {}", full_url);
        trace!(" Authorization: Bearer <{} chars>", key.len());
        trace!(" Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            trace!(" Messages: {} messages", msg_array.len());
        }
        if let Some(tools) = tools {
            trace!(" Tools: {} tools provided", tools.len());
        }

        let token_key = if model.contains("gpt-5") {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        let use_stream = !(model.contains("gpt-oss") && (self.base_url.contains("nvidia") || self.base_url.contains("cerebras")));
        if !use_stream {
            info!("Setting stream=false for NVIDIA model: {}", model);
        }
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": use_stream,
            token_key: if self.base_url.contains("groq") { 4096 } else { 65536 },
            "temperature": 1.0,
            "top_p": 1.0
        });

        // Only add chat_template_kwargs for native z.ai/GLM API, not for opencode.ai proxy
        if (model.contains("kimi") || model.contains("glm")) && self.base_url.contains("z.ai") {
            let kwargs = if model.contains("glm") {
                serde_json::json!({"enable_thinking": true, "clear_thinking": false})
            } else {
                serde_json::json!({"thinking": true})
            };
            request_body["chat_template_kwargs"] = kwargs;
            info!("Model factory: enabled thinking mode for {} (chat_template_kwargs)", model);
        }

        if let Some(tools_value) = tools {
            if !tools_value.is_empty() {
                request_body["tools"] = serde_json::json!(tools_value);
                request_body["tool_choice"] = serde_json::json!("auto");
                info!("Added {} tools to LLM request (tool_choice=auto)", tools_value.len());

                if let Some(first_tool) = tools_value.first().and_then(|t| t.get("function")) {
                    info!("First tool: name={:?}", first_tool.get("name"));
                }

                if model.contains("deepseek") {
                    let legacy_functions: Vec<Value> = tools_value
                        .iter()
                        .filter_map(|t| t.get("function").cloned())
                        .collect();
                    if !legacy_functions.is_empty() {
                        request_body["functions"] = serde_json::json!(legacy_functions);
                        trace!("Added {} legacy functions for DeepSeek compatibility", legacy_functions.len());
                    }
                }
            }
        }

        let (chunk_tx, mut chunk_rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, String>>(100000);

        let max_retries = 2;
        let mut stream_started = false;
        info!("LLM request URL: {}, body size: {} bytes, stream={}", full_url, request_body.to_string().len(), use_stream);
        'retry_loop: for attempt in 0..=max_retries {
            let response = match self.client
                .post(&full_url)
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
            {
                Ok(r) => r,
                    Err(e) => {
                        let err_msg = format!("HTTP error: {:#}", e);
                        let _ = chunk_tx.send(Err(err_msg.clone())).await;
                        if attempt < max_retries {
                            warn!("LLM generate_stream attempt {} failed (connection): {}, retrying...", attempt + 1, err_msg);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            continue;
                        }
                        log::error!("LLM generate_stream failed after {} retries: {}", max_retries, err_msg);
                        return Err("LLM stream request failed after retries".into());
                    }
            };

            let status = response.status();
            info!("LLM HTTP response status: {} for model {} (attempt {})", status, model, attempt + 1);
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                warn!("LLM HTTP error body: {}", error_text);
                let _ = chunk_tx.send(Err(format!("Status {}: {}", status, error_text))).await;
                if attempt < max_retries {
                    warn!("LLM generate_stream attempt {} failed (status {}): {}, retrying...", attempt + 1, status, error_text);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                log::error!("LLM generate_stream failed after {} retries (status {}): {}", max_retries, status, error_text);
                return Err(format!("LLM request failed with status: {}: {}", status, error_text).into());
            }

            stream_started = true;

            if !use_stream {
                let body = response.text().await.map_err(|e| format!("Failed: {}", e))?;
                if let Ok(val) = serde_json::from_str::<Value>(&body) {
                    let choice = &val["choices"][0];
                    if let Some(reasoning) = choice["message"]["reasoning_content"].as_str().or(choice["message"]["reasoning"].as_str()) {
                        let _ = tx.send(serde_json::json!({"__reasoning__": reasoning}).to_string()).await;
                    }
                    if let Some(content) = choice["message"]["content"].as_str() {
                        if !content.is_empty() {
                            let _ = tx.send(content.to_string()).await;
                        }
                    }
                    if let Some(tcs) = choice["message"]["tool_calls"].as_array() {
                        for tc in tcs {
                            if let Some(func) = tc.get("function") {
                                let args = func["arguments"].as_str().unwrap_or("{}");
                                let msg = serde_json::json!({"__tool_call__": true, "name": func["name"], "arguments": args});
                                let _ = tx.send(msg.to_string()).await;
                            }
                        }
                    }
                }
                return Ok(());
            }

            let mut stream = response.bytes_stream();
            use futures_util::StreamExt;
            loop {
                match stream.next().await {
                    Some(Ok(bytes)) => {
                        if chunk_tx.send(Ok(bytes.to_vec())).await.is_err() {
                            break 'retry_loop;
                        }
                    }
                    Some(Err(e)) => {
                        let err_msg = format!("Stream read error: {}", e);
                        let _ = chunk_tx.send(Err(err_msg.clone())).await;
                        log::error!("LLM generate_stream stream error: {}", err_msg);
                        return Err(err_msg.into());
                    }
                    None => break,
                }
            }
            break;
        }

        if !stream_started {
            return Err("LLM stream request failed to start".into());
        }
        drop(chunk_tx);

        let handler = get_handler(model);
        let mut stream_state = String::new();
        let mut line_buffer = String::new(); // for SSE line fragmentation across chunks
        let mut first_bytes: Option<String> = None;
        let mut last_bytes = String::new();
        let mut total_size: usize = 0;
        let mut content_sent: usize = 0;
        let mut tool_call_name = String::new();
        let mut tool_call_args = String::new();
        let mut reasoning_from_field = String::new();   // from reasoning_content/reasoning API field
        let mut reasoning_from_content = String::new(); // from <think> tags inside content

        info!("LLM stream starting for model: {}", model);

        let idle_timeout = std::time::Duration::from_secs(60);
        loop {
            let chunk_result = match tokio::time::timeout(idle_timeout, chunk_rx.recv()).await {
                Ok(Some(Ok(chunk))) => chunk,
                Ok(Some(Err(e))) => return Err(format!("Stream error: {}", e).into()),
                Ok(None) => break,
                Err(_) => {
                    log::error!("LLM stream idle timeout after 5s (no data received)");
                    return Err("LLM stream idle timeout after 5s".into());
                }
            };
            let chunk_str = String::from_utf8_lossy(&chunk_result);
            total_size += chunk_result.len();
            trace!("LLM chunk raw: {} bytes", chunk_result.len());
            if first_bytes.is_none() {
                first_bytes = Some(chunk_str.chars().take(100).collect());
            }
            last_bytes = chunk_str.chars().take(100).collect();

            // Accumulate lines across chunks to handle SSE fragmentation
            line_buffer.push_str(&chunk_str);
            let mut pos = 0;
            while let Some(newline) = line_buffer[pos..].find('\n') {
                let end = pos + newline;
                let line = line_buffer[pos..end].trim_end_matches('\r').to_string();
                pos = end + 1;
                if !line.starts_with("data: ") || line.contains("[DONE]") {
                    continue;
                }
                if let Ok(data) = serde_json::from_str::<Value>(&line[6..]) {
                        // Log only when finish_reason or function/tool calls present
                        let fr = &data["choices"][0]["finish_reason"];
                        let has_fr = !fr.is_null();
                        let has_tc = !data["choices"][0]["delta"]["tool_calls"].is_null();
                        let has_fc = !data["choices"][0]["delta"]["function_call"].is_null();
                        if has_fr || has_tc || has_fc {
                            info!("SSE finish_reason={} tool_calls={} function_call={}", fr, has_tc, has_fc);
                        }
                        if let Some(filter_result) = data["choices"][0]["delta"]["content_filter_result"].as_object() {
                            if let Some(error) = filter_result.get("error") {
                                let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
                                let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("no message");
                                log::error!("LLM Content filter error: code={}, message={}", code, message);
                            } else {
                                trace!("LLM Content filter result (no error): {:?}", filter_result);
                            }
                        }

                        // Different models put data in different fields:
                        //   NVIDIA/Nemotron:    content=answer, reasoning=thinking (separate chunks)
                        //   DeepSeek paid:      content=answer, reasoning_content=thinking
                        let content_text = data["choices"][0]["delta"]["content"].as_str()
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty());

                        // Reasoning/thinking from separate API field
                        let reasoning_text = data["choices"][0]["delta"]["reasoning"].as_str()
                            .or_else(|| data["choices"][0]["delta"]["reasoning_content"].as_str())
                            .map(|s| s.to_string());

                        // Always accumulate reasoning from separate API field
                        if let Some(ref reasoning) = reasoning_text {
                            reasoning_from_field.push_str(reasoning);
                        }

                        // Process content through handler — this extracts think-tag reasoning
                        if let Some(ref content) = content_text {
                            if !content.is_empty() {
                                let processed = handler.process_content_streaming(content, &mut stream_state);
                                if !processed.content.is_empty() {
                                    content_sent += processed.content.len();
                                    let _ = tx.send(processed.content).await;
                                }
                                if !processed.reasoning.is_empty() {
                                    reasoning_from_content.push_str(&processed.reasoning);
                                    // Stream reasoning chunks live to the frontend
                                    let reasoning_chunk = serde_json::json!({
                                        "__reasoning__": processed.reasoning.trim()
                                    }).to_string();
                                    let _ = tx.send(reasoning_chunk).await;
                                }
                            }
                        }

                        // Handle legacy function_call format (Groq legacy functions API)
                        if tool_call_name.is_empty() {
                            if let Some(func_call) = data["choices"][0]["delta"]["function_call"].as_object() {
                                if let Some(name) = func_call.get("name").and_then(|n| n.as_str()) {
                                    tool_call_name = name.to_string();
                                    tool_call_args.clear();
                                }
                                if let Some(args) = func_call.get("arguments").and_then(|a| a.as_str()) {
                                    tool_call_args.push_str(args);
                                }
                            }
                        }

                        // Handle modern tool_calls format (OpenAI tools API)
                        // Accumulate across streaming chunks, don't send via tx
                        if let Some(tool_calls) = data["choices"][0]["delta"]["tool_calls"].as_array() {
                            if !tool_calls.is_empty() {
                                info!("SSE delta with tool_calls: {} entries, first: {:?}", tool_calls.len(), tool_calls[0].get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()));
                            }
                            for tool_call in tool_calls {
                                if let Some(func) = tool_call.get("function") {
                                    if tool_call_name.is_empty() {
                                        if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                            tool_call_name = name.to_string();
                                            tool_call_args.clear();
                                        }
                                    }
                                    if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                        tool_call_args.push_str(args);
                    }
                }
            }

                        // Handle legacy function_call format (NVIDIA, some open-source models)
                        if let Some(func_call) = data["choices"][0]["delta"]["function_call"].as_object() {
                            if tool_call_name.is_empty() {
                                if let Some(name) = func_call.get("name").and_then(|n| n.as_str()) {
                                    tool_call_name = name.to_string();
                                    tool_call_args.clear();
                                }
                            }
                            if let Some(args) = func_call.get("arguments").and_then(|a| a.as_str()) {
                                tool_call_args.push_str(args);
                            }
                        }
                    }
                }
            }
            // Keep trailing partial line for next chunk
            line_buffer = line_buffer[pos..].to_string();
        }

        // Combine all reasoning from both sources
        let mut total_reasoning = reasoning_from_field;
        if !reasoning_from_content.is_empty() {
            if !total_reasoning.is_empty() {
                total_reasoning.push('\n');
            }
            total_reasoning.push_str(&reasoning_from_content);
        }

        // Send accumulated reasoning to frontend as a thinking block
        if !total_reasoning.is_empty() {
            let reasoning_msg = serde_json::json!({
                "__reasoning__": total_reasoning.trim()
            }).to_string();
            let _ = tx.send(reasoning_msg).await;
            info!("LLM reasoning sent: {} bytes", total_reasoning.len());
        }

        // After streaming ends, if tool_calls were accumulated, send them to tx
        if !tool_call_name.is_empty() && !tool_call_args.is_empty() {
            let tool_call_msg = serde_json::json!({
                "__tool_call__": true,
                "name": tool_call_name,
                "arguments": tool_call_args
            });
            let _ = tx.send(tool_call_msg.to_string()).await;
            tokio::task::yield_now().await;
            info!("LLM tool_call accumulated: {} with {} bytes args", tool_call_name, tool_call_args.len());
        }

        info!("LLM stream done: size={} bytes, content_sent={}, reasoning={}B, first={:?}, last={}",
            total_size, content_sent, total_reasoning.len(), first_bytes, last_bytes);

        Ok(())
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMProviderType {
    OpenAI,
    Claude,
    AzureClaude,
    AzureGPT5,
    GLM,
    Bedrock,
    Vertex,
}

impl From<&str> for LLMProviderType {
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("claude") || lower.contains("anthropic") {
            if lower.contains("azure") {
                Self::AzureClaude
            } else {
                Self::Claude
            }
        } else if lower.contains("azuregpt5") || lower.contains("gpt5") || (lower.contains("openai.azure.com") && lower.contains("responses")) {
            Self::AzureGPT5
        } else if lower.contains("z.ai") || lower.contains("glm") {
            Self::GLM
        } else if lower.contains("bedrock") {
            Self::Bedrock
        } else if lower.contains("googleapis.com") || lower.contains("vertex") || lower.contains("generativelanguage") {
            Self::Vertex
        } else {
            Self::OpenAI
        }
    }
}

pub fn create_llm_provider(
    provider_type: LLMProviderType,
    base_url: String,
    deployment_name: Option<String>,
    endpoint_path: Option<String>,
) -> Arc<dyn LLMProvider> {
    match provider_type {
        LLMProviderType::OpenAI => {
            info!("Creating OpenAI LLM provider with URL: {}", base_url);
            Arc::new(OpenAIClient::new(
                "empty".to_string(),
                Some(base_url),
                endpoint_path,
            ))
        }
        LLMProviderType::Claude => {
            info!("Creating Claude LLM provider with URL: {}", base_url);
            Arc::new(ClaudeClient::new(base_url, deployment_name))
        }
        LLMProviderType::AzureClaude => {
            let deployment = deployment_name.unwrap_or_else(|| "claude-opus-4-5".to_string());
            info!(
                "Creating Azure Claude LLM provider with URL: {}, deployment: {}",
                base_url, deployment
            );
            Arc::new(ClaudeClient::azure(base_url, deployment))
        }
        LLMProviderType::AzureGPT5 => {
            info!("Creating Azure GPT-5/Responses LLM provider with URL: {}", base_url);
            Arc::new(AzureGPT5Client::new(base_url, endpoint_path))
        }
        LLMProviderType::GLM => {
            info!("Creating GLM/z.ai LLM provider with URL: {}", base_url);
            Arc::new(GLMClient::new(base_url))
        }
        LLMProviderType::Bedrock => {
            info!("Creating Bedrock LLM provider with exact URL: {}", base_url);
            Arc::new(BedrockClient::new(base_url))
        }
        LLMProviderType::Vertex => {
            info!("Creating Vertex/Gemini LLM provider with URL: {}", base_url);
            Arc::new(vertex::VertexClient::new(base_url, endpoint_path))
        }
    }
}

pub fn create_llm_provider_from_url(
    url: &str,
    model: Option<String>,
    endpoint_path: Option<String>,
    explicit_provider: Option<LLMProviderType>,
) -> Arc<dyn LLMProvider> {
    let detected = LLMProviderType::from(url);
    let provider_type = explicit_provider.as_ref().map(|p| *p).unwrap_or(detected);
    info!("LLM provider: explicit={:?}, detected={:?}, URL={}", explicit_provider, detected, url);
    if explicit_provider.is_some() {
        info!("Using explicit LLM provider type: {:?} for URL: {}", provider_type, url);
    }
    create_llm_provider(provider_type, url.to_string(), model, endpoint_path)
}

pub struct DynamicLLMProvider {
    inner: RwLock<Arc<dyn LLMProvider>>,
}

impl std::fmt::Debug for DynamicLLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicLLMProvider").finish()
    }
}

impl DynamicLLMProvider {
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self {
            inner: RwLock::new(provider),
        }
    }

    pub async fn update_provider(&self, new_provider: Arc<dyn LLMProvider>) {
        let mut guard = self.inner.write().await;
        *guard = new_provider;
        info!("LLM provider updated dynamically");
    }

    pub async fn update_from_config(
        &self,
        url: &str,
        model: Option<String>,
        endpoint_path: Option<String>,
        explicit_provider: Option<LLMProviderType>,
    ) {
        let new_provider = create_llm_provider_from_url(url, model, endpoint_path, explicit_provider);
        self.update_provider(new_provider).await;
    }

    async fn get_provider(&self) -> Arc<dyn LLMProvider> {
        self.inner.read().await.clone()
    }
}

#[async_trait]
impl LLMProvider for DynamicLLMProvider {
    async fn generate(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.get_provider()
            .await
            .generate(prompt, config, model, key)
            .await
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
        self.get_provider()
            .await
            .generate_stream(prompt, config, tx, model, key, tools)
            .await
    }

    async fn cancel_job(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.get_provider().await.cancel_job(session_id).await
    }
}
