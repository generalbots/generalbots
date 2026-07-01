use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::constants::get_ui_root;

fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

pub async fn redirect_to_login() -> Response {
    Redirect::to(&get_login_url()).into_response()
}

async fn serve_cloud_file(file_path: std::path::PathBuf) -> Response {
    #[cfg(feature = "embed-ui")]
    {
        let cloud_root = get_ui_root().join("cloud");
        let relative = file_path.strip_prefix(&cloud_root).unwrap_or(&file_path);
        let asset_path = format!("cloud/{}", relative.display()).replace('\\', "/");
        if let Some(content) = Assets::get(&asset_path) {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            return (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response();
        }
        return StatusCode::NOT_FOUND.into_response();
    }

    #[cfg(not(feature = "embed-ui"))]
    {
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
        // Static asset: /cloud/js/file.js, /cloud/css/file.css
        cloud_root.join(normalized)
    } else if normalized.is_empty() || normalized.ends_with('/') {
        cloud_root.join("index.html")
    } else {
        // Use first path segment as page name
        // e.g., /cloud/store/gpu → store.html, /cloud/dashboard → dashboard.html
        let first_seg = normalized.split('/').next().unwrap_or(normalized);
        cloud_root.join(format!("{first_seg}.html"))
    };

    serve_cloud_file(file_path).await
}
