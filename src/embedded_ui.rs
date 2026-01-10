use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    routing::get,
    Router,
};
use rust_embed::Embed;
use std::path::Path;

#[derive(Embed)]
#[folder = "../botui/ui/suite/"]
#[prefix = ""]
struct EmbeddedUi;

fn get_mime_type(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "pdf" => "application/pdf",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

async fn serve_embedded_file(req: Request<Body>) -> Response<Body> {
    let path = req.uri().path().trim_start_matches('/');

    let file_path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path
    };

    let try_paths = [
        file_path.to_string(),
        format!("{}/index.html", file_path.trim_end_matches('/')),
        format!("{}.html", file_path),
    ];

    for try_path in &try_paths {
        if let Some(content) = EmbeddedUi::get(try_path) {
            let mime = get_mime_type(try_path);

            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content.data.into_owned()))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap()
                });
        }
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(
            r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
<h1>404 - Not Found</h1>
<p>The requested file was not found in embedded UI.</p>
<p><a href="/">Go to Home</a></p>
</body>
</html>"#,
        ))
        .unwrap()
}

pub fn embedded_ui_router() -> Router {
    Router::new().fallback(get(serve_embedded_file))
}

pub fn has_embedded_ui() -> bool {
    EmbeddedUi::get("index.html").is_some()
}

pub fn list_embedded_files() -> Vec<String> {
    EmbeddedUi::iter().map(|f| f.to_string()).collect()
}
