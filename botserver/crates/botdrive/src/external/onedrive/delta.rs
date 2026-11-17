use crate::external::types::{DeltaPage, ExternalFile, SyncError};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Graph endpoint used for the change feed.
pub(super) const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";
/// First call of a delta run (no cursor yet).
pub(super) const DELTA_ROOT: &str = "https://graph.microsoft.com/v1.0/me/drive/root/delta";
/// Token endpoint (tenant is part of the path).
pub(super) const TOKEN_PATH: &str = "/oauth2/v2.0/token";

/// Fetch one page of the change feed. `cursor` is a previously stored
/// `@odata.deltaLink`; `None` starts (or restarts) a full traversal.
pub async fn fetch_page(
    http: &reqwest::Client,
    access_token: &str,
    cursor: Option<&str>,
) -> Result<DeltaPage, SyncError> {
    let url = cursor.unwrap_or(DELTA_ROOT).to_string();
    let res = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("delta request failed: {e}")))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(classify_status(status.as_u16(), &body));
    }
    let json: Value = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("delta response unreadable: {e}")))?;
    Ok(parse_delta(&json))
}

/// Download the bytes of one item.
pub async fn download(
    http: &reqwest::Client,
    access_token: &str,
    remote_id: &str,
) -> Result<Vec<u8>, SyncError> {
    let url = format!(
        "{GRAPH_BASE}/me/drive/items/{}/content",
        percent_encode(remote_id)
    );
    let res = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("download request failed: {e}")))?;
    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(classify_status(status.as_u16(), &body));
    }
    res.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| SyncError::Transient(format!("download body failed: {e}")))
}

/// Turn a Graph error into the right retry decision.
pub fn classify_status(status: u16, body: &str) -> SyncError {
    if status == 401 || status == 403 {
        // A revoked consent or an expired refresh token: retrying is pointless
        // until the user reconnects.
        return SyncError::Auth(truncate(body));
    }
    if status == 429 {
        return SyncError::RateLimited(None);
    }
    if status >= 500 {
        return SyncError::Transient(format!("graph {status}: {}", truncate(body)));
    }
    SyncError::Permanent(format!("graph {status}: {}", truncate(body)))
}

/// Parse a `delta` response into a page of changes.
///
/// Handles both page shapes: `@odata.nextLink` while the traversal is still
/// running, `@odata.deltaLink` on the page that closes it, and the `deleted`
/// facet for removals.
pub fn parse_delta(json: &Value) -> DeltaPage {
    let mut page = DeltaPage::default();
    if let Some(values) = json.get("value").and_then(Value::as_array) {
        for item in values {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            if item.get("deleted").is_some() {
                page.removed.push(id.to_string());
                continue;
            }
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let is_folder = item.get("folder").is_some();
            let path = item
                .get("parentReference")
                .and_then(|p| p.get("path"))
                .and_then(Value::as_str)
                .map(|p| format!("{}/{}", p.trim_matches('/'), name))
                .unwrap_or_else(|| name.clone());
            let mut f = ExternalFile::new(id, name, path, is_folder);
            f.mime_type = item
                .get("file")
                .and_then(|fi| fi.get("mimeType"))
                .and_then(Value::as_str)
                .map(String::from);
            f.size_bytes = item.get("size").and_then(Value::as_i64).unwrap_or(0);
            f.modified_at = item
                .get("lastModifiedDateTime")
                .and_then(Value::as_str)
                .and_then(parse_ts);
            f.revision = item
                .get("eTag")
                .and_then(Value::as_str)
                .map(|s| s.trim_matches(['"', '{', '}']).to_string());
            f.parent_remote_id = item
                .get("parentReference")
                .and_then(|p| p.get("id"))
                .and_then(Value::as_str)
                .map(String::from);
            page.items.push(f);
        }
    }
    page.next_cursor = json
        .get("@odata.nextLink")
        .and_then(Value::as_str)
        .map(String::from);
    page.delta = json
        .get("@odata.deltaLink")
        .and_then(Value::as_str)
        .map(String::from);
    page
}

/// Parse an ISO-8601 timestamp, ignoring anything malformed.
pub(crate) fn parse_ts(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
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
