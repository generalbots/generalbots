//! Document-level requests: save, load, search and the save response.

use serde::{Deserialize, Serialize};

use super::super::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    /// Document title. Optional so the save endpoint can serve both full-state
    /// saves and rename/flush actions that operate on an existing session.
    pub name: Option<String>,
    /// Full worksheet set. Empty when the client only wants the session
    /// flushed or the document renamed (the session already holds live state).
    #[serde(default)]
    pub worksheets: Vec<Worksheet>,
    /// Explicit action: `rename` renames the document in its session;
    /// anything else is a full save (or a session flush when `worksheets`
    /// is empty and the document already exists).
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFromDriveRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
