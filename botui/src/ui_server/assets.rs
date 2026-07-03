use axum::{
    http::StatusCode,
    response::IntoResponse,
};

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::{Assets, ROOT_FILES, SUITE_DIRS};
#[cfg(not(feature = "embed-ui"))]
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


