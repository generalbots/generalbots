//! OneDrive uploads (#2047).
//!
//! Simple upload: `PUT /me/drive/root:/{path}:/content` for a new file and
//! `PUT /me/drive/items/{item-id}/content` to replace one. The body is the raw
//! bytes, which keeps the request small enough for the JSON API shape the chat
//! uses (a file above the simple-upload limit is rejected before this runs).
//! Kept apart from the read client so each file stays focused.

use serde_json::Value;

use super::onedrive::{percent_encode, GRAPH_BASE};

/// Where a write is allowed to land. `Files.ReadWrite.AppFolder` only grants
/// the app's own folder, so a save must not be addressed to the drive root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneDriveBase {
    /// The account's drive root (needs `Files.ReadWrite`/`Files.ReadWrite.All`).
    Root,
    /// This app's folder (covered by `Files.ReadWrite.AppFolder`).
    AppFolder,
}

/// A fully prepared upload request. Pure data so the URL/body construction is
/// testable without a network (#2047).
#[derive(Debug, Clone, PartialEq)]
pub struct UploadTarget {
    /// Always `PUT` for Graph simple upload.
    pub method: &'static str,
    /// Absolute URL.
    pub url: String,
    /// `Content-Type` header for the body.
    pub content_type: String,
    /// Request body (the raw file bytes).
    pub body: Vec<u8>,
}

/// Percent-encodes each path segment, leaving `/` as the separator so Graph
/// reads the `root:/folder/file:` syntax correctly.
fn encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(percent_encode)
        .collect::<Vec<_>>()
        .join("/")
}

/// Builds the upload request. `path` is the destination relative to the drive
/// root (or to the app folder, which is where a `Files.ReadWrite.AppFolder`
/// grant may write); `replace_remote_id` switches to the item endpoint.
pub fn build_upload(
    path: &str,
    name: &str,
    mime_type: &str,
    bytes: &[u8],
    replace_remote_id: Option<&str>,
    base: OneDriveBase,
) -> UploadTarget {
    let full_path = match (path.trim_matches('/'), name) {
        ("", name) => name.to_string(),
        (path, name) => format!("{path}/{name}"),
    };
    let url = match replace_remote_id.filter(|id| !id.is_empty()) {
        Some(id) => format!("{GRAPH_BASE}/me/drive/items/{}/content", percent_encode(id)),
        None => format!(
            "{GRAPH_BASE}/me/drive/{}:/{}:/content",
            match base {
                OneDriveBase::Root => "root",
                OneDriveBase::AppFolder => "special/approot",
            },
            encode_path(&full_path)
        ),
    };
    UploadTarget {
        method: "PUT",
        url,
        content_type: if mime_type.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            mime_type.to_string()
        },
        body: bytes.to_vec(),
    }
}

/// Parses a Graph item response into the fields the API reports back.
pub fn parse_upload(
    json: &Value,
    fallback_name: &str,
    bytes: usize,
) -> (String, String, Option<String>, usize) {
    let remote_id = json
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let name = json
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback_name)
        .to_string();
    let link = json
        .get("webUrl")
        .and_then(Value::as_str)
        .map(str::to_string);
    let size = json
        .get("size")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(bytes);
    (remote_id, name, link, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_new_file_uses_the_root_path_endpoint() {
        let target = build_upload(
            "Relatórios 2026",
            "pib.xlsx",
            "application/vnd.ms-excel",
            b"x",
            None,
            OneDriveBase::Root,
        );
        assert_eq!(target.method, "PUT");
        assert!(target.url.contains("/me/drive/root:/Relat%C3%B3rios%202026/pib.xlsx:/content"));
        assert_eq!(target.content_type, "application/vnd.ms-excel");
        assert_eq!(target.body, b"x");
    }

    #[test]
    fn replacing_uses_the_item_endpoint_and_ignores_the_path() {
        let target = build_upload("ignored", "pib.xlsx", "", b"y", Some("item-42"), OneDriveBase::Root);
        assert!(target.url.ends_with("/me/drive/items/item-42/content"));
        assert_eq!(target.content_type, "application/octet-stream");
    }

    #[test]
    fn an_app_folder_grant_writes_inside_the_app_folder() {
        let target = build_upload("artifacts", "pib.xlsx", "text/plain", b"z", None, OneDriveBase::AppFolder);
        assert!(target
            .url
            .contains("/me/drive/special/approot:/artifacts/pib.xlsx:/content"));
    }

    #[test]
    fn a_response_reports_id_link_and_size() {
        let json = json!({
            "id": "item-1",
            "name": "pib.xlsx",
            "webUrl": "https://onedrive.example/pib.xlsx",
            "size": 12
        });
        let (id, name, link, size) = parse_upload(&json, "fallback.xlsx", 4);
        assert_eq!(id, "item-1");
        assert_eq!(name, "pib.xlsx");
        assert_eq!(link.as_deref(), Some("https://onedrive.example/pib.xlsx"));
        assert_eq!(size, 12);
    }
}
