use async_trait::async_trait;
use futures::StreamExt;
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use super::{llm_models::get_handler, LLMProvider};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStreamDelta {
    #[serde(rename = "type")]
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

#[derive(Debug)]
pub struct ClaudeClient {
    client: reqwest::Client,
    base_url: String,
    deployment_name: String,
    is_azure: bool,
}

impl ClaudeClient {
    pub fn new(base_url: String, deployment_name: Option<String>) -> Self {
        let is_azure = base_url.contains("azure.com") || base_url.contains("openai.azure.com");

        Self {
            client: reqwest::Client::new(),
            base_url,
            deployment_name: deployment_name.unwrap_or_else(|| "claude-opus-4-5".to_string()),
            is_azure,
        }
    }

    pub fn azure(endpoint: String, deployment_name: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: endpoint,
            deployment_name,
            is_azure: true,
        }
    }

    fn build_url(&self) -> String {
        if self.is_azure {
            // Azure Claude exposes Anthropic API directly at /v1/messages
            format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
        }
    }

    fn build_headers(&self, api_key: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        // Both Azure Claude and direct Anthropic use the same headers
        // Azure Claude proxies the Anthropic API format
        if let Ok(val) = api_key.parse() {
            headers.insert("x-api-key", val);
        }
        if let Ok(val) = "2023-06-01".parse() {
            headers.insert("anthropic-version", val);
        }

        if let Ok(val) = "application/json".parse() {
            headers.insert(reqwest::header::CONTENT_TYPE, val);
        }

        headers
    }

    pub fn build_messages(
        system_prompt: &str,
        context_data: &str,
        history: &[(String, String)],
    ) -> (Option<String>, Vec<ClaudeMessage>) {
        let mut system_parts = Vec::new();

        if !system_prompt.is_empty() {
            system_parts.push(system_prompt.to_string());
        }
        if !context_data.is_empty() {
            system_parts.push(context_data.to_string());
        }

        let system = if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n\n"))
        };

        let messages: Vec<ClaudeMessage> = history
            .iter()
            .map(|(role, content)| ClaudeMessage {
                role: role.clone(),
                content: content.clone(),
            })
            .collect();

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
        let url = self.build_url();
        let headers = self.build_headers(key);

        let model_name = if model.is_empty() {
            &self.deployment_name
        } else {
            model
        };

        let empty_vec = vec![];
        let claude_messages: Vec<ClaudeMessage> = if messages.is_array() {
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
                        if role == "system" {
                            None
                        } else {
                            Some(ClaudeMessage {
                                role: role.to_string(),
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

        let system_prompt: Option<String> = if messages.is_array() {
            messages
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter(|m| m["role"].as_str() == Some("system"))
                .map(|m| m["content"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
                .join("\n\n")
                .into()
        } else {
            None
        };

        let system = system_prompt.filter(|s| !s.is_empty());

        let request = ClaudeRequest {
            model: model_name.to_string(),
            max_tokens: 4096,
            messages: claude_messages,
            system,
            stream: None,
        };

        info!("Claude request to {}: model={}", url, model_name);
        trace!("Claude request body: {:?}", serde_json::to_string(&request));

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            warn!("Claude API error ({}): {}", status, error_text);
            return Err(format!("Claude API error ({}): {}", status, error_text).into());
        }

        let result: ClaudeResponse = response.json().await?;
        let raw_content = self.extract_text_from_response(&result);

        let handler = get_handler(model_name);
        let content = handler.process_content(&raw_content);

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
        let url = self.build_url();
        let headers = self.build_headers(key);

        let model_name = if model.is_empty() {
            &self.deployment_name
        } else {
            model
        };

        let empty_vec = vec![];
        let claude_messages: Vec<ClaudeMessage> = if messages.is_array() {
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
                        if role == "system" {
                            None
                        } else {
                            Some(ClaudeMessage {
                                role: role.to_string(),
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

        let system_prompt: Option<String> = if messages.is_array() {
            messages
                .as_array()
                .unwrap_or(&empty_vec)
                .iter()
                .filter(|m| m["role"].as_str() == Some("system"))
                .map(|m| m["content"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
                .join("\n\n")
                .into()
        } else {
            None
        };

        let system = system_prompt.filter(|s| !s.is_empty());

        let request = ClaudeRequest {
            model: model_name.to_string(),
            max_tokens: 4096,
            messages: claude_messages,
            system,
            stream: Some(true),
        };

        info!("Claude streaming request to {}: model={}", url, model_name);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            warn!("Claude streaming API error ({}): {}", status, error_text);
            return Err(format!("Claude streaming API error ({}): {}", status, error_text).into());
        }

        let handler = get_handler(model_name);
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                let line = line.trim();

                if line.starts_with("data: ") {
                    let data = &line[6..];

                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(data) {
                        if event.event_type == "content_block_delta" {
                            if let Some(delta) = event.delta {
                                if delta.delta_type == "text_delta" && !delta.text.is_empty() {
                                    let processed = handler.process_content(&delta.text);
                                    if !processed.is_empty() {
                                        let _ = tx.send(processed).await;
                                    }
                                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_client_new() {
        let client = ClaudeClient::new(
            "https://api.anthropic.com".to_string(),
            Some("claude-3-opus".to_string()),
        );
        assert!(!client.is_azure);
        assert_eq!(client.deployment_name, "claude-3-opus");
    }

    #[test]
    fn test_claude_client_azure() {
        let client = ClaudeClient::azure(
            "https://myendpoint.openai.azure.com/anthropic".to_string(),
            "claude-opus-4-5".to_string(),
        );
        assert!(client.is_azure);
        assert_eq!(client.deployment_name, "claude-opus-4-5");
    }

    #[test]
    fn test_build_url_azure() {
        let client = ClaudeClient::azure(
            "https://myendpoint.openai.azure.com/anthropic".to_string(),
            "claude-opus-4-5".to_string(),
        );
        let url = client.build_url();
        // Azure Claude uses Anthropic API format directly
        assert!(url.contains("/v1/messages"));
    }

    #[test]
    fn test_build_url_anthropic() {
        let client = ClaudeClient::new(
            "https://api.anthropic.com".to_string(),
            None,
        );
        let url = client.build_url();
        assert_eq!(url, "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_build_messages_empty() {
        let (system, messages) = ClaudeClient::build_messages("", "", &[]);
        assert!(system.is_none());
        assert!(messages.is_empty());
    }

    #[test]
    fn test_build_messages_with_system() {
        let (system, messages) = ClaudeClient::build_messages(
            "You are a helpful assistant.",
            "",
            &[],
        );
        assert_eq!(system, Some("You are a helpful assistant.".to_string()));
        assert!(messages.is_empty());
    }

    #[test]
    fn test_build_messages_with_history() {
        let history = vec![
            ("user".to_string(), "Hello".to_string()),
            ("assistant".to_string(), "Hi there!".to_string()),
        ];
        let (system, messages) = ClaudeClient::build_messages("", "", &history);
        assert!(system.is_none());
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_build_messages_full() {
        let history = vec![
            ("user".to_string(), "What is 2+2?".to_string()),
        ];
        let (system, messages) = ClaudeClient::build_messages(
            "You are a math tutor.",
            "Focus on step-by-step explanations.",
            &history,
        );
        assert!(system.is_some());
        assert!(system.unwrap().contains("math tutor"));
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_claude_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-3-opus".to_string(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: Some("Be helpful".to_string()),
            stream: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-3-opus"));
        assert!(json.contains("max_tokens"));
        assert!(json.contains("Be helpful"));
    }

    #[test]
    fn test_claude_response_deserialization() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-3-opus",
            "stop_reason": "end_turn"
        }"#;

        let response: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "msg_123");
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, "Hello!");
    }
}
