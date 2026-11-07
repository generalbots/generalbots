//! Native Save dialog for suite downloads (#1834).
//!
//! The suite runs inside a WebView, where the browser-only
//! "blob + `<a download>`" pattern produces no visible result (WebKitGTK has
//! no download handler wired on the window). Chat attachments and Drive both
//! call this command instead: it asks for a destination with the native dialog
//! and writes the bytes there.

use std::path::PathBuf;

use tauri_plugin_dialog::DialogExt;

/// Upper bound for one download. Office documents and conversation archives
/// are far below this; the cap only stops a runaway endpoint from exhausting
/// memory before the dialog is confirmed.
const MAX_DOWNLOAD_BYTES: usize = 512 * 1024 * 1024;

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
                .map_err(|e| format!("invalid download URL: {e}"))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err("only http(s) downloads are supported".to_string());
            }
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| format!("HTTP client: {e}"))?;
            let response = client
                .get(parsed)
                .send()
                .await
                .map_err(|e| format!("download failed: {e}"))?;
            let status = response.status();
            if !status.is_success() {
                return Err(format!("download failed: HTTP {status}"));
            }
            if let Some(length) = response.content_length() {
                if length > MAX_DOWNLOAD_BYTES as u64 {
                    return Err("the file is too large to save".to_string());
                }
            }
            response
                .bytes()
                .await
                .map_err(|e| format!("download failed: {e}"))?
                .to_vec()
        }
        (None, Some(bytes)) => {
            if bytes.len() > MAX_DOWNLOAD_BYTES {
                return Err("the file is too large to save".to_string());
            }
            bytes
        }
        _ => return Err("provide either a download URL or the file bytes".to_string()),
    };

    let Some(target) = app
        .dialog()
        .file()
        .set_title("Save file")
        .set_file_name(safe_file_name(&file_name))
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path: PathBuf = target
        .into_path()
        .map_err(|e| format!("invalid destination: {e}"))?;
    tokio::fs::write(&path, payload)
        .await
        .map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(Some(path.to_string_lossy().to_string()))
}
