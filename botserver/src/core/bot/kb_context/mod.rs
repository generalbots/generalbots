use serde::{Deserialize, Serialize};

pub mod kb_context_ops;
pub mod kb_context_search;

pub use kb_context_ops::*;
pub use kb_context_search::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbSearchResult {
    pub content: String,
    pub document_path: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbContext {
    pub kb_name: String,
    pub search_results: Vec<KbSearchResult>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKbAssociation {
    pub kb_name: String,
    pub qdrant_collection: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWebsiteAssociation {
    pub website_url: String,
    pub collection_name: String,
    pub is_active: bool,
}
