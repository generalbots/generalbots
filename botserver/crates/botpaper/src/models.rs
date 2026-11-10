use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub owner_id: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub word_count: usize,
    pub storage_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub save_as_named: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    #[serde(rename = "selected-text")]
    pub selected_text: Option<String>,
    #[serde(rename = "selectedText")]
    pub selected_text_alt: Option<String>,
    pub text: Option<String>,
    pub tone: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "translate-lang")]
    pub translate_lang: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQuery {
    pub id: Option<String>,
}
