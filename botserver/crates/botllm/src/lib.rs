pub mod bedrock;
pub mod claude;
pub mod glm;
pub mod hallucination_detector;
pub mod kimi;
pub mod llm_models;
pub mod rate_limiter;
pub mod vertex;
pub mod cache;
pub mod episodic_memory;
pub mod local;
pub mod smart_router;
pub mod observability;
pub mod pipeline;

pub use rate_limiter::{ApiRateLimiter, RateLimits};
pub use hallucination_detector::HallucinationDetector;
pub use llm_models::get_handler;
pub use claude::ClaudeClient;
pub use glm::GLMClient;
pub use vertex::VertexTokenManager;
pub use bedrock::BedrockClient;
pub use pipeline::{PipelineConfig, LlmPipeline, MessageBuilder, KbContextManager, PromptManager};

use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info, trace};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

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

        Self {
            client: reqwest::Client::new(),
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
            error!("Rate limit exceeded: {}", e);
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
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            token_key: 16384,
            "temperature": 1.0,
            "top_p": 1.0
        });

        if model.contains("kimi") || model.contains("glm") {
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
        let mut last_bytes = String::new();
        let mut total_size: usize = 0;
        let mut content_sent: usize = 0;

        info!("LLM stream starting for model: {}", model);

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
                        if let Some(filter_result) = data["choices"][0]["delta"]["content_filter_result"].as_object() {
                            if let Some(error) = filter_result.get("error") {
                                let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
                                let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("no message");
                                error!("LLM Content filter error: code={}, message={}", code, message);
                            } else {
                                trace!("LLM Content filter result (no error): {:?}", filter_result);
                            }
                        }

                        let reasoning_text = data["choices"][0]["delta"]["reasoning_content"].as_str().map(|s| s.to_string());
                        let content_text = data["choices"][0]["delta"]["content"].as_str().map(|s| s.to_string());

                        // Send reasoning_content only if there's no content delta (thinking-only chunks)
                        if let Some(ref reasoning) = reasoning_text {
                            if !reasoning.is_empty() && content_text.as_ref().map_or(true, |c| c.is_empty()) {
                                let processed = handler.process_content_streaming(reasoning, &mut stream_state);
                                if !processed.is_empty() {
                                    content_sent += processed.len();
                                    let _ = tx.send(processed).await;
                                }
                            }
                        }

                        if let Some(ref content) = content_text {
                            if !content.is_empty() {
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
