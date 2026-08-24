use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Container {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RawItem {
    pub external_id: String,
    pub title: String,
    pub body: Option<String>,
    pub container_id: String,
    pub acl_principals: Vec<String>,
    pub external_url: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A pluggable external knowledge source. Implementations are REST-configurable:
/// every endpoint (base URL and path template) is read from the connection's
/// credentials JSON, never hardcoded to a vendor.
pub trait KnowledgeConnector: Send + Sync {
    fn kind(&self) -> &'static str;
    fn list_containers(&self, credentials: &Value) -> Result<Vec<Container>, String>;
    fn iter_items(
        &self,
        credentials: &Value,
        container: &Container,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RawItem>, String>;
}

const DEFAULT_CONTAINERS_PATH_CHAT: &str = "/api/v1/channels";
const DEFAULT_ITEMS_PATH_CHAT: &str = "/api/v1/channels/{container}/messages";
const DEFAULT_CONTAINERS_PATH_MAIL: &str = "/api/v1/folders";
const DEFAULT_ITEMS_PATH_MAIL: &str = "/api/v1/folders/{container}/messages";
const DEFAULT_CONTAINERS_PATH_DRIVE: &str = "/api/v1/drives";
const DEFAULT_ITEMS_PATH_DRIVE: &str = "/api/v1/drives/{container}/files";
const HTTP_TIMEOUT_SECS: u64 = 30;

/// REST-configurable chat connector.
///
/// Expected credential shape (JSON object stored in Vault at the connection's
/// `vault_token_ref`):
///
/// ```json
/// {
///   "base_url": "https://chat.example.com",
///   "token": "<secret bearer material>",
///   "containers_path": "/api/v1/channels",
///   "items_path": "/api/v1/channels/{container}/messages",
///   "token_header": "Authorization",
///   "token_scheme": "Bearer",
///   "since_param": "since",
///   "limit_param": "limit",
///   "limit": "200"
/// }
/// ```
///
/// Only `base_url` and `token` are required; the remaining keys fall back to
/// the defaults shown above. `{container}` in `items_path` is substituted with
/// the percent-encoded container identifier. Containers are objects with
/// `id`/`name`; items are objects with `id`, `title|subject|name`,
/// `body|text|content`, optional `url|external_url|web_url`, optional
/// `acl|acl_principals` (array of principal strings) and optional
/// `updated_at|edited_at` (ISO-8601).
pub struct ChatConnector;

/// REST-configurable mail connector (folders/messages over a generic API).
///
/// Expected credential shape — identical contract to [`ChatConnector`] with
/// mail-specific defaults:
///
/// ```json
/// {
///   "base_url": "https://mail.example.com",
///   "token": "<secret bearer material>",
///   "containers_path": "/api/v1/folders",
///   "items_path": "/api/v1/folders/{container}/messages",
///   "token_header": "Authorization",
///   "token_scheme": "Bearer",
///   "since_param": "since",
///   "limit_param": "limit",
///   "limit": "200"
/// }
/// ```
///
/// Containers are folders (`id`/`name`); items are messages resolved as
/// `id`, subject from `title|subject`, body from `body|preview|text|content`,
/// sender-independent ACL from `acl|acl_principals`, timestamp from
/// `updated_at|received_at|date`.
pub struct MailConnector;

/// REST-configurable drive connector (drives/files over a generic API).
///
/// Expected credential shape — identical contract to [`ChatConnector`] with
/// drive-specific defaults:
///
/// ```json
/// {
///   "base_url": "https://drive.example.com",
///   "token": "<secret bearer material>",
///   "containers_path": "/api/v1/drives",
///   "items_path": "/api/v1/drives/{container}/files",
///   "token_header": "Authorization",
///   "token_scheme": "Bearer",
///   "since_param": "modified_since",
///   "limit_param": "page_size",
///   "limit": "200"
/// }
/// ```
///
/// Containers are drives/folders (`id`/`name`); items are files resolved as
/// `id`, name from `title|name|file_name`, textual content from
/// `body|content|text` (plain text exports only), link from
/// `url|external_url|web_url`, sharing principals from `acl|acl_principals`
/// and revision time from `updated_at|modified_at`.
pub struct DriveConnector;

fn cred_string(cred: &Value, key: &str, default: &str) -> String {
    cred.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| default.to_string())
}

fn cred_token(cred: &Value) -> Option<String> {
    for key in ["token", "access_token", "api_key"] {
        if let Some(t) = cred.get(key).and_then(Value::as_str).filter(|t| !t.is_empty()) {
            return Some(t.to_string());
        }
    }
    None
}

fn base_url(cred: &Value) -> Result<String, String> {
    let raw = cred_string(cred, "base_url", "");
    if raw.is_empty() {
        return Err("credentials.base_url is required".to_string());
    }
    Ok(raw.trim_end_matches('/').to_string())
}

fn resolve_path(cred: &Value, key: &str, default: &str) -> Result<String, String> {
    let path = cred_string(cred, key, default);
    if path.starts_with("http://") || path.starts_with("https://") {
        Ok(path.trim_end_matches('/').to_string())
    } else {
        Ok(format!("{}{}", base_url(cred)?, path))
    }
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn http_get_json(url: &str, cred: &Value) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client build failed: {e}"))?;
    let mut request = client.get(url);
    if let Some(token) = cred_token(cred) {
        let header = cred_string(cred, "token_header", "Authorization");
        let scheme = cred_string(cred, "token_scheme", "Bearer");
        let value =
            if scheme.is_empty() { token } else { format!("{scheme} {token}") };
        request = request.header(header, value);
    }
    let response =
        request.send().map_err(|e| format!("GET {url} failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("GET {url} returned status {status}"));
    }
    response.json::<Value>().map_err(|e| format!("GET {url} body decode: {e}"))
}

fn response_items(root: &Value) -> Vec<Value> {
    if let Some(arr) = root.as_array() {
        return arr.clone();
    }
    for key in ["items", "value", "data", "channels", "folders", "drives", "files", "messages", "results"] {
        if let Some(arr) = root.get(key).and_then(Value::as_array) {
            return arr.clone();
        }
    }
    Vec::new()
}

fn pick<'a>(obj: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(v) = obj.get(*key).and_then(Value::as_str).filter(|s| !s.is_empty()) {
            return Some(v);
        }
    }
    None
}

fn parse_time(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|t| t.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|t| t.and_utc())
        })
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|t| t.and_utc())
        })
}

fn item_updated_at(obj: &Value) -> Option<DateTime<Utc>> {
    pick(obj, &["updated_at", "edited_at", "modified_at", "received_at", "date"]).and_then(parse_time)
}

fn item_acl(obj: &Value) -> Vec<String> {
    for key in ["acl", "acl_principals"] {
        if let Some(arr) = obj.get(key).and_then(Value::as_array) {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    Vec::new()
}

#[derive(Deserialize)]
struct GenericConnectorConfig {
    #[serde(default)]
    containers_path: Option<String>,
    #[serde(default)]
    items_path: Option<String>,
}

impl GenericConnectorConfig {
    fn from_cred(cred: &Value) -> Self {
        serde_json::from_value(cred.clone()).unwrap_or(Self {
            containers_path: None,
            items_path: None,
        })
    }

    fn containers_path(&self, default: &str) -> String {
        self.containers_path.clone().unwrap_or_else(|| default.to_string())
    }

    fn items_path(&self, default: &str) -> String {
        self.items_path.clone().unwrap_or_else(|| default.to_string())
    }
}

fn list_containers_generic(
    cred: &Value,
    default_containers_path: &str,
) -> Result<Vec<Container>, String> {
    let url = resolve_path(cred, "containers_path", default_containers_path)?;
    let root = http_get_json(&url, cred)?;
    Ok(response_items(&root)
        .into_iter()
        .filter_map(|entry| {
            let id = pick(&entry, &["id", "key", "folder_id", "channel_id", "drive_id"])?.to_string();
            let name = pick(&entry, &["name", "display_name", "title", "label"])
                .unwrap_or(&id)
                .to_string();
            Some(Container { id, name })
        })
        .collect())
}

fn iter_items_generic(
    cred: &Value,
    container: &Container,
    since: Option<DateTime<Utc>>,
    default_items_path: &str,
) -> Result<Vec<RawItem>, String> {
    let config = GenericConnectorConfig::from_cred(cred);
    let mut url = resolve_path(cred, "items_path", default_items_path)?
        .replace("{container}", &percent_encode(&container.id));
    let mut params: Vec<(String, String)> = Vec::new();
    if let Some(ts) = since {
        params.push((cred_string(cred, "since_param", "since"), ts.to_rfc3339()));
    }
    params.push((
        cred_string(cred, "limit_param", "limit"),
        cred_string(cred, "limit", "200"),
    ));
    let query: Vec<String> =
        params.iter().map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v))).collect();
    url.push('?');
    url.push_str(&query.join("&"));

    let root = http_get_json(&url, cred)?;
    Ok(response_items(&root)
        .into_iter()
        .filter_map(|entry| {
            let external_id = pick(&entry, &["id", "message_id", "file_id", "external_id"])?.to_string();
            let title = pick(&entry, &["title", "subject", "name", "file_name", "summary"])
                .unwrap_or(&external_id)
                .to_string();
            let body = pick(&entry, &["body", "text", "content", "preview", "snippet"])
                .map(|s| s.to_string());
            let external_url =
                pick(&entry, &["url", "external_url", "web_url", "link"]).map(|s| s.to_string());
            Some(RawItem {
                external_id,
                title,
                body,
                container_id: container.id.clone(),
                acl_principals: item_acl(&entry),
                external_url,
                updated_at: item_updated_at(&entry),
            })
        })
        .collect())
}

impl KnowledgeConnector for ChatConnector {
    fn kind(&self) -> &'static str {
        "chat"
    }

    fn list_containers(&self, cred: &Value) -> Result<Vec<Container>, String> {
        list_containers_generic(cred, DEFAULT_CONTAINERS_PATH_CHAT)
    }

    fn iter_items(
        &self,
        cred: &Value,
        container: &Container,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RawItem>, String> {
        iter_items_generic(cred, container, since, DEFAULT_ITEMS_PATH_CHAT)
    }
}

impl KnowledgeConnector for MailConnector {
    fn kind(&self) -> &'static str {
        "mail"
    }

    fn list_containers(&self, cred: &Value) -> Result<Vec<Container>, String> {
        list_containers_generic(cred, DEFAULT_CONTAINERS_PATH_MAIL)
    }

    fn iter_items(
        &self,
        cred: &Value,
        container: &Container,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RawItem>, String> {
        iter_items_generic(cred, container, since, DEFAULT_ITEMS_PATH_MAIL)
    }
}

impl KnowledgeConnector for DriveConnector {
    fn kind(&self) -> &'static str {
        "drive"
    }

    fn list_containers(&self, cred: &Value) -> Result<Vec<Container>, String> {
        list_containers_generic(cred, DEFAULT_CONTAINERS_PATH_DRIVE)
    }

    fn iter_items(
        &self,
        cred: &Value,
        container: &Container,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<RawItem>, String> {
        iter_items_generic(cred, container, since, DEFAULT_ITEMS_PATH_DRIVE)
    }
}
