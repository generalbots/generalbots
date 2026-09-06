use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SharePointItemType {
    File,
    Folder,
    ListItem,
    DocumentSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointSite {
    pub id: String,
    pub display_name: String,
    pub url: String,
    pub description: Option<String>,
    pub web_template: Option<String>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointDrive {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub drive_type: String,
    pub quota_total_bytes: Option<i64>,
    pub quota_used_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointItem {
    pub id: String,
    pub name: String,
    pub item_type: SharePointItemType,
    pub web_url: String,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub parent_path: Option<String>,
    pub etag: Option<String>,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointList {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub list_template: String,
    pub item_count: i32,
    pub columns: Vec<SharePointColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointColumn {
    pub name: String,
    pub display_name: String,
    pub column_type: String,
    pub required: bool,
    pub indexed: bool,
}

#[derive(Clone)]
pub struct SharePointClient {
    pub tenant_id: String,
    pub access_token: String,
    pub graph_base_url: String,
    pub http_client: reqwest::blocking::Client,
}

impl SharePointClient {
    /// Microsoft Graph host — request URLs are always checked against this
    /// constant before use, so tokens cannot be sent to a different server.
    pub const GRAPH_HOST: &str = "https://graph.microsoft.com/v1.0";

    pub fn new(tenant_id: String, access_token: String) -> Self {
        Self {
            tenant_id,
            access_token,
            graph_base_url: Self::GRAPH_HOST.to_string(),
            http_client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Server-side request forgery guard for every URL built on top of the
    /// Graph base: the host must be the pinned Microsoft Graph endpoint.
    pub fn validate_url_for_graph(url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("invalid Graph URL: {e}"))?;
        let host = parsed.host_str().unwrap_or("");
        if parsed.scheme() != "https" || host != "graph.microsoft.com" {
            return Err(format!("Graph requests must target graph.microsoft.com, got {host}"));
        }
        Ok(())
    }

    pub fn list_sites_url(&self) -> Result<String, String> {
        let url = format!("{}/sites?search=*", self.graph_base_url);
        Self::validate_url_for_graph(&url)?;
        Ok(url)
    }

    pub fn site_drives_url(&self, site_id: &str) -> Result<String, String> {
        let encoded: String = site_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        let url = format!("{}/sites/{encoded}/drives", self.graph_base_url);
        Self::validate_url_for_graph(&url)?;
        Ok(url)
    }

    pub fn drive_items_url(&self, drive_id: &str, path: Option<&str>) -> Result<String, String> {
        let encoded: String = drive_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        let url = match path {
            Some(p) => {
                let p = p.replace('/', "%2F");
                format!("{}/drives/{encoded}/root:/{p}:/children", self.graph_base_url)
            }
            None => format!("{}/drives/{encoded}/root/children", self.graph_base_url),
        };
        Self::validate_url_for_graph(&url)?;
        Ok(url)
    }

    pub fn build_auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}

pub fn parse_drive_response(raw: &serde_json::Value) -> Vec<SharePointItem> {
    let mut items = Vec::new();
    if let Some(arr) = raw.get("value").and_then(|v| v.as_array()) {
        for entry in arr {
            let name = entry.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let id = entry.get("id").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let web_url = entry.get("webUrl").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let size = entry.get("size").and_then(|n| n.as_i64()).unwrap_or(0);
            let mime = entry
                .get("file")
                .and_then(|f| f.get("mimeType"))
                .and_then(|m| m.as_str())
                .map(String::from);
            let folder = entry.get("folder").is_some();
            let item_type = if folder {
                SharePointItemType::Folder
            } else if mime.is_some() {
                SharePointItemType::File
            } else {
                SharePointItemType::ListItem
            };
            let last_modified = entry
                .get("lastModifiedDateTime")
                .and_then(|t| t.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            let created = entry
                .get("createdDateTime")
                .and_then(|t| t.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            items.push(SharePointItem {
                id,
                name,
                item_type,
                web_url,
                size_bytes: size,
                mime_type: mime,
                parent_path: entry
                    .get("parentReference")
                    .and_then(|p| p.get("path"))
                    .and_then(|p| p.as_str())
                    .map(String::from),
                etag: entry.get("eTag").and_then(|e| e.as_str()).map(String::from),
                last_modified,
                created_at: created,
                created_by: entry
                    .get("createdBy")
                    .and_then(|u| u.get("user"))
                    .and_then(|u| u.get("displayName"))
                    .and_then(|n| n.as_str())
                    .map(String::from),
                checksum: entry
                    .get("file")
                    .and_then(|f| f.get("hashes"))
                    .and_then(|h| h.get("crc32Hash"))
                    .and_then(|c| c.as_str())
                    .map(String::from),
            });
        }
    }
    items
}

pub fn site_id_from(site: &SharePointSite) -> String {
    site.id.clone()
}

pub fn build_drive_label(drive: &SharePointDrive) -> String {
    let used = drive.quota_used_bytes.unwrap_or(0);
    let total = drive.quota_total_bytes.unwrap_or(0);
    if total == 0 {
        return format!("{} (unlimited)", drive.name);
    }
    let pct = (used as f64 / total as f64) * 100.0;
    format!("{} ({:.1}% used)", drive.name, pct)
}
