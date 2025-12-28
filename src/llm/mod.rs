use async_trait::async_trait;
use futures::StreamExt;
use log::{info, trace};
use serde_json::Value;
use tokio::sync::mpsc;

pub mod cache;
pub mod episodic_memory;
pub mod llm_models;
pub mod local;
pub mod observability;

pub use llm_models::get_handler;

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
}

impl OpenAIClient {
    pub fn new(_api_key: String, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
        }
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
                "content": system_prompt
            }));
        }
        if !context_data.is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": context_data
            }));
        }
        for (role, content) in history {
            messages.push(serde_json::json!({
                "role": role,
                "content": content
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
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", key))
            .json(&serde_json::json!({
                "model": model,
                "messages": if messages.is_array() && !messages.as_array().unwrap().is_empty() {
                    messages
                } else {
                    &default_messages
                }
            }))
            .send()
            .await?;

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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", key))
            .json(&serde_json::json!({
                "model": model,
                "messages": if messages.is_array() && !messages.as_array().unwrap().is_empty() {
                    info!("Using provided messages: {:?}", messages);
                    messages
                } else {
                    &default_messages
                },
                "stream": true
            }))
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            let error_text = response.text().await.unwrap_or_default();
            trace!("LLM generate_stream error: {}", error_text);
            return Err(format!("LLM request failed with status: {}", status).into());
        }

        let handler = get_handler(model);
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
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

pub fn start_llm_services(state: &std::sync::Arc<crate::shared::state::AppState>) {
    episodic_memory::start_episodic_memory_scheduler(std::sync::Arc::clone(state));
    info!("LLM services started (episodic memory scheduler)");
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
        let client = OpenAIClient::new("test_key".to_string(), None);
        assert_eq!(client.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_openai_client_new_custom_url() {
        let client = OpenAIClient::new(
            "test_key".to_string(),
            Some("http://localhost:8080".to_string()),
        );
        assert_eq!(client.base_url, "http://localhost:8080");
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
