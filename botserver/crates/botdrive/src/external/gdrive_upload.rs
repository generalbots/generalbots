//! Google Drive uploads (#2047).
//!
//! `multipart/related` in one request: a metadata part (`name`, optional
//! `parents`) followed by the file bytes. Create uses `files.create`; replace
//! uses `files.update` on the known remote id. Kept apart from the read client
//! so each file stays focused (and under the size limit).

use serde_json::Value;

use super::gdrive::{percent_encode, DRIVE_API};

/// Fields returned by an upload; `size` is a string in Drive v3.
const UPLOAD_FIELDS: &str = "id,name,webViewLink,size";
/// Boundary for `multipart/related` uploads. Fixed: the body is generated and
/// sent in one request, so there is nothing to negotiate.
const UPLOAD_BOUNDARY: &str = "gb_drive_upload_boundary";

/// A fully prepared upload request. Pure data so it can be asserted in a test
/// without a network or a fake HTTP server (#2047).
#[derive(Debug, Clone, PartialEq)]
pub struct UploadTarget {
    /// HTTP method (`POST` to create, `PATCH` to replace).
    pub method: &'static str,
    /// Absolute URL.
    pub url: String,
    /// `Content-Type` header for the body.
    pub content_type: String,
    /// Request body.
    pub body: Vec<u8>,
}

/// Builds a Drive `multipart/related` upload: a metadata part (`name`, and
/// `parents` when a folder is given) followed by the file bytes.
pub fn build_upload(
    name: &str,
    parent_id: Option<&str>,
    mime_type: &str,
    bytes: &[u8],
    replace_remote_id: Option<&str>,
) -> UploadTarget {
    let mut metadata = serde_json::Map::new();
    metadata.insert("name".into(), Value::String(name.to_string()));
    if let Some(parent) = parent_id.filter(|parent| !parent.is_empty()) {
        metadata.insert(
            "parents".into(),
            Value::Array(vec![Value::String(parent.to_string())]),
        );
    }
    let metadata = Value::Object(metadata).to_string();
    let mime_type = if mime_type.trim().is_empty() {
        "application/octet-stream"
    } else {
        mime_type
    };

    let mut body = Vec::with_capacity(metadata.len() + bytes.len() + 256);
    body.extend_from_slice(
        format!(
            "--{UPLOAD_BOUNDARY}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!("--{UPLOAD_BOUNDARY}\r\nContent-Type: {mime_type}\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{UPLOAD_BOUNDARY}--\r\n").as_bytes());

    let (method, url) = match replace_remote_id.filter(|id| !id.is_empty()) {
        Some(id) => (
            "PATCH",
            format!(
                "{DRIVE_API}/files/{}?uploadType=multipart&fields={}",
                percent_encode(id),
                percent_encode(UPLOAD_FIELDS)
            ),
        ),
        None => (
            "POST",
            format!(
                "{DRIVE_API}/files?uploadType=multipart&fields={}",
                percent_encode(UPLOAD_FIELDS)
            ),
        ),
    };
    UploadTarget {
        method,
        url,
        content_type: format!("multipart/related; boundary={UPLOAD_BOUNDARY}"),
        body,
    }
}

/// Parses an upload response into the fields the API reports back.
pub fn parse_upload(json: &Value, fallback_name: &str, bytes: usize) -> (String, String, Option<String>, usize) {
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
        .get("webViewLink")
        .and_then(Value::as_str)
        .map(str::to_string);
    let size = json
        .get("size")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(bytes);
    (remote_id, name, link, size)
}

