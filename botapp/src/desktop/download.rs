//! Native Save dialog for suite downloads (#1834).
//!
//! The suite runs inside a WebView, where the browser-only
//! "blob + `<a download>`" pattern produces no visible result (WebKitGTK has
//! no download handler wired on the window). Chat attachments and Drive both
//! call this command instead: it asks for a destination with the native dialog
//! and writes the bytes there.

use std::path::PathBuf;

use super::drive::path_scope;
use super::{fs_scope, i18n};
use tauri_plugin_dialog::DialogExt;

/// Upper bound for one download. Office documents and conversation archives
/// are far below this; the cap only stops a runaway endpoint from exhausting
/// memory before the dialog is confirmed.
const MAX_DOWNLOAD_BYTES: usize = 512 * 1024 * 1024;

/// #2690 — the webview supplies the URL, so an unrestricted fetch is an SSRF
/// primitive against the host's loopback/link-local services. Only the local
/// sidecar (and the dev server in a development build) may be fetched.
fn is_allowed_download_url(url: &reqwest::Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if !matches!(url.host_str(), Some("localhost") | Some("127.0.0.1")) {
        return false;
    }
    let allowed_ports = [super::sidecar::probe::sidecar_port(), 3000];
    url.port_or_known_default()
        .is_some_and(|port| allowed_ports.contains(&port))
}

#[cfg(test)]
mod url_tests {
    use super::is_allowed_download_url;

    #[test]
    fn only_loopback_sidecar_urls_are_allowed() {
        let port = super::super::sidecar::probe::sidecar_port();
        let good = reqwest::Url::parse(&format!("http://127.0.0.1:{port}/api/files/x")).unwrap();
        assert!(is_allowed_download_url(&good));
        for hostile in [
            "http://169.254.169.254/latest/meta-data/",
            "http://localhost:22/",
            "https://example.com/file",
            "http://user@127.0.0.1:8080/file",
            "file:///etc/passwd",
        ] {
            let url = reqwest::Url::parse(hostile).unwrap();
            assert!(!is_allowed_download_url(&url), "SSRF URL accepted: {hostile}");
        }
    }
}

/// Keep only the base name the Save dialog should suggest. The user still
/// chooses the final path; path separators and control characters never reach
/// the native dialog.
fn safe_file_name(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw).trim();
    let clean: String = base
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '(' | ')' | '[' | ']')
        })
        .collect();
    if clean.trim().is_empty() {
        "download".to_string()
    } else {
        clean
    }
}

/// Save one download to a user-chosen path.
///
/// Supply either `url` (the suite fetches it, keeping the bearer token out of
/// the native layer) or `bytes` (a Blob the caller already fetched from a POST
/// endpoint such as Drive's export). Returns the saved path, or `None` when the
/// user cancelled the dialog.
///
/// # Errors
/// Returns a message when the URL is invalid, the download fails, or the file
/// cannot be written.
#[tauri::command]
pub async fn save_download(
    app: tauri::AppHandle,
    file_name: String,
    url: Option<String>,
    bytes: Option<Vec<u8>>,
) -> Result<Option<String>, String> {
    let payload = match (url, bytes) {
        (Some(url), None) => {
            let parsed = reqwest::Url::parse(url.trim())
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            if !is_allowed_download_url(&parsed) {
                return Err(i18n::t("download-unsupported-scheme"));
            }
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                // A redirect could move the request from the sidecar to an
                // internal host after the URL check — never follow one (#2690).
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let mut response = client
                .get(parsed)
                .send()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let status = response.status();
            if !status.is_success() {
                return Err(i18n::t_args("download-failed", &[("error", &format!("HTTP {status}"))]));
            }
            if response.content_length().is_some_and(|length| length > MAX_DOWNLOAD_BYTES as u64) {
                return Err(i18n::t("download-too-large"));
            }
            // `content_length()` is optional on chunked responses; enforce the
            // cap while streaming so an unbounded body cannot allocate memory.
            let mut payload = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?
            {
                if payload.len().saturating_add(chunk.len()) > MAX_DOWNLOAD_BYTES {
                    return Err(i18n::t("download-too-large"));
                }
                payload.extend_from_slice(&chunk);
            }
            payload
        }
        (None, Some(bytes)) => {
            if bytes.len() > MAX_DOWNLOAD_BYTES {
                return Err(i18n::t("download-too-large"));
            }
            bytes
        }
        _ => return Err(i18n::t("download-provide-source")),
    };

    let name = safe_file_name(&file_name);
    let Some(target) = app
        .dialog()
        .file()
        .set_title(i18n::t("dialog-save-file"))
        .set_file_name(name.clone())
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path: PathBuf = target
        .into_path()
        .map_err(|e| i18n::t_args("download-invalid-destination", &[("error", &e.to_string())]))?;
    // #2497 — the native picker is the human confirmation, but its result is
    // still caller-adjacent data: canonicalise and scope-check before writing
    // so a symlink or a forged dialog result cannot drop bytes outside the
    // folders the shell owns.
    let scoped = path_scope::resolve_in_scopes(&path.to_string_lossy(), &fs_scope::scopes())?;
    tokio::fs::write(&scoped, payload)
        .await
        .map_err(|e| {
            log::warn!("download IPC: write failed for {}: {e}", scoped.display());
            // Only the sanitised base name crosses the IPC boundary; the full
            // target stays in the local log above.
            i18n::t_args("download-write-failed", &[("path", &name), ("error", &e.to_string())])
        })?;
    log::info!("download IPC: saved {}", scoped.display());
    Ok(Some(scoped.to_string_lossy().to_string()))
}
