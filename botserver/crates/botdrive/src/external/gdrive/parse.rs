use crate::external::types::{DeltaPage, ExternalFile, SyncError};
use serde_json::Value;

/// Drive API root.
pub(super) const DRIVE_API: &str = "https://www.googleapis.com/drive/v3";
/// OAuth endpoints.
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Token endpoint; overridable for a local mock in tests or a proxy mirror.
fn token_url() -> String {
    std::env::var("EXTERNAL_DRIVE_GOOGLE_TOKEN_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TOKEN_URL.to_string())
}
/// Folder mime type — the only one that is a directory rather than a blob.
pub(super) const FOLDER_MIME: &str = "application/vnd.google-apps.folder";
/// The field mask keeps responses small; the default projection omits nothing
/// we need but is much larger.
pub(super) const FILE_FIELDS: &str =
    "nextPageToken,files(id,name,mimeType,size,modifiedTime,version,parents,trashed)";
pub(super) const CHANGE_FIELDS: &str = "nextPageToken,newStartPageToken,\
changes(fileId,removed,file(id,name,mimeType,size,modifiedTime,version,parents,trashed))";

/// A token set as returned by the Google token endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSet {
    /// Bearer access token.
    pub access_token: String,
    /// Refresh token (issued because we request `access_type=offline`).
    pub refresh_token: Option<String>,
    /// Absolute expiry, derived from `expires_in`.
    pub expires_at: DateTime<Utc>,
    /// Scopes the provider reports as granted (#1643). Falls back to the
    /// requested list when the provider omits it.
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
}

impl TokenResponse {
    fn into_token_set(self, now: DateTime<Utc>) -> TokenSet {
        TokenSet {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: now + chrono::Duration::seconds(self.expires_in.unwrap_or(3600)),
            scope: self.scope,
        }
    }
}

/// Build the authorization URL. `access_type=offline` + `prompt=consent` is
/// what makes Google issue a refresh token for a desktop-style client.
pub fn authorize_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    authorize_url_with_pkce(client_id, redirect_uri, state, None)
}

/// Build the authorization URL with an optional PKCE challenge. Managed
/// installed-app clients have no secret, so the desktop flow always supplies
/// the S256 challenge derived from the verifier carried in the signed state.
pub fn authorize_url_with_pkce(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: Option<&str>,
) -> String {
    // #2047 — one consent asks for read + write, so a save target exists from
    // the first connection; existing read-only grants are not rewritten.
    let scope = crate::external::types::Provider::GoogleDrive
        .scopes()
        .join(" ");

/// Turn a Drive error into the right retry decision.
///
/// Google signals throttling with 403 as often as 429, and the reason is only
/// in the body — so the status alone must not be trusted here.
pub fn classify_status(status: u16, body: &str) -> SyncError {
    if status == 401 {
        return SyncError::Auth(truncate(body));
    }
    if status == 429 || (status == 403 && body.contains("rateLimitExceeded")) {
        return SyncError::RateLimited(None);
    }
    if status == 403 {
        // No consent for this scope, or the app is not allowed to see the file.
        return SyncError::Auth(truncate(body));
    }
    if status >= 500 {
        return SyncError::Transient(format!("drive {status}: {}", truncate(body)));
    }
    SyncError::Permanent(format!("drive {status}: {}", truncate(body)))
}

/// Parse either a `files.list` or a `changes.list` response.
///
/// Both shapes are accepted by the same function on purpose: the first sync
/// walks files, later syncs walk changes, and the sync engine should not care
/// which one it is reading.
pub fn parse_page(json: &Value) -> DeltaPage {
    let mut page = DeltaPage::default();
    if let Some(files) = json.get("files").and_then(Value::as_array) {
        for file in files {
            if let Some(f) = parse_file(file) {
                page.items.push(f);
            }
        }
    }
    if let Some(changes) = json.get("changes").and_then(Value::as_array) {
        for change in changes {
            let Some(id) = change.get("fileId").and_then(Value::as_str).or_else(|| {
                change
                    .get("file")
                    .and_then(|f| f.get("id"))
                    .and_then(Value::as_str)
            }) else {
                continue;
            };
            if change
                .get("removed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                page.removed.push(id.to_string());
                continue;
            }
            // A change for a trashed file is a removal for our purposes: the
            // user deleted it from their Drive and we must not keep serving it.
            match change.get("file") {
                Some(file) if file.get("trashed").and_then(Value::as_bool) == Some(true) => {
                    page.removed.push(id.to_string());
                }
                // The parent comes from the file's own `parents` list; the
                // change entry's `fileId` is the item itself, not its folder.
                Some(file) => page.items.extend(parse_file(file)),
                None => {}
            }
        }
    }
    page.next_cursor = json
        .get("nextPageToken")
        .and_then(Value::as_str)
        .map(String::from);
    page.delta = json
        .get("newStartPageToken")
        .and_then(Value::as_str)
        .map(String::from);
    page
}

/// Parse one file entry, or `None` when it carries nothing usable.
fn parse_file(file: &Value) -> Option<ExternalFile> {
    let id = file.get("id").and_then(Value::as_str)?;
    let name = file.get("name").and_then(Value::as_str).unwrap_or_default();
    if name.is_empty() {
        return None;
    }
    let mime = file.get("mimeType").and_then(Value::as_str);
    let is_folder = mime == Some(FOLDER_MIME);
    // Drive is flat: the path is derived from the parent chain by the caller
    // when it walks, so the name alone is the best single-page answer here.
    let mut f = ExternalFile::new(id, name, name, is_folder);
    f.mime_type = mime.map(String::from);
    f.size_bytes = file
        .get("size")
        .and_then(|s| match s {
            Value::String(v) => v.parse::<i64>().ok(),
            Value::Number(n) => n.as_i64(),
            _ => None,
        })
        .unwrap_or(0);
    f.modified_at = file
        .get("modifiedTime")
        .and_then(Value::as_str)
        .and_then(crate::external::onedrive::parse_ts);
    f.revision = file
        .get("version")
        .and_then(Value::as_str)
        .map(String::from);
    f.parent_remote_id = file
        .get("parents")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .map(String::from);
    Some(f)
}

pub(super) fn truncate(body: &str) -> String {
    botlib::text::truncate_with_suffix(body.trim(), 300, "…")
}

pub(super) fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            other => format!("%{:02X}", other),
        })
        .collect()
}
