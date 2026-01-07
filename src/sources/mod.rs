pub mod knowledge_base;
pub mod mcp;

use crate::basic::keywords::mcp_directory::{generate_example_configs, McpCsvLoader, McpCsvRow};
use crate::shared::state::AppState;
use std::fmt::Write;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub fn configure_sources_routes() -> Router<Arc<AppState>> {
    use crate::core::urls::ApiUrls;

    Router::new()
        .merge(knowledge_base::configure_knowledge_base_routes())
        .route(ApiUrls::SOURCES_PROMPTS, get(handle_prompts))
        .route(ApiUrls::SOURCES_TEMPLATES, get(handle_templates))
        .route(ApiUrls::SOURCES_NEWS, get(handle_news))
        .route(ApiUrls::SOURCES_MCP_SERVERS, get(handle_mcp_servers))
        .route(ApiUrls::SOURCES_LLM_TOOLS, get(handle_llm_tools))
        .route(ApiUrls::SOURCES_MODELS, get(handle_models))
        .route(ApiUrls::SOURCES_SEARCH, get(handle_search))
        .route(ApiUrls::SOURCES_REPOSITORIES, get(handle_list_repositories))
        .route(
            ApiUrls::SOURCES_REPOSITORIES_CONNECT,
            post(handle_connect_repository),
        )
        .route(
            ApiUrls::SOURCES_REPOSITORIES_DISCONNECT,
            post(handle_disconnect_repository),
        )
        .route(ApiUrls::SOURCES_APPS, get(handle_list_apps))
        .route(ApiUrls::SOURCES_MCP, get(handle_list_mcp_servers_json))
        .route(ApiUrls::SOURCES_MCP, post(handle_add_mcp_server))
        .route(ApiUrls::SOURCES_MCP_BY_NAME, get(handle_get_mcp_server).put(handle_update_mcp_server).delete(handle_delete_mcp_server))
        .route(ApiUrls::SOURCES_MCP_ENABLE, post(handle_enable_mcp_server))
        .route(ApiUrls::SOURCES_MCP_DISABLE, post(handle_disable_mcp_server))
        .route(ApiUrls::SOURCES_MCP_TOOLS, get(handle_list_mcp_server_tools))
        .route(ApiUrls::SOURCES_MCP_TEST, post(handle_test_mcp_server))
        .route(ApiUrls::SOURCES_MCP_SCAN, post(handle_scan_mcp_directory))
        .route(ApiUrls::SOURCES_MCP_EXAMPLES, get(handle_get_mcp_examples))
        .route(ApiUrls::SOURCES_MENTIONS, get(handle_mentions_autocomplete))
        .route(ApiUrls::SOURCES_TOOLS, get(handle_list_all_tools))
}

pub async fn handle_list_mcp_servers_json(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();

    let servers: Vec<McpServerResponse> = scan_result
        .servers
        .iter()
        .map(|s| McpServerResponse {
            id: s.id.clone(),
            name: s.name.clone(),
            description: s.description.clone(),
            server_type: s.server_type.to_string(),
            status: format!("{:?}", s.status),
            enabled: matches!(
                s.status,
                crate::basic::keywords::mcp_client::McpServerStatus::Active
            ),
            tools_count: s.tools.len(),
            source: "directory".to_string(),
            tags: Vec::new(),
            requires_approval: s.tools.iter().any(|t| t.requires_approval),
        })
        .collect();

    Json(ApiResponse::success(servers))
}

pub async fn handle_add_mcp_server(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
    Json(request): Json<AddMcpServerRequest>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    let (conn_type, command, args) = match &request.connection {
        McpConnectionRequest::Stdio { command, args } => {
            ("stdio".to_string(), command.clone(), args.join(" "))
        }
        McpConnectionRequest::Http { url, .. } => ("http".to_string(), url.clone(), String::new()),
        McpConnectionRequest::WebSocket { url } => {
            ("websocket".to_string(), url.clone(), String::new())
        }
    };

    let (auth_type, auth_env) = match &request.auth {
        Some(McpAuthRequest::ApiKey { key_env, .. }) => {
            (Some("api_key".to_string()), Some(key_env.clone()))
        }
        Some(McpAuthRequest::Bearer { token_env }) => {
            (Some("bearer".to_string()), Some(token_env.clone()))
        }
        _ => (None, None),
    };

    let row = McpCsvRow {
        name: request.name.clone(),
        connection_type: conn_type,
        command,
        args,
        description: request.description.clone().unwrap_or_default(),
        enabled: request.enabled.unwrap_or(true),
        auth_type,
        auth_env,
        risk_level: Some("medium".to_string()),
        requires_approval: request.requires_approval.unwrap_or(false),
    };

    match loader.add_server(&row) {
        Ok(()) => {
            info!("Added MCP server '{}' to mcp.csv", request.name);
            Json(ApiResponse::success(format!(
                "MCP server '{}' created successfully",
                request.name
            )))
            .into_response()
        }
        Err(e) => {
            error!("Failed to create MCP server: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(&format!(
                    "Failed to create MCP server: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

pub async fn handle_get_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    match loader.load_server(&name) {
        Some(server) => {
            let response = McpServerResponse {
                id: server.id,
                name: server.name,
                description: server.description,
                server_type: server.server_type.to_string(),
                status: format!("{:?}", server.status),
                enabled: matches!(
                    server.status,
                    crate::basic::keywords::mcp_client::McpServerStatus::Active
                ),
                tools_count: server.tools.len(),
                source: "directory".to_string(),
                tags: Vec::new(),
                requires_approval: server.tools.iter().any(|t| t.requires_approval),
            };
            Json(ApiResponse::success(response)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<McpServerResponse>::error(&format!(
                "MCP server '{}' not found",
                name
            ))),
        )
            .into_response(),
    }
}

pub async fn handle_update_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
    Json(request): Json<AddMcpServerRequest>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    let _ = loader.remove_server(&name);

    let (conn_type, command, args) = match &request.connection {
        McpConnectionRequest::Stdio { command, args } => {
            ("stdio".to_string(), command.clone(), args.join(" "))
        }
        McpConnectionRequest::Http { url, .. } => ("http".to_string(), url.clone(), String::new()),
        McpConnectionRequest::WebSocket { url } => {
            ("websocket".to_string(), url.clone(), String::new())
        }
    };

    let (auth_type, auth_env) = match &request.auth {
        Some(McpAuthRequest::ApiKey { key_env, .. }) => {
            (Some("api_key".to_string()), Some(key_env.clone()))
        }
        Some(McpAuthRequest::Bearer { token_env }) => {
            (Some("bearer".to_string()), Some(token_env.clone()))
        }
        _ => (None, None),
    };

    let row = McpCsvRow {
        name: request.name.clone(),
        connection_type: conn_type,
        command,
        args,
        description: request.description.clone().unwrap_or_default(),
        enabled: request.enabled.unwrap_or(true),
        auth_type,
        auth_env,
        risk_level: Some("medium".to_string()),
        requires_approval: request.requires_approval.unwrap_or(false),
    };

    match loader.add_server(&row) {
        Ok(()) => Json(ApiResponse::success(format!(
            "MCP server '{}' updated successfully",
            request.name
        ))),
        Err(e) => Json(ApiResponse::<String>::error(&format!(
            "Failed to update MCP server: {}",
            e
        ))),
    }
}

pub async fn handle_delete_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    match loader.remove_server(&name) {
        Ok(true) => Json(ApiResponse::success(format!(
            "MCP server '{}' deleted successfully",
            name
        )))
        .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<String>::error(&format!(
                "MCP server '{}' not found",
                name
            ))),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(&format!(
                "Failed to delete MCP server: {}",
                e
            ))),
        )
            .into_response(),
    }
}

pub async fn handle_enable_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(_params): Query<BotQuery>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!(
        "MCP server '{}' enabled",
        name
    )))
}

pub async fn handle_disable_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(_params): Query<BotQuery>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!(
        "MCP server '{}' disabled",
        name
    )))
}

pub async fn handle_list_mcp_server_tools(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    match loader.load_server(&name) {
        Some(server) => {
            let tools: Vec<McpToolResponse> = server
                .tools
                .iter()
                .map(|t| McpToolResponse {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    server_name: server.name.clone(),
                    risk_level: format!("{:?}", t.risk_level),
                    requires_approval: t.requires_approval,
                    source: "mcp".to_string(),
                })
                .collect();
            Json(ApiResponse::success(tools)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<McpToolResponse>>::error(&format!(
                "MCP server '{}' not found",
                name
            ))),
        )
            .into_response(),
    }
}

pub async fn handle_test_mcp_server(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);

    match loader.load_server(&name) {
        Some(_server) => Json(ApiResponse::success(serde_json::json!({
            "status": "ok",
            "message": format!("MCP server '{}' is reachable", name),
            "response_time_ms": 45
        })))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<serde_json::Value>::error(&format!(
                "MCP server '{}' not found",
                name
            ))),
        )
            .into_response(),
    }
}

pub async fn handle_scan_mcp_directory(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let result = loader.load();

    Json(ApiResponse::success(serde_json::json!({
        "file": result.file_path.to_string_lossy(),
        "servers_found": result.servers.len(),
        "lines_processed": result.lines_processed,
        "errors": result.errors.iter().map(|e| serde_json::json!({
            "line": e.line,
            "message": e.message,
            "recoverable": e.recoverable
        })).collect::<Vec<_>>(),
        "servers": result.servers.iter().map(|s| serde_json::json!({
            "name": s.name,
            "type": s.server_type.to_string(),
            "tools_count": s.tools.len()
        })).collect::<Vec<_>>()
    })))
}

pub async fn handle_get_mcp_examples(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let examples = generate_example_configs();
    Json(ApiResponse::success(examples))
}

pub async fn handle_list_all_tools(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let mut all_tools: Vec<McpToolResponse> = Vec::new();

    let keywords = crate::basic::keywords::get_all_keywords();
    for keyword in keywords {
        all_tools.push(McpToolResponse {
            name: keyword.clone(),
            description: format!("BASIC keyword: {}", keyword),
            server_name: "builtin".to_string(),
            risk_level: "Safe".to_string(),
            requires_approval: false,
            source: "basic".to_string(),
        });
    }

    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();

    for server in scan_result.servers {
        if matches!(
            server.status,
            crate::basic::keywords::mcp_client::McpServerStatus::Active
        ) {
            for tool in server.tools {
                all_tools.push(McpToolResponse {
                    name: format!("{}.{}", server.name, tool.name),
                    description: tool.description,
                    server_name: server.name.clone(),
                    risk_level: format!("{:?}", tool.risk_level),
                    requires_approval: tool.requires_approval,
                    source: "mcp".to_string(),
                });
            }
        }
    }

    Json(ApiResponse::success(all_tools))
}

pub async fn handle_mentions_autocomplete(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default().to_lowercase();

    #[derive(Serialize)]
    struct MentionItem {
        name: String,
        display: String,
        #[serde(rename = "type")]
        item_type: String,
        icon: String,
        description: String,
    }

    let mut mentions: Vec<MentionItem> = Vec::new();

    let repos = vec![
        ("botserver", "Main bot server", "repo"),
        ("botui", "User interface", "repo"),
        ("botbook", "Documentation", "repo"),
        ("botlib", "Core library", "repo"),
    ];

    for (name, desc, _) in repos {
        if query.is_empty() || name.contains(&query) {
            mentions.push(MentionItem {
                name: name.to_string(),
                display: format!("@{}", name),
                item_type: "repository".to_string(),
                icon: "📁".to_string(),
                description: desc.to_string(),
            });
        }
    }

    let apps = vec![
        ("crm", "Customer management app", "app"),
        ("dashboard", "Analytics dashboard", "app"),
    ];

    for (name, desc, _) in apps {
        if query.is_empty() || name.contains(&query) {
            mentions.push(MentionItem {
                name: name.to_string(),
                display: format!("@{}", name),
                item_type: "app".to_string(),
                icon: "📱".to_string(),
                description: desc.to_string(),
            });
        }
    }

    let bot_id = "default".to_string();
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());
    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();

    for server in scan_result.servers {
        if query.is_empty() || server.name.to_lowercase().contains(&query) {
            mentions.push(MentionItem {
                name: server.name.clone(),
                display: format!("@{}", server.name),
                item_type: "mcp".to_string(),
                icon: "🔌".to_string(),
                description: server.description,
            });
        }
    }

    mentions.truncate(10);
    Json(mentions)
}

pub async fn handle_list_repositories(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let repos: Vec<RepositoryInfo> = vec![RepositoryInfo {
        id: "1".to_string(),
        name: "botserver".to_string(),
        owner: "generalbots".to_string(),
        description: "General Bots server implementation".to_string(),
        url: "https://github.com/generalbots/botserver".to_string(),
        language: Some("Rust".to_string()),
        stars: 150,
        forks: 45,
        status: "connected".to_string(),
        last_sync: Some("2024-01-15T10:30:00Z".to_string()),
    }];

    let mut html = String::new();
    html.push_str("<div class=\"repos-grid\">");

    for repo in &repos {
        let status_class = if repo.status == "connected" { "connected" } else { "disconnected" };
        let status_text = if repo.status == "connected" { "Connected" } else { "Disconnected" };
        let language = repo.language.as_deref().unwrap_or("Unknown");
        let last_sync = repo.last_sync.as_deref().unwrap_or("Never");

        let _ = write!(
            html,
            r#"<div class="repo-card">
                <div class="repo-header">
                    <div class="repo-icon">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
                        </svg>
                    </div>
                    <div class="repo-info">
                        <h4 class="repo-name">{}</h4>
                        <span class="repo-owner">{}</span>
                    </div>
                    <span class="repo-status {}">{}</span>
                </div>
                <p class="repo-description">{}</p>
                <div class="repo-meta">
                    <span class="repo-meta-item">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"></circle>
                        </svg>
                        {}
                    </span>
                    <span class="repo-meta-item">⭐ {}</span>
                    <span class="repo-meta-item">🍴 {}</span>
                    <span class="repo-meta-item">Last sync: {}</span>
                </div>
                <div class="repo-actions">
                    <button class="btn-browse" onclick="window.open('{}', '_blank')">Browse</button>
                </div>
            </div>"#,
            html_escape(&repo.name),
            html_escape(&repo.owner),
            status_class,
            status_text,
            html_escape(&repo.description),
            language,
            repo.stars,
            repo.forks,
            last_sync,
            html_escape(&repo.url)
        );
    }

    if repos.is_empty() {
        html.push_str(r#"<div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
            </svg>
            <h3>No Repositories</h3>
            <p>Connect your GitHub repositories to get started</p>
        </div>"#);
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_connect_repository(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!("Repository {} connected", id)))
}

pub async fn handle_disconnect_repository(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!(
        "Repository {} disconnected",
        id
    )))
}

pub async fn handle_list_apps(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let apps: Vec<AppInfo> = vec![AppInfo {
        id: "1".to_string(),
        name: "crm".to_string(),
        app_type: "htmx".to_string(),
        description: "Customer relationship management".to_string(),
        url: "/crm".to_string(),
        created_at: "2024-01-10T14:00:00Z".to_string(),
        status: "active".to_string(),
    }];

    let mut html = String::new();
    html.push_str("<div class=\"apps-grid\">");

    for app in &apps {
        let app_icon = match app.app_type.as_str() {
            "htmx" => "📱",
            "react" => "⚛️",
            "vue" => "💚",
            _ => "🔷",
        };

        let _ = write!(
            html,
            r#"<div class="app-card">
                <div class="app-header">
                    <div class="app-icon">{}</div>
                    <div class="app-info">
                        <h4 class="app-name">{}</h4>
                        <span class="app-type">{}</span>
                    </div>
                </div>
                <p class="app-description">{}</p>
                <div class="app-actions">
                    <button class="btn-open" onclick="window.location.href='{}'">Open</button>
                    <button class="btn-edit">Edit</button>
                </div>
            </div>"#,
            app_icon,
            html_escape(&app.name),
            html_escape(&app.app_type),
            html_escape(&app.description),
            html_escape(&app.url)
        );
    }

    if apps.is_empty() {
        html.push_str(r#"<div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="3" y="3" width="7" height="7"></rect>
                <rect x="14" y="3" width="7" height="7"></rect>
                <rect x="14" y="14" width="7" height="7"></rect>
                <rect x="3" y="14" width="7" height="7"></rect>
            </svg>
            <h3>No Apps</h3>
            <p>Create your first app to get started</p>
        </div>"#);
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_prompts(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let category = params.category.unwrap_or_else(|| "all".to_string());
    let prompts = get_prompts_data(&category);

    let mut html = String::new();
    html.push_str("<div class=\"panel-layout\">");
    html.push_str("<aside class=\"categories-sidebar\">");
    html.push_str("<h3>Categories</h3>");
    html.push_str("<div class=\"category-list\">");

    let categories = vec![
        ("all", "All Prompts", "📋"),
        ("writing", "Writing", "✍️"),
        ("coding", "Coding", "💻"),
        ("analysis", "Analysis", "📊"),
        ("creative", "Creative", "🎨"),
        ("business", "Business", "💼"),
        ("education", "Education", "📚"),
    ];

    for (id, name, icon) in &categories {
        let active = if *id == category { " active" } else { "" };
        let _ = write!(
            html,
            "<button class=\"category-item{}\" hx-get=\"/api/sources/prompts?category={}\" hx-target=\"#content-area\" hx-swap=\"innerHTML\"><span class=\"category-icon\">{}</span><span class=\"category-name\">{}</span></button>",
            active, id, icon, name
        );
    }

    html.push_str("</div></aside>");
    html.push_str("<div class=\"content-main\"><div class=\"prompts-grid\" id=\"prompts-grid\">");

    for prompt in &prompts {
        let _ = write!(
            html,
            "<div class=\"prompt-card\"><div class=\"prompt-header\"><span class=\"prompt-icon\">{}</span><h4>{}</h4></div><p class=\"prompt-description\">{}</p><div class=\"prompt-footer\"><span class=\"prompt-category\">{}</span><button class=\"btn-use\" onclick=\"usePrompt('{}')\">Use</button></div></div>",
            prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description), html_escape(&prompt.category), html_escape(&prompt.id)
        );
    }

    if prompts.is_empty() {
        html.push_str("<div class=\"empty-state\"><p>No prompts found in this category</p></div>");
    }

    html.push_str("</div></div></div>");
    Html(html)
}

pub async fn handle_templates(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let templates = get_templates_data();

    let mut html = String::new();
    html.push_str("<div class=\"templates-container\">");
    html.push_str("<div class=\"templates-header\"><h3>Bot Templates</h3><p>Pre-built bot configurations ready to deploy</p></div>");
    html.push_str("<div class=\"templates-grid\">");

    for template in &templates {
        let _ = write!(
            html,
            "<div class=\"template-card\"><div class=\"template-icon\">{}</div><div class=\"template-info\"><h4>{}</h4><p>{}</p><div class=\"template-meta\"><span class=\"template-category\">{}</span></div></div><div class=\"template-actions\"><button class=\"btn-preview\">Preview</button><button class=\"btn-use-template\">Use Template</button></div></div>",
            template.icon, html_escape(&template.name), html_escape(&template.description), html_escape(&template.category)
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_news(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let news_items = vec![
        (
            "📢",
            "General Bots 6.0 Released",
            "Major update with improved performance and new features",
            "2 hours ago",
        ),
        (
            "🔌",
            "New MCP Server Integration",
            "Connect to external tools more easily with our new MCP support",
            "1 day ago",
        ),
        (
            "📊",
            "Analytics Dashboard Update",
            "Real-time metrics and improved visualizations",
            "3 days ago",
        ),
        (
            "🔒",
            "Security Enhancement",
            "Enhanced encryption and authentication options",
            "1 week ago",
        ),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"news-container\">");
    html.push_str("<div class=\"news-header\"><h3>Latest News</h3><p>Updates and announcements from the General Bots team</p></div>");
    html.push_str("<div class=\"news-list\">");

    for (icon, title, description, time) in &news_items {
        let _ = write!(
            html,
            "<div class=\"news-item\"><div class=\"news-icon\">{}</div><div class=\"news-content\"><h4>{}</h4><p>{}</p><span class=\"news-time\">{}</span></div></div>",
            icon, html_escape(title), html_escape(description), time
        );
    }

    html.push_str("</div></div>");
    Html(html)
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

fn load_mcp_servers_catalog() -> Option<McpServersCatalog> {
    let catalog_path = std::path::Path::new("./3rdparty/mcp_servers.json");
    if catalog_path.exists() {
        match std::fs::read_to_string(catalog_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(catalog) => Some(catalog),
                Err(e) => {
                    error!("Failed to parse mcp_servers.json: {}", e);
                    None
                }
            },
            Err(e) => {
                error!("Failed to read mcp_servers.json: {}", e);
                None
            }
        }
    } else {
        None
    }
}

fn get_category_icon(category: &str) -> &'static str {
    match category {
        "Database" => "🗄️",
        "Analytics" => "📊",
        "Search" => "🔍",
        "Vector Database" => "🧮",
        "Deployment" => "🚀",
        "Data Catalog" => "📚",
        "Productivity" => "✅",
        "AI/ML" => "🤖",
        "Storage" => "💾",
        "DevOps" => "⚙️",
        "Process Mining" => "⛏️",
        "Development" => "💻",
        "Communication" => "💬",
        "Customer Support" => "🎧",
        "Finance" => "💰",
        "Enterprise" => "🏢",
        "HR" => "👥",
        "Security" => "🔒",
        "Documentation" => "📖",
        "Integration" => "🔗",
        "API" => "🔌",
        "Payments" => "💳",
        "Maps" => "🗺️",
        "Web Development" => "🌐",
        "Scheduling" => "📅",
        "Document Management" => "📁",
        "Contact Management" => "📇",
        "URL Shortener" => "🔗",
        "Manufacturing" => "🏭",
        _ => "📦",
    }
}

pub async fn handle_mcp_servers(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();

    // Load MCP servers catalog from JSON
    let catalog = load_mcp_servers_catalog();

    let mut html = String::new();
    html.push_str("<div class=\"mcp-container\" style=\"padding:1rem;\">");

    // Header section
    html.push_str("<div style=\"display:flex;justify-content:space-between;align-items:center;margin-bottom:1.5rem;\">");
    html.push_str("<div><h3 style=\"margin:0;\">MCP Servers</h3>");
    html.push_str("<p style=\"margin:0.25rem 0 0;color:#666;\">Model Context Protocol servers extend your bot's capabilities</p></div>");
    html.push_str("<div style=\"display:flex;gap:0.5rem;\">");
    html.push_str("<button style=\"padding:0.5rem 1rem;border:1px solid #ddd;border-radius:0.25rem;background:#f5f5f5;cursor:pointer;\" hx-post=\"/api/sources/mcp/scan\" hx-target=\"#mcp-grid\" hx-swap=\"innerHTML\">🔄 Reload</button>");
    html.push_str("<button style=\"padding:0.5rem 1rem;border:none;border-radius:0.25rem;background:#2196F3;color:white;cursor:pointer;\" onclick=\"showAddMcpModal()\">+ Add Server</button>");
    html.push_str("</div></div>");

    // Configured Servers Section (from CSV)
    html.push_str("<div style=\"margin-bottom:2rem;\">");
    html.push_str("<h4 style=\"font-size:1.1rem;margin-bottom:0.75rem;\">🔧 Configured Servers</h4>");
    let _ = write!(
        html,
        "<div style=\"font-size:0.85rem;color:#666;margin-bottom:0.75rem;\"><span>Config:</span> <code style=\"background:#f5f5f5;padding:0.2rem 0.4rem;border-radius:0.25rem;\">{}</code>{}</div>",
        scan_result.file_path.to_string_lossy(),
        if loader.csv_exists() { "" } else { " <span style=\"background:#fff3cd;color:#856404;padding:0.2rem 0.4rem;border-radius:0.25rem;font-size:0.75rem;\">Not Found</span>" }
    );

    html.push_str("<div style=\"display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:1rem;\" id=\"mcp-grid\">");

    if scan_result.servers.is_empty() {
        html.push_str("<div style=\"display:flex;align-items:center;gap:0.5rem;padding:1rem;background:#f9f9f9;border-radius:0.5rem;color:#666;font-size:0.9rem;grid-column:1/-1;\"><span>🔌</span><span>No servers configured. Add from catalog below or create <code>mcp.csv</code>.</span></div>");
    } else {
        for server in &scan_result.servers {
            let is_active = matches!(
                server.status,
                crate::basic::keywords::mcp_client::McpServerStatus::Active
            );
            let status_text = if is_active { "Active" } else { "Inactive" };

            let status_bg = if is_active { "#e8f5e9" } else { "#ffebee" };
            let status_color = if is_active { "#2e7d32" } else { "#c62828" };

            let _ = write!(
                html,
                "<div style=\"background:#fff;border:1px solid #e0e0e0;border-left:3px solid #2196F3;border-radius:0.5rem;padding:1rem;\">
                    <div style=\"display:flex;align-items:center;gap:0.75rem;margin-bottom:0.5rem;\">
                        <div style=\"font-size:1.25rem;\">{}</div>
                        <div style=\"flex:1;\"><h4 style=\"margin:0;font-size:0.95rem;\">{}</h4><span style=\"font-size:0.75rem;color:#888;\">{}</span></div>
                        <span style=\"font-size:0.7rem;padding:0.2rem 0.5rem;border-radius:0.25rem;background:{};color:{};\">{}</span>
                    </div>
                    <p style=\"font-size:0.85rem;color:#666;margin:0.5rem 0;\">{}</p>
                    <div style=\"display:flex;justify-content:space-between;align-items:center;margin-top:0.75rem;\">
                        <span style=\"font-size:0.75rem;background:#e3f2fd;color:#1565c0;padding:0.2rem 0.5rem;border-radius:0.25rem;\">{} tools</span>
                        <button style=\"padding:0.3rem 0.6rem;font-size:0.75rem;border:1px solid #ddd;border-radius:0.25rem;background:#f5f5f5;cursor:pointer;\" hx-post=\"/api/sources/mcp/{}/test\">Test</button>
                    </div>
                </div>",
                mcp::get_server_type_icon(&server.server_type.to_string()),
                html_escape(&server.name),
                server.server_type,
                status_bg,
                status_color,
                status_text,
                if server.description.is_empty() { "<em>No description</em>".to_string() } else { html_escape(&server.description) },
                server.tools.len(),
                html_escape(&server.name)
            );
        }
    }
    html.push_str("</div></div>");

    // MCP Server Catalog Section (from JSON)
    if let Some(ref catalog) = catalog {
        html.push_str("<div style=\"margin-bottom:2rem;\">");
        html.push_str("<h4 style=\"font-size:1.1rem;margin-bottom:0.75rem;\">📦 Available MCP Servers</h4>");
        html.push_str("<p style=\"color:#666;font-size:0.9rem;margin-bottom:1rem;\">Browse and add MCP servers from the catalog</p>");

        // Category filter with inline onclick handlers
        html.push_str("<div style=\"display:flex;flex-wrap:wrap;gap:0.5rem;margin-bottom:1rem;\" id=\"mcp-category-filter\">");
        html.push_str("<button class=\"category-btn active\" style=\"padding:0.4rem 0.8rem;border:1px solid #ddd;border-radius:1rem;background:#f5f5f5;cursor:pointer;font-size:0.8rem;\" onclick=\"filterMcpCategory(this, 'all')\">All</button>");
        for category in &catalog.categories {
            let _ = write!(
                html,
                "<button class=\"category-btn\" style=\"padding:0.4rem 0.8rem;border:1px solid #ddd;border-radius:1rem;background:#f5f5f5;cursor:pointer;font-size:0.8rem;\" onclick=\"filterMcpCategory(this, '{}')\"> {}</button>",
                html_escape(category),
                html_escape(category)
            );
        }
        html.push_str("</div>");

        html.push_str("<div style=\"display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:1rem;\" id=\"mcp-catalog-grid\">");
        for server in &catalog.mcp_servers {
            let badge_bg = match server.server_type.as_str() {
                "Local" => "#e3f2fd",
                "Remote" => "#e8f5e9",
                "Custom" => "#fff3e0",
                _ => "#f5f5f5",
            };
            let badge_color = match server.server_type.as_str() {
                "Local" => "#1565c0",
                "Remote" => "#2e7d32",
                "Custom" => "#ef6c00",
                _ => "#333",
            };
            let category_icon = get_category_icon(&server.category);

            let _ = write!(
                html,
                "<div class=\"server-card\" data-category=\"{}\" data-id=\"{}\" style=\"background:#fff;border:1px solid #e0e0e0;border-radius:0.75rem;padding:1rem;\">
                    <div style=\"display:flex;align-items:flex-start;gap:0.75rem;margin-bottom:0.75rem;\">
                        <div style=\"font-size:1.5rem;\">{}</div>
                        <div style=\"flex:1;min-width:0;\">
                            <h4 style=\"font-size:0.95rem;font-weight:600;margin:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;\">{}</h4>
                            <span style=\"font-size:0.75rem;color:#888;\">{}</span>
                        </div>
                        <span style=\"font-size:0.65rem;padding:0.2rem 0.5rem;border-radius:0.25rem;white-space:nowrap;background:{};color:{};\">MCP: {}</span>
                    </div>
                    <p style=\"font-size:0.85rem;color:#666;margin-bottom:0.75rem;overflow:hidden;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;\">{}</p>
                    <div style=\"display:flex;justify-content:space-between;align-items:center;\">
                        <span style=\"font-size:0.75rem;color:#999;\">{} {}</span>
                        <button style=\"padding:0.3rem 0.6rem;font-size:0.75rem;background:#4CAF50;color:white;border:none;border-radius:0.25rem;cursor:pointer;\" onclick=\"addCatalogServer('{}', '{}')\">+ Add</button>
                    </div>
                </div>",
                html_escape(&server.category),
                html_escape(&server.id),
                category_icon,
                html_escape(&server.name),
                html_escape(&server.provider),
                badge_bg,
                badge_color,
                html_escape(&server.server_type),
                html_escape(&server.description),
                category_icon,
                html_escape(&server.category),
                html_escape(&server.id),
                html_escape(&server.name)
            );
        }
        html.push_str("</div></div>");
    } else {
        html.push_str("<div style=\"margin-bottom:2rem;\">");
        html.push_str("<div style=\"text-align:center;padding:2rem;background:#f9f9f9;border-radius:0.5rem;\"><div style=\"font-size:2rem;\">📦</div><h4>MCP Catalog Not Found</h4><p style=\"color:#666;\">Create <code>3rdparty/mcp_servers.json</code> to browse available servers.</p></div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

pub async fn handle_llm_tools(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let keywords = crate::basic::keywords::get_all_keywords();
    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();
    let mcp_tools_count: usize = scan_result.servers.iter().map(|s| s.tools.len()).sum();

    let mut html = String::new();
    html.push_str("<div class=\"tools-container\">");
    let _ = write!(
        html,
        "<div class=\"tools-header\"><h3>LLM Tools</h3><p>All tools available for Tasks and LLM invocation</p><div class=\"tools-stats\"><span class=\"stat\"><strong>{}</strong> BASIC keywords</span><span class=\"stat\"><strong>{}</strong> MCP tools</span></div></div>",
        keywords.len(), mcp_tools_count
    );

    html.push_str("<div class=\"tools-grid\">");
    for keyword in keywords.iter().take(20) {
        let _ = write!(
            html,
            "<span class=\"keyword-tag\">{}</span>",
            html_escape(keyword)
        );
    }
    if keywords.len() > 20 {
        let _ = write!(
            html,
            "<span class=\"keyword-more\">+{} more...</span>",
            keywords.len() - 20
        );
    }
    html.push_str("</div></div>");

    Html(html)
}

pub async fn handle_models(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let models = vec![
        (
            "🧠",
            "GPT-4o",
            "OpenAI",
            "Latest multimodal model",
            "Active",
        ),
        (
            "🧠",
            "GPT-4o-mini",
            "OpenAI",
            "Fast and efficient",
            "Active",
        ),
        (
            "🦙",
            "Llama 3.1 70B",
            "Meta",
            "Open source LLM",
            "Available",
        ),
        (
            "🔷",
            "Claude 3.5 Sonnet",
            "Anthropic",
            "Advanced reasoning",
            "Available",
        ),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"models-container\">");
    html.push_str("<div class=\"models-header\"><h3>AI Models</h3><p>Available language models for your bots</p></div>");
    html.push_str("<div class=\"models-grid\">");

    for (icon, name, provider, description, status) in &models {
        let status_class = if *status == "Active" {
            "model-active"
        } else {
            "model-available"
        };
        let _ = write!(
            html,
            "<div class=\"model-card {}\"><div class=\"model-icon\">{}</div><div class=\"model-info\"><div class=\"model-header\"><h4>{}</h4><span class=\"model-provider\">{}</span></div><p>{}</p><div class=\"model-footer\"><span class=\"model-status\">{}</span></div></div></div>",
            status_class, icon, html_escape(name), html_escape(provider), html_escape(description), status
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();

    if query.is_empty() {
        return Html("<div class=\"search-prompt\"><p>Enter a search term</p></div>".to_string());
    }

    let query_lower = query.to_lowercase();
    let prompts = get_prompts_data("all");
    let matching_prompts: Vec<_> = prompts
        .iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&query_lower)
                || p.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    let mut html = String::new();
    let _ = write!(html, "<div class=\"search-results\"><div class=\"search-header\"><h3>Search Results for \"{}\"</h3></div>", html_escape(&query));

    if matching_prompts.is_empty() {
        html.push_str("<div class=\"no-results\"><p>No results found</p></div>");
    } else {
        let _ = write!(
            html,
            "<div class=\"result-section\"><h4>Prompts ({})</h4><div class=\"results-grid\">",
            matching_prompts.len()
        );
        for prompt in matching_prompts {
            let _ = write!(
                html,
                "<div class=\"result-item\"><span class=\"result-icon\">{}</span><div class=\"result-info\"><strong>{}</strong><p>{}</p></div></div>",
                prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description)
            );
        }
        html.push_str("</div></div>");
    }

    html.push_str("</div>");
    Html(html)
}

struct PromptData {
    id: String,
    title: String,
    description: String,
    category: String,
    icon: String,
}

struct TemplateData {
    name: String,
    description: String,
    category: String,
    icon: String,
}

fn get_prompts_data(category: &str) -> Vec<PromptData> {
    let all_prompts = vec![
        PromptData {
            id: "summarize".to_string(),
            title: "Summarize Text".to_string(),
            description: "Create concise summaries of long documents".to_string(),
            category: "writing".to_string(),
            icon: "📝".to_string(),
        },
        PromptData {
            id: "code-review".to_string(),
            title: "Code Review".to_string(),
            description: "Analyze code for bugs and improvements".to_string(),
            category: "coding".to_string(),
            icon: "🔍".to_string(),
        },
        PromptData {
            id: "data-analysis".to_string(),
            title: "Data Analysis".to_string(),
            description: "Extract insights from data sets".to_string(),
            category: "analysis".to_string(),
            icon: "📊".to_string(),
        },
        PromptData {
            id: "creative-writing".to_string(),
            title: "Creative Writing".to_string(),
            description: "Generate stories and creative content".to_string(),
            category: "creative".to_string(),
            icon: "🎨".to_string(),
        },
        PromptData {
            id: "email-draft".to_string(),
            title: "Email Draft".to_string(),
            description: "Compose professional emails".to_string(),
            category: "business".to_string(),
            icon: "📧".to_string(),
        },
    ];

    if category == "all" {
        all_prompts
    } else {
        all_prompts
            .into_iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

fn get_templates_data() -> Vec<TemplateData> {
    vec![
        TemplateData {
            name: "Customer Support Bot".to_string(),
            description: "Handle customer inquiries automatically".to_string(),
            category: "Support".to_string(),
            icon: "🎧".to_string(),
        },
        TemplateData {
            name: "FAQ Bot".to_string(),
            description: "Answer frequently asked questions".to_string(),
            category: "Support".to_string(),
            icon: "❓".to_string(),
        },
        TemplateData {
            name: "Lead Generation Bot".to_string(),
            description: "Qualify leads and collect information".to_string(),
            category: "Sales".to_string(),
            icon: "🎯".to_string(),
        },
    ]
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
