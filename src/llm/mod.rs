use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info, trace};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub mod cache;
pub mod claude;
pub mod episodic_memory;
pub mod glm;
pub mod hallucination_detector;
pub mod llm_models;
pub mod local;
pub mod rate_limiter;
pub mod smart_router;
pub mod vertex;
pub mod bedrock;

pub use claude::ClaudeClient;
pub use glm::GLMClient;
pub use llm_models::get_handler;
pub use rate_limiter::{ApiRateLimiter, RateLimits};
pub use vertex::VertexTokenManager;
pub use bedrock::BedrockClient;

#[async_trait]
pub trait LLMProvider: Send + Sync {
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
        Self {
            client: reqwest::Client::new(),
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

        // Check rate limits before making the request
        if let Err(e) = self.rate_limiter.acquire(4096).await {
            error!("Rate limit exceeded: {}", e);
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
            error!("AzureGPT5 generate error: {}", error_text);
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
    /// Estimates token count for a text string (roughly 4 characters per token for English)
    fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: ~4 characters per token for English text
        // This is a heuristic and may not be accurate for all languages
        text.len().div_ceil(4)
    }

    /// Estimates total tokens for a messages array
    fn estimate_messages_tokens(messages: &Value) -> usize {
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

    /// Truncates messages to fit within the max_tokens limit
    /// Keeps system messages and the most recent user/assistant messages
    fn truncate_messages(messages: &Value, max_tokens: usize) -> Value {
        let mut result = Vec::new();
        let mut token_count = 0;

        if let Some(msg_array) = messages.as_array() {
            // First pass: keep all system messages
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

            // Second pass: add user/assistant messages from newest to oldest
            let mut recent_messages: Vec<&Value> = msg_array
                .iter()
                .filter(|msg| msg.get("role").and_then(|r| r.as_str()) != Some("system"))
                .collect();

            // Reverse to get newest first
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

            // Reverse back to chronological order for non-system messages
            // But keep system messages at the beginning
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

    /// Ensures messages fit within model's context limit
    fn ensure_token_limit(messages: &Value, model_context_limit: usize) -> Value {
        let estimated_tokens = Self::estimate_messages_tokens(messages);

        // Use 90% of context limit to leave room for response
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

        // Detect if the base URL already contains a completions path
        let has_v1_path = trimmed_base.contains("/v1/chat/completions");
        let has_chat_path = !has_v1_path && trimmed_base.contains("/chat/completions");

        let endpoint = if let Some(path) = endpoint_path {
            path
        } else if has_v1_path || (has_chat_path && !trimmed_base.contains("z.ai")) {
            // Path already in base_url, use empty endpoint
            "".to_string()
        } else if trimmed_base.contains("z.ai") || trimmed_base.contains("/v4") {
            "/chat/completions".to_string() // z.ai uses /chat/completions, not /v1/chat/completions
        } else {
            "/v1/chat/completions".to_string()
        };

        // Final normalization: if the base URL already ends with the endpoint, empty the endpoint
        let (final_base, final_endpoint) = if !endpoint.is_empty() && trimmed_base.ends_with(&endpoint) {
            (trimmed_base, "".to_string())
        } else {
            (trimmed_base, endpoint)
        };

        // Detect API provider and set appropriate rate limits
        let rate_limiter = if base.contains("groq.com") {
            ApiRateLimiter::new(RateLimits::groq_free_tier())
        } else if base.contains("openai.com") {
            ApiRateLimiter::new(RateLimits::openai_free_tier())
        } else {
            // Default to unlimited for other providers (local models, etc.)
            ApiRateLimiter::unlimited()
        };

        Self {
            client: reqwest::Client::new(),
            base_url: final_base,
            endpoint_path: final_endpoint,
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    /// Sanitizes a string by removing invalid UTF-8 surrogate characters
    /// that cannot be encoded in valid UTF-8 (surrogates are only valid in UTF-16)
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
            // Filter out internal roles not valid for OpenAI API
            let api_role = match role.as_str() {
                "user" | "assistant" | "system" | "developer" | "tool" => role.as_str(),
                // Convert internal roles to valid API roles
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

        // Get the messages to use
        let raw_messages =
            if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
                messages
            } else {
                &default_messages
            };

        // Ensure messages fit within model's context limit
        // GLM-4.7 has 202750 tokens, other models vary
        let context_limit = if model.contains("glm-4") || model.contains("GLM-4") {
            202750
        } else if model.contains("gemini") {
            1000000 // Google Gemini models have 1M+ token context
        } else if model.contains("gpt-oss") || model.contains("gpt-4") {
            128000 // Cerebras gpt-oss models and GPT-4 variants
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768 // Local llama.cpp server context limit
        } else {
            32768 // Default conservative limit for modern models
        };

        let messages = OpenAIClient::ensure_token_limit(raw_messages, context_limit);

        let full_url = format!("{}{}", self.base_url, self.endpoint_path);
        let auth_header = format!("Bearer {}", key);

        // Debug logging to help troubleshoot 401 errors
        trace!("LLM Request Details:");
        trace!("  URL: {}", full_url);
        trace!("  Authorization: Bearer <{} chars>", key.len());
        trace!("  Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            trace!("  Messages: {} messages", msg_array.len());
        }
        trace!("  API Key First 8 chars: '{}...'", &key.chars().take(8).collect::<String>());
        trace!("  API Key Last 8 chars: '...{}'", &key.chars().rev().take(8).collect::<String>());

        // Build the request body (no tools for non-streaming generate)
        let response = self
            .client
            .post(&full_url)
            .header("Authorization", &auth_header)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": false
            }))
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            let error_text = response.text().await.unwrap_or_default();
            error!("LLM generate error: {}", error_text);
            return Err(format!("LLM request failed with status: {}", status).into());
        }

        let result: Value = response.json().await?;
        let raw_content = result["choices"][0]["message"]["content"]
            .as_str()
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
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);

        // Get the messages to use
        let raw_messages =
            if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
                info!("Using provided messages: {:?}", messages);
                messages
            } else {
                &default_messages
            };

        // Ensure messages fit within model's context limit
        // GLM-4.7 has 202750 tokens, other models vary
        let context_limit = if model.contains("glm-4") || model.contains("GLM-4") {
            202750
        } else if model.contains("gemini") {
            1000000 // Google Gemini models have 1M+ token context
        } else if model.contains("gpt-oss") || model.contains("gpt-4") {
            128000 // Cerebras gpt-oss models and GPT-4 variants
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768 // Local llama.cpp server context limit
        } else {
            32768 // Default conservative limit for modern models
        };

        let messages = OpenAIClient::ensure_token_limit(raw_messages, context_limit);

        // Check rate limits before making the request
        let estimated_tokens = OpenAIClient::estimate_messages_tokens(&messages);
        if let Err(e) = self.rate_limiter.acquire(estimated_tokens).await {
            error!("Rate limit exceeded: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
        }

        let full_url = format!("{}{}", self.base_url, self.endpoint_path);
        let auth_header = format!("Bearer {}", key);

        // Debug logging to help troubleshoot 401 errors
        trace!("LLM Request Details:");
        trace!("  URL: {}", full_url);
        trace!("  Authorization: Bearer <{} chars>", key.len());
        trace!("  Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            trace!("  Messages: {} messages", msg_array.len());
        }
        if let Some(tools) = tools {
            trace!("  Tools: {} tools provided", tools.len());
        }

        // Build the request body - include tools if provided
        // GPT-5 models use max_completion_tokens instead of max_tokens
        let token_key = if model.contains("gpt-5") {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            token_key: 16384,
            "temperature": 1.0,
            "top_p": 1.0
        });

        // GLM 4.7 / Kimi K2.5 factory: enable thinking mode via chat_template_kwargs
        if model.contains("kimi") || model.contains("glm") {
            let kwargs = if model.contains("glm") {
                serde_json::json!({"enable_thinking": true, "clear_thinking": false})
            } else {
                serde_json::json!({"thinking": true})
            };
            request_body["chat_template_kwargs"] = kwargs;
            info!("Model factory: enabled thinking mode for {} (chat_template_kwargs)", model);
        }

        // Add tools to the request if provided
        if let Some(tools_value) = tools {
            if !tools_value.is_empty() {
                request_body["tools"] = serde_json::json!(tools_value);
                info!("Added {} tools to LLM request", tools_value.len());
            }
        }

        let response = self
            .client
            .post(&full_url)
            .header("Authorization", &auth_header)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            let error_text = response.text().await.unwrap_or_default();
            error!("LLM generate_stream error: {}", error_text);
            return Err(format!("LLM request failed with status: {}", status).into());
        }

    let handler = get_handler(model);
    let mut stream_state = String::new();
    let mut stream = response.bytes_stream();
    let mut first_bytes: Option<String> = None;
    let mut last_bytes: String = String::new();
    let mut total_size: usize = 0;
    let mut content_sent: usize = 0;

    info!("LLM stream starting for model: {}", model);

    let mut in_reasoning = false;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        total_size += chunk.len();
        let chunk_str = String::from_utf8_lossy(&chunk);
        trace!("LLM chunk raw: {} bytes", chunk.len());
        if first_bytes.is_none() {
            first_bytes = Some(chunk_str.chars().take(100).collect());
        }
        last_bytes = chunk_str.chars().take(100).collect();
        for line in chunk_str.lines() {
            if line.starts_with("data: ") && !line.contains("[DONE]") {
              if let Ok(data) = serde_json::from_str::<Value>(&line[6..]) {
                // Check for content filter errors
                if let Some(filter_result) = data["choices"][0]["delta"]["content_filter_result"].as_object() {
                  if let Some(error) = filter_result.get("error") {
                    let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
                    let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("no message");
                    error!("LLM Content filter error: code={}, message={}", code, message);
                  } else {
                    trace!("LLM Content filter result (no error): {:?}", filter_result);
                  }
                }

                if let Some(reasoning) = data["choices"][0]["delta"]["reasoning_content"].as_str() {
                  if !reasoning.is_empty() {
                    if !in_reasoning {
                      in_reasoning = true;
                    }
                    let thinking_msg = serde_json::json!({
                      "type": "thinking",
                      "content": reasoning
                    }).to_string();
                    let _ = tx.send(thinking_msg).await;
                  }
                }

                if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                  if !content.is_empty() {
                    if in_reasoning {
                      in_reasoning = false;
                      let clear_msg = serde_json::json!({"type": "thinking_clear"}).to_string();
                      let _ = tx.send(clear_msg).await;
                    }
                    let processed = handler.process_content_streaming(content, &mut stream_state);
                    if !processed.is_empty() {
                      content_sent += processed.len();
                      let _ = tx.send(processed).await;
                    }
                  }
                }

                if let Some(tool_calls) = data["choices"][0]["delta"]["tool_calls"].as_array() {
                  for tool_call in tool_calls {
                    if let Some(func) = tool_call.get("function") {
                      if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                        let _ = tx.send(args.to_string()).await;
                      }
                    }
                  }
                }
              }
            }
        }
    }

    info!("LLM stream done: size={} bytes, content_sent={}, first={:?}, last={}", 
          total_size, content_sent, first_bytes, last_bytes);

    Ok(())
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

pub fn start_llm_services(state: &std::sync::Arc<crate::core::shared::state::AppState>) {
    episodic_memory::start_episodic_memory_scheduler(std::sync::Arc::clone(state));
    info!("LLM services started (episodic memory scheduler)");
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
        } else if lower.contains("azuregpt5") || lower.contains("gpt5") {
            Self::AzureGPT5
        } else if lower.contains("openai.azure.com") && lower.contains("responses") {
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
) -> std::sync::Arc<dyn LLMProvider> {
    match provider_type {
        LLMProviderType::OpenAI => {
            info!("Creating OpenAI LLM provider with URL: {}", base_url);
            std::sync::Arc::new(OpenAIClient::new(
                "empty".to_string(),
                Some(base_url),
                endpoint_path,
            ))
        }
        LLMProviderType::Claude => {
            info!("Creating Claude LLM provider with URL: {}", base_url);
            std::sync::Arc::new(ClaudeClient::new(base_url, deployment_name))
        }
        LLMProviderType::AzureClaude => {
            let deployment = deployment_name.unwrap_or_else(|| "claude-opus-4-5".to_string());
            info!(
                "Creating Azure Claude LLM provider with URL: {}, deployment: {}",
                base_url, deployment
            );
            std::sync::Arc::new(ClaudeClient::azure(base_url, deployment))
        }
        LLMProviderType::AzureGPT5 => {
            info!("Creating Azure GPT-5/Responses LLM provider with URL: {}", base_url);
            std::sync::Arc::new(AzureGPT5Client::new(base_url, endpoint_path))
        }
        LLMProviderType::GLM => {
            info!("Creating GLM/z.ai LLM provider with URL: {}", base_url);
            std::sync::Arc::new(GLMClient::new(base_url))
        }
        LLMProviderType::Bedrock => {
            info!("Creating Bedrock LLM provider with exact URL: {}", base_url);
            std::sync::Arc::new(BedrockClient::new(base_url))
        }
        LLMProviderType::Vertex => {
            info!("Creating Vertex/Gemini LLM provider with URL: {}", base_url);
            // Re-export the struct if we haven't already
            std::sync::Arc::new(crate::llm::vertex::VertexClient::new(base_url, endpoint_path))
        }
    }
}

/// Create LLM provider from URL with optional explicit provider type override.
/// If explicit_provider is Some, it takes precedence over URL-based detection.
pub fn create_llm_provider_from_url(
    url: &str,
    model: Option<String>,
    endpoint_path: Option<String>,
    explicit_provider: Option<LLMProviderType>,
) -> std::sync::Arc<dyn LLMProvider> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ToolCall {
        pub id: String,
        #[serde(rename = "type")]
        pub r#type: String,
        pub function: ToolFunction,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ToolFunction {
        pub name: String,
        pub arguments: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ChatMessage {
        role: String,
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ChatCompletionResponse {
        id: String,
        object: String,
        created: i64,
        model: String,
        choices: Vec<ChatChoice>,
        usage: Usage,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ChatChoice {
        index: i32,
        message: ChatMessage,
        finish_reason: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Usage {
        #[serde(rename = "prompt_tokens")]
        prompt: i32,
        #[serde(rename = "completion_tokens")]
        completion: i32,
        #[serde(rename = "total_tokens")]
        total: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ErrorResponse {
        error: ErrorDetail,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ErrorDetail {
        message: String,
        #[serde(rename = "type")]
        r#type: String,
        code: String,
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "get_weather".to_string(),
                arguments: r#"{"location": "NYC"}"#.to_string(),
            },
        };

        let json = serde_json::to_string(&tool_call).unwrap();
        assert!(json.contains("get_weather"));
        assert!(json.contains("call_123"));
    }

    #[test]
    fn test_chat_completion_response_serialization() {
        let response = ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1_234_567_890,
            model: "gpt-4".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: Some("Hello!".to_string()),
                    tool_calls: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt: 10,
                completion: 5,
                total: 15,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("chat.completion"));
        assert!(json.contains("Hello!"));
        assert!(json.contains("gpt-4"));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: ErrorDetail {
                message: "Test error".to_string(),
                r#type: "test_error".to_string(),
                code: "test_code".to_string(),
            },
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Test error"));
        assert!(json.contains("test_code"));
    }

    #[test]
    fn test_build_messages_empty() {
        let messages = OpenAIClient::build_messages("", "", &[]);
        assert!(messages.is_array());
        assert!(messages.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_build_messages_with_system_prompt() {
        let messages = OpenAIClient::build_messages("You are a helpful assistant.", "", &[]);
        let arr = messages.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["role"], "system");
        assert_eq!(arr[0]["content"], "You are a helpful assistant.");
    }

    #[test]
    fn test_build_messages_with_context() {
        let messages = OpenAIClient::build_messages("System prompt", "Context data", &[]);
        let arr = messages.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["content"], "System prompt");
        assert_eq!(arr[1]["content"], "Context data");
    }

    #[test]
    fn test_build_messages_with_history() {
        let history = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];
        let messages = OpenAIClient::build_messages("", "", &history);
        let arr = messages.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["role"], "user");
        assert_eq!(arr[0]["content"], "Hello");
        assert_eq!(arr[1]["role"], "assistant");
        assert_eq!(arr[1]["content"], "Hi there!");
    }

    #[test]
    fn test_build_messages_full() {
        let history = vec![("user".to_string(), "What is the weather?".to_string())];
        let messages = OpenAIClient::build_messages(
            "You are a weather bot.",
            "Current location: NYC",
            &history,
        );
        let arr = messages.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["role"], "system");
        assert_eq!(arr[1]["role"], "system");
        assert_eq!(arr[2]["role"], "user");
    }

    #[test]
    fn test_openai_client_new_default_url() {
        let client = OpenAIClient::new("test_key".to_string(), None, None);
        assert_eq!(client.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_openai_client_new_custom_url() {
        let client = OpenAIClient::new(
            "test_key".to_string(),
            Some("http://localhost:9000".to_string()),
            None,
        );
        assert_eq!(client.base_url, "http://localhost:9000");
    }

    #[test]
    fn test_chat_message_with_tool_calls() {
        let message = ChatMessage {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(vec![ToolCall {
                id: "call_1".to_string(),
                r#type: "function".to_string(),
                function: ToolFunction {
                    name: "search".to_string(),
                    arguments: r#"{"query": "test"}"#.to_string(),
                },
            }]),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("tool_calls"));
        assert!(json.contains("search"));
    }

    #[test]
    fn test_usage_calculation() {
        let usage = Usage {
            prompt: 100,
            completion: 50,
            total: 150,
        };
        assert_eq!(usage.prompt + usage.completion, usage.total);
    }

    #[test]
    fn test_chat_choice_finish_reasons() {
        let stop_choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: Some("Done".to_string()),
                tool_calls: None,
            },
            finish_reason: "stop".to_string(),
        };
        assert_eq!(stop_choice.finish_reason, "stop");

        let tool_choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![]),
            },
            finish_reason: "tool_calls".to_string(),
        };
        assert_eq!(tool_choice.finish_reason, "tool_calls");
    }
}
// Cache test 1776459528
// Force rebuild 1776460876
// Trigger 1776462135
// Force 1776462809
