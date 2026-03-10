use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::llm::LLMProvider;

#[derive(Debug)]
pub struct BedrockClient {
    client: reqwest::Client,
    base_url: String,
}

impl BedrockClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl LLMProvider for BedrockClient {
    async fn generate(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let default_messages = serde_json::json!([{"role": "user", "content": prompt}]);

        let raw_messages = if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
            messages
        } else {
            &default_messages
        };


        let mut messages_limited = Vec::new();
        if let Some(msg_array) = raw_messages.as_array() {
            for msg in msg_array {
                messages_limited.push(msg.clone());
            }
        }
        let formatted_messages = serde_json::Value::Array(messages_limited);

        let auth_header = if key.starts_with("Bearer ") {
            key.to_string()
        } else {
            format!("Bearer {}", key)
        };

        let request_body = serde_json::json!({
            "model": model,
            "messages": formatted_messages,
            "stream": false
        });

        info!("Sending request to Bedrock endpoint: {}", self.base_url);

        let response = self.client
            .post(&self.base_url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Bedrock generate error: {}", error_text);
            return Err(format!("Bedrock API error ({}): {}", status, error_text).into());
        }

        let json: Value = response.json().await?;
        
        if let Some(choices) = json.get("choices") {
            if let Some(first_choice) = choices.get(0) {
                if let Some(message) = first_choice.get("message") {
                    if let Some(content) = message.get("content") {
                        if let Some(content_str) = content.as_str() {
                            return Ok(content_str.to_string());
                        }
                    }
                }
            }
        }

        Err("Failed to parse response from Bedrock".into())
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

        let raw_messages = if messages.is_array() && !messages.as_array().unwrap_or(&vec![]).is_empty() {
            messages
        } else {
            &default_messages
        };

        let mut messages_limited = Vec::new();
        if let Some(msg_array) = raw_messages.as_array() {
            for msg in msg_array {
                messages_limited.push(msg.clone());
            }
        }
        let formatted_messages = serde_json::Value::Array(messages_limited);

        let auth_header = if key.starts_with("Bearer ") {
            key.to_string()
        } else {
            format!("Bearer {}", key)
        };

        let mut request_body = serde_json::json!({
            "model": model,
            "messages": formatted_messages,
            "stream": true
        });

        if let Some(tools_value) = tools {
            if !tools_value.is_empty() {
                request_body["tools"] = serde_json::json!(tools_value);
                info!("Added {} tools to Bedrock request", tools_value.len());
            }
        }

        let stream_url = if self.base_url.ends_with("/invoke") {
            self.base_url.replace("/invoke", "/invoke-with-response-stream")
        } else {
            self.base_url.clone()
        };

        info!("Sending streaming request to Bedrock endpoint: {}", stream_url);

        let response = self.client
            .post(&stream_url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Bedrock generate_stream error: {}", error_text);
            return Err(format!("Bedrock API error ({}): {}", status, error_text).into());
        }

        let mut stream = response.bytes_stream();
        let mut tool_call_buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Ok(text) = std::str::from_utf8(&chunk) {
                        for line in text.split('\n') {
                            let line = line.trim();
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    continue;
                                }

                                if let Ok(json) = serde_json::from_str::<Value>(data) {
                                    if let Some(choices) = json.get("choices") {
                                        if let Some(first_choice) = choices.get(0) {
                                            if let Some(delta) = first_choice.get("delta") {
                                                // Handle standard content streaming
                                                if let Some(content) = delta.get("content") {
                                                    if let Some(content_str) = content.as_str() {
                                                        if !content_str.is_empty() && tx.send(content_str.to_string()).await.is_err() {
                                                            return Ok(());
                                                        }
                                                    }
                                                }
                                                
                                                // Handle tool calls streaming
                                                if let Some(tool_calls) = delta.get("tool_calls") {
                                                    if let Some(calls_array) = tool_calls.as_array() {
                                                        if let Some(first_call) = calls_array.first() {
                                                            if let Some(function) = first_call.get("function") {
                                                                // Stream function JSON representation just like OpenAI does
                                                                if let Some(name) = function.get("name") {
                                                                    if let Some(name_str) = name.as_str() {
                                                                        tool_call_buffer = format!("{{\"name\": \"{}\", \"arguments\": \"", name_str);
                                                                    }
                                                                }
                                                                if let Some(args) = function.get("arguments") {
                                                                    if let Some(args_str) = args.as_str() {
                                                                        tool_call_buffer.push_str(args_str);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Bedrock stream reading error: {}", e);
                    break;
                }
            }
        }
        
        // Finalize tool call JSON parsing
        if !tool_call_buffer.is_empty() {
            tool_call_buffer.push_str("\"}");
            if tx.send(format!("`tool_call`: {}", tool_call_buffer)).await.is_err() {
                return Ok(());
            }
        }

        Ok(())
    }

    async fn cancel_job(&self, _session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
