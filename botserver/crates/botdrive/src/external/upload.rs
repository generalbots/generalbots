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
                "the file is {size} bytes; the simple upload limit is {MAX_UPLOAD_BYTES} bytes"
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

/// Prepares the pure request data for one upload. Kept separate from the send
/// so the URL/body construction is testable without a network.
pub fn build_target(req: &UploadRequest, granted: &str) -> Result<PreparedUpload, UploadError> {
    if req.content.len() > MAX_UPLOAD_BYTES {
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
            crate::external::gdrive_upload::parse_upload(&json, &fallback, 0)
        }
        Provider::OneDrive => {
            crate::external::onedrive_upload::parse_upload(&json, &fallback, 0)
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
mod tests {
    use super::*;

    fn request(provider: Provider, name: &str) -> UploadRequest {
        UploadRequest {
            provider,
            path: "artifacts".to_string(),
            name: name.to_string(),
            mime_type: "text/plain".to_string(),
            content: b"hello".to_vec(),
            replace_remote_id: None,
            parent_id: None,
        }
    }

    #[test]
    fn oversized_content_is_refused_before_building_a_request() {
        let mut req = request(Provider::GoogleDrive, "big.txt");
        req.content = vec![0; MAX_UPLOAD_BYTES + 1];
        assert_eq!(
            build_target(&req, ""),
            Err(UploadError::TooLarge(MAX_UPLOAD_BYTES + 1))
        );
        assert_eq!(UploadError::TooLarge(1).kind(), "too_large");
    }

    #[test]
    fn error_kinds_are_the_contract_the_ui_reads() {
        assert_eq!(UploadError::NotConfigured.kind(), "not_configured");
        assert_eq!(UploadError::NotConnected.kind(), "not_connected");
        assert_eq!(
            UploadError::MissingWriteScope.kind(),
            "missing_write_scope"
        );
        assert_eq!(UploadError::Provider("x".into()).kind(), "provider_error");
    }

    #[test]
    fn write_capability_comes_from_the_recorded_scope() {
        assert!(allows_write(
            Provider::GoogleDrive,
            "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/drive.file"
        ));
        assert!(!allows_write(
            Provider::GoogleDrive,
            "https://www.googleapis.com/auth/drive.readonly"
        ));
        assert!(!allows_write(Provider::OneDrive, ""));
        assert!(allows_write(
            Provider::OneDrive,
            "offline_access Files.Read.All Files.ReadWrite.AppFolder User.Read"
        ));
    }

    #[test]
    fn onedrive_uses_the_app_folder_unless_a_broad_write_scope_was_granted() {
        assert!(onedrive_app_folder_only(
            "offline_access Files.Read.All Files.ReadWrite.AppFolder"
        ));
        assert!(!onedrive_app_folder_only("Files.ReadWrite"));
        assert!(!onedrive_app_folder_only("Files.ReadWrite.All"));
    }

    #[test]
    fn google_target_is_multipart_related_with_metadata() {
        let mut req = request(Provider::GoogleDrive, "pib.csv");
        req.parent_id = Some("folder-9".to_string());
        let prepared = build_target(&req, "").expect("target");
        assert_eq!(prepared.method, "POST");
        assert!(prepared.url.contains("uploadType=multipart"));
        assert!(prepared
            .content_type
            .starts_with("multipart/related; boundary="));
        let body = String::from_utf8_lossy(&prepared.body);
        assert!(body.contains(r#""name":"pib.csv""#));
        assert!(body.contains(r#""parents":["folder-9"]"#));
        assert!(body.contains("hello"));
    }

    #[test]
    fn onedrive_target_puts_the_bytes_to_the_path() {
        let prepared = build_target(
            &request(Provider::OneDrive, "pib.csv"),
            "Files.ReadWrite.AppFolder",
        )
        .expect("target");
        assert_eq!(prepared.method, "PUT");
        assert!(prepared
            .url
            .contains("/me/drive/special/approot:/artifacts/pib.csv:/content"));
        assert_eq!(prepared.body, b"hello");
    }
}
