use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use log::error;
use std::{fs, path::PathBuf};
use tower_http::services::ServeDir;

pub async fn index() -> impl IntoResponse {
    match fs::read_to_string("ui/desktop/index.html") {
        Ok(html) => (StatusCode::OK, [("content-type", "text/html")], Html(html)),
        Err(e) => {
            error!("Failed to load index page: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain")],
                Html("Failed to load index page".to_string()),
            )
        }
    }
}

pub fn configure_router() -> Router {
    let static_path = PathBuf::from("./ui/desktop");

    Router::new()
        // Serve all JS files
        .nest_service("/js", ServeDir::new(static_path.join("js")))
        // Serve CSS files
        .nest_service("/css", ServeDir::new(static_path.join("css")))
        // Serve public assets (themes, etc.)
        .nest_service("/public", ServeDir::new(static_path.join("public")))
        .nest_service("/drive", ServeDir::new(static_path.join("drive")))
        .nest_service("/chat", ServeDir::new(static_path.join("chat")))
        .nest_service("/mail", ServeDir::new(static_path.join("mail")))
        .nest_service("/tasks", ServeDir::new(static_path.join("tasks")))
        // Fallback: serve static files and index.html for SPA routing
        .fallback_service(
            ServeDir::new(static_path.clone()).fallback(
                ServeDir::new(static_path.clone()).append_index_html_on_directories(true),
            ),
        )
        .route("/", get(index))
}
