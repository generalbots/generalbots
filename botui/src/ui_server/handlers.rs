use axum::{extract::{State, OriginalUri, Query}, response::{Response, IntoResponse, Redirect}, http::HeaderMap};
use crate::ui_server::{AppState, get_ui_root, SuiteQueryParams, Assets};
use log::{info, warn};
use std::fs;
use std::path::PathBuf;

// Include the handlers from the previously extracted content
pub async fn index(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    headers: axum::http::HeaderMap,
) -> Response {
    let path = uri.path();
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

    if let Some(ref bot) = bot_name {
        // Access Check
        let mut has_token = false;
        if let Some(cookie_header) = headers.get(axum::http::header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                if cookie_str.contains("gb-access-token") {
                    has_token = true;
                }
            }
        }
        
        if has_token && bot != "default" {
            let target_url = format!("{}/api/bots/{}/access", state.client.base_url(), bot);
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
                
            let mut req = client.get(&target_url);
            for (k, v) in headers.iter() {
                if k != axum::http::header::HOST {
                    req = req.header(k, v);
                }
            }
            
            if let Ok(resp) = req.send().await {
                if resp.status() == axum::http::StatusCode::FORBIDDEN || resp.status() == axum::http::StatusCode::UNAUTHORIZED {
                    info!("index: Access denied for bot {}", bot);
                    return axum::response::Redirect::to("/auth/login.html").into_response();
                }
            }
        }
    }

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
    serve_suite_impl(&state, bot_name, headers.clone()).await
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

#[derive(Deserialize)]
pub struct SuiteQueryParams {
