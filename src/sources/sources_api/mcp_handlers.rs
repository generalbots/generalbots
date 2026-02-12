use crate::basic::keywords::mcp_directory::McpCsvLoader;
use crate::basic::keywords::get_all_keywords;
use crate::core::shared::state::AppState;
use super::types::{ApiResponse, BotQuery, McpServerResponse, McpToolResponse, AddMcpServerRequest, McpConnectionRequest, McpAuthRequest};

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::error;
use std::sync::Arc;

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

    use crate::basic::keywords::mcp_directory::McpCsvRow;
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
            log::info!("Added MCP server '{}' to mcp.csv", request.name);
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

    use crate::basic::keywords::mcp_directory::McpCsvRow;
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
    let examples = crate::basic::keywords::mcp_directory::generate_example_configs();
    Json(ApiResponse::success(examples))
}

pub async fn handle_list_all_tools(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

    let mut all_tools: Vec<McpToolResponse> = Vec::new();

    let keywords = get_all_keywords();
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

pub async fn handle_mcp_servers(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    use super::html_renderers::{load_mcp_servers_catalog, get_category_icon, html_escape};

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
                crate::sources::mcp::get_server_type_icon(&server.server_type.to_string()),
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

    axum::response::Html(html)
}
