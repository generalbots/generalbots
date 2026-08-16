//! Microsoft Graph client interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::sharepoint::{SharePointList, SharePointListItem};
use super::sharepoint::SharePointSite;

/// Microsoft Graph client abstraction.
#[async_trait]
pub trait GraphClient: Send + Sync {
    /// List all SharePoint sites the user has access to.
    async fn list_sites(&self) -> Result<Vec<SharePointSite>, GraphError>;

    /// List all lists in a SharePoint site.
    async fn list_lists(&self, site_id: &str) -> Result<Vec<SharePointList>, GraphError>;

    /// Fetch all items in a list, paging internally.
    async fn list_items(
        &self,
        site_id: &str,
        list_id: &str,
    ) -> Result<Vec<SharePointListItem>, GraphError>;
}

/// Errors raised by the Graph client.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum GraphError {
    /// HTTP transport failure.
    #[error("transport: {0}")]
    Transport(String),
    /// Graph returned a non-2xx status.
    #[error("graph returned {status}: {body}")]
    Http {
        /// Status code.
        status: u16,
        /// Body excerpt.
        body: String,
    },
    /// Deserialization failure.
    #[error("decode: {0}")]
    Decode(String),
    /// Rate limited (HTTP 429).
    #[error("rate limited; retry after {0}s")]
    RateLimited(u32),
}

/// Lightweight metadata for a SharePoint site (re-exported for `SharePoint`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSite {
    /// Site ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// URL.
    pub web_url: String,
}

/// Type alias kept for API compatibility with `GraphList`.
pub type GraphList = SharePointList;
/// Type alias kept for API compatibility with `GraphDriveItem`.
pub type GraphDriveItem = SharePointListItem;
