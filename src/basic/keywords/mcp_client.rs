//! MCP Client - Model Context Protocol Server Integration
//!
//! This module provides the MCP (Model Context Protocol) client functionality
//! that enables BASIC programs to call registered MCP servers for extended
//! capabilities. It supports tool discovery, invocation, and result handling.
//!
//! # Architecture
//!
//! ```text
//! BASIC Program → USE_MCP → MCP Client → MCP Server → Execute Tool → Return Result
//!       ↓            ↓           ↓            ↓             ↓            ↓
//!   USE_MCP      Resolve     Connect     Invoke tool    Process     Return to
//!   "server"     server      & auth      with params    result      BASIC
//! ```
//!
//! # Supported MCP Servers
//!
//! - Database Server: PostgreSQL, MySQL, SQLite connections
//! - Filesystem Server: Local and cloud file access
//! - Web Server: HTTP/REST API integrations
//! - Email Server: SMTP/IMAP email handling
//! - Slack Server: Slack workspace integration
//! - Analytics Server: Data processing and reporting
//! - Custom Servers: User-defined MCP servers
//!
//! # Example BASIC Usage
//!
//! ```basic
//! ' Use MCP server to query database
//! result = USE_MCP "database", "query", {"sql": "SELECT * FROM users"}
//!
//! ' Use MCP server to send Slack message
//! USE_MCP "slack", "send_message", {"channel": "#general", "text": "Hello!"}
//!
//! ' List available tools from a server
//! tools = MCP_LIST_TOOLS "filesystem"
//! ```

use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// MCP DATA STRUCTURES
// ============================================================================

/// Represents a registered MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// Unique server identifier
    pub id: String,
    /// Server name (used in BASIC as identifier)
    pub name: String,
    /// Server description
    pub description: String,
    /// Server type/category
    pub server_type: McpServerType,
    /// Connection configuration
    pub connection: McpConnection,
    /// Authentication configuration
    pub auth: McpAuth,
    /// Available tools on this server
    pub tools: Vec<McpTool>,
    /// Server capabilities
    pub capabilities: McpCapabilities,
    /// Server status
    pub status: McpServerStatus,
    /// Bot ID that owns this server config
    pub bot_id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Last health check timestamp
    pub last_health_check: Option<DateTime<Utc>>,
    /// Health check status
    pub health_status: HealthStatus,
}

/// MCP server types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        McpServerType::Custom("unknown".to_string())
    }
}

impl From<&str> for McpServerType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "database" | "db" => McpServerType::Database,
            "filesystem" | "fs" | "file" => McpServerType::Filesystem,
            "web" | "http" | "rest" | "api" => McpServerType::Web,
            "email" | "mail" | "smtp" | "imap" => McpServerType::Email,
            "slack" => McpServerType::Slack,
            "teams" | "microsoft-teams" => McpServerType::Teams,
            "analytics" | "data" => McpServerType::Analytics,
            "search" | "elasticsearch" | "opensearch" => McpServerType::Search,
            "storage" | "s3" | "blob" | "gcs" => McpServerType::Storage,
            "compute" | "lambda" | "function" => McpServerType::Compute,
            other => McpServerType::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for McpServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpServerType::Database => write!(f, "database"),
            McpServerType::Filesystem => write!(f, "filesystem"),
            McpServerType::Web => write!(f, "web"),
            McpServerType::Email => write!(f, "email"),
            McpServerType::Slack => write!(f, "slack"),
            McpServerType::Teams => write!(f, "teams"),
            McpServerType::Analytics => write!(f, "analytics"),
            McpServerType::Search => write!(f, "search"),
            McpServerType::Storage => write!(f, "storage"),
            McpServerType::Compute => write!(f, "compute"),
            McpServerType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// MCP connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConnection {
    /// Connection type
    pub connection_type: ConnectionType,
    /// Server URL or path
    pub url: String,
    /// Connection port (if applicable)
    pub port: Option<u16>,
    /// Connection timeout in seconds
    pub timeout_seconds: i32,
    /// Maximum retries
    pub max_retries: i32,
    /// Retry backoff in milliseconds
    pub retry_backoff_ms: i32,
    /// Keep-alive settings
    pub keep_alive: bool,
    /// SSL/TLS configuration
    pub tls_config: Option<TlsConfig>,
}

impl Default for McpConnection {
    fn default() -> Self {
        McpConnection {
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

/// Connection type for MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    /// HTTP/HTTPS connection
    Http,
    /// WebSocket connection
    WebSocket,
    /// gRPC connection
    Grpc,
    /// Unix socket
    UnixSocket,
    /// Standard IO (for local processes)
    Stdio,
    /// TCP socket
    Tcp,
}

impl Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::Http
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub verify_certificates: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
}

/// MCP authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuth {
    /// Authentication type
    pub auth_type: McpAuthType,
    /// Credentials (stored securely, reference only)
    pub credentials: McpCredentials,
}

impl Default for McpAuth {
    fn default() -> Self {
        McpAuth {
            auth_type: McpAuthType::None,
            credentials: McpCredentials::None,
        }
    }
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpAuthType {
    None,
    ApiKey,
    Bearer,
    Basic,
    OAuth2,
    Certificate,
    Custom(String),
}

impl Default for McpAuthType {
    fn default() -> Self {
        McpAuthType::None
    }
}

/// Credentials storage (references, not actual secrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpCredentials {
    None,
    ApiKey {
        header_name: String,
        key_ref: String, // Reference to secret storage
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

impl Default for McpCredentials {
    fn default() -> Self {
        McpCredentials::None
    }
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
    /// Output schema (JSON Schema)
    pub output_schema: Option<serde_json::Value>,
    /// Required permissions
    pub required_permissions: Vec<String>,
    /// Risk level of this tool
    pub risk_level: ToolRiskLevel,
    /// Whether this tool modifies data
    pub is_destructive: bool,
    /// Whether this tool requires approval
    pub requires_approval: bool,
    /// Rate limit (calls per minute)
    pub rate_limit: Option<i32>,
    /// Timeout for this specific tool
    pub timeout_seconds: Option<i32>,
}

/// Tool risk level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolRiskLevel {
    Safe,     // Read-only, no side effects
    Low,      // Minor side effects, easily reversible
    Medium,   // Moderate side effects, reversible with effort
    High,     // Significant side effects, difficult to reverse
    Critical, // Irreversible actions, requires approval
}

impl Default for ToolRiskLevel {
    fn default() -> Self {
        ToolRiskLevel::Low
    }
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpCapabilities {
    /// Supports tool listing
    pub tools: bool,
    /// Supports resource listing
    pub resources: bool,
    /// Supports prompts
    pub prompts: bool,
    /// Supports logging
    pub logging: bool,
    /// Supports streaming responses
    pub streaming: bool,
    /// Supports cancellation
    pub cancellation: bool,
    /// Custom capabilities
    pub custom: HashMap<String, bool>,
}

/// MCP server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerStatus {
    Active,
    Inactive,
    Connecting,
    Error(String),
    Maintenance,
    Unknown,
}

impl Default for McpServerStatus {
    fn default() -> Self {
        McpServerStatus::Inactive
    }
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub last_check: Option<DateTime<Utc>>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub consecutive_failures: i32,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus {
            healthy: false,
            last_check: None,
            response_time_ms: None,
            error_message: None,
            consecutive_failures: 0,
        }
    }
}

// ============================================================================
// MCP REQUEST/RESPONSE
// ============================================================================

/// MCP tool invocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// Request ID for tracking
    pub id: String,
    /// Target server name
    pub server: String,
    /// Tool to invoke
    pub tool: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
    /// Request context
    pub context: McpRequestContext,
    /// Timeout override
    pub timeout_seconds: Option<i32>,
}

/// Request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequestContext {
    pub session_id: String,
    pub bot_id: String,
    pub user_id: String,
    pub task_id: Option<String>,
    pub step_id: Option<String>,
    pub correlation_id: Option<String>,
}

/// MCP tool invocation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Response ID (matches request ID)
    pub id: String,
    /// Whether the invocation succeeded
    pub success: bool,
    /// Result data
    pub result: Option<serde_json::Value>,
    /// Error information
    pub error: Option<McpError>,
    /// Execution metadata
    pub metadata: McpResponseMetadata,
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub retryable: bool,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseMetadata {
    pub duration_ms: i64,
    pub server_version: Option<String>,
    pub rate_limit_remaining: Option<i32>,
    pub rate_limit_reset: Option<DateTime<Utc>>,
}

// ============================================================================
// MCP CLIENT
// ============================================================================

/// The MCP Client for managing server connections and tool invocations
pub struct McpClient {
    state: Arc<AppState>,
    config: McpClientConfig,
    servers: HashMap<String, McpServer>,
    http_client: reqwest::Client,
}

/// MCP Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// Enable/disable MCP functionality
    pub enabled: bool,
    /// Default timeout for all requests
    pub default_timeout_seconds: i32,
    /// Maximum concurrent requests
    pub max_concurrent_requests: i32,
    /// Enable request caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: i32,
    /// Enable audit logging
    pub audit_enabled: bool,
    /// Health check interval in seconds
    pub health_check_interval_seconds: i32,
    /// Auto-retry failed requests
    pub auto_retry: bool,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: i32,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        McpClientConfig {
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
            .finish()
    }
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(state: Arc<AppState>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        McpClient {
            state,
            config: McpClientConfig::default(),
            servers: HashMap::new(),
            http_client,
        }
    }

    /// Create a new MCP client with custom configuration
    pub fn with_config(state: Arc<AppState>, config: McpClientConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.default_timeout_seconds as u64))
            .build()
            .unwrap_or_default();

        McpClient {
            state,
            config,
            servers: HashMap::new(),
            http_client,
        }
    }

    /// Load servers from database for a bot
    pub async fn load_servers(
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
                    "inactive" => McpServerStatus::Inactive,
                    "error" => McpServerStatus::Error("Unknown error".to_string()),
                    "maintenance" => McpServerStatus::Maintenance,
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

    /// Register a new MCP server
    pub async fn register_server(
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

    /// Get a server by name
    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    /// List all registered servers
    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    /// List tools from a specific server
    pub async fn list_tools(
        &self,
        server_name: &str,
    ) -> Result<Vec<McpTool>, Box<dyn std::error::Error + Send + Sync>> {
        let server = self
            .servers
            .get(server_name)
            .ok_or_else(|| format!("MCP server '{}' not found", server_name))?;

        // For HTTP-based servers, call the tools/list endpoint
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

        // Return cached tools if available
        Ok(server.tools.clone())
    }

    /// Invoke a tool on an MCP server
    pub async fn invoke_tool(
        &self,
        request: McpRequest,
    ) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Get server
        let server = self
            .servers
            .get(&request.server)
            .ok_or_else(|| format!("MCP server '{}' not found", request.server))?;

        // Check server status
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

        // Audit the request
        if self.config.audit_enabled {
            info!(
                "MCP request: server={} tool={}",
                request.server, request.tool
            );
        }

        // Execute based on connection type
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

                // Audit log the response
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

                // Audit log the error
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

    /// Invoke tool via HTTP
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

        // Add authentication headers
        http_request = self.add_auth_headers(http_request, &server.auth);

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

    /// Invoke tool via stdio (local process)
    async fn invoke_stdio(
        &self,
        server: &McpServer,
        request: &McpRequest,
    ) -> Result<McpResponse, Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        let _input = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": request.tool,
                "arguments": request.arguments
            },
            "id": request.id
        });

        let output = Command::new(&server.connection.url)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;

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

    /// Add authentication headers to request
    fn add_auth_headers(
        &self,
        mut request: reqwest::RequestBuilder,
        auth: &McpAuth,
    ) -> reqwest::RequestBuilder {
        match &auth.credentials {
            McpCredentials::ApiKey {
                header_name,
                key_ref,
            } => {
                // In production, resolve key_ref from secret storage
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

    /// Perform health check on a server
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
