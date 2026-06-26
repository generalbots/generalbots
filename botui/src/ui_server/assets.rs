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

use axum::response::Redirect;

/// Login and logout now served by dedicated login app on port 5000.
/// Redirect requests to the login server URL.
fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

pub async fn serve_login() -> impl IntoResponse {
    Redirect::to(&get_login_url()).into_response()
}

pub async fn serve_logout() -> impl IntoResponse {
    Redirect::to(&format!("{}/logout", get_login_url())).into_response()
}
