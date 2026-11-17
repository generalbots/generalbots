use super::parse::{classify_status, parse_page, percent_encode, CHANGE_FIELDS, DRIVE_API, FILE_FIELDS};
use crate::external::types::{DeltaPage, SyncError};
use serde_json::Value;

/// Obtain the cursor a delta run starts from. Called once per connection.
pub async fn start_page_token(
    http: &reqwest::Client,
    access_token: &str,
) -> Result<String, SyncError> {
    let url = format!("{DRIVE_API}/changes/startPageToken");
    let res = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("startPageToken failed: {e}")))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(classify_status(status.as_u16(), &body));
    }
    let json: Value = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("startPageToken unreadable: {e}")))?;
    json.get("startPageToken")
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| SyncError::Transient("startPageToken missing from response".into()))
}

/// Fetch one page of the change feed. `cursor` is a stored page token.
pub async fn fetch_page(
    http: &reqwest::Client,
    access_token: &str,
    cursor: Option<&str>,
) -> Result<DeltaPage, SyncError> {
    match cursor {
        // No cursor yet: full traversal of the user's files.
        None => fetch_list_page(http, access_token, None).await,
        Some(token) => fetch_change_page(http, access_token, Some(token)).await,
    }
}

/// One page of the initial `files.list` traversal. `page_token` continues a
/// traversal already in progress — it is *not* a stored cursor, which is why
/// the two functions are separate.
pub async fn fetch_list_page(
    http: &reqwest::Client,
    access_token: &str,
    page_token: Option<&str>,
) -> Result<DeltaPage, SyncError> {
    let url = match page_token {
        Some(token) => format!(
            "{DRIVE_API}/files?pageSize=100&pageToken={}&fields={FILE_FIELDS}",
            percent_encode(token)
        ),
        None => format!("{DRIVE_API}/files?pageSize=100&fields={FILE_FIELDS}"),
    };
    get_page(http, &url, access_token).await
}

/// One page of `changes.list`, replayed from a stored start page token.
pub async fn fetch_change_page(
    http: &reqwest::Client,
    access_token: &str,
    page_token: Option<&str>,
) -> Result<DeltaPage, SyncError> {
    let url = match page_token {
        Some(token) => format!(
            "{DRIVE_API}/changes?pageToken={}&fields={CHANGE_FIELDS}",
            percent_encode(token)
        ),
        None => format!("{DRIVE_API}/changes?fields={CHANGE_FIELDS}"),
    };
    get_page(http, &url, access_token).await
}

async fn get_page(
    http: &reqwest::Client,
    url: &str,
    access_token: &str,
) -> Result<DeltaPage, SyncError> {
    let res = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("drive request failed: {e}")))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(classify_status(status.as_u16(), &body));
    }
    let json: Value = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("drive response unreadable: {e}")))?;
    Ok(parse_page(&json))
}

/// Download the bytes of one file.
pub async fn download(
    http: &reqwest::Client,
    access_token: &str,
    remote_id: &str,
) -> Result<Vec<u8>, SyncError> {
    let url = format!("{DRIVE_API}/files/{}?alt=media", percent_encode(remote_id));
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
