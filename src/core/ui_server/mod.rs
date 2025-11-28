use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use log::error;
use std::{fs, path::PathBuf};
use tower_http::services::ServeDir;

// Serve minimal UI (default at /)
pub async fn index() -> impl IntoResponse {
    serve_minimal().await
}

// Handler for minimal UI
pub async fn serve_minimal() -> impl IntoResponse {
    match fs::read_to_string("ui/minimal/index.html") {
        Ok(html) => (StatusCode::OK, [("content-type", "text/html")], Html(html)),
        Err(e) => {
            error!("Failed to load minimal UI: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                Html("Failed to load minimal interface".to_string()),
            )
        }
    }
}

// Handler for suite UI
pub async fn serve_suite() -> impl IntoResponse {
    match fs::read_to_string("ui/suite/index.html") {
        Ok(html) => (StatusCode::OK, [("content-type", "text/html")], Html(html)),
        Err(e) => {
            error!("Failed to load suite UI: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                Html("Failed to load suite interface".to_string()),
            )
        }
    }
}

pub fn configure_router() -> Router {
    let suite_path = PathBuf::from("./ui/suite");
    let minimal_path = PathBuf::from("./ui/minimal");

    Router::new()
        // Default route serves minimal UI
        .route("/", get(index))
        .route("/minimal", get(serve_minimal))
        // Suite UI route
        .route("/suite", get(serve_suite))
        // Suite static assets (when accessing /suite/*)
        .nest_service("/suite/js", ServeDir::new(suite_path.join("js")))
        .nest_service("/suite/css", ServeDir::new(suite_path.join("css")))
        .nest_service("/suite/public", ServeDir::new(suite_path.join("public")))
        .nest_service("/suite/drive", ServeDir::new(suite_path.join("drive")))
        .nest_service("/suite/chat", ServeDir::new(suite_path.join("chat")))
        .nest_service("/suite/mail", ServeDir::new(suite_path.join("mail")))
        .nest_service("/suite/tasks", ServeDir::new(suite_path.join("tasks")))
        // Legacy paths for backward compatibility (serve suite assets)
        .nest_service("/js", ServeDir::new(suite_path.join("js")))
        .nest_service("/css", ServeDir::new(suite_path.join("css")))
        .nest_service("/public", ServeDir::new(suite_path.join("public")))
        .nest_service("/drive", ServeDir::new(suite_path.join("drive")))
        .nest_service("/chat", ServeDir::new(suite_path.join("chat")))
        .nest_service("/mail", ServeDir::new(suite_path.join("mail")))
        .nest_service("/tasks", ServeDir::new(suite_path.join("tasks")))
        // Fallback for other static files
        .fallback_service(
            ServeDir::new(minimal_path.clone()).fallback(
                ServeDir::new(minimal_path.clone()).append_index_html_on_directories(true),
            ),
        )
}
