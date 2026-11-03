//! Declared MCP UI resources (#1675).
//!
//! MCP servers advertise interactive apps through `_meta.ui.resourceUri`.
//! Discovery is cached by the server process, but the Integrations surface must
//! also work offline (desktop, air-gapped installs) and must be pinnable. The
//! bot therefore keeps a small sidecar catalog, `mcp_ui.json`, next to
//! `mcp.csv`:
//!
//! ```json
//! { "servers": { "my-server": [ {
//!     "uri": "ui://dashboard", "name": "Dashboard",
//!     "mime_type": "text/html;profile=mcp-app",
//!     "html": "<!doctype html>…", "pin": "<approved hash>",
//!     "permissions": ["clipboard-write"],
//!     "csp": { "connectDomains": ["https://api.example.com"] }
//! } ] } }
//! ```
//!
//! The catalog is authored by discovery or by hand; either way the hash is
//! recomputed here so a changed resource can never be shown under an old
//! approval.

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::types::McpUiResourceView;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCatalog {
    #[serde(default)]
    pub servers: std::collections::HashMap<String, Vec<UiCatalogEntry>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCatalogEntry {
    pub uri: String,
    pub name: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub html: String,
    #[serde(default)]
    pub pin: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub csp: Value,
}

/// Loads the sidecar for one bot. A missing or malformed file is an empty
/// catalog, never an error: MCP management must survive a bad edit.
#[must_use]
pub fn load_catalog(work_path: &str, bot_id: &str) -> UiCatalog {
    let path = std::path::Path::new(work_path)
        .join(format!("{bot_id}.gbai"))
        .join("mcp_ui.json");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return UiCatalog::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

#[must_use]
pub fn entries_for(catalog: &UiCatalog, server: &str) -> Vec<UiCatalogEntry> {
    catalog.servers.get(server).cloned().unwrap_or_default()
}

#[must_use]
pub fn hash_html(html: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(html.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[must_use]
pub fn views(entries: &[UiCatalogEntry]) -> Vec<McpUiResourceView> {
    entries
        .iter()
        .map(|entry| {
            let hash = hash_html(&entry.html);
            let pin_changed = entry
                .pin
                .as_deref()
                .map(|pin| pin != hash)
                .unwrap_or(false);
            let csp = entry.csp.get("csp").unwrap_or(&entry.csp);
            McpUiResourceView {
                uri: entry.uri.clone(),
                name: entry.name.clone(),
                mime_type: if entry.mime_type.is_empty() {
                    "text/html;profile=mcp-app".to_string()
                } else {
                    entry.mime_type.clone()
                },
                permissions: entry.permissions.clone(),
                connect_domains: string_list(csp.get("connectDomains")),
                resource_domains: string_list(csp.get("resourceDomains")),
                hash,
                pin_changed,
            }
        })
        .collect()
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Base64 payload for the preview. The client builds a Blob and renders it in
/// an opaque sandbox; the raw HTML never becomes a suite-origin document.
#[must_use]
pub fn preview_payload(entry: &UiCatalogEntry) -> (String, String) {
    let encoded = base64::engine::general_purpose::STANDARD.encode(entry.html.as_bytes());
    (entry.name.clone(), encoded)
}
