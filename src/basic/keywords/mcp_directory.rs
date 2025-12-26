use chrono::Utc;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use super::mcp_client::{
    ConnectionType, HealthStatus, McpAuth, McpCapabilities, McpConnection, McpServer,
    McpServerStatus, McpServerType,
};

#[derive(Debug, Clone)]
pub struct McpCsvRow {
    pub name: String,

    pub connection_type: String,

    pub command: String,

    pub args: String,

    pub description: String,

    pub enabled: bool,

    pub auth_type: Option<String>,

    pub auth_env: Option<String>,

    pub risk_level: Option<String>,

    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,

    #[serde(default)]
    pub description: String,

    #[serde(rename = "type")]
    pub server_type: String,

    #[serde(default = "default_enabled")]
    pub enabled: bool,

    pub connection: McpConnectionConfig,

    #[serde(default)]
    pub auth: Option<McpAuthConfig>,

    #[serde(default)]
    pub tools: Vec<McpToolConfig>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub risk_level: String,

    #[serde(default)]
    pub requires_approval: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpConnectionConfig {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,

        #[serde(default)]
        args: Vec<String>,

        #[serde(default)]
        cwd: Option<String>,
    },

    #[serde(rename = "http")]
    Http {
        url: String,

        #[serde(default = "default_timeout")]
        timeout: u32,

        #[serde(default)]
        headers: HashMap<String, String>,
    },

    #[serde(rename = "websocket")]
    WebSocket {
        url: String,

        #[serde(default = "default_timeout")]
        timeout: u32,
    },

    #[serde(rename = "tcp")]
    Tcp { host: String, port: u16 },
}

fn default_timeout() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpAuthConfig {
    #[serde(rename = "none")]
    None,

    #[serde(rename = "api_key")]
    ApiKey {
        #[serde(default = "default_api_key_header")]
        header: String,

        key_env: String,
    },

    #[serde(rename = "bearer")]
    Bearer { token_env: String },

    #[serde(rename = "basic")]
    Basic {
        username_env: String,

        password_env: String,
    },
}

fn default_api_key_header() -> String {
    "X-API-Key".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolConfig {
    pub name: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,

    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,

    #[serde(default)]
    pub risk_level: Option<String>,

    #[serde(default)]
    pub requires_approval: bool,

    #[serde(default)]
    pub rate_limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct McpLoadResult {
    pub servers: Vec<McpServer>,

    pub errors: Vec<McpLoadError>,

    pub lines_processed: usize,

    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct McpLoadError {
    pub line: usize,

    pub message: String,

    pub recoverable: bool,
}

#[derive(Debug)]
pub struct McpCsvLoader {
    work_path: String,

    bot_id: String,
}

impl McpCsvLoader {
    pub fn new(work_path: &str, bot_id: &str) -> Self {
        Self {
            work_path: work_path.to_string(),
            bot_id: bot_id.to_string(),
        }
    }

    pub fn get_csv_path(&self) -> PathBuf {
        PathBuf::from(&self.work_path)
            .join(format!("{}.gbai", self.bot_id))
            .join("mcp.csv")
    }

    pub fn csv_exists(&self) -> bool {
        self.get_csv_path().exists()
    }

    pub fn load(&self) -> McpLoadResult {
        let csv_path = self.get_csv_path();

        info!("Loading MCP servers from: {}", csv_path.display());

        let mut result = McpLoadResult {
            servers: Vec::new(),
            errors: Vec::new(),
            lines_processed: 0,
            file_path: csv_path.clone(),
        };

        if !csv_path.exists() {
            debug!("MCP CSV file does not exist: {}", csv_path.display());
            return result;
        }

        let content = match std::fs::read_to_string(&csv_path) {
            Ok(c) => c,
            Err(e) => {
                result.errors.push(McpLoadError {
                    line: 0,
                    message: format!("Failed to read mcp.csv: {}", e),
                    recoverable: false,
                });
                return result;
            }
        };

        let mut lines = content.lines().enumerate();

        if let Some((_, header)) = lines.next() {
            let header_lower = header.to_lowercase();
            if !header_lower.starts_with("name,") && !header_lower.contains(",type,") {
                if let Some(server) = self.parse_csv_line(1, header, &mut result.errors) {
                    result.servers.push(server);
                }
            }
            result.lines_processed += 1;
        }

        for (line_num, line) in lines {
            let line_number = line_num + 1;
            result.lines_processed += 1;

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }

            if let Some(server) = self.parse_csv_line(line_number, line, &mut result.errors) {
                result.servers.push(server);
            }
        }

        info!(
            "MCP load complete: {} servers loaded, {} errors, {} lines processed",
            result.servers.len(),
            result.errors.len(),
            result.lines_processed
        );

        result
    }

    fn parse_csv_line(
        &self,
        line_num: usize,
        line: &str,
        errors: &mut Vec<McpLoadError>,
    ) -> Option<McpServer> {
        let columns = Self::parse_csv_columns(line);

        if columns.len() < 3 {
            errors.push(McpLoadError {
                line: line_num,
                message: format!(
                    "Invalid CSV: expected at least 3 columns (name,type,command), got {}",
                    columns.len()
                ),
                recoverable: true,
            });
            return None;
        }

        let name = columns[0].trim().to_string();
        let conn_type = columns[1].trim().to_lowercase();
        let command = columns[2].trim().to_string();
        let args = columns
            .get(3)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let description = columns
            .get(4)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let enabled = columns
            .get(5)
            .map(|s| s.trim().to_lowercase() != "false")
            .unwrap_or(true);
        let auth_type = columns
            .get(6)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let auth_env = columns
            .get(7)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let _risk_level = columns
            .get(8)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let _requires_approval = columns
            .get(9)
            .map(|s| s.trim().to_lowercase() == "true")
            .unwrap_or(false);

        if name.is_empty() {
            errors.push(McpLoadError {
                line: line_num,
                message: "Server name cannot be empty".to_string(),
                recoverable: true,
            });
            return None;
        }

        if command.is_empty() {
            errors.push(McpLoadError {
                line: line_num,
                message: format!("Command/URL cannot be empty for server '{}'", name),
                recoverable: true,
            });
            return None;
        }

        let connection = match conn_type.as_str() {
            "stdio" => {
                let args_vec: Vec<String> = if args.is_empty() {
                    Vec::new()
                } else {
                    Self::parse_args(&args)
                };
                McpConnection {
                    connection_type: ConnectionType::Stdio,
                    url: format!("{}:{}", command, args_vec.join(" ")),
                    port: None,
                    timeout_seconds: 30,
                    max_retries: 3,
                    retry_backoff_ms: 1000,
                    keep_alive: true,
                    tls_config: None,
                }
            }
            "http" => McpConnection {
                connection_type: ConnectionType::Http,
                url: command.clone(),
                port: None,
                timeout_seconds: 30,
                max_retries: 3,
                retry_backoff_ms: 1000,
                keep_alive: true,
                tls_config: None,
            },
            "websocket" | "ws" => McpConnection {
                connection_type: ConnectionType::WebSocket,
                url: command.clone(),
                port: None,
                timeout_seconds: 30,
                max_retries: 3,
                retry_backoff_ms: 1000,
                keep_alive: true,
                tls_config: None,
            },
            "tcp" => {
                let parts: Vec<&str> = command.split(':').collect();
                let host = parts
                    .first()
                    .map(|s| (*s).to_string())
                    .unwrap_or_else(|| "localhost".to_string());
                let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(9000);
                McpConnection {
                    connection_type: ConnectionType::Tcp,
                    url: host,
                    port: Some(port),
                    timeout_seconds: 30,
                    max_retries: 3,
                    retry_backoff_ms: 1000,
                    keep_alive: true,
                    tls_config: None,
                }
            }
            _ => {
                errors.push(McpLoadError {
                    line: line_num,
                    message: format!(
                        "Unknown connection type '{}' for server '{}'. Use: stdio, http, websocket, tcp",
                        conn_type, name
                    ),
                    recoverable: true,
                });
                return None;
            }
        };

        let auth = match (auth_type.as_deref(), auth_env.as_ref()) {
            (Some("api_key"), Some(env)) => {
                use super::mcp_client::{McpAuthType, McpCredentials};
                McpAuth {
                    auth_type: McpAuthType::ApiKey,
                    credentials: McpCredentials::ApiKey {
                        header_name: "X-API-Key".to_string(),
                        key_ref: env.clone(),
                    },
                }
            }
            (Some("bearer"), Some(env)) => {
                use super::mcp_client::{McpAuthType, McpCredentials};
                McpAuth {
                    auth_type: McpAuthType::Bearer,
                    credentials: McpCredentials::Bearer {
                        token_ref: env.clone(),
                    },
                }
            }
            _ => McpAuth::default(),
        };

        let server_type = Self::infer_server_type(&name, &conn_type, &command);

        let status = if enabled {
            McpServerStatus::Active
        } else {
            McpServerStatus::Inactive
        };

        debug!(
            "Loaded MCP server '{}' (type={}, enabled={})",
            name, server_type, enabled
        );

        Some(McpServer {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            server_type,
            connection,
            auth,
            tools: Vec::new(),
            capabilities: McpCapabilities {
                tools: true,
                resources: false,
                prompts: false,
                logging: false,
                streaming: false,
                cancellation: false,
                custom: HashMap::new(),
            },
            status,
            bot_id: self.bot_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_health_check: None,
            health_status: HealthStatus::default(),
        })
    }

    fn parse_csv_columns(line: &str) -> Vec<String> {
        let mut columns = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' if !in_quotes => {
                    in_quotes = true;
                }
                '"' if in_quotes => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        current.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                ',' if !in_quotes => {
                    columns.push(current.clone());
                    current.clear();
                }
                _ => {
                    current.push(c);
                }
            }
        }
        columns.push(current);

        columns
    }

    fn parse_args(args: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for c in args.chars() {
            match c {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = c;
                }
                c if in_quotes && c == quote_char => {
                    in_quotes = false;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
        if !current.is_empty() {
            result.push(current);
        }

        result
    }

    fn infer_server_type(name: &str, conn_type: &str, command: &str) -> McpServerType {
        let name_lower = name.to_lowercase();
        let cmd_lower = command.to_lowercase();

        if name_lower.contains("filesystem") || cmd_lower.contains("filesystem") {
            McpServerType::Filesystem
        } else if name_lower.contains("github") || cmd_lower.contains("github") {
            McpServerType::Web
        } else if name_lower.contains("postgres")
            || name_lower.contains("mysql")
            || name_lower.contains("sqlite")
            || name_lower.contains("database")
            || cmd_lower.contains("postgres")
            || cmd_lower.contains("mysql")
            || cmd_lower.contains("sqlite")
        {
            McpServerType::Database
        } else if name_lower.contains("slack") || cmd_lower.contains("slack") {
            McpServerType::Slack
        } else if name_lower.contains("teams") || cmd_lower.contains("teams") {
            McpServerType::Teams
        } else if name_lower.contains("email")
            || name_lower.contains("smtp")
            || name_lower.contains("imap")
        {
            McpServerType::Email
        } else if name_lower.contains("analytics") {
            McpServerType::Analytics
        } else if name_lower.contains("search") {
            McpServerType::Search
        } else if name_lower.contains("storage")
            || name_lower.contains("s3")
            || name_lower.contains("minio")
        {
            McpServerType::Storage
        } else if conn_type == "http" || conn_type == "websocket" {
            McpServerType::Web
        } else {
            McpServerType::Custom("custom".to_string())
        }
    }

    pub fn load_server(&self, name: &str) -> Option<McpServer> {
        let result = self.load();
        result.servers.into_iter().find(|s| s.name == name)
    }

    pub fn create_example_csv(&self) -> std::io::Result<PathBuf> {
        let csv_path = self.get_csv_path();

        if let Some(parent) = csv_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let example_content = r#"name,type,command,args,description,enabled
# MCP Server Configuration
# Columns: name,type,command,args,description,enabled,auth_type,auth_env,risk_level,requires_approval
#
# type: stdio (local process), http (REST API), websocket, tcp
# auth_type: none, api_key, bearer
# risk_level: safe, low, medium, high, critical
#
# Example servers:
filesystem,stdio,npx,"-y @modelcontextprotocol/server-filesystem /data",Access local files and directories,true
# github,stdio,npx,"-y @modelcontextprotocol/server-github",GitHub API integration,true,bearer,GITHUB_TOKEN
# postgres,stdio,npx,"-y @modelcontextprotocol/server-postgres",PostgreSQL database queries,false
# slack,stdio,npx,"-y @modelcontextprotocol/server-slack",Slack messaging,false,bearer,SLACK_BOT_TOKEN
# myapi,http,https://api.example.com/mcp,,Custom API server,true,api_key,MY_API_KEY
"#;

        std::fs::write(&csv_path, example_content)?;
        info!("Created example mcp.csv at {}", csv_path.display());

        Ok(csv_path)
    }

    pub fn add_server(&self, row: &McpCsvRow) -> std::io::Result<()> {
        let csv_path = self.get_csv_path();

        if !csv_path.exists() {
            if let Some(parent) = csv_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&csv_path, "name,type,command,args,description,enabled,auth_type,auth_env,risk_level,requires_approval\n")?;
        }

        let line = format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            Self::escape_csv(&row.name),
            Self::escape_csv(&row.connection_type),
            Self::escape_csv(&row.command),
            Self::escape_csv(&row.args),
            Self::escape_csv(&row.description),
            row.enabled,
            row.auth_type.as_deref().unwrap_or(""),
            row.auth_env.as_deref().unwrap_or(""),
            row.risk_level.as_deref().unwrap_or("medium"),
            row.requires_approval
        );

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(&csv_path)?;
        file.write_all(line.as_bytes())?;

        info!("Added MCP server '{}' to {}", row.name, csv_path.display());
        Ok(())
    }

    pub fn remove_server(&self, name: &str) -> std::io::Result<bool> {
        let csv_path = self.get_csv_path();

        if !csv_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&csv_path)?;
        let mut new_lines: Vec<&str> = Vec::new();
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                new_lines.push(line);
                continue;
            }

            let columns = Self::parse_csv_columns(line);
            if columns.first().map(|s| s.trim()) == Some(name) {
                found = true;
                continue;
            }

            new_lines.push(line);
        }

        if found {
            std::fs::write(&csv_path, new_lines.join("\n") + "\n")?;
            info!("Removed MCP server '{}' from {}", name, csv_path.display());
        }

        Ok(found)
    }

    fn escape_csv(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

pub fn generate_example_configs() -> Vec<McpServerConfig> {
    vec![
        McpServerConfig {
            name: "filesystem".to_string(),
            description: "Access local files and directories".to_string(),
            server_type: "filesystem".to_string(),
            enabled: true,
            connection: McpConnectionConfig::Stdio {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    "/data".to_string(),
                ],
                cwd: None,
            },
            auth: None,
            tools: vec![
                McpToolConfig {
                    name: "read_file".to_string(),
                    description: "Read contents of a file".to_string(),
                    input_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "File path to read"}
                        },
                        "required": ["path"]
                    })),
                    output_schema: None,
                    risk_level: Some("low".to_string()),
                    requires_approval: false,
                    rate_limit: None,
                },
                McpToolConfig {
                    name: "write_file".to_string(),
                    description: "Write contents to a file".to_string(),
                    input_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "File path to write"},
                            "content": {"type": "string", "description": "Content to write"}
                        },
                        "required": ["path", "content"]
                    })),
                    output_schema: None,
                    risk_level: Some("medium".to_string()),
                    requires_approval: true,
                    rate_limit: None,
                },
                McpToolConfig {
                    name: "list_directory".to_string(),
                    description: "List files in a directory".to_string(),
                    input_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Directory path"}
                        },
                        "required": ["path"]
                    })),
                    output_schema: None,
                    risk_level: Some("safe".to_string()),
                    requires_approval: false,
                    rate_limit: None,
                },
            ],
            env: HashMap::new(),
            tags: vec!["storage".to_string(), "files".to_string()],
            risk_level: "low".to_string(),
            requires_approval: false,
        },
        McpServerConfig {
            name: "github".to_string(),
            description: "Interact with GitHub repositories".to_string(),
            server_type: "github".to_string(),
            enabled: true,
            connection: McpConnectionConfig::Stdio {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-github".to_string(),
                ],
                cwd: None,
            },
            auth: Some(McpAuthConfig::Bearer {
                token_env: "GITHUB_TOKEN".to_string(),
            }),
            tools: vec![McpToolConfig {
                name: "search_repositories".to_string(),
                description: "Search GitHub repositories".to_string(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                })),
                output_schema: None,
                risk_level: Some("safe".to_string()),
                requires_approval: false,
                rate_limit: Some(30),
            }],
            env: HashMap::new(),
            tags: vec!["vcs".to_string(), "github".to_string()],
            risk_level: "medium".to_string(),
            requires_approval: false,
        },
    ]
}

pub type McpDirectoryScanner = McpCsvLoader;
pub type McpDirectoryScanResult = McpLoadResult;
pub type McpScanError = McpLoadError;
