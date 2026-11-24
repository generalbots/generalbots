use axum::{
    extract::OriginalUri,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
#[cfg(not(feature = "embed-ui"))]
use log::info;

use crate::ui_server::constants::get_ui_root;
#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::js_escape::js_window_script;
use crate::ui_server::safe_path::{is_safe_relative_path, resolve_static_path};

fn get_login_url() -> String {
    std::env::var("LOGIN_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
}

fn get_chat_url() -> String {
    std::env::var("CHAT_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn get_platform_domain() -> String {
    std::env::var("GB_PLATFORM_DOMAIN").unwrap_or_else(|_| "generalbots.org".to_string())
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

fn build_html_injection_script(login_url: &str, chat_url: &str, platform_domain: &str) -> String {
    js_window_script(&[
        ("GB_LOGIN_URL", login_url),
        ("GB_CHAT_URL", chat_url),
        ("GB_PLATFORM_DOMAIN", platform_domain),
    ])
}

fn get_html_injection_script() -> String {
    build_html_injection_script(
        &get_login_url(),
        &get_chat_url(),
        &get_platform_domain(),
    )
}

async fn serve_cloud_file(file_path: std::path::PathBuf) -> Response {
    let ui_root = get_ui_root();
    // #2700: wildcard captures are attacker-controlled. Reject traversal
    // before either the embedded or the filesystem read path uses the path.
    let relative = file_path
        .strip_prefix(&ui_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    if !is_safe_relative_path(&relative) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let injection = get_html_injection_script();
    let banner = get_preview_banner_html();

    #[cfg(feature = "embed-ui")]
    {
        let cloud_root = get_ui_root().join("cloud");
        let relative = file_path.strip_prefix(&cloud_root).unwrap_or(&file_path);
        if !is_safe_relative_path(&relative.to_string_lossy().replace('\\', "/")) {
            return StatusCode::NOT_FOUND.into_response();
        }
        let asset_path = format!("cloud/{}", relative.display()).replace('\\', "/");
        if let Some(content) = Assets::get(&asset_path) {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            let data = if mime.as_ref() == "text/html" {
                let d = inject_script_into_html(&content.data, &injection);
                inject_banner_into_html(&d, &banner).into()
            } else {
                content.data
            };
            return ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], data).into_response();
        }
        return StatusCode::NOT_FOUND.into_response();
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        // Canonicalize so a symlink inside the UI tree cannot point at `.env`
        // or any other file outside the served root.
        let Some(safe_path) = resolve_static_path(&ui_root, &file_path) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        match tokio::fs::read(&safe_path).await {
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
    if !is_safe_relative_path(normalized) {
        return StatusCode::NOT_FOUND.into_response();
    }
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

pub async fn serve_cloud(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    if path == "signup"
        || path == "login"
        || path.starts_with("signup/")
        || path.starts_with("login/")
    {
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
        let suite_url =
            std::env::var("SUITE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        return Redirect::to(&format!("{suite_url}/")).into_response();
    }
    resolve_and_serve_cloud(path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_wildcards_reject_traversal_before_join() {
        // Axum decodes `%2e%2e%2f` into `../` before the handler sees it; the
        // same guard also catches double-encoded forms from front proxies.
        for path in [
            "../.env",
            "js/../../../.env",
            "%2e%2e%2f.env",
            "%252e%252e%2f.env",
            "/etc/passwd",
            "js\\..\\..\\.env",
        ] {
            assert!(!is_safe_relative_path(path), "must reject: {path}");
        }
        for path in ["store.html", "js/cloud.js", "css/site.css", "plans"] {
            assert!(is_safe_relative_path(path), "must accept: {path}");
        }
    }

    #[test]
    fn hostile_env_values_cannot_break_out_of_the_injected_script() {
        // #2702: each value below can come from LOGIN_URL / CHAT_URL /
        // GB_PLATFORM_DOMAIN. None may close the string or the <script> tag.
        let hostile = "</script><img src=x onerror=alert(1)>\";\u{2028}&";
        let script = build_html_injection_script(hostile, hostile, hostile);
        assert_eq!(script.matches("</script>").count(), 1, "{script}");
        assert!(!script.contains("<img"), "{script}");
        assert!(!script.contains('\u{2028}'), "{script}");
        assert!(script.contains("\\u003c/script\\u003e"), "{script}");
        assert!(script.contains("\\\""), "{script}");
        assert!(script.contains("\\u0026"), "{script}");
    }

    #[tokio::test]
    async fn cloud_handler_rejects_traversal_payloads() {
        // Exercise the real handler, not only the string gate: a payload that
        // slipped past `is_safe_relative_path` must still fail containment and
        // never return file bytes from outside the UI root.
        for payload in [
            "../.env",
            "js/../../../.env",
            "%2e%2e%2f.env",
            "%252e%252e%2f.env",
            "/etc/passwd",
            "//etc/passwd",
            "js\\..\\..\\.env",
            "C:\\Windows\\win.ini",
            "js/%00/../../etc/passwd",
        ] {
            let response = resolve_and_serve_cloud(payload).await;
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "must reject: {payload}"
            );
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap_or_default();
            assert!(body.is_empty(), "no bytes may leak for: {payload}");
        }
    }

    #[tokio::test]
    async fn cloud_handler_serves_legitimate_asset() {
        let response = resolve_and_serve_cloud("js/cloud.js").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        assert!(!body.is_empty(), "legitimate asset must be served");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cloud_handler_rejects_symlink_outside_ui_root() {
        use std::os::unix::fs::symlink;

        let base = std::env::temp_dir().join(format!(
            "gb-cloud-link-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let outside = base.join("outside");
        std::fs::create_dir_all(&outside).expect("create temp dir");
        std::fs::write(outside.join("secret.txt"), b"TOP SECRET").expect("write secret");
        let link = base.join("escape.txt");
        symlink(outside.join("secret.txt"), &link).expect("create symlink");

        let response = serve_cloud_file(link).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        assert!(!String::from_utf8_lossy(&body).contains("TOP SECRET"));

        std::fs::remove_dir_all(&base).ok();
    }
}
