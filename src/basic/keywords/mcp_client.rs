use crate::security::command_guard::SafeCommand;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,

    pub name: String,

    pub description: String,

    pub server_type: McpServerType,

    pub connection: McpConnection,

    pub auth: McpAuth,

    pub tools: Vec<McpTool>,

    pub capabilities: McpCapabilities,

    pub status: McpServerStatus,

    pub bot_id: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub last_health_check: Option<DateTime<Utc>>,

    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum McpServerType {
    Database,
    Filesystem,
    Web,
    Email,
    Slack,
    Teams,
    Analytics,
    Search,
    Storage,
    Compute,
    Custom(String),
}

impl Default for McpServerType {
    fn default() -> Self {
        Self::Custom("unknown".to_string())
    }
}

impl From<&str> for McpServerType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "database" | "db" => Self::Database,
            "filesystem" | "fs" | "file" => Self::Filesystem,
            "web" | "http" | "rest" | "api" => Self::Web,
            "email" | "mail" | "smtp" | "imap" => Self::Email,
            "slack" => Self::Slack,
            "teams" | "microsoft-teams" => Self::Teams,
            "analytics" | "data" => Self::Analytics,
            "search" | "elasticsearch" | "opensearch" => Self::Search,
            "storage" | "s3" | "blob" | "gcs" => Self::Storage,
            "compute" | "lambda" | "function" => Self::Compute,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for McpServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database => write!(f, "database"),
            Self::Filesystem => write!(f, "filesystem"),
            Self::Web => write!(f, "web"),
            Self::Email => write!(f, "email"),
            Self::Slack => write!(f, "slack"),
            Self::Teams => write!(f, "teams"),
            Self::Analytics => write!(f, "analytics"),
            Self::Search => write!(f, "search"),
            Self::Storage => write!(f, "storage"),
            Self::Compute => write!(f, "compute"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnection {
    pub connection_type: ConnectionType,

    pub url: String,

    pub port: Option<u16>,

    pub timeout_seconds: i32,

    pub max_retries: i32,

    pub retry_backoff_ms: i32,

    pub keep_alive: bool,

    pub tls_config: Option<TlsConfig>,
}

impl Default for McpConnection {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Http,
            url: "http://localhost:8080".to_string(),
            port: None,
            timeout_seconds: 30,
            max_retries: 3,
            retry_backoff_ms: 1000,
            keep_alive: true,
            tls_config: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ConnectionType {
    #[default]
    Http,

    WebSocket,

    Grpc,

    UnixSocket,

    Stdio,

    Tcp,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub verify_certificates: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuth {
    pub auth_type: McpAuthType,

    pub credentials: McpCredentials,
}

impl Default for McpAuth {
    fn default() -> Self {
        Self {
            auth_type: McpAuthType::None,
            credentials: McpCredentials::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum McpAuthType {
    #[default]
    None,
    ApiKey,
    Bearer,
    Basic,
    OAuth2,
    Certificate,
    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum McpCredentials {
    #[default]
    None,
    ApiKey {
        header_name: String,
        key_ref: String,
    },
    Bearer {
        token_ref: String,
    },
    Basic {
        username_ref: String,
        password_ref: String,
    },
    OAuth2 {
        client_id_ref: String,
        client_secret_ref: String,
        token_url: String,
        scopes: Vec<String>,
    },
    Certificate {
        cert_ref: String,
        key_ref: String,
    },
    Custom(HashMap<String, String>),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,

    pub description: String,

    pub input_schema: serde_json::Value,

    pub output_schema: Option<serde_json::Value>,

    pub required_permissions: Vec<String>,

    pub risk_level: ToolRiskLevel,

    pub is_destructive: bool,

    pub requires_approval: bool,

    pub rate_limit: Option<i32>,

    pub timeout_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ToolRiskLevel {
    Safe,
    #[default]
    Low,
    Medium,
    High,
    Critical,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpCapabilities {
    pub tools: bool,

    pub resources: bool,

    pub prompts: bool,

    pub logging: bool,

    pub streaming: bool,

    pub cancellation: bool,

    pub custom: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum McpServerStatus {
    Active,
    #[default]
    Inactive,
    Connecting,
    Error(String),
    Maintenance,
    Unknown,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct HealthStatus {
    pub healthy: bool,
    pub last_check: Option<DateTime<Utc>>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub consecutive_failures: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,

    pub server: String,

    pub tool: String,

    pub arguments: serde_json::Value,

    pub context: McpRequestContext,

    pub timeout_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequestContext {
    pub session_id: String,
    pub bot_id: String,
    pub user_id: String,
    pub task_id: Option<String>,
    pub step_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,

    pub success: bool,

    pub result: Option<serde_json::Value>,

    pub error: Option<McpError>,

    pub metadata: McpResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseMetadata {
    pub duration_ms: i64,
    pub server_version: Option<String>,
    pub rate_limit_remaining: Option<i32>,
    pub rate_limit_reset: Option<DateTime<Utc>>,
}

pub struct McpClient {
    state: Arc<AppState>,
    config: McpClientConfig,
    servers: HashMap<String, McpServer>,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    pub enabled: bool,

    pub default_timeout_seconds: i32,

    pub max_concurrent_requests: i32,

    pub cache_enabled: bool,

    pub cache_ttl_seconds: i32,

    pub audit_enabled: bool,

    pub health_check_interval_seconds: i32,

    pub auto_retry: bool,

    pub circuit_breaker_threshold: i32,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_timeout_seconds: 30,
            max_concurrent_requests: 10,
            cache_enabled: true,
            cache_ttl_seconds: 300,
            audit_enabled: true,
            health_check_interval_seconds: 60,
            auto_retry: true,
            circuit_breaker_threshold: 5,
        }
    }
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient")
            .field("config", &self.config)
            .field("servers_count", &self.servers.len())
            .finish_non_exhaustive()
    }
}

impl McpClient {
    pub fn new(state: Arc<AppState>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            state,
            config: McpClientConfig::default(),
            servers: HashMap::new(),
            http_client,
        }
    }

    pub fn with_config(state: Arc<AppState>, config: McpClientConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.default_timeout_seconds as u64))
            .build()
            .unwrap_or_default();

        Self {
            state,
            config,
            servers: HashMap::new(),
            http_client,
        }
    }

    pub fn load_servers(
        &mut self,
        bot_id: &Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB error: {}", e))?;
        let bot_id_str = bot_id.to_string();

        let query = diesel::sql_query(
            "SELECT id, name, description, server_type, config, status, created_at, updated_at
             FROM mcp_servers WHERE bot_id = $1 AND status != 'deleted'",
        )
        .bind::<diesel::sql_types::Text, _>(&bot_id_str);

        #[derive(QueryableByName)]
        struct ServerRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            id: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            server_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
        }

        let rows: Vec<ServerRow> = query.load(&mut *conn).unwrap_or_default();

        for row in rows {
            let server = McpServer {
                id: row.id.clone(),
                name: row.name.clone(),
                description: row.description.unwrap_or_default(),
                server_type: McpServerType::from(row.server_type.as_str()),
                connection: serde_json::from_str(&row.config).unwrap_or_default(),
                auth: McpAuth::default(),
                tools: Vec::new(),
                capabilities: McpCapabilities::default(),
                status: match row.status.as_str() {
                    "active" => McpServerStatus::Active,
                    "maintenance" => McpServerStatus::Maintenance,
                    "error" => McpServerStatus::Error("Unknown error".to_string()),
                    _ => McpServerStatus::Inactive,
                },
                bot_id: bot_id_str.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_health_check: None,
                health_status: HealthStatus::default(),
            };

            self.servers.insert(row.name, server);
        }

        info!(
            "Loaded {} MCP servers for bot {}",
            self.servers.len(),
            bot_id
        );
        Ok(())
    }

    pub fn register_server(
        &mut self,
        server: McpServer,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB error: {}", e))?;

        let config_json = serde_json::to_string(&server.connection)?;
        let now = Utc::now().to_rfc3339();

        let server_type_str = server.server_type.to_string();
        let query = diesel::sql_query(
            "INSERT INTO mcp_servers (id, bot_id, name, description, server_type, config, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (bot_id, name) DO UPDATE SET
                description = EXCLUDED.description,
                server_type = EXCLUDED.server_type,
                config = EXCLUDED.config,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at"
        )
            .bind::<diesel::sql_types::Text, _>(&server.id)
            .bind::<diesel::sql_types::Text, _>(&server.bot_id)
            .bind::<diesel::sql_types::Text, _>(&server.name)
            .bind::<diesel::sql_types::Text, _>(&server.description)
            .bind::<diesel::sql_types::Text, _>(&server_type_str)
            .bind::<diesel::sql_types::Text, _>(&config_json)
            .bind::<diesel::sql_types::Text, _>("active")
            .bind::<diesel::sql_types::Text, _>(&now)
            .bind::<diesel::sql_types::Text, _>(&now);

        query
            .execute(&mut *conn)
            .map_err(|e| format!("Failed to register MCP server: {}", e))?;

        self.servers.insert(server.name.clone(), server);
        Ok(())
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    pub async fn list_tools(
        &self,
        server_name: &str,
    ) -> Result<Vec<McpTool>, Box<dyn std::error::Error + Send + Sync>> {
        let server = self
            .servers
            .get(server_name)
            .ok_or_else(|| format!("MCP server '{}' not found", server_name))?;

        if server.connection.connection_type == ConnectionType::Http {
            let url = format!("{}/tools/list", server.connection.url);
            let response = self
                .http_client
                .get(&url)
                .timeout(Duration::from_secs(
                    server.connection.timeout_seconds as u64,
                ))
                .send()
                .await?;

            if response.status().is_success() {
                let tools: Vec<McpTool> = response.json().await?;
                return Ok(tools);
            }
        }

        Ok(server.tools.clone())
    }

    pub async fn invoke_tool(
        &self,
        request: McpRequest,
    ) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        let server = self
            .servers
            .get(&request.server)
            .ok_or_else(|| format!("MCP server '{}' not found", request.server))?;

        if server.status != McpServerStatus::Active {
            return Ok(McpResponse {
                id: request.id,
                success: false,
                result: None,
                error: Some(McpError {
                    code: "SERVER_UNAVAILABLE".to_string(),
                    message: format!(
                        "MCP server '{}' is not active (status: {:?})",
                        request.server, server.status
                    ),
                    details: None,
                    retryable: true,
                }),
                metadata: McpResponseMetadata {
                    duration_ms: start_time.elapsed().as_millis() as i64,
                    server_version: None,
                    rate_limit_remaining: None,
                    rate_limit_reset: None,
                },
            });
        }

        if self.config.audit_enabled {
            info!(
                "MCP request: server={} tool={}",
                request.server, request.tool
            );
        }

        let result = match server.connection.connection_type {
            ConnectionType::Http => self.invoke_http(server, &request).await,
            ConnectionType::Stdio => self.invoke_stdio(server, &request).await,
            _ => Err(format!(
                "Connection type {:?} not yet supported",
                server.connection.connection_type
            )
            .into()),
        };

        let duration_ms = start_time.elapsed().as_millis() as i64;

        match result {
            Ok(mut response) => {
                response.metadata.duration_ms = duration_ms;

                if self.config.audit_enabled {
                    info!(
                        "MCP response: id={} success={}",
                        response.id, response.success
                    );
                }

                Ok(response)
            }
            Err(e) => {
                let response = McpResponse {
                    id: request.id.clone(),
                    success: false,
                    result: None,
                    error: Some(McpError {
                        code: "INVOCATION_ERROR".to_string(),
                        message: e.to_string(),
                        details: None,
                        retryable: true,
                    }),
                    metadata: McpResponseMetadata {
                        duration_ms,
                        server_version: None,
                        rate_limit_remaining: None,
                        rate_limit_reset: None,
                    },
                };

                if self.config.audit_enabled {
                    info!(
                        "MCP error response: id={} error={:?}",
                        response.id, response.error
                    );
                }

                Ok(response)
            }
        }
    }

    async fn invoke_http(
        &self,
        server: &McpServer,
        request: &McpRequest,
    ) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/tools/call", server.connection.url);

        let body = serde_json::json!({
            "name": request.tool,
            "arguments": request.arguments
        });

        let timeout = request
            .timeout_seconds
            .unwrap_or(server.connection.timeout_seconds);

        let mut http_request = self
            .http_client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(timeout as u64));

        http_request = Self::add_auth_headers(http_request, &server.auth);

        let response = http_request.send().await?;
        let status = response.status();

        if status.is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(McpResponse {
                id: request.id.clone(),
                success: true,
                result: Some(result),
                error: None,
                metadata: McpResponseMetadata {
                    duration_ms: 0,
                    server_version: None,
                    rate_limit_remaining: None,
                    rate_limit_reset: None,
                },
            })
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Ok(McpResponse {
                id: request.id.clone(),
                success: false,
                result: None,
                error: Some(McpError {
                    code: format!("HTTP_{}", status.as_u16()),
                    message: error_text,
                    details: None,
                    retryable: status.as_u16() >= 500,
                }),
                metadata: McpResponseMetadata {
                    duration_ms: 0,
                    server_version: None,
                    rate_limit_remaining: None,
                    rate_limit_reset: None,
                },
            })
        }
    }

    async fn invoke_stdio(
        &self,
        server: &McpServer,
        request: &McpRequest,
    ) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let _input = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": request.tool,
                "arguments": request.arguments
            },
            "id": request.id
        });

        let cmd = SafeCommand::new(&server.connection.url)
            .map_err(|e| anyhow::anyhow!("Failed to build MCP command: {}", e))?;

        let output = cmd.execute_async()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute MCP command: {}", e))?;

        if output.status.success() {
            let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
            Ok(McpResponse {
                id: request.id.clone(),
                success: true,
                result: result.get("result").cloned(),
                error: None,
                metadata: McpResponseMetadata {
                    duration_ms: 0,
                    server_version: None,
                    rate_limit_remaining: None,
                    rate_limit_reset: None,
                },
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(McpResponse {
                id: request.id.clone(),
                success: false,
                result: None,
                error: Some(McpError {
                    code: "STDIO_ERROR".to_string(),
                    message: stderr.to_string(),
                    details: None,
                    retryable: false,
                }),
                metadata: McpResponseMetadata {
                    duration_ms: 0,
                    server_version: None,
                    rate_limit_remaining: None,
                    rate_limit_reset: None,
                },
            })
        }
    }

    fn add_auth_headers(
        mut request: reqwest::RequestBuilder,
        auth: &McpAuth,
    ) -> reqwest::RequestBuilder {
        match &auth.credentials {
            McpCredentials::ApiKey {
                header_name,
                key_ref,
            } => {
                request = request.header(header_name.as_str(), key_ref.as_str());
            }
            McpCredentials::Bearer { token_ref } => {
                request = request.bearer_auth(token_ref);
            }
            McpCredentials::Basic {
                username_ref,
                password_ref,
            } => {
                request = request.basic_auth(username_ref, Some(password_ref));
            }
            _ => {}
        }
        request
    }

    pub async fn health_check(
        &mut self,
        server_name: &str,
    ) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>> {
        let server = self
            .servers
            .get_mut(server_name)
            .ok_or_else(|| format!("MCP server '{}' not found", server_name))?;

        let start_time = std::time::Instant::now();

        let health_url = format!("{}/health", server.connection.url);
        let result = self
            .http_client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        let latency_ms = start_time.elapsed().as_millis() as i64;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    server.status = McpServerStatus::Active;
                    Ok(HealthStatus {
                        healthy: true,
                        last_check: Some(Utc::now()),
                        response_time_ms: Some(latency_ms),
                        error_message: None,
                        consecutive_failures: 0,
                    })
                } else {
                    server.status = McpServerStatus::Error(format!("HTTP {}", response.status()));
                    Ok(HealthStatus {
                        healthy: false,
                        last_check: Some(Utc::now()),
                        response_time_ms: Some(latency_ms),
                        error_message: Some(format!(
                            "Server returned status {}",
                            response.status()
                        )),
                        consecutive_failures: 1,
                    })
                }
            }
            Err(e) => {
                server.status = McpServerStatus::Unknown;
                Ok(HealthStatus {
                    healthy: false,
                    last_check: Some(Utc::now()),
                    response_time_ms: Some(latency_ms),
                    error_message: Some(format!("Health check failed: {}", e)),
                    consecutive_failures: 1,
                })
            }
        }
    }
}
