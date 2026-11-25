use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::ui_server::constants::get_ui_root;
#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
use crate::ui_server::js_escape::js_window_script;
use crate::ui_server::safe_path::{is_safe_relative_path, resolve_static_path};

fn get_cloud_url() -> String {
    std::env::var("CLOUD_URL").unwrap_or_else(|_| "http://localhost:4000".to_string())
}

/// Build the config script served on auth pages (#2702): the env-derived
/// `CLOUD_URL` is escaped by the shared helper before it reaches the HTML.
fn build_cloud_script(cloud_url: &str) -> String {
    js_window_script(&[("GB_CLOUD_URL", cloud_url)])
}

/// Pure header-shaping rule for every response served by the auth origin
/// (login/signup HTML and the injected `GB_CLOUD_URL` config JS, #2705).
///
/// `no-store` plus the legacy `Pragma`/`Expires` trio keeps credentials and
/// the injected config out of browser and intermediary caches even for clients
/// that ignore `Cache-Control`. Kept pure (mime in, headers out) so the
/// never-store invariant is unit-testable without building an HTTP response.
fn auth_response_headers(mime: &str) -> [(axum::http::HeaderName, String); 4] {
    [
        (axum::http::header::CONTENT_TYPE, mime.to_string()),
        (axum::http::header::CACHE_CONTROL, "no-store".to_string()),
        (axum::http::header::PRAGMA, "no-cache".to_string()),
        (axum::http::header::EXPIRES, "0".to_string()),
    ]
}

fn login_response(mime: &str, data: Vec<u8>) -> Response {
    (auth_response_headers(mime), data).into_response()
}

async fn serve_login_file(file_path: std::path::PathBuf) -> Response {
    let injection = build_cloud_script(&get_cloud_url());

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

/// `/suite/:dir/*path` on the login server, restricted to the shared static
/// trees the auth pages actually link: the palette tokens
/// (`/suite/css/palette-tokens.css`) and the touch icons
/// (`/suite/assets/icons/apple-touch-icon.png`). The login router has no
/// catch-all static mount, so those requests used to fall through to
/// `serve_login_index` and the browser refused the HTML body as a stylesheet
/// ("MIME type ('text/html') is not a supported stylesheet MIME type").
pub async fn serve_login_suite(
    axum::extract::Path((dir, path)): axum::extract::Path<(String, String)>,
) -> Response {
    if !matches!(dir.as_str(), "css" | "assets") || !is_safe_relative_path(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let suite_root = get_ui_root().join("suite");
    serve_login_file(suite_root.join(&dir).join(path.as_str())).await
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
    fn hostile_env_values_cannot_break_out_of_the_injected_script() {
        // #2702: CLOUD_URL is env-controlled and must stay inert in the
        // <script> injected into every auth page.
        for hostile in [
            "\";alert(1);//",
            "</script><img src=x onerror=alert(1)>",
            "a\u{2028}b\u{2029}c",
            "&<>",
        ] {
            let script = build_cloud_script(hostile);
            assert_eq!(script.matches("</script>").count(), 1, "{script}");
            assert!(!script.contains("<img"), "{script}");
            assert!(!script.contains('\u{2028}'), "{script}");
            assert!(!script.contains('\u{2029}'), "{script}");
        }
        let script = build_cloud_script("</script>");
        assert!(script.contains("\\u003c/script\\u003e"), "{script}");
    }

    // #2700: exercise the real handlers, not only the string gate. Axum
    // decodes wildcard captures before the handler runs, so both literal and
    // encoded `..` reach these functions.
    #[tokio::test]
    async fn login_handlers_reject_traversal_payloads() {
        use axum::extract::Path;

        for payload in [
            "../.env",
            "..%2f.env",
            "%2e%2e%2f.env",
            "%252e%252e%2f.env",
            "js/../../../.env",
            "..\\..\\.env",
            "/etc/passwd",
            "//etc/passwd",
            "C:\\Windows\\win.ini",
            "js/%00/../../etc/passwd",
        ] {
            let js = serve_login_js(Path(payload.to_string())).await;
            assert_eq!(js.status(), StatusCode::NOT_FOUND, "js: {payload}");
            let images = serve_login_images(Path(payload.to_string())).await;
            assert_eq!(images.status(), StatusCode::NOT_FOUND, "images: {payload}");
            let cloud = serve_login_cloud(Path(payload.to_string())).await;
            assert_eq!(cloud.status(), StatusCode::NOT_FOUND, "cloud: {payload}");
            let suite = serve_login_suite(Path(("css".to_string(), payload.to_string()))).await;
            assert_eq!(suite.status(), StatusCode::NOT_FOUND, "suite: {payload}");
        }
    }

    #[tokio::test]
    async fn login_suite_handler_only_serves_shared_static_trees() {
        use axum::extract::Path;

        // The auth router has no catch-all static mount: only the trees the
        // login/signup pages link are exposed.
        let denied = serve_login_suite(Path(("chat".to_string(), "chat.html".to_string()))).await;
        assert_eq!(denied.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn login_file_rejects_path_outside_ui_root() {
        // `ui/../Cargo.toml` exists but sits outside the served UI tree.
        let outside = get_ui_root().join("../Cargo.toml");
        let response = serve_login_file(outside).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn login_handlers_serve_legitimate_assets() {
        use axum::extract::Path;

        let js = serve_login_js(Path("cloud-auth.js".to_string())).await;
        assert_eq!(js.status(), StatusCode::OK);
        let cloud = serve_login_cloud(Path("js/cloud.js".to_string())).await;
        assert_eq!(cloud.status(), StatusCode::OK);
        let suite =
            serve_login_suite(Path(("css".to_string(), "palette-tokens.css".to_string()))).await;
        assert_eq!(suite.status(), StatusCode::OK);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn login_file_rejects_symlink_outside_ui_root() {
        use std::os::unix::fs::symlink;

        let base = std::env::temp_dir().join(format!(
            "gb-login-link-{}-{}",
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

        let response = serve_login_file(link).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        assert!(!String::from_utf8_lossy(&body).contains("TOP SECRET"));

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn auth_header_shaping_marks_every_mime_never_storable() {
        use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, EXPIRES, PRAGMA};
        // Login/signup HTML and every shared-JS MIME served by the auth origin
        // must carry the full no-store trio (#2705).
        for mime in [
            "text/html",
            "text/javascript",
            "application/javascript",
            "text/css",
            "image/svg+xml",
        ] {
            let shaped = auth_response_headers(mime);
            assert_eq!(shaped[0].0, CONTENT_TYPE);
            assert_eq!(shaped[0].1.as_str(), mime);
            assert_eq!(shaped[1].0, CACHE_CONTROL);
            assert_eq!(shaped[1].1.as_str(), "no-store");
            assert_eq!(shaped[2].0, PRAGMA);
            assert_eq!(shaped[2].1.as_str(), "no-cache");
            assert_eq!(shaped[3].0, EXPIRES);
            assert_eq!(shaped[3].1.as_str(), "0");
        }
    }

    #[test]
    fn auth_responses_are_never_stored() {
        use axum::http::header::{CACHE_CONTROL, EXPIRES, PRAGMA};
        let response = login_response("text/html", b"page".to_vec());
        let headers = response.headers();
        assert_eq!(
            headers.get(CACHE_CONTROL).and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
        assert_eq!(
            headers.get(PRAGMA).and_then(|v| v.to_str().ok()),
            Some("no-cache")
        );
        assert_eq!(
            headers.get(EXPIRES).and_then(|v| v.to_str().ok()),
            Some("0")
        );
    }
}
