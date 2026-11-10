//! Native Save dialog for suite downloads (#1834).
//!
//! The suite runs inside a WebView, where the browser-only
//! "blob + `<a download>`" pattern produces no visible result (WebKitGTK has
//! no download handler wired on the window). Chat attachments and Drive both
//! call this command instead: it asks for a destination with the native dialog
//! and writes the bytes there.

use std::path::PathBuf;

use super::i18n;
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
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(i18n::t("download-unsupported-scheme"));
            }
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let response = client
                .get(parsed)
                .send()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let status = response.status();
            if !status.is_success() {
                return Err(i18n::t_args("download-failed", &[("error", &format!("HTTP {status}"))]));
            }
            if let Some(length) = response.content_length() {
                if length > MAX_DOWNLOAD_BYTES as u64 {
                    return Err(i18n::t("download-too-large"));
                }
            }
            response
                .bytes()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?
                .to_vec()
        }
        (None, Some(bytes)) => {
            if bytes.len() > MAX_DOWNLOAD_BYTES {
                return Err(i18n::t("download-too-large"));
            }
            bytes
        }
        _ => return Err(i18n::t("download-provide-source")),
    };

    let Some(target) = app
        .dialog()
        .file()
        .set_title(i18n::t("dialog-save-file"))
        .set_file_name(safe_file_name(&file_name))
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path: PathBuf = target
        .into_path()
        .map_err(|e| i18n::t_args("download-invalid-destination", &[("error", &e.to_string())]))?;
    tokio::fs::write(&path, payload)
        .await
        .map_err(|e| {
            i18n::t_args(
                "download-write-failed",
                &[("path", &path.display().to_string()), ("error", &e.to_string())],
            )
        })?;
    Ok(Some(path.to_string_lossy().to_string()))
}
