//! External workbook link requests.

use serde::{Deserialize, Serialize};

use super::super::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExternalLinkRequest {
    pub sheet_id: String,
    pub source_path: String,
    pub link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sheet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshExternalLinkRequest {
    pub sheet_id: String,
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveExternalLinkRequest {
    pub sheet_id: String,
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExternalLinksResponse {
    pub links: Vec<ExternalLink>,
}
