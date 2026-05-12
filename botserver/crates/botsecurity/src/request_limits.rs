use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub const DEFAULT_MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

pub const MAX_UPLOAD_SIZE: usize = 100 * 1024 * 1024;

pub async fn request_size_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let content_length = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(len) = content_length {
        if len > DEFAULT_MAX_REQUEST_SIZE {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                axum::Json(serde_json::json!({
                    "error": "request_too_large",
                    "message": format!("Request body {} bytes exceeds maximum {}", len, DEFAULT_MAX_REQUEST_SIZE),
                    "max_size": DEFAULT_MAX_REQUEST_SIZE
                })),
            )
                .into_response();
        }
    }

    next.run(req).await
}

pub async fn upload_size_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let content_length = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(len) = content_length {
        if len > MAX_UPLOAD_SIZE {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                axum::Json(serde_json::json!({
                    "error": "upload_too_large",
                    "message": format!("Upload {} bytes exceeds maximum {}", len, MAX_UPLOAD_SIZE),
                    "max_size": MAX_UPLOAD_SIZE
                })),
            )
                .into_response();
        }
    }

    next.run(req).await
}

