use axum::{
    extract::OriginalUri,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
#[cfg(not(feature = "embed-ui"))]
use log::info;

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::constants::get_ui_root;

fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

fn get_chat_url() -> String {
    std::env::var("CHAT_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

pub async fn redirect_to_login() -> Response {
    Redirect::to(&get_login_url()).into_response()
}

pub async fn redirect_to_signup() -> Response {
    Redirect::to(&format!("{}/signup", get_login_url())).into_response()
}

pub async fn redirect_to_store() -> Response {
    Redirect::to("/store").into_response()
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

fn get_preview_banner_html() -> String {
    String::new()
}

fn inject_banner_into_html(bytes: &[u8], banner: &str) -> Vec<u8> {
    if let Ok(content) = std::str::from_utf8(bytes) {
        if let Some(body_start) = content.find("<body") {
            if let Some(body_end) = content[body_start..].find('>') {
                let insert_pos = body_start + body_end + 1;
                let mut new_content = String::with_capacity(content.len() + banner.len());
                new_content.push_str(&content[..insert_pos]);
                new_content.push_str(banner);
                new_content.push_str(&content[insert_pos..]);
                return new_content.into_bytes();
            }
        }
    }
    bytes.to_vec()
}

fn get_html_injection_script() -> String {
    let login_url = get_login_url();
    let chat_url = get_chat_url();
    format!(
        r#"<script>window.GB_LOGIN_URL = "{}";window.GB_CHAT_URL = "{}";</script>"#,
        login_url, chat_url
    )
}

async fn serve_cloud_file(file_path: std::path::PathBuf) -> Response {
    let injection = get_html_injection_script();
    let banner = get_preview_banner_html();

    #[cfg(feature = "embed-ui")]
    {
        let cloud_root = get_ui_root().join("cloud");
        let relative = file_path.strip_prefix(&cloud_root).unwrap_or(&file_path);
        let asset_path = format!("cloud/{}", relative.display()).replace('\\', "/");
        if let Some(content) = Assets::get(&asset_path) {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            let data = if mime.as_ref() == "text/html" {
                let d = inject_script_into_html(&content.data, &injection);
                inject_banner_into_html(&d, &banner).into()
            } else {
                content.data
            };
            return (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                data,
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
                let data = if mime.as_ref() == "text/html" {
                    let d = inject_script_into_html(&bytes, &injection);
                    let d2 = inject_banner_into_html(&d, &banner);
                    info!(
                        "Injected GB_LOGIN_URL, GB_CHAT_URL, and preview banner into {}",
                        file_path.display()
                    );
                    d2
                } else {
                    bytes
                };
                (
                    [
                        (axum::http::header::CONTENT_TYPE, mime.as_ref()),
                        (axum::http::header::CACHE_CONTROL, "no-store"),
                    ],
                    data,
                )
                    .into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

/// Resolve a path to a cloud file and serve it.
/// Used by both `/cloud/*path` (legacy) and root-level page fallback.
async fn resolve_and_serve_cloud(path: &str) -> Response {
    let cloud_root = get_ui_root().join("cloud");
    let normalized = path.strip_prefix('/').unwrap_or(path);
    let file_path = if normalized.contains('.') {
        cloud_root.join(normalized)
    } else if normalized.is_empty() || normalized.ends_with('/') {
        cloud_root.join("index.html")
    } else {
        let first_seg = normalized.split('/').next().unwrap_or(normalized);
        cloud_root.join(format!("{first_seg}.html"))
    };

    serve_cloud_file(file_path).await
}

pub async fn serve_cloud(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    if path == "signup" || path == "login" || path.starts_with("signup/") || path.starts_with("login/") {
        return Redirect::to(&format!("{}/{}", get_login_url(), path)).into_response();
    }
    resolve_and_serve_cloud(&path).await
}

/// Cloud page fallback: serves cloud HTML/JS/CSS from root-level paths.
/// `/store` → `cloud/store.html`, `/js/cloud.js` → `cloud/js/cloud.js`
/// Login/signup routes are served exclusively on port 5000, never on cloud.
pub async fn serve_cloud_fallback(original_uri: OriginalUri) -> Response {
    let path = original_uri.path();
    if path == "/login" || path == "/login/" {
        return Redirect::to(&format!("{}/login", get_login_url())).into_response();
    }
    if path == "/signup" || path == "/signup/" {
        return Redirect::to(&format!("{}/signup", get_login_url())).into_response();
    }
    if path == "/chat" || path == "/chat/" {
        let suite_url = std::env::var("SUITE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        return Redirect::to(&format!("{suite_url}/")).into_response();
    }
    resolve_and_serve_cloud(path).await
}
