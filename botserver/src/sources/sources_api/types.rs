use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotQuery {
    pub bot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub server_type: String,
    pub status: String,
    pub enabled: bool,
    pub tools_count: usize,
    pub source: String,
    pub tags: Vec<String>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResponse {
    pub name: String,
    pub description: String,
    pub server_name: String,
    pub risk_level: String,
    pub requires_approval: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMcpServerRequest {
    pub name: String,
    pub description: Option<String>,
    pub server_type: String,
    pub connection: McpConnectionRequest,
    pub auth: Option<McpAuthRequest>,
    pub enabled: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub requires_approval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpConnectionRequest {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(default = "default_timeout")]
        timeout: u32,
    },
    #[serde(rename = "websocket")]
    WebSocket { url: String },
}

fn default_timeout() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpAuthRequest {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "api_key")]
    ApiKey { header: String, key_env: String },
    #[serde(rename = "bearer")]
    Bearer { token_env: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub description: String,
    pub url: String,
    pub language: Option<String>,
    pub stars: u32,
    pub forks: u32,
    pub status: String,
    pub last_sync: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub app_type: String,
    pub description: String,
    pub url: String,
    pub created_at: String,
    pub status: String,
}

/// MCP Server from JSON catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub category: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersCatalog {
    pub mcp_servers: Vec<McpServerCatalogEntry>,
    pub categories: Vec<String>,
    pub types: Vec<McpServerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerType {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PromptData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub struct TemplateData {
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
}
