use axum::{
    body::Body,
    extract::{
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
        OriginalUri, Query, State,
    },
    http::{Request, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
#[cfg(feature = "embed-ui")]
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite,
    tungstenite::protocol::Message as TungsteniteMessage,
};
#[cfg(not(feature = "embed-ui"))]
use tower_http::services::{ServeDir, ServeFile};

#[cfg(feature = "embed-ui")]
#[derive(RustEmbed)]
#[folder = "ui"]
struct Assets;

use crate::shared::AppState;

const SUITE_DIRS: &[&str] = &[
    "js",
    "css",
    "public",
    "assets",
    "partials",
    // Core & Support
    "settings",
    "auth",
    "about",
    // Core Apps
    #[cfg(feature = "drive")]
    "drive",
    #[cfg(feature = "chat")]
    "chat",
    #[cfg(feature = "mail")]
    "mail",
    #[cfg(feature = "tasks")]
    "tasks",
    #[cfg(feature = "calendar")]
    "calendar",
    #[cfg(feature = "meet")]
    "meet",
    // Document Apps
    #[cfg(feature = "paper")]
    "paper",
    #[cfg(feature = "sheet")]
    "sheet",
    #[cfg(feature = "slides")]
    "slides",
    #[cfg(feature = "docs")]
    "docs",
    // Research & Learning
    #[cfg(feature = "research")]
    "research",
    #[cfg(feature = "sources")]
    "sources",
    #[cfg(feature = "learn")]
    "learn",
    // Analytics
    #[cfg(feature = "analytics")]
    "analytics",
    #[cfg(feature = "dashboards")]
    "dashboards",
    #[cfg(feature = "monitoring")]
    "monitoring",
    // Admin & Tools
    #[cfg(feature = "admin")]
    "admin",
    #[cfg(feature = "attendant")]
    "attendant",
    #[cfg(feature = "tools")]
    "tools",
    // Media
    #[cfg(feature = "video")]
    "video",
    #[cfg(feature = "player")]
    "player",
    #[cfg(feature = "canvas")]
    "canvas",
    // Social
    #[cfg(feature = "social")]
    "social",
    #[cfg(feature = "people")]
    "people",
    #[cfg(feature = "people")]
    "crm",
    #[cfg(feature = "tickets")]
    "tickets",
    // Business
    #[cfg(feature = "billing")]
    "billing",
    #[cfg(feature = "products")]
    "products",
    // Development
    #[cfg(feature = "designer")]
    "designer",
    #[cfg(feature = "workspace")]
    "workspace",
    #[cfg(feature = "project")]
    "project",
    #[cfg(feature = "goals")]
    "goals",
    "vibe",
];

const ROOT_FILES: &[&str] = &[
    "designer.html",
    "designer.css",
    "designer.js",
    "editor.html",
    "editor.css",
    "editor.js",
    "home.html",
    "base.html",
    "base-layout.html",
    "base-layout.css",
    "desktop.html",
    "default.gbui",
    "single.gbui",
];

pub async fn index(OriginalUri(uri): OriginalUri) -> Response {
    let path = uri.path();

    // Check if path contains static asset directories - serve them directly
    let path_lower = path.to_lowercase();
    if path_lower.contains("/js/")
        || path_lower.contains("/css/")
        || path_lower.contains("/vendor/")
        || path_lower.contains("/assets/")
        || path_lower.contains("/public/")
        || path_lower.contains("/partials/")
        || path_lower.contains("/crm/")
        || path_lower.contains("/tasks/")
        || path_lower.contains("/drive/")
        || path_lower.contains("/terminal/")
        || path_lower.contains("/browser/")
        || path_lower.contains("/editor/")
        || path_lower.ends_with(".js")
        || path_lower.ends_with(".css")
        || path_lower.ends_with(".html")
        || path_lower.ends_with(".png")
        || path_lower.ends_with(".jpg")
        || path_lower.ends_with(".jpeg")
        || path_lower.ends_with(".gif")
        || path_lower.ends_with(".svg")
        || path_lower.ends_with(".ico")
        || path_lower.ends_with(".woff")
        || path_lower.ends_with(".woff2")
        || path_lower.ends_with(".ttf")
        || path_lower.ends_with(".eot")
        || path_lower.ends_with(".mp4")
        || path_lower.ends_with(".webm")
        || path_lower.ends_with(".mp3")
        || path_lower.ends_with(".wav")
    {
        // Remove bot name prefix if present (e.g., /edu/suite/js/file.js -> suite/js/file.js)
        let path_parts: Vec<&str> = path.split('/').collect();
        let fs_path = if path_parts.len() > 1 {
            let mut start_idx = 1;
            let known_dirs = ["suite", "js", "css", "vendor", "assets", "public", "partials", "settings", "auth", "about", "drive", "chat", "tasks", "admin", "mail", "calendar", "meet", "docs", "sheet", "slides", "paper", "research", "sources", "learn", "analytics", "dashboards", "monitoring", "people", "crm", "tickets", "billing", "products", "video", "player", "canvas", "social", "project", "goals", "workspace", "designer", "vibe"];
            let suite_dirs = ["drive", "chat", "tasks", "admin", "mail", "calendar", "meet", "docs", "sheet", "slides", "paper", "research", "sources", "learn", "analytics", "dashboards", "monitoring", "people", "crm", "tickets", "billing", "products", "video", "player", "canvas", "social", "project", "goals", "workspace", "designer", "vibe"];

            // If the first segment is already a known dir (suite subdirectory), prepend "suite/"
            if known_dirs.contains(&path_parts[1]) {
                if suite_dirs.contains(&path_parts[1]) {
                    // e.g., /crm/crm.css -> suite/crm/crm.css
                    format!("suite/{}", path_parts[1..].join("/"))
                } else {
                    // e.g., /js/..., /suite/..., /css/...
                    path_parts[1..].join("/")
                }
            } else {
                // Special case: /auth/suite/* should map to suite/* (auth is a route, not a directory)
                if path_parts.get(1) == Some(&"auth") && path_parts.get(2) == Some(&"suite") {
                    start_idx = 2;
                }
                // Skip bot name if present (first segment is not a known dir, second segment is)
                else if path_parts.len() > start_idx + 1
                    && known_dirs.contains(&path_parts[start_idx + 1])
                {
                    start_idx += 1;
                }

                path_parts[start_idx..].join("/")
            }
        } else {
            path.to_string()
        };

        let full_path = get_ui_root().join(&fs_path);

        info!("index: Serving static file: {} -> {:?} (fs_path: {})", path, full_path, fs_path);

        #[cfg(feature = "embed-ui")]
        {
            let asset_path = fs_path.trim_start_matches('/');
            if let Some(content) = Assets::get(asset_path) {
                let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
                return ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response();
            }
        }

        #[cfg(not(feature = "embed-ui"))]
        {
            if let Ok(bytes) = tokio::fs::read(&full_path).await {
                let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
                return (StatusCode::OK, [("content-type", mime.as_ref())], bytes).into_response();
            }
        }

        warn!("index: Static file not found: {} -> {:?}", path, full_path);
        return StatusCode::NOT_FOUND.into_response();
    }

    let path_parts: Vec<&str> = path.split('/').collect();
    let bot_name = path_parts
        .iter()
        .rev()
        .find(|part| {
            !part.is_empty()
                && **part != "chat"
                && **part != "app"
                && **part != "ws"
                && **part != "ui"
                && **part != "api"
                && **part != "auth"
                && **part != "suite"
                && !part.ends_with(".js")
                && !part.ends_with(".css")
        })
        .map(|s| s.to_string());

    info!(
        "index: Extracted bot_name: {:?} from path: {}",
        bot_name,
        path
    );
    serve_suite(bot_name).await.into_response()
}

pub fn get_ui_root() -> PathBuf {
    #[cfg(feature = "embed-ui")]
    {
        PathBuf::from("ui")
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let candidates = [
            "ui",
            "botui/ui",
            "../botui/ui",
            "../../botui/ui",
            "../../../botui/ui",
        ];

        for path_str in candidates {
            let path = PathBuf::from(path_str);
            if path.exists() {
                info!("Found UI root at: {:?}", path);
                return path;
            }
        }

        let default = PathBuf::from("ui");
        error!(
            "Could not find 'ui' directory in candidates: {:?}. Defaulting to 'ui' (CWD: {:?})",
            candidates,
            std::env::current_dir()
        );
        default
    }
}

pub async fn serve_minimal() -> impl IntoResponse {
    let html_res = {
        #[cfg(feature = "embed-ui")]
        {
            Assets::get("minimal/index.html")
                .map(|f| String::from_utf8(f.data.into_owned()).map_err(|e| e.to_string()))
                .unwrap_or(Err("Asset not found".to_string()))
        }
        #[cfg(not(feature = "embed-ui"))]
        {
            let path = get_ui_root().join("minimal/index.html");
            fs::read_to_string(&path).map_err(|e| {
                format!(
                    "Failed to read {:?} (CWD: {:?}): {}",
                    path,
                    std::env::current_dir(),
                    e
                )
            })
        }
    };

    match html_res {
        Ok(html) => (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], Html(html)),
        Err(e) => {
            error!("Failed to load minimal UI: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                Html("Failed to load minimal interface".to_string()),
            )
        }
    }
}

pub async fn serve_suite(bot_name: Option<String>) -> impl IntoResponse {
    let raw_html_res = {
        #[cfg(feature = "embed-ui")]
        {
            match Assets::get("suite/desktop.html") {
                Some(f) => String::from_utf8(f.data.into_owned()).map_err(|e| e.to_string()),
                None => {
                    let path = get_ui_root().join("suite/desktop.html");
                    log::warn!("Asset .suite/desktop.html. not found in embedded binary, falling back to filesystem: {:?}", path);
                    fs::read_to_string(&path).map_err(|e| {
                        format!(
                            "Asset not found in binary AND failed to read {:?} (CWD: {:?}): {}",
                            path,
                            std::env::current_dir(),
                            e
                        )
                    })
                }
            }
        }
        #[cfg(not(feature = "embed-ui"))]
        {
            let path = get_ui_root().join("suite/desktop.html");
            fs::read_to_string(&path).map_err(|e| {
                format!(
                    "Failed to read {:?} (CWD: {:?}): {}",
                    path,
                    std::env::current_dir(),
                    e
                )
            })
        }
    };

    match raw_html_res {
        Ok(raw_html) => {
            let mut html = raw_html;
            let _ = &mut html; // Suppress unused_mut if no features are disabled

            // Inject base tag and bot_name into the page
            if let Some(head_end) = html.find("</head>") {
                // Check if bot_name is actually an auth page (login.html, register.html, etc.)
                // These are not actual bots, so we should use "/" as base href
                let is_auth_page = bot_name.as_ref()
                    .map(|n| n.ends_with(".html") || n == "login" || n == "register" || n == "forgot-password" || n == "reset-password")
                    .unwrap_or(false);

                // Set base href to include bot context if present (e.g., /edu/)
                // But NOT for auth pages - those use root
                let base_href = if is_auth_page {
                    "/".to_string()
                } else if let Some(ref name) = bot_name {
                    format!("/{}/", name)
                } else {
                    "/".to_string()
                };
                let base_tag = format!(r#"<base href="{}">"#, base_href);
                html.insert_str(head_end, &base_tag);

                // Only inject bot_name script for actual bots, not auth pages
                if !is_auth_page {
                    if let Some(name) = bot_name {
                        info!("serve_suite: Injecting bot_name '{}' into page with base href='{}'", name, base_href);
                        let bot_script = format!(
                            r#"<script>window.__INITIAL_BOT_NAME__ = "{}";</script>"#,
                            &name
                        );
                        html.insert_str(head_end + base_tag.len(), &bot_script);
                        info!("serve_suite: Successfully injected base tag and bot_name script");
                    } else {
                        info!("serve_suite: Successfully injected base tag (no bot_name)");
                    }
                } else {
                    info!("serve_suite: Auth page detected, skipping bot_name injection (base href='{}')", base_href);
                }
            } else {
                error!("serve_suite: Failed to find </head> tag to inject content");
            }

            // Core Apps
            #[cfg(not(feature = "chat"))]
            {
                html = remove_section(&html, "chat");
            }
            #[cfg(not(feature = "mail"))]
            {
                html = remove_section(&html, "mail");
            }
            #[cfg(not(feature = "calendar"))]
            {
                html = remove_section(&html, "calendar");
            }
            #[cfg(not(feature = "drive"))]
            {
                html = remove_section(&html, "drive");
            }
            #[cfg(not(feature = "tasks"))]
            {
                html = remove_section(&html, "tasks");
            }
            #[cfg(not(feature = "meet"))]
            {
                html = remove_section(&html, "meet");
            }

            // Documents
            #[cfg(not(feature = "docs"))]
            {
                html = remove_section(&html, "docs");
            }
            #[cfg(not(feature = "sheet"))]
            {
                html = remove_section(&html, "sheet");
            }
            #[cfg(not(feature = "slides"))]
            {
                html = remove_section(&html, "slides");
            }
            #[cfg(not(feature = "paper"))]
            {
                html = remove_section(&html, "paper");
            }

            // Research
            #[cfg(not(feature = "research"))]
            {
                html = remove_section(&html, "research");
            }
            #[cfg(not(feature = "sources"))]
            {
                html = remove_section(&html, "sources");
            }
            #[cfg(not(feature = "learn"))]
            {
                html = remove_section(&html, "learn");
            }

            // Analytics
            #[cfg(not(feature = "analytics"))]
            {
                html = remove_section(&html, "analytics");
            }
            #[cfg(not(feature = "dashboards"))]
            {
                html = remove_section(&html, "dashboards");
            }
            #[cfg(not(feature = "monitoring"))]
            {
                html = remove_section(&html, "monitoring");
            }

            // Business
            #[cfg(not(feature = "people"))]
            {
                html = remove_section(&html, "people");
                html = remove_section(&html, "crm");
            }
            #[cfg(not(feature = "billing"))]
            {
                html = remove_section(&html, "billing");
            }
            #[cfg(not(feature = "products"))]
            {
                html = remove_section(&html, "products");
            }
            #[cfg(not(feature = "tickets"))]
            {
                html = remove_section(&html, "tickets");
            }

            // Media
            #[cfg(not(feature = "video"))]
            {
                html = remove_section(&html, "video");
            }
            #[cfg(not(feature = "player"))]
            {
                html = remove_section(&html, "player");
            }
            #[cfg(not(feature = "canvas"))]
            {
                html = remove_section(&html, "canvas");
            }

            // Social & Project
            #[cfg(not(feature = "social"))]
            {
                html = remove_section(&html, "social");
            }
            #[cfg(not(feature = "project"))]
            {
                html = remove_section(&html, "project");
            }
            #[cfg(not(feature = "goals"))]
            {
                html = remove_section(&html, "goals");
            }
            #[cfg(not(feature = "workspace"))]
            {
                html = remove_section(&html, "workspace");
            }

            // Admin/Tools
            #[cfg(not(feature = "admin"))]
            {
                html = remove_section(&html, "admin");
            }
            // Mapped security to tools feature
            #[cfg(not(feature = "tools"))]
            {
                html = remove_section(&html, "security");
            }
            #[cfg(not(feature = "attendant"))]
            {
                html = remove_section(&html, "attendant");
            }
            #[cfg(not(feature = "designer"))]
            {
                html = remove_section(&html, "designer");
            }
            #[cfg(not(feature = "editor"))]
            {
                html = remove_section(&html, "editor");
            }
            #[cfg(not(feature = "settings"))]
            {
                html = remove_section(&html, "settings");
            }

            (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], Html(html))
        }
        Err(e) => {
            error!("Failed to load suite UI: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                Html("Failed to load suite interface".to_string()),
            )
        }
    }
}

pub fn remove_section(html: &str, section: &str) -> String {
    let start_marker = format!("<!-- SECTION:{} -->", section);
    let end_marker = format!("<!-- ENDSECTION:{} -->", section);

    let mut result = String::with_capacity(html.len());
    let mut current_pos = 0;

    // Process multiple occurrences of the section
    while let Some(start_idx) = html[current_pos..].find(&start_marker) {
        let abs_start = current_pos + start_idx;
        // Append content up to the marker
        result.push_str(&html[current_pos..abs_start]);

        // Find end marker
        if let Some(end_idx) = html[abs_start..].find(&end_marker) {
            // Skip past the end marker
            current_pos = abs_start + end_idx + end_marker.len();
        } else {
            // No end marker? This shouldn't happen with our script,
            // but if it does, just skip the start marker and continue
            // or consume everything?
            // Safety: Skip start marker only
            current_pos = abs_start + start_marker.len();
        }
    }

    // Append remaining content
    result.push_str(&html[current_pos..]);
    result
}

async fn health(State(state): State<AppState>) -> (StatusCode, axum::Json<serde_json::Value>) {
    let commit = option_env!("BOTUI_COMMIT").unwrap_or("unknown");
    if state.health_check().await {
        (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "healthy",
                "service": "botui",
                "mode": "web",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "status": "unhealthy",
                "service": "botui",
                "error": "botserver unreachable",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    }
}

async fn api_health(State(state): State<AppState>) -> (StatusCode, axum::Json<serde_json::Value>) {
    let commit = option_env!("BOTUI_COMMIT").unwrap_or("unknown");
    if state.health_check().await {
        (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "ok",
                "botserver": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "status": "error",
                "botserver": "unhealthy",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    }
}

fn extract_app_context(headers: &axum::http::HeaderMap, path: &str) -> Option<String> {
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            if let Some(start) = referer_str.find("/apps/") {
                let after_apps = &referer_str[start + 6..];
                if let Some(end) = after_apps.find('/') {
                    return Some(after_apps[..end].to_string());
                } else if !after_apps.is_empty() {
                    return Some(after_apps.to_string());
                }
            }
        }
    }

    if let Some(after_apps) = path.strip_prefix("/apps/") {
        if let Some(end) = after_apps.find('/') {
            return Some(after_apps[..end].to_string());
        }
    }

    None
}

async fn proxy_api(
    State(state): State<AppState>,
    original_uri: OriginalUri,
    req: Request<Body>,
) -> Response<Body> {
    let path = original_uri.path();
    let query = original_uri
        .query()
        .map_or_else(String::new, |q| format!("?{q}"));
    let method = req.method().clone();
    let headers = req.headers().clone();

    let app_context = extract_app_context(&headers, path);

    let target_url = format!("{}{path}{query}", state.client.base_url());
    debug!("Proxying {method} {path} to {target_url} (app: {app_context:?})");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let mut proxy_req = client.request(method.clone(), &target_url);

    for (name, value) in &headers {
        if name != "host" {
            if let Ok(v) = value.to_str() {
                proxy_req = proxy_req.header(name.as_str(), v);
            }
        }
    }

    if let Some(app) = app_context {
        proxy_req = proxy_req.header("X-App-Context", app);
    }

    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {e}");
            return build_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read request body",
            );
        }
    };

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes.to_vec());
    }

    match proxy_req.send().await {
        Ok(resp) => build_proxy_response(resp).await,
        Err(e) => {
            error!("Proxy request failed: {e}");
            build_error_response(StatusCode::BAD_GATEWAY, &format!("Proxy error: {e}"))
        }
    }
}

fn build_error_response(status: StatusCode, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.to_string()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to build error response"))
                .unwrap_or_default()
        })
}

async fn build_proxy_response(resp: reqwest::Response) -> Response<Body> {
    let status = resp.status();
    let headers = resp.headers().clone();

    match resp.bytes().await {
        Ok(body) => {
            let mut response = Response::builder().status(status);

            for (name, value) in &headers {
                response = response.header(name, value);
            }

            response.body(Body::from(body)).unwrap_or_else(|_| {
                build_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to build response",
                )
            })
        }
        Err(e) => {
            error!("Failed to read response body: {e}");
            build_error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Failed to read response: {e}"),
            )
        }
    }
}

fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(api_health))
        .route("/client-error", axum::routing::post(handle_client_error))
        .fallback(any(proxy_api))
}

#[derive(Debug, Deserialize)]
struct WsQuery {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    user_id: String,
    bot_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClientError {
    message: String,
    stack: Option<String>,
    source: String,
    url: String,
    user_agent: String,
    timestamp: String,
}

async fn handle_client_error(Json(error): Json<ClientError>) -> impl IntoResponse {
    warn!(
        "CLIENT:{}: {} at {} ({}) - {}",
        error.source.to_uppercase(),
        error.message,
        error.url,
        error.timestamp,
        error.user_agent
    );

    if let Some(stack) = &error.stack {
        if !stack.is_empty() {
            warn!("CLIENT:STACK: {}", stack);
        }
    }

    StatusCode::OK
}

#[derive(Debug, Default, Deserialize)]
struct OptionalWsQuery {
    task_id: Option<String>,
}

async fn ws_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    // Extract bot_name from URL path (e.g., /edu, /chat/edu)
    let path_parts: Vec<&str> = uri.path().split('/').collect();
    let bot_name = params
        .bot_name
        .filter(|name| name != "ws" && !name.is_empty())
        .or_else(|| {
            // Try to extract from path like /edu or /app/edu
            path_parts
                .iter()
                .find(|part| {
                    !part.is_empty() && **part != "chat" && **part != "app" && **part != "ws"
                })
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "default".to_string());

    let params_with_bot = WsQuery {
        bot_name: Some(bot_name),
        ..params
    };

    ws.on_upgrade(move |socket| handle_ws_proxy(socket, state, params_with_bot))
}

async fn handle_ws_proxy(
    client_socket: WebSocket,
    state: AppState,
    params: WsQuery,
) {
    let bot_name = params.bot_name.unwrap_or_else(|| "default".to_string());
    let backend_url = format!(
        "{}/ws/{}?session_id={}&user_id={}",
        state
            .client
            .base_url()
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
        bot_name,
        params.session_id,
        params.user_id
    );

    info!("Proxying WebSocket to: {backend_url}");

    let Ok(tls_connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
    else {
        error!("Failed to build TLS connector for WebSocket proxy");
        return;
    };

    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);

    let backend_result =
        connect_async_tls_with_config(&backend_url, None, false, Some(connector)).await;

    let backend_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    > = match backend_result {
        Ok((socket, _)) => socket,
        Err(e) => {
            error!("Failed to connect to backend WebSocket: {e}");
            return;
        }
    };

    info!("Connected to backend WebSocket");

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    let client_to_backend = async {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(AxumMessage::Text(text)) => {
                    if backend_tx.send(TungsteniteMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Binary(data)) => {
                    if backend_tx.send(TungsteniteMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Ping(data)) => {
                    if backend_tx.send(TungsteniteMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Pong(data)) => {
                    if backend_tx.send(TungsteniteMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Close(_)) | Err(_) => break,
            }
        }
    };

    let backend_to_client = async {
        while let Some(msg) = backend_rx.next().await {
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    if client_tx.send(AxumMessage::Text(text)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Binary(data)) => {
                    if client_tx.send(AxumMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Ping(data)) => {
                    if client_tx.send(AxumMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Pong(data)) => {
                    if client_tx.send(AxumMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Close(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    };

    tokio::select! {
        () = client_to_backend => info!("Client connection closed"),
        () = backend_to_client => info!("Backend connection closed"),
    }
}

async fn ws_task_progress_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<OptionalWsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_task_progress_ws_proxy(socket, state, params))
}

async fn handle_task_progress_ws_proxy(
    client_socket: WebSocket,
    state: AppState,
    params: OptionalWsQuery,
) {
    let mut backend_url = format!(
        "{}/ws/task-progress",
        state
            .client
            .base_url()
            .replace("https://", "wss://")
            .replace("http://", "ws://"),
    );

    if let Some(task_id) = &params.task_id {
        backend_url = format!("{}/{}", backend_url, task_id);
    }

    info!("Proxying task-progress WebSocket to: {backend_url}");

    let Ok(tls_connector) = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
    else {
        error!("Failed to build TLS connector for task-progress");
        return;
    };

    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);

    let backend_result =
        connect_async_tls_with_config(&backend_url, None, false, Some(connector)).await;

    let backend_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    > = match backend_result {
        Ok((socket, _)) => socket,
        Err(e) => {
            error!("Failed to connect to backend task-progress WebSocket: {e}");
            return;
        }
    };

    info!("Connected to backend task-progress WebSocket");

    let (mut client_tx, mut client_rx) = client_socket.split();
    let (mut backend_tx, mut backend_rx) = backend_socket.split();

    let client_to_backend = async {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(AxumMessage::Text(text)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Text(text)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Binary(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Binary(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Ping(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Ping(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Pong(data)) => {
                    let res: Result<(), tungstenite::Error> =
                        backend_tx.send(TungsteniteMessage::Pong(data)).await;
                    if res.is_err() {
                        break;
                    }
                }
                Ok(AxumMessage::Close(_)) | Err(_) => break,
            }
        }
    };

    let backend_to_client = async {
        while let Some(msg) =
            backend_rx.next().await as Option<Result<TungsteniteMessage, tungstenite::Error>>
        {
            match msg {
                Ok(TungsteniteMessage::Text(text)) => {
                    // Log manifest_update messages for debugging
                    let is_manifest = text.contains("manifest_update");
                    if is_manifest {
                    } else if text.contains("task_progress") {
                        debug!("[WS_PROXY] Forwarding task_progress to client");
                    }
                    match client_tx.send(AxumMessage::Text(text)).await {
                        Ok(()) => {
                            if is_manifest {
                            }
                        }
                        Err(e) => {
                            error!("[WS_PROXY] Failed to send message to client: {:?}", e);
                            break;
                        }
                    }
                }
                Ok(TungsteniteMessage::Binary(data)) => {
                    if client_tx.send(AxumMessage::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Ping(data)) => {
                    if client_tx.send(AxumMessage::Ping(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Pong(data)) => {
                    if client_tx.send(AxumMessage::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(TungsteniteMessage::Close(_)) | Err(_) => break,
                Ok(_) => {}
            }
        }
    };

    tokio::select! {
        () = client_to_backend => info!("Task-progress client connection closed"),
        () = backend_to_client => info!("Task-progress backend connection closed"),
    }
}

fn create_ws_router() -> Router<AppState> {
    Router::new()
        .route("/task-progress", get(ws_task_progress_proxy))
        .route("/task-progress/:task_id", get(ws_task_progress_proxy))
        .route("/autotask", get(ws_task_progress_proxy))
        .fallback(any(ws_proxy))
}

fn create_apps_router() -> Router<AppState> {
    Router::new().fallback(any(proxy_api))
}

fn create_ui_router() -> Router<AppState> {
    Router::new().fallback(any(proxy_api))
}

async fn serve_favicon() -> impl IntoResponse {
    #[cfg(feature = "embed-ui")]
    {
        match Assets::get("suite/public/favicon.ico") {
            Some(content) => (
                StatusCode::OK,
                [("content-type", "image/x-icon")],
                content.data,
            )
                .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
    #[cfg(not(feature = "embed-ui"))]
    {
        let favicon_path = get_ui_root().join("suite/public/favicon.ico");
        match tokio::fs::read(&favicon_path).await {
            Ok(bytes) => {
                (StatusCode::OK, [("content-type", "image/x-icon")], bytes).into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

#[cfg(feature = "embed-ui")]
async fn handle_embedded_asset(
    axum::extract::Path((dir, path)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if !SUITE_DIRS.contains(&dir.as_str()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let asset_path = format!("suite/{}/{}", dir, path);
    match Assets::get(&asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(feature = "embed-ui")]
async fn handle_embedded_root_asset(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> impl IntoResponse {
    if !ROOT_FILES.contains(&filename.as_str()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let asset_path = format!("suite/{}", filename);
    match Assets::get(&asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(feature = "embed-ui")]
async fn handle_auth_asset(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let normalized_path = path.strip_prefix('/').unwrap_or(&path);
    let asset_path = format!("suite/auth/{}", normalized_path);
    match Assets::get(&asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serve login page at clean /login route (hides physical path /suite/auth/login.html)
async fn serve_login() -> impl IntoResponse {
    #[cfg(feature = "embed-ui")]
    {
        let asset_path = "suite/auth/login.html";
        match Assets::get(asset_path) {
            Some(content) => {
                let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
                (
                    [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                    content.data,
                )
                    .into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let login_path = get_ui_root().join("suite/auth/login.html");
        match tokio::fs::read(&login_path).await {
            Ok(content) => {
                let mime = mime_guess::from_path(&login_path).first_or_octet_stream();
                (
                    [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                    content,
                )
                    .into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

/// Serve logout page at clean /logout route (hides physical path /suite/auth/logout.html)
async fn serve_logout() -> impl IntoResponse {
    #[cfg(feature = "embed-ui")]
    {
        let asset_path = "suite/auth/logout.html";
        match Assets::get(asset_path) {
            Some(content) => {
                let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
                (
                    [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                    content.data,
                )
                    .into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let logout_path = get_ui_root().join("suite/auth/logout.html");
        match tokio::fs::read(&logout_path).await {
            Ok(content) => {
                let mime = mime_guess::from_path(&logout_path).first_or_octet_stream();
                (
                    [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                    content,
                )
                    .into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

fn add_static_routes(router: Router<AppState>, _suite_path: &Path) -> Router<AppState> {
    #[cfg(feature = "embed-ui")]
    {
        let mut r = router.route("/suite/:dir/*path", get(handle_embedded_asset));

        // Add root files only under /suite/
        for file in ROOT_FILES {
            r = r.route(&format!("/suite/{}", file), get(handle_embedded_root_asset));
        }
        r
    }
    #[cfg(not(feature = "embed-ui"))]
    {
        let mut r = router;
        // Only serve suite directories under /suite/{dir} path
        // Root-level paths (/:dir) are handled by fallback to support bot prefixes
        for dir in SUITE_DIRS {
            let path = _suite_path.join(dir);
            info!("Adding route for /suite/{} -> {:?}", dir, path);
            r = r.nest_service(&format!("/suite/{dir}"), ServeDir::new(path.clone()));
        }

        for file in ROOT_FILES {
            let path = _suite_path.join(file);
            r = r.nest_service(&format!("/suite/{}", file), ServeFile::new(path));
        }
        r
    }
}

pub fn configure_router() -> Router {
    let suite_path = get_ui_root().join("suite");
    let state = AppState::new();

    let mut router = Router::new()
        .route("/health", get(health))
        .route("/favicon.ico", get(serve_favicon))
        .route("/login", get(serve_login))
        .route("/logout", get(serve_logout))
        .nest("/api", create_api_router())
        .nest("/ui", create_ui_router())
        .nest("/ws", create_ws_router())
        .nest("/apps", create_apps_router())
        .route("/", get(index))
        .route("/minimal", get(serve_minimal))
        .route("/suite", get(serve_suite));

    #[cfg(not(feature = "embed-ui"))]
    {
        router = router.nest_service("/auth", ServeDir::new(suite_path.join("auth")));
    }

    #[cfg(feature = "embed-ui")]
    {
        router = router.route("/auth/*path", get(handle_auth_asset));
    }

    router = add_static_routes(router, &suite_path);

    router.fallback(get(index)).with_state(state)
}
