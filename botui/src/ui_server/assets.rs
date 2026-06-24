use axum::{
    http::StatusCode,
    response::IntoResponse,
};

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::{Assets, ROOT_FILES, SUITE_DIRS};
use crate::ui_server::constants::get_ui_root;

pub async fn serve_favicon() -> impl IntoResponse {
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
pub async fn handle_embedded_asset(
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
pub async fn handle_embedded_root_asset(
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
pub async fn handle_auth_asset(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
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
pub async fn serve_login() -> impl IntoResponse {
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
pub async fn serve_logout() -> impl IntoResponse {
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
