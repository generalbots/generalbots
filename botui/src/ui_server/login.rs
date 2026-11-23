use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::ui_server::constants::get_ui_root;
#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::js_escape::js_string;
use crate::ui_server::safe_path::{is_safe_relative_path, resolve_static_path};

fn get_cloud_url() -> String {
    std::env::var("CLOUD_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
}

/// Auth pages and the injected config script must never be cached by an
/// intermediary (#2705); cloud pages already use `no-store`.
fn login_response(mime: &str, data: Vec<u8>) -> Response {
    (
        [
            (axum::http::header::CONTENT_TYPE, mime.to_string()),
            (axum::http::header::CACHE_CONTROL, "no-store".to_string()),
        ],
        data,
    )
        .into_response()
}

async fn serve_login_file(file_path: std::path::PathBuf) -> Response {
    let injection = format!(
        r#"<script>window.GB_CLOUD_URL = "{}";</script>"#,
        js_string(&get_cloud_url())
    );

    let ui_root = get_ui_root();
    // #2700: every wildcard capture is attacker-controlled. Reject `..`,
    // encoded traversal and absolute paths before the path is used at all.
    let relative = file_path
        .strip_prefix(&ui_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    if !is_safe_relative_path(&relative) {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Login pages reference shared cloud assets (e.g. /js/cloud-auth.js from
    // cloud/js/) — the login UI has no copy of them. Fall back to the cloud
    // tree per the "login serves CSS/JS/images from cloud via proxy" design;
    // otherwise the 404 HTML body reaches the browser as the script response
    // and strict MIME checking blocks it ("MIME type ('text/html') is not
    // executable").
    let cloud_fallback = file_path
        .strip_prefix(&ui_root)
        .ok()
        .and_then(|rel| rel.to_str())
        .and_then(|rel| rel.strip_prefix("login/"))
        .map(|rel| ui_root.join("cloud").join(rel));

    #[cfg(feature = "embed-ui")]
    {
        let relative = file_path.strip_prefix(&ui_root).unwrap_or(&file_path);
        let asset_path = relative.display().to_string().replace('\\', "/");
        let embedded = Assets::get(&asset_path).or_else(|| {
            cloud_fallback
                .as_ref()
                .and_then(|p| p.strip_prefix(&ui_root).ok())
                .and_then(|rel| rel.to_str())
                .map(|rel| rel.replace('\\', "/"))
                .and_then(|rel| Assets::get(&rel))
        });
        if let Some(content) = embedded {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            let data = if mime.as_ref() == "text/html" {
                inject_script_into_html(&content.data, &injection)
            } else {
                content.data.to_vec()
            };
            return login_response(mime.as_ref(), data);
        }
        return StatusCode::NOT_FOUND.into_response();
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        // Canonicalize both candidates: a symlink inside the UI tree must not
        // be able to expose files outside it, including the repository `.env`.
        if let Some(safe_path) = resolve_static_path(&ui_root, &file_path) {
            if let Ok(bytes) = tokio::fs::read(&safe_path).await {
                let mime = mime_guess::from_path(&safe_path).first_or_octet_stream();
                let data = if mime.as_ref() == "text/html" {
                    inject_script_into_html(&bytes, &injection)
                } else {
                    bytes
                };
                return login_response(mime.as_ref(), data);
            }
        }
        if let Some(fallback) = cloud_fallback {
            if let Some(safe_fallback) = resolve_static_path(&ui_root, &fallback) {
                if let Ok(bytes) = tokio::fs::read(&safe_fallback).await {
                    let mime = mime_guess::from_path(&safe_fallback).first_or_octet_stream();
                    return login_response(mime.as_ref(), bytes);
                }
            }
        }
        StatusCode::NOT_FOUND.into_response()
    }
}

fn inject_script_into_html(bytes: &[u8], script: &str) -> Vec<u8> {
    if let Ok(content) = std::str::from_utf8(bytes) {
        if let Some(head_end) = content.find("</head>") {
            let mut new_content = String::with_capacity(content.len() + script.len());
            new_content.push_str(&content[..head_end]);
            new_content.push_str(script);
            new_content.push_str(&content[head_end..]);
            return new_content.into_bytes();
        }
    }
    bytes.to_vec()
}

pub async fn serve_login_index() -> Response {
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("index.html")).await
}

pub async fn serve_login_signup() -> Response {
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("signup.html")).await
}

pub async fn serve_login_js(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    if !is_safe_relative_path(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("js").join(path.as_str())).await
}

pub async fn serve_login_images(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    if !is_safe_relative_path(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let login_root = get_ui_root().join("login");
    serve_login_file(login_root.join("images").join(path.as_str())).await
}

/// `/cloud/*path` on the login server — serves shared cloud assets (js,
/// images, css) directly from the cloud UI tree. Deployed login pages
/// reference `/cloud/js/cloud-auth.js` and `/cloud/images/logo.svg`; without
/// this route the request fell through to the index fallback and the browser
/// refused the HTML body as a script (strict MIME checking).
pub async fn serve_login_cloud(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    if !is_safe_relative_path(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let cloud_root = get_ui_root().join("cloud");
    serve_login_file(cloud_root.join(path.as_str())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_wildcards_reject_traversal() {
        for path in [
            "..%2f.env",
            "%2e%2e%2f.env",
            "js/../../../.env",
            "..\\..\\.env",
            "/etc/passwd",
        ] {
            assert!(!is_safe_relative_path(path), "must reject: {path}");
        }
        assert!(is_safe_relative_path("js/cloud-auth.js"));
    }

    #[test]
    fn auth_responses_are_never_stored() {
        use axum::http::header::CACHE_CONTROL;
        let response = login_response("text/html", b"page".to_vec());
        assert_eq!(
            response.headers().get(CACHE_CONTROL).and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
    }
}
