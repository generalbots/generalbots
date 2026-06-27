use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use log::error;
#[cfg(not(feature = "embed-ui"))]
use std::fs;

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
#[cfg(not(feature = "embed-ui"))]
use crate::ui_server::constants::get_ui_root;
use crate::shared::AppState;

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

pub fn remove_section(html: &str, section: &str) -> String {
    let start_marker = format!("<!-- SECTION:{} -->", section);
    let end_marker = format!("<!-- ENDSECTION:{} -->", section);

    let mut result = String::with_capacity(html.len());
    let mut current_pos = 0;

    while let Some(start_idx) = html[current_pos..].find(&start_marker) {
        let abs_start = current_pos + start_idx;
        result.push_str(&html[current_pos..abs_start]);

        if let Some(end_idx) = html[abs_start..].find(&end_marker) {
            current_pos = abs_start + end_idx + end_marker.len();
        } else {
            current_pos = abs_start + start_marker.len();
        }
    }

    result.push_str(&html[current_pos..]);
    result
}

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let commit = option_env!("BOTUI_COMMIT").unwrap_or("unknown");
    if state.health_check().await {
        (
            StatusCode::OK,
            Json(serde_json::json!({
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
            Json(serde_json::json!({
                "status": "unhealthy",
                "service": "botui",
                "error": "botserver unreachable",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    }
}

pub async fn api_health(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let commit = option_env!("BOTUI_COMMIT").unwrap_or("unknown");
    if state.health_check().await {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "botserver": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "error",
                "botserver": "unhealthy",
                "version": env!("CARGO_PKG_VERSION"),
                "commit": commit
            })),
        )
    }
}
