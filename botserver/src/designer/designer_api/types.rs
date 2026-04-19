use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
    pub connections: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DialogRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub line: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicRequest {
    pub nodes: Vec<MagicNode>,
    pub connections: i32,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicResponse {
    pub improved_code: Option<String>,
    pub explanation: Option<String>,
    pub suggestions: Option<Vec<MagicSuggestion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSuggestion {
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerModifyRequest {
    pub app_name: String,
    pub current_page: Option<String>,
    pub message: String,
    pub context: Option<DesignerContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerContext {
    pub page_html: Option<String>,
    pub tables: Option<Vec<String>>,
    pub recent_changes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerModifyResponse {
    pub success: bool,
    pub message: String,
    pub changes: Vec<DesignerChange>,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerChange {
    pub change_type: String,
    pub file_path: String,
    pub description: String,
    pub preview: Option<String>,
}
