use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub content_hash: String,
    pub chunk_count: i32,
    pub status: SourceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Pdf,
    Docx,
    Txt,
    Markdown,
    Html,
    Csv,
    Xlsx,
    Url,
    Custom,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pdf => write!(f, "pdf"),
            Self::Docx => write!(f, "docx"),
            Self::Txt => write!(f, "txt"),
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
            Self::Csv => write!(f, "csv"),
            Self::Xlsx => write!(f, "xlsx"),
            Self::Url => write!(f, "url"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl From<&str> for SourceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "docx" | "doc" => Self::Docx,
            "txt" | "text" => Self::Txt,
            "md" | "markdown" => Self::Markdown,
            "html" | "htm" => Self::Html,
            "csv" => Self::Csv,
            "xlsx" | "xls" => Self::Xlsx,
            "url" | "web" => Self::Url,
            _ => Self::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Pending,
    Processing,
    Indexed,
    Failed,
    Reindexing,
}

impl std::fmt::Display for SourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Processing => write!(f, "processing"),
            Self::Indexed => write!(f, "indexed"),
            Self::Failed => write!(f, "failed"),
            Self::Reindexing => write!(f, "reindexing"),
        }
    }
}

impl From<&str> for SourceStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "processing" => Self::Processing,
            "indexed" => Self::Indexed,
            "failed" => Self::Failed,
            "reindexing" => Self::Reindexing,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub source_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub token_count: i32,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub source_id: Option<String>,
    pub message: String,
    pub chunks_created: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub collection: Option<String>,
    pub top_k: Option<usize>,
    pub min_score: Option<f32>,
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub content: String,
    pub source_name: String,
    pub source_id: String,
    pub chunk_index: i32,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub query: String,
    pub total_results: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSourcesQuery {
    pub status: Option<String>,
    pub source_type: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexRequest {
    pub source_ids: Option<Vec<String>>,
    pub force: Option<bool>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KnowledgeSourceRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub source_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub file_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub url: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content_hash: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub chunk_count: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub created_at: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub updated_at: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub indexed_at: Option<String>,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SearchResultRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub chunk_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub source_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub source_name: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub chunk_index: i32,
    #[diesel(sql_type = diesel::sql_types::Float)]
    pub score: f32,
}
