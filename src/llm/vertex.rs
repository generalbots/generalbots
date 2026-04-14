//! Google Vertex AI and Gemini Native API Integration
//! Support for both OpenAI-compatible endpoints and native Google AI Studio / Vertex AI endpoints.

use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    token_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: std::time::Instant,
}

/// Manages OAuth2 access tokens for Google Vertex AI
#[derive(Debug, Clone)]
pub struct VertexTokenManager {
    credentials_path: String,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl VertexTokenManager {
    pub fn new(credentials_path: &str) -> Self {
        Self {
            credentials_path: credentials_path.to_string(),
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a valid access token, refreshing if necessary
    pub async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check cached token
        {
            let cache = self.cached_token.read().await;
            if let Some(ref token) = *cache {
                // Return cached token if it expires in more than 60 seconds
                if token.expires_at > std::time::Instant::now() + std::time::Duration::from_secs(60) {
                    trace!("Using cached Vertex AI access token");
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Generate new token
        info!("Generating new Vertex AI access token from service account");
        let token = self.generate_token().await?;

        // Cache it
        {
            let mut cache = self.cached_token.write().await;
            *cache = Some(token.clone());
        }

        Ok(token.access_token)
    }

    async fn generate_token(&self) -> Result<CachedToken, Box<dyn std::error::Error + Send + Sync>> {
        let content = if self.credentials_path.trim().starts_with('{') {
            self.credentials_path.clone()
        } else {
            // Expand ~ to home directory
            let expanded_path = if self.credentials_path.starts_with('~') {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
                self.credentials_path.replacen('~', &home, 1)
            } else {
                self.credentials_path.clone()
            };

            tokio::fs::read_to_string(&expanded_path)
                .await
                .map_err(|e| format!("Failed to read service account key from '{}': {}", expanded_path, e))?
        };

        let sa_key: ServiceAccountKey = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse service account JSON: {}", e))?;

        let token_uri = sa_key.token_uri.as_deref().unwrap_or(GOOGLE_TOKEN_URL);

        // Create JWT assertion
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("System time error: {}", e))?
            .as_secs();

        let jwt_claims = JwtClaims {
            iss: sa_key.client_email.clone(),
            scope: GOOGLE_SCOPE.to_string(),
            aud: token_uri.to_string(),
            iat: now,
            exp: now + 3600,
        };

        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(sa_key.private_key.as_bytes())
            .map_err(|e| format!("Failed to parse RSA private key: {}", e))?;

        let jwt = jsonwebtoken::encode(&header, &jwt_claims, &encoding_key)
            .map_err(|e| format!("Failed to encode JWT: {}", e))?;

        // Exchange JWT for access token
        let client = reqwest::Client::new();
        let response = client
            .post(token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to request token: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Google OAuth2 token request failed: {} - {}", status, body);
            return Err(format!("Token request failed with status {}: {}", status, body).into());
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        info!("Successfully obtained Vertex AI access token (expires in {}s)", token_response.expires_in);

        Ok(CachedToken {
            access_token: token_response.access_token,
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(token_response.expires_in),
        })
    }
}

#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

/// Builds the Vertex AI OpenAI-compatible endpoint URL
pub fn build_vertex_url(project: &str, location: &str) -> String {
    format!(
        "https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/endpoints/openapi"
    )
}

/// Check if a URL is a Vertex AI endpoint
pub fn is_vertex_ai_url(url: &str) -> bool {
    url.contains("aiplatform.googleapis.com") || url.contains("generativelanguage.googleapis.com")
}

use crate::llm::LLMProvider;
use serde_json::Value;
use tokio::sync::mpsc;
use async_trait::async_trait;
use futures::StreamExt;

#[derive(Debug)]
pub struct VertexClient {
    client: reqwest::Client,
    base_url: String,
    endpoint_path: String,
    token_manager: Arc<tokio::sync::RwLock<Option<Arc<VertexTokenManager>>>>,
}

impl VertexClient {
    pub fn new(base_url: String, endpoint_path: Option<String>) -> Self {
        let endpoint = endpoint_path.unwrap_or_else(|| {
            if base_url.contains("generativelanguage.googleapis.com") {
                // Default to OpenAI compatibility if possible, but generate_stream will auto-detect
                "/v1beta/openai/chat/completions".to_string()
            } else if base_url.contains("aiplatform.googleapis.com") && !base_url.contains("openapi") {
                // If it's a naked aiplatform URL, it might be missing the project/location path
                // We'll leave it empty and let the caller provide a full URL if needed
                "".to_string()
            } else {
                "".to_string()
            }
        });

        Self {
            client: reqwest::Client::new(),
            base_url,
            endpoint_path: endpoint,
            token_manager: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    async fn get_auth_header(&self, key: &str) -> (&'static str, String) {
        // If the key is a path or JSON (starts with {), it's a Service Account
        if key.starts_with('{') || key.starts_with('~') || key.starts_with('/') || key.ends_with(".json") {
            let mut manager_opt = self.token_manager.write().await;
            if manager_opt.is_none() {
                *manager_opt = Some(Arc::new(VertexTokenManager::new(key)));
            }
            if let Some(manager) = manager_opt.as_ref() {
                match manager.get_access_token().await {
                    Ok(token) => return ("Authorization", format!("Bearer {}", token)),
                    Err(e) => error!("Failed to get Vertex OAuth token: {}", e),
                }
            }
        }
        
        // Default to Google API Key (Google AI Studio / Gemini Developer APIs)
        ("x-goog-api-key", key.to_string())
    }

    fn convert_messages_to_gemini(&self, messages: &Value) -> Value {
        let mut contents = Vec::new();
        if let Some(msg_array) = messages.as_array() {
            for msg in msg_array {
                let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                
                let gemini_role = match role {
                    "user" => "user",
                    "assistant" => "model",
                    "system" | "episodic" | "compact" => "user",
                    "tool" => "user",
                    _ => "user",
                };

                contents.push(serde_json::json!({
                    "role": gemini_role,
                    "parts": [{"text": content}]
                }));
            }
        }
        serde_json::json!(contents)
    }
}

#[async_trait]
impl LLMProvider for VertexClient {
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

        let (header_name, auth_value) = self.get_auth_header(key).await;
        
        // Auto-detect if we should use native Gemini API
        let is_native = self.endpoint_path.contains(":generateContent") || 
                        self.base_url.contains("generativelanguage") && !self.endpoint_path.contains("openai");

        let full_url = if is_native && !self.endpoint_path.contains(":generateContent") {
            // Build native URL for generativelanguage
            format!("{}/v1beta/models/{}:generateContent", self.base_url.trim_end_matches('/'), model)
        } else {
            format!("{}{}", self.base_url, self.endpoint_path)
        }.trim_end_matches('/').to_string();

        let request_body = if is_native {
            serde_json::json!({
                "contents": self.convert_messages_to_gemini(raw_messages),
                "generationConfig": {
                    "temperature": 0.7,
                }
            })
        } else {
            serde_json::json!({
                "model": model,
                "messages": raw_messages,
                "stream": false
            })
        };

        info!("Sending request to Vertex/Gemini endpoint: {}", full_url);

        let response = self.client
            .post(full_url)
            .header(header_name, &auth_value)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Vertex/Gemini generate error: {}", error_text);
            return Err(format!("Google API error ({}): {}", status, error_text).into());
        }

        let json: Value = response.json().await?;
        
        // Try parsing OpenAI format
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
        
        // Try parsing Native Gemini format
        if let Some(candidates) = json.get("candidates") {
            if let Some(first_candidate) = candidates.get(0) {
                if let Some(content) = first_candidate.get("content") {
                    if let Some(parts) = content.get("parts") {
                        if let Some(first_part) = parts.get(0) {
                            if let Some(text) = first_part.get("text") {
                                if let Some(text_str) = text.as_str() {
                                    return Ok(text_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("Failed to parse response from Vertex/Gemini (unknown format)".into())
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

        let (header_name, auth_value) = self.get_auth_header(key).await;
        
        // Auto-detect if we should use native Gemini API
        let is_native = self.endpoint_path.contains("GenerateContent") || 
                        (self.base_url.contains("googleapis.com") && 
                         !self.endpoint_path.contains("openai") && 
                         !self.endpoint_path.contains("completions"));

        let full_url = if is_native && !self.endpoint_path.contains("GenerateContent") {
             // Build native URL for Gemini if it looks like we need it
             if self.base_url.contains("aiplatform") {
                 // Global Vertex endpoint format
                 format!("{}/v1/publishers/google/models/{}:streamGenerateContent", self.base_url.trim_end_matches('/'), model)
             } else {
                 // Google AI Studio format
                 format!("{}/v1beta/models/{}:streamGenerateContent", self.base_url.trim_end_matches('/'), model)
             }
        } else {
            format!("{}{}", self.base_url, self.endpoint_path)
        }.trim_end_matches('/').to_string();

        let mut request_body = if is_native {
            let mut body = serde_json::json!({
                "contents": self.convert_messages_to_gemini(raw_messages),
                "generationConfig": {
                    "temperature": 0.7,
                }
            });
            
            // Handle thinking models if requested
            if model.contains("preview") || model.contains("3.1-flash") {
                body["generationConfig"]["thinkingConfig"] = serde_json::json!({
                    "thinkingLevel": "LOW"
                });
            }
            body
        } else {
            serde_json::json!({
                "model": model,
                "messages": raw_messages,
                "stream": true
            })
        };

        if let Some(tools_value) = tools {
            if !tools_value.is_empty() {
                request_body["tools"] = serde_json::json!(tools_value);
                info!("Added {} tools to Vertex request", tools_value.len());
            }
        }

        info!("Sending streaming request to Vertex/Gemini endpoint: {}", full_url);

        let response = self.client
            .post(full_url)
            .header(header_name, &auth_value)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Vertex/Gemini generate_stream error: {}", error_text);
            return Err(format!("Google API error ({}): {}", status, error_text).into());
        }

        let mut stream = response.bytes_stream();
        let mut tool_call_buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Ok(text) = std::str::from_utf8(&chunk) {
                        for line in text.split('\n') {
                            let line = line.trim();
                            if line.is_empty() { continue; }

                            if let Some(data) = line.strip_prefix("data: ") {
                                // --- OpenAI SSE Format ---
                                if data == "[DONE]" { continue; }
                                if let Ok(json) = serde_json::from_str::<Value>(data) {
                                    if let Some(choices) = json.get("choices") {
                                        if let Some(first_choice) = choices.get(0) {
                                            if let Some(delta) = first_choice.get("delta") {
                                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                    if !content.is_empty() {
                                                        let _ = tx.send(content.to_string()).await;
                                                    }
                                                }
                                                
                                                // Handle tool calls...
                                                if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                                    if let Some(first_call) = tool_calls.first() {
                                                        if let Some(function) = first_call.get("function") {
                                                            if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                                                tool_call_buffer = format!("{{\"name\": \"{}\", \"arguments\": \"", name);
                                                            }
                                                            if let Some(args) = function.get("arguments").and_then(|a| a.as_str()) {
                                                                tool_call_buffer.push_str(args);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // --- Native Gemini JSON format ---
                                // It usually arrives as raw JSON objects, sometimes with leading commas or brackets in a stream array
                                let trimmed = line.trim_start_matches([',', '[', ']']).trim_end_matches(']');
                                if trimmed.is_empty() { continue; }
                                
                                if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
                                    if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                                        if let Some(candidate) = candidates.first() {
                                            if let Some(content) = candidate.get("content") {
                                                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                                    for part in parts {
                                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                            if !text.is_empty() {
                                                                let _ = tx.send(text.to_string()).await;
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
                    error!("Vertex stream reading error: {}", e);
                    break;
                }
            }
        }
        
        if !tool_call_buffer.is_empty() {
            tool_call_buffer.push_str("\"}");
            let _ = tx.send(format!("`tool_call`: {}", tool_call_buffer)).await;
        }

        Ok(())
    }

    async fn cancel_job(&self, _session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
