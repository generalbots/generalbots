//! Sources-module connector descriptor for M365.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of M365 source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum M365SourceKind {
    /// A SharePoint list (rows are exposed to the bot).
    SharePointList,
    /// A document library (files exposed as blobs).
    DocumentLibrary,
    /// A calendar (events).
    Calendar,
}

/// Configuration for an M365 source registered in the Sources module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M365SourceConfig {
    /// Source ID in the Sources module.
    pub id: Uuid,
    /// Tenant (bot) owning this source.
    pub tenant_id: String,
    /// Display name.
    pub name: String,
    /// Kind.
    pub kind: M365SourceKind,
    /// SharePoint site ID (for SharePoint kinds).
    pub site_id: Option<String>,
    /// List ID (for `SharePointList`).
    pub list_id: Option<String>,
    /// Calendar ID (for `Calendar`).
    pub calendar_id: Option<String>,
    /// OAuth2 token used to read this source.
    pub token_ref: String,
    /// When the connector was created.
    pub created_at: DateTime<Utc>,
    /// Sync interval in minutes (0 = manual).
    pub sync_interval_min: u32,
}

/// Alias kept for backward compatibility.
pub type M365Source = M365SourceConfig;

/// Errors raised by the connector.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ConnectorError {
    /// Source not found in the registry.
    #[error("source {0} not found")]
    NotFound(Uuid),
    /// Token referenced by the source has expired.
    #[error("token expired for source {0}")]
    TokenExpired(Uuid),
    /// Graph returned a 4xx/5xx status.
    #[error("graph error: {0}")]
    Graph(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_config_holds_values() {
        let c = M365SourceConfig {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            name: "Invoices list".to_string(),
            kind: M365SourceKind::SharePointList,
            site_id: Some("site-1".to_string()),
            list_id: Some("list-1".to_string()),
            calendar_id: None,
            token_ref: "vault://m365/token".to_string(),
            created_at: Utc::now(),
            sync_interval_min: 60,
        };
        assert_eq!(c.kind, M365SourceKind::SharePointList);
    }
}
