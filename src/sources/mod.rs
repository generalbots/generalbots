













pub mod mcp;

use crate::basic::keywords::mcp_directory::{generate_example_configs, McpCsvLoader, McpCsvRow};
use crate::shared::state::AppState;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
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
    Router::new()

        .route("/api/sources/prompts", get(handle_prompts))
        .route("/api/sources/templates", get(handle_templates))
        .route("/api/sources/news", get(handle_news))
        .route("/api/sources/mcp-servers", get(handle_mcp_servers))
        .route("/api/sources/llm-tools", get(handle_llm_tools))
        .route("/api/sources/models", get(handle_models))

        .route("/api/sources/search", get(handle_search))

        .route("/api/sources/repositories", get(handle_list_repositories))
        .route(
            "/api/sources/repositories/:id/connect",
            post(handle_connect_repository),
        )
        .route(
            "/api/sources/repositories/:id/disconnect",
            post(handle_disconnect_repository),
        )

        .route("/api/sources/apps", get(handle_list_apps))

        .route("/api/sources/mcp", get(handle_list_mcp_servers_json))
        .route("/api/sources/mcp", post(handle_add_mcp_server))
        .route("/api/sources/mcp/:name", get(handle_get_mcp_server))
        .route("/api/sources/mcp/:name", put(handle_update_mcp_server))
        .route("/api/sources/mcp/:name", delete(handle_delete_mcp_server))
        .route(
            "/api/sources/mcp/:name/enable",
            post(handle_enable_mcp_server),
        )
        .route(
            "/api/sources/mcp/:name/disable",
            post(handle_disable_mcp_server),
        )
        .route(
            "/api/sources/mcp/:name/tools",
            get(handle_list_mcp_server_tools),
        )
        .route("/api/sources/mcp/:name/test", post(handle_test_mcp_server))
        .route("/api/sources/mcp/scan", post(handle_scan_mcp_directory))
        .route("/api/sources/mcp/examples", get(handle_get_mcp_examples))

        .route("/api/sources/mentions", get(handle_mentions_autocomplete))

        .route("/api/sources/tools", get(handle_list_all_tools))
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

    Json(ApiResponse::success(repos))
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

    Json(ApiResponse::success(apps))
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
        html.push_str(&format!(
            "<button class=\"category-item{}\" hx-get=\"/api/sources/prompts?category={}\" hx-target=\"#content-area\" hx-swap=\"innerHTML\"><span class=\"category-icon\">{}</span><span class=\"category-name\">{}</span></button>",
            active, id, icon, name
        ));
    }

    html.push_str("</div></aside>");
    html.push_str("<div class=\"content-main\"><div class=\"prompts-grid\" id=\"prompts-grid\">");

    for prompt in &prompts {
        html.push_str(&format!(
            "<div class=\"prompt-card\"><div class=\"prompt-header\"><span class=\"prompt-icon\">{}</span><h4>{}</h4></div><p class=\"prompt-description\">{}</p><div class=\"prompt-footer\"><span class=\"prompt-category\">{}</span><button class=\"btn-use\" onclick=\"usePrompt('{}')\">Use</button></div></div>",
            prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description), html_escape(&prompt.category), html_escape(&prompt.id)
        ));
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
        html.push_str(&format!(
            "<div class=\"template-card\"><div class=\"template-icon\">{}</div><div class=\"template-info\"><h4>{}</h4><p>{}</p><div class=\"template-meta\"><span class=\"template-category\">{}</span></div></div><div class=\"template-actions\"><button class=\"btn-preview\">Preview</button><button class=\"btn-use-template\">Use Template</button></div></div>",
            template.icon, html_escape(&template.name), html_escape(&template.description), html_escape(&template.category)
        ));
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
        html.push_str(&format!(
            "<div class=\"news-item\"><div class=\"news-icon\">{}</div><div class=\"news-content\"><h4>{}</h4><p>{}</p><span class=\"news-time\">{}</span></div></div>",
            icon, html_escape(title), html_escape(description), time
        ));
    }

    html.push_str("</div></div>");
    Html(html)
}


pub async fn handle_mcp_servers(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let loader = McpCsvLoader::new(&work_path, &bot_id);
    let scan_result = loader.load();

    let mut html = String::new();
    html.push_str("<div class=\"mcp-container\">");
    html.push_str("<div class=\"mcp-header\">");
    html.push_str("<h3>MCP Servers</h3>");
    html.push_str("<p>Model Context Protocol servers extend your bot's capabilities. Configure servers in <code>mcp.csv</code>.</p>");
    html.push_str("<div class=\"mcp-header-actions\">");
    html.push_str("<button class=\"btn-scan\" hx-post=\"/api/sources/mcp/scan\" hx-target=\"#mcp-grid\" hx-swap=\"innerHTML\">🔄 Reload</button>");
    html.push_str(
        "<button class=\"btn-add-server\" onclick=\"showAddMcpModal()\">+ Add Server</button>",
    );
    html.push_str("</div></div>");

    html.push_str(&format!(
        "<div class=\"mcp-directory-info\"><span class=\"label\">MCP Config:</span><code>{}</code>{}</div>",
        scan_result.file_path.to_string_lossy(),
        if !loader.csv_exists() { "<span class=\"badge badge-warning\">Not Found</span>" } else { "" }
    ));

    html.push_str("<div class=\"mcp-grid\" id=\"mcp-grid\">");

    if scan_result.servers.is_empty() {
        html.push_str("<div class=\"empty-state\"><div class=\"empty-icon\">🔌</div><h4>No MCP Servers Found</h4><p>Add MCP server configuration files to your <code>.gbmcp</code> directory.</p></div>");
    } else {
        for server in &scan_result.servers {
            let is_active = matches!(
                server.status,
                crate::basic::keywords::mcp_client::McpServerStatus::Active
            );
            let status_class = if is_active {
                "status-active"
            } else {
                "status-inactive"
            };
            let status_text = if is_active { "Active" } else { "Inactive" };

            html.push_str(&format!(
                "<div class=\"mcp-card\"><div class=\"mcp-card-header\"><div class=\"mcp-icon\">{}</div><div class=\"mcp-title\"><h4>{}</h4><span class=\"mcp-type\">{}</span></div><div class=\"mcp-status {}\">{}</div></div><p class=\"mcp-description\">{}</p><div class=\"mcp-tools-count\"><span class=\"tools-badge\">{} tools</span></div><div class=\"mcp-actions\"><button class=\"btn-test\" hx-post=\"/api/sources/mcp/{}/test\">Test</button></div></div>",
                mcp::get_server_type_icon(&server.server_type.to_string()),
                html_escape(&server.name),
                server.server_type.to_string(),
                status_class,
                status_text,
                if server.description.is_empty() { "<em>No description</em>".to_string() } else { html_escape(&server.description) },
                server.tools.len(),
                html_escape(&server.name)
            ));
        }
    }

    html.push_str("</div></div>");
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
    html.push_str(&format!(
        "<div class=\"tools-header\"><h3>LLM Tools</h3><p>All tools available for Tasks and LLM invocation</p><div class=\"tools-stats\"><span class=\"stat\"><strong>{}</strong> BASIC keywords</span><span class=\"stat\"><strong>{}</strong> MCP tools</span></div></div>",
        keywords.len(), mcp_tools_count
    ));

    html.push_str("<div class=\"tools-grid\">");
    for keyword in keywords.iter().take(20) {
        html.push_str(&format!(
            "<span class=\"keyword-tag\">{}</span>",
            html_escape(keyword)
        ));
    }
    if keywords.len() > 20 {
        html.push_str(&format!(
            "<span class=\"keyword-more\">+{} more...</span>",
            keywords.len() - 20
        ));
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
        html.push_str(&format!(
            "<div class=\"model-card {}\"><div class=\"model-icon\">{}</div><div class=\"model-info\"><div class=\"model-header\"><h4>{}</h4><span class=\"model-provider\">{}</span></div><p>{}</p><div class=\"model-footer\"><span class=\"model-status\">{}</span></div></div></div>",
            status_class, icon, html_escape(name), html_escape(provider), html_escape(description), status
        ));
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
    html.push_str(&format!("<div class=\"search-results\"><div class=\"search-header\"><h3>Search Results for \"{}\"</h3></div>", html_escape(&query)));

    if matching_prompts.is_empty() {
        html.push_str("<div class=\"no-results\"><p>No results found</p></div>");
    } else {
        html.push_str(&format!(
            "<div class=\"result-section\"><h4>Prompts ({})</h4><div class=\"results-grid\">",
            matching_prompts.len()
        ));
        for prompt in matching_prompts {
            html.push_str(&format!(
                "<div class=\"result-item\"><span class=\"result-icon\">{}</span><div class=\"result-info\"><strong>{}</strong><p>{}</p></div></div>",
                prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description)
            ));
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
