use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub mod cache;
pub mod claude;
pub mod episodic_memory;
pub mod glm;
pub mod llm_models;
pub mod local;
pub mod rate_limiter;
pub mod smart_router;

pub use claude::ClaudeClient;
pub use glm::GLMClient;
pub use llm_models::get_handler;
pub use rate_limiter::{ApiRateLimiter, RateLimits};

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

        // For z.ai API, use different endpoint path
        let endpoint = if let Some(path) = endpoint_path {
            path
        } else if base.contains("z.ai") || base.contains("/v4") {
            "/chat/completions".to_string()  // z.ai uses /chat/completions, not /v1/chat/completions
        } else {
            "/v1/chat/completions".to_string()
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
            base_url: base,
            endpoint_path: endpoint,
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
            messages.push(serde_json::json!({
                "role": role,
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
        } else if model.contains("gpt-4") {
            128000
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768 // Local llama.cpp server context limit
        } else {
            4096 // Default conservative limit
        };

        let messages = OpenAIClient::ensure_token_limit(raw_messages, context_limit);

        let full_url = format!("{}{}", self.base_url, self.endpoint_path);
        let auth_header = format!("Bearer {}", key);

        // Debug logging to help troubleshoot 401 errors
        info!("LLM Request Details:");
        info!("  URL: {}", full_url);
        info!("  Authorization: Bearer <{} chars>", key.len());
        info!("  Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            info!("  Messages: {} messages", msg_array.len());
        }
        info!("  API Key First 8 chars: '{}...'", &key.chars().take(8).collect::<String>());
        info!("  API Key Last 8 chars: '...{}'", &key.chars().rev().take(8).collect::<String>());

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
        } else if model.contains("gpt-4") {
            128000
        } else if model.contains("gpt-3.5") {
            16385
        } else if model.starts_with("http://localhost:808") || model == "local" {
            768 // Local llama.cpp server context limit
        } else {
            4096 // Default conservative limit
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
        info!("LLM Request Details:");
        info!("  URL: {}", full_url);
        info!("  Authorization: Bearer <{} chars>", key.len());
        info!("  Model: {}", model);
        if let Some(msg_array) = messages.as_array() {
            info!("  Messages: {} messages", msg_array.len());
        }
        if let Some(tools) = tools {
            info!("  Tools: {} tools provided", tools.len());
        }

        // Build the request body - include tools if provided
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true
        });

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
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            for line in chunk_str.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    if let Ok(data) = serde_json::from_str::<Value>(&line[6..]) {
                        if let Some(content) = data["choices"][0]["delta"]["content"].as_str() {
                            let processed = handler.process_content(content);
                            if !processed.is_empty() {
                                let _ = tx.send(processed).await;
                            }
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

pub fn start_llm_services(state: &std::sync::Arc<crate::core::shared::state::AppState>) {
    episodic_memory::start_episodic_memory_scheduler(std::sync::Arc::clone(state));
    info!("LLM services started (episodic memory scheduler)");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LLMProviderType {
    OpenAI,
    Claude,
    AzureClaude,
    GLM,
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
        } else if lower.contains("z.ai") || lower.contains("glm") {
            Self::GLM
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
        LLMProviderType::GLM => {
            info!("Creating GLM/z.ai LLM provider with URL: {}", base_url);
            std::sync::Arc::new(GLMClient::new(base_url))
        }
    }
}

pub fn create_llm_provider_from_url(
    url: &str,
    model: Option<String>,
    endpoint_path: Option<String>,
) -> std::sync::Arc<dyn LLMProvider> {
    let provider_type = LLMProviderType::from(url);
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
    ) {
        let new_provider = create_llm_provider_from_url(url, model, endpoint_path);
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
