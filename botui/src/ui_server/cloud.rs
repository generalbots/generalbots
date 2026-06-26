use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use crate::ui_server::constants::get_ui_root;

fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

pub async fn redirect_to_login() -> Response {
    Redirect::to(&get_login_url()).into_response()
}

async fn serve_cloud_file(file_path: std::path::PathBuf) -> Response {
    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            (
                [(
                    axum::http::header::CONTENT_TYPE,
                    mime.as_ref(),
                )],
                bytes,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn serve_cloud_index() -> Response {
    let cloud_root = get_ui_root().join("cloud");
    serve_cloud_file(cloud_root.join("index.html")).await
}

pub async fn serve_cloud(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let cloud_root = get_ui_root().join("cloud");
    let normalized = path.strip_prefix('/').unwrap_or(&path);
    let file_path = if normalized.contains('.') {
        cloud_root.join(normalized)
    } else if normalized.is_empty() || normalized.ends_with('/') {
        cloud_root.join("index.html")
    } else {
        cloud_root.join(format!("{normalized}.html"))
    };

    serve_cloud_file(file_path).await
}
