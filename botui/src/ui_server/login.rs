use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::ui_server::constants::get_ui_root;

async fn serve_login_file(file_path: std::path::PathBuf) -> Response {
    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], bytes).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn serve_login_index() -> Response {
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("index.html")).await
}

pub async fn serve_login_signup() -> Response {
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("signup.html")).await
}

pub async fn serve_login_cloud_css(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let cloud_root = get_ui_root().join("cloud");
    serve_login_file(cloud_root.join("css").join(path.as_str())).await
}

pub async fn serve_login_cloud_js(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let cloud_root = get_ui_root().join("cloud");
    serve_login_file(cloud_root.join("js").join(path.as_str())).await
}

pub async fn serve_login_cloud_images(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let cloud_root = get_ui_root().join("cloud");
    serve_login_file(cloud_root.join("images").join(path.as_str())).await
}
