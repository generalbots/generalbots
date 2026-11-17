use crate::external::types::{DeltaPage, ExternalFile, SyncError};
use serde_json::Value;

/// Drive API root.
pub(super) const DRIVE_API: &str = "https://www.googleapis.com/drive/v3";
/// Folder mime type — the only one that is a directory rather than a blob.
pub(super) const FOLDER_MIME: &str = "application/vnd.google-apps.folder";
/// The field mask keeps responses small; the default projection omits nothing
/// we need but is much larger.
pub(super) const FILE_FIELDS: &str =
    "nextPageToken,files(id,name,mimeType,size,modifiedTime,version,parents,trashed)";
pub(super) const CHANGE_FIELDS: &str = "nextPageToken,newStartPageToken,\
changes(fileId,removed,file(id,name,mimeType,size,modifiedTime,version,parents,trashed))";

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
