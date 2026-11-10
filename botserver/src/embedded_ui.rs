#[cfg(feature = "embed-ui")]
use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    response::IntoResponse,
    Router,
};
#[cfg(not(feature = "embed-ui"))]
use axum::Router;

// The desktop edition embeds the suite instead of depending on a `ui/` folder
// next to the sidecar (#1811 roadmap): the release tarball, the single-file
// self-extractor, the raw `botserver-linux-x86_64` artifact and the Tauri
// deb/rpm/AppImage all then answer `/suite/desktop.html` from inside the
// binary. A `ui/` folder on disk still wins when present (see
// `run_axum_server.rs`), so the payload copies shipped by the packaging
// pipeline keep the same behavior (custom edited UI, hot reload policies).
//
// BotUI is a sibling crate in the monorepo, hence the `..` folders; those are
// resolved at compile time and are relative to the botserver manifest.
#[cfg(feature = "embed-ui")]
#[derive(RustEmbed)]
#[folder = "../botui/ui/suite"]
struct EmbeddedSuite;

#[cfg(feature = "embed-ui")]
#[derive(RustEmbed)]
#[folder = "../botui/ui/themes"]
struct EmbeddedThemes;

#[cfg(feature = "embed-ui")]
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

/// Answers a suite file with the copy embedded in this binary.
///
/// The client-side shell routes applications (`/drive`, `/chat/...`), so a
/// lookup miss is answered with `desktop.html` for user-facing paths — the
/// same contract `fallback_response` implements for the on-disk layout —
/// while API, DAV and webhook paths get an explicit 404 so missing routes
/// stay visible.
#[cfg(feature = "embed-ui")]
async fn serve_embedded_file(req: Request<Body>) -> Response<Body> {
    let requested = req.uri().path().trim_start_matches('/');

    // `/themes/*` is its own nest on the disk layout; in the embedded tree the
    // files live under botui/ui/themes with the prefix already stripped.
    if let Some(theme_rel) = requested.strip_prefix("themes/") {
        if let Some(content) = EmbeddedThemes::get(theme_rel) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, get_mime_type(theme_rel))
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content.data.into_owned()))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap_or_else(|_| Response::new(Body::from("Critical Error")))
                });
        }
    }

    // Everything else belongs to the suite tree, addressed without the
    // `/suite` prefix. Requests for `/suite` or `/` open the desktop shell —
    // botui/ui/suite has no index.html; the suite is a routed application.
    let rel = if requested.is_empty() || requested == "suite" {
        "desktop.html".to_string()
    } else {
        requested.strip_prefix("suite/").unwrap_or(requested).to_string()
    };

    let try_paths = [
        rel.clone(),
        format!("{}/index.html", rel.trim_end_matches('/')),
        format!("{}.html", rel),
    ];

    for try_path in &try_paths {
        if let Some(content) = EmbeddedSuite::get(try_path) {
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
                        .unwrap_or_else(|_| Response::new(Body::from("Critical Error")))
                });
        }
    }

    // Not found in the embedded tree. Mirror the on-disk fallback: machine
    // facing paths report the miss instead of a 200 HTML answer, user-facing
    // paths receive the desktop shell so the client router takes over.
    let machine_facing = ["api/", "caldav", "webhook", ".well-known/"]
        .iter()
        .any(|prefix| requested.starts_with(prefix));
    if machine_facing {
        let locale = botcore::i18n::locale_from_headers(req.headers());
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "error": "not_found",
                "message": botcore::i18n::t(&locale, "error-http-no-route"),
                "path": format!("/{requested}"),
            })),
        )
            .into_response();
    }

    match EmbeddedSuite::get("desktop.html") {
        Some(shell) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(shell.data.into_owned()))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap_or_else(|_| Response::new(Body::from("Critical Error")))
            }),
        None => {
            if requested.is_empty() || requested == "suite" {
                log::warn!("Embedded UI is empty: desktop.html not found at build time");
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
</body>
</html>"#,
                ))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap_or_else(|_| Response::new(Body::from("Critical Error")))
                })
        }
    }
}

#[cfg(feature = "embed-ui")]
pub fn embedded_ui_router() -> Router {
    use axum::routing::any;
    Router::new().fallback(any(serve_embedded_file))
}

#[cfg(feature = "embed-ui")]
pub fn has_embedded_ui() -> bool {
    let has_index = EmbeddedSuite::get("desktop.html").is_some();
    if has_index {
        log::info!("Embedded UI detected - desktop.html found");
    } else {
        log::warn!("No embedded UI found - desktop.html not embedded");
    }
    has_index
}

#[cfg(feature = "embed-ui")]
pub fn list_embedded_files() -> Vec<String> {
    let files: Vec<String> = EmbeddedSuite::iter().map(|f| f.to_string()).collect();
    log::debug!("Embedded UI contains {} files", files.len());
    files
}

#[cfg(not(feature = "embed-ui"))]
pub fn has_embedded_ui() -> bool {
    false
}

#[cfg(not(feature = "embed-ui"))]
pub fn list_embedded_files() -> Vec<String> {
    Vec::new()
}

#[cfg(not(feature = "embed-ui"))]
pub fn embedded_ui_router() -> Router {
    Router::new()
}

#[cfg(feature = "embed-ui")]
use rust_embed::RustEmbed;
#[cfg(feature = "embed-ui")]
use std::path::Path;
