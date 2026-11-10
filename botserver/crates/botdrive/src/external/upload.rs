//! Saving chat artifacts to an external drive (#2047).
//!
//! The external drives were read-only: the sync engine mirrored metadata and
//! downloaded bytes, and the OAuth grants asked for read scope only. This
//! module is the write half — one `upload_file` for both providers, backed by
//! the pure request builders in [`super::gdrive_upload`] and
//! [`super::onedrive_upload`].
//!
//! Capability is never inferred: a connection whose recorded consent has no
//! write scope fails with [`UploadError::MissingWriteScope`] instead of quietly
//! writing into General Bots Drive. That is the caller's cue to offer the
//! "grant write access" consent again.

use crate::external::types::{Provider, SyncError};

/// Simple-upload ceiling for the JSON API (the chat sends base64, so a larger
/// file would also blow the request-size limits). The providers have their own
/// limits; this is the one this endpoint enforces.
pub const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;

/// Per-provider ceiling for the single-request upload this module builds.
/// Google's `multipart/related` upload is documented for files up to 5 MB —
/// a larger body is rejected by Drive, so refusing it here keeps the error
/// honest (`too_large`) instead of surfacing a provider rejection. OneDrive's
/// simple PUT accepts far more, so the local request limit is the binding one.
pub const fn max_upload_bytes(provider: Provider) -> usize {
    let provider_cap = match provider {
        Provider::GoogleDrive => 5 * 1024 * 1024,
        Provider::OneDrive => MAX_UPLOAD_BYTES,
    };
    if provider_cap < MAX_UPLOAD_BYTES {
        provider_cap
    } else {
        MAX_UPLOAD_BYTES
    }
}

/// What to write, and where.
#[derive(Debug, Clone)]
pub struct UploadRequest {
    /// Provider to write to.
    pub provider: Provider,
    /// Destination folder relative to the provider root. Empty means the
    /// provider's default location (Drive root / OneDrive app folder).
    pub path: String,
    /// File name shown in the provider.
    pub name: String,
    /// MIME type; empty falls back to `application/octet-stream`.
    pub mime_type: String,
    /// File bytes.
    pub content: Vec<u8>,
    /// Replace this remote file instead of creating a new one.
    pub replace_remote_id: Option<String>,
    /// Google Drive parent folder id, when the folder is addressed by id.
    pub parent_id: Option<String>,
}

/// The provider's answer, mirrored into the local listing by the route.
#[derive(Debug, Clone, PartialEq)]
pub struct UploadedFile {
    /// Provider slug (`onedrive` / `gdrive`).
    pub provider: &'static str,
    /// Provider-assigned id.
    pub remote_id: String,
    /// Name the provider stored.
    pub name: String,
    /// Shareable link, when the provider returns one.
    pub web_view_link: Option<String>,
    /// Stored size in bytes.
    pub bytes: usize,
}

/// Why an upload could not happen. The route maps each variant onto a status
/// and a machine-readable `kind`, so the UI can act on it (offer consent,
/// explain the size, retry) instead of showing a generic failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadError {
    /// No OAuth client configured for the provider on this deployment.
    NotConfigured,
    /// No stored connection for the caller's branch.
    NotConnected,
    /// The connection consented to read only.
    MissingWriteScope,
    /// The content exceeds [`MAX_UPLOAD_BYTES`].
    TooLarge(usize),
    /// The provider rejected the write.
    Provider(String),
}

impl UploadError {
    /// Short tag for the JSON payload and the logs.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::NotConnected => "not_connected",
            Self::MissingWriteScope => "missing_write_scope",
            Self::TooLarge(_) => "too_large",
            Self::Provider(_) => "provider_error",
        }
    }

    /// Message shown to the user.
    pub fn message(&self) -> String {
        match self {
            Self::NotConfigured => {
                "this deployment has no OAuth client for that drive".to_string()
            }
            Self::NotConnected => "the account is not connected".to_string(),
            Self::MissingWriteScope => {
                "the connection only allows reading; grant write access and try again".to_string()
            }
            Self::TooLarge(size) => format!(
                "the file is {size} bytes; it exceeds the single-request upload limit of this drive"
            ),
            Self::Provider(detail) => format!("the provider rejected the upload: {detail}"),
        }
    }
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for UploadError {}

/// True when the provider's recorded consent allows writing. `Files.ReadWrite`
/// on OneDrive implies the root is writable; `Files.ReadWrite.AppFolder` limits
/// writes to the app folder.
pub fn allows_write(provider: Provider, granted: &str) -> bool {
    provider.granted_scopes_allow_write(granted)
}

/// Whether OneDrive writes must stay inside the app folder.
pub fn onedrive_app_folder_only(granted: &str) -> bool {
    let granted = granted.to_ascii_lowercase();
    let broad = granted
        .split_whitespace()
        .any(|scope| scope == "files.readwrite" || scope == "files.readwrite.all");
    !broad
}

/// Path recorded in the local mirror for a file that was just uploaded. Uses
/// the same segment rules as the OneDrive request builder so the listing shows
/// the path the provider actually stored (and a `../` in the request can never
/// land in the mirror either).
pub fn mirror_path(folder: &str, name: &str) -> String {
    let folder = crate::external::onedrive_upload::sanitize_path(folder);
    let name = crate::external::onedrive_upload::sanitize_name(name);
    if folder.is_empty() {
        name
    } else {
        format!("{folder}/{name}")
    }
}

/// Prepares the pure request data for one upload. Kept separate from the send
/// so the URL/body construction is testable without a network.
pub fn build_target(req: &UploadRequest, granted: &str) -> Result<PreparedUpload, UploadError> {
    if req.content.len() > max_upload_bytes(req.provider) {
        return Err(UploadError::TooLarge(req.content.len()));
    }
    match req.provider {
        Provider::GoogleDrive => {
            let target = crate::external::gdrive_upload::build_upload(
                &req.name,
                req.parent_id.as_deref(),
                &req.mime_type,
                &req.content,
                req.replace_remote_id.as_deref(),
            );
            Ok(PreparedUpload {
                method: target.method,
                url: target.url,
                content_type: target.content_type,
                body: target.body,
            })
        }
        Provider::OneDrive => {
            let base = if onedrive_app_folder_only(granted) {
                crate::external::onedrive_upload::OneDriveBase::AppFolder
            } else {
                crate::external::onedrive_upload::OneDriveBase::Root
            };
            let target = crate::external::onedrive_upload::build_upload(
                &req.path,
                &req.name,
                &req.mime_type,
                &req.content,
                req.replace_remote_id.as_deref(),
                base,
            );
            Ok(PreparedUpload {
                method: target.method,
                url: target.url,
                content_type: target.content_type,
                body: target.body,
            })
        }
    }
}

/// The wire-ready request built from an [`UploadRequest`].
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedUpload {
    pub method: &'static str,
    pub url: String,
    pub content_type: String,
    pub body: Vec<u8>,
}

/// Sends a prepared upload and parses the provider's answer.
pub async fn send(
    http: &reqwest::Client,
    provider: Provider,
    access_token: &str,
    name: &str,
    prepared: PreparedUpload,
) -> Result<UploadedFile, SyncError> {
    let method = match prepared.method {
        "PATCH" => reqwest::Method::PATCH,
        "POST" => reqwest::Method::POST,
        _ => reqwest::Method::PUT,
    };
    // What we actually sent, used when the provider omits `size` from its
    // answer — reporting 0 bytes for a stored file would be a lie the UI
    // shows back to the user.
    let sent_bytes = prepared.body.len();
    let res = http
        .request(method, &prepared.url)
        .bearer_auth(access_token)
        .header(reqwest::header::CONTENT_TYPE, prepared.content_type)
        .body(prepared.body)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("upload request failed: {e}")))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(classify_upload_status(
            status.as_u16(),
            &body,
            provider,
        ));
    }
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("upload response unreadable: {e}")))?;
    let fallback = name.to_string();
    let (remote_id, stored_name, link, bytes) = match provider {
        Provider::GoogleDrive => {
            crate::external::gdrive_upload::parse_upload(&json, &fallback, sent_bytes)
        }
        Provider::OneDrive => {
            crate::external::onedrive_upload::parse_upload(&json, &fallback, sent_bytes)
        }
    };
    if remote_id.is_empty() {
        return Err(SyncError::Permanent(
            "the provider did not return an id for the uploaded file".to_string(),
        ));
    }
    Ok(UploadedFile {
        provider: provider.as_str(),
        remote_id,
        name: stored_name,
        web_view_link: link,
        bytes,
    })
}

/// Maps a provider rejection onto the sync error vocabulary. A 401/403 on an
/// upload means the grant is insufficient (scope or revoked consent), which the
/// route reports as `missing_write_scope` for the consent button.
fn classify_upload_status(status: u16, body: &str, provider: Provider) -> SyncError {
    if status == 401 || status == 403 {
        return SyncError::Auth(format!(
            "{} refused the write (HTTP {status}); the grant may be read-only",
            provider.display_name()
        ));
    }
    match provider {
        Provider::GoogleDrive => crate::external::gdrive::classify_status(status, body),
        Provider::OneDrive => crate::external::onedrive::classify_status(status, body),
    }
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod tests;
