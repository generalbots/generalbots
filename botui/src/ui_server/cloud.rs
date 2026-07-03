use axum::{
    extract::OriginalUri,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use log::info;

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::constants::get_ui_root;

fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

pub async fn redirect_to_login() -> Response {
    Redirect::to(&get_login_url()).into_response()
}

pub async fn redirect_to_store() -> Response {
    Redirect::to("/store").into_response()
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
            Ok(mut bytes) => {
                let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
                // Inject GB_LOGIN_URL into HTML pages (same pattern as suite)
                if mime.as_ref() == "text/html" {
                    if let Ok(content) = std::str::from_utf8(&bytes) {
                        if let Some(head_end) = content.find("</head>") {
                            let login_url = get_login_url();
                            let script = format!(
                                r#"<script>window.GB_LOGIN_URL = "{}";</script>"#,
                                login_url
                            );
                            let mut new_content = String::with_capacity(content.len() + script.len());
                            new_content.push_str(&content[..head_end]);
                            new_content.push_str(&script);
                            new_content.push_str(&content[head_end..]);
                            bytes = new_content.into_bytes();
                            info!(
                                "Injected GB_LOGIN_URL ({}) into {}",
                                login_url,
                                file_path.display()
                            );
                        }
                    }
                }
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
    resolve_and_serve_cloud(path).await
}
