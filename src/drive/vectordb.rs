#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(feature = "vectordb")]
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

#[cfg(feature = "vectordb")]
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        vectors_config::Config, CreateCollection, Distance, PointStruct, VectorParams,
        VectorsConfig,
    },
};

/// File metadata for vector DB indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDocument {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub bucket: String,
    pub content_text: String,
    pub content_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    pub mime_type: Option<String>,
    pub tags: Vec<String>,
}

/// File search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchQuery {
    pub query_text: String,
    pub bucket: Option<String>,
    pub file_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub limit: usize,
}

/// File search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub file: FileDocument,
    pub score: f32,
    pub snippet: String,
    pub highlights: Vec<String>,
}

/// Per-user drive vector DB manager
#[derive(Debug)]
pub struct UserDriveVectorDB {
    _user_id: Uuid,
    _bot_id: Uuid,
    _collection_name: String,
    db_path: PathBuf,
    #[cfg(feature = "vectordb")]
    client: Option<Arc<QdrantClient>>,
}

impl UserDriveVectorDB {
    /// Create new user drive vector DB instance
    pub fn new(user_id: Uuid, bot_id: Uuid, db_path: PathBuf) -> Self {
        let collection_name = format!("drive_{}_{}", bot_id, user_id);

        Self {
            _user_id: user_id,
            _bot_id: bot_id,
            _collection_name: collection_name,
            db_path,
            #[cfg(feature = "vectordb")]
            client: None,
        }
    }

    /// Initialize vector DB collection
    #[cfg(feature = "vectordb")]
    pub async fn initialize(&mut self, qdrant_url: &str) -> Result<()> {
        let client = qdrant_client::Qdrant::from_url(qdrant_url).build()?;

        // Check if collection exists
        let collections = client.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self._collection_name);

        if !exists {
            // Create collection for file embeddings (1536 dimensions for OpenAI embeddings)
            client
                .create_collection(&CreateCollection {
                    collection_name: self._collection_name.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(Config::Params(VectorParams {
                            size: 1536,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        })),
                    }),
                    ..Default::default()
                })
                .await?;

            log::info!(
                "Initialized vector DB collection: {}",
                self._collection_name
            );
        }

        self.client = Some(Arc::new(client));
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn initialize(&mut self, _qdrant_url: &str) -> Result<()> {
        log::warn!("Vector DB feature not enabled, using fallback storage");
        fs::create_dir_all(&self.db_path).await?;
        Ok(())
    }

    /// Index a single file (on-demand)
    #[cfg(feature = "vectordb")]
    pub async fn index_file(&self, file: &FileDocument, embedding: Vec<f32>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let payload = serde_json::to_value(file)?
            .as_object()
            .map(|m| m.clone())
            .unwrap_or_default();
        let point = PointStruct::new(file.id.clone(), embedding, payload);

        client
            .upsert_points(self._collection_name.clone(), None, vec![point], None)
            .await?;

        log::debug!("Indexed file: {} - {}", file.id, file.file_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn index_file(&self, file: &FileDocument, _embedding: Vec<f32>) -> Result<()> {
        // Fallback: Store in JSON file
        let file_path = self.db_path.join(format!("{}.json", file.id));
        let json = serde_json::to_string_pretty(file)?;
        fs::write(file_path, json).await?;
        Ok(())
    }

    /// Index multiple files in batch
    pub async fn index_files_batch(&self, files: &[(FileDocument, Vec<f32>)]) -> Result<()> {
        #[cfg(feature = "vectordb")]
        {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

            let points: Vec<PointStruct> = files
                .iter()
                .filter_map(|(file, embedding)| {
                    serde_json::to_value(file).ok().and_then(|v| {
                        v.as_object().map(|m| {
                            PointStruct::new(file.id.clone(), embedding.clone(), m.clone())
                        })
                    })
                })
                .collect();

            if !points.is_empty() {
                client
                    .upsert_points(self._collection_name.clone(), None, points, None)
                    .await?;
            }
        }

        #[cfg(not(feature = "vectordb"))]
        {
            for (file, embedding) in files {
                self.index_file(file, embedding.clone()).await?;
            }
        }

        Ok(())
    }

    /// Search files using vector similarity
    #[cfg(feature = "vectordb")]
    pub async fn search(
        &self,
        query: &FileSearchQuery,
        query_embedding: Vec<f32>,
    ) -> Result<Vec<FileSearchResult>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        // Build filter if specified
        let mut filter = None;
        if query.bucket.is_some() || query.file_type.is_some() || !query.tags.is_empty() {
            let mut conditions = vec![];

            if let Some(bucket) = &query.bucket {
                conditions.push(qdrant_client::qdrant::Condition::matches(
                    "bucket",
                    bucket.clone(),
                ));
            }

            if let Some(file_type) = &query.file_type {
                conditions.push(qdrant_client::qdrant::Condition::matches(
                    "file_type",
                    file_type.clone(),
                ));
            }

            for tag in &query.tags {
                conditions.push(qdrant_client::qdrant::Condition::matches(
                    "tags",
                    tag.clone(),
                ));
            }

            if !conditions.is_empty() {
                filter = Some(qdrant_client::qdrant::Filter::must(conditions));
            }
        }

        let search_result = client
            .search_points(&qdrant_client::qdrant::SearchPoints {
                collection_name: self._collection_name.clone(),
                vector: query_embedding,
                limit: query.limit as u64,
                filter,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        let mut results = Vec::new();
        for point in search_result.result {
            if let Some(payload) = point.payload {
                let file: FileDocument = serde_json::from_value(serde_json::to_value(&payload)?)?;

                // Create snippet and highlights
                let snippet = self.create_snippet(&file.content_text, &query.query_text, 200);
                let highlights = self.extract_highlights(&file.content_text, &query.query_text, 3);

                results.push(FileSearchResult {
                    file,
                    score: point.score,
                    snippet,
                    highlights,
                });
            }
        }

        Ok(results)
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn search(
        &self,
        query: &FileSearchQuery,
        _query_embedding: Vec<f32>,
    ) -> Result<Vec<FileSearchResult>> {
        // Fallback: Simple text search in JSON files
        let mut results = Vec::new();
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(file) = serde_json::from_str::<FileDocument>(&content) {
                    // Apply filters
                    if let Some(bucket) = &query.bucket {
                        if &file.bucket != bucket {
                            continue;
                        }
                    }

                    if let Some(file_type) = &query.file_type {
                        if &file.file_type != file_type {
                            continue;
                        }
                    }

                    // Simple text matching
                    let query_lower = query.query_text.to_lowercase();
                    if file.file_name.to_lowercase().contains(&query_lower)
                        || file.content_text.to_lowercase().contains(&query_lower)
                        || file
                            .content_summary
                            .as_ref()
                            .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    {
                        let snippet =
                            self.create_snippet(&file.content_text, &query.query_text, 200);
                        let highlights =
                            self.extract_highlights(&file.content_text, &query.query_text, 3);

                        results.push(FileSearchResult {
                            file,
                            score: 1.0,
                            snippet,
                            highlights,
                        });
                    }
                }

                if results.len() >= query.limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Create a snippet around the query match
    fn create_snippet(&self, content: &str, query: &str, max_length: usize) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();

        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(max_length / 2);
            let end = (pos + query.len() + max_length / 2).min(content.len());
            let snippet = &content[start..end];

            if start > 0 && end < content.len() {
                format!("...{}...", snippet)
            } else if start > 0 {
                format!("...{}", snippet)
            } else if end < content.len() {
                format!("{}...", snippet)
            } else {
                snippet.to_string()
            }
        } else if content.len() > max_length {
            format!("{}...", &content[..max_length])
        } else {
            content.to_string()
        }
    }

    /// Extract highlighted segments containing the query
    fn extract_highlights(&self, content: &str, query: &str, max_highlights: usize) -> Vec<String> {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        let mut highlights = Vec::new();
        let mut pos = 0;

        while let Some(found_pos) = content_lower[pos..].find(&query_lower) {
            let actual_pos = pos + found_pos;
            let start = actual_pos.saturating_sub(40);
            let end = (actual_pos + query.len() + 40).min(content.len());

            highlights.push(content[start..end].to_string());

            if highlights.len() >= max_highlights {
                break;
            }

            pos = actual_pos + query.len();
        }

        highlights
    }

    /// Delete file from index
    #[cfg(feature = "vectordb")]
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client
            .delete_points(
                self._collection_name.clone(),
                &vec![file_id.into()].into(),
                None,
            )
            .await?;

        log::debug!("Deleted file from index: {}", file_id);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", file_id));
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    /// Get indexed file count
    #[cfg(feature = "vectordb")]
    pub async fn get_count(&self) -> Result<u64> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let info = client
            .collection_info(self._collection_name.clone())
            .await?;

        Ok(info.result.unwrap().points_count.unwrap_or(0))
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn get_count(&self) -> Result<u64> {
        let mut count = 0;
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Update file metadata without re-indexing content
    pub async fn update_file_metadata(&self, file_id: &str, tags: Vec<String>) -> Result<()> {
        // Read existing file
        #[cfg(not(feature = "vectordb"))]
        {
            let file_path = self.db_path.join(format!("{}.json", file_id));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path).await?;
                let mut file: FileDocument = serde_json::from_str(&content)?;
                file.tags = tags;
                let json = serde_json::to_string_pretty(&file)?;
                fs::write(file_path, json).await?;
            }
        }

        #[cfg(feature = "vectordb")]
        {
            // Update payload in Qdrant
            log::warn!("Metadata update not yet implemented for Qdrant backend");
        }

        Ok(())
    }

    /// Clear all indexed files
    #[cfg(feature = "vectordb")]
    pub async fn clear(&self) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client
            .delete_collection(self._collection_name.clone())
            .await?;

        // Recreate empty collection
        client
            .create_collection(&CreateCollection {
                collection_name: self._collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;

        log::info!("Cleared drive vector collection: {}", self._collection_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn clear(&self) -> Result<()> {
        if self.db_path.exists() {
            fs::remove_dir_all(&self.db_path).await?;
            fs::create_dir_all(&self.db_path).await?;
        }
        Ok(())
    }
}

/// File content extractor for different file types
#[derive(Debug)]
pub struct FileContentExtractor;

impl FileContentExtractor {
    /// Extract text content from file based on type
    pub async fn extract_text(file_path: &PathBuf, mime_type: &str) -> Result<String> {
        match mime_type {
            // Plain text files
            "text/plain" | "text/markdown" | "text/csv" => {
                let content = fs::read_to_string(file_path).await?;
                Ok(content)
            }

            // Code files
            t if t.starts_with("text/") => {
                let content = fs::read_to_string(file_path).await?;
                Ok(content)
            }

            // PDF files
            "application/pdf" => {
                log::info!("PDF extraction requested for {:?}", file_path);
                // Return placeholder for PDF files - requires pdf-extract crate
                Ok(format!("[PDF content from {:?}]", file_path))
            }

            // Microsoft Word documents
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/msword" => {
                log::info!("Word document extraction requested for {:?}", file_path);
                // Return placeholder for Word documents - requires docx-rs crate
                Ok(format!("[Word document content from {:?}]", file_path))
            }

            // Excel/Spreadsheet files
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel" => {
                log::info!("Spreadsheet extraction requested for {:?}", file_path);
                // Return placeholder for spreadsheets - requires calamine crate
                Ok(format!("[Spreadsheet content from {:?}]", file_path))
            }

            // JSON files
            "application/json" => {
                let content = fs::read_to_string(file_path).await?;
                // Pretty print JSON for better indexing
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => Ok(serde_json::to_string_pretty(&json)?),
                    Err(_) => Ok(content),
                }
            }

            // XML/HTML files
            "text/xml" | "application/xml" | "text/html" => {
                let content = fs::read_to_string(file_path).await?;
                // Basic HTML/XML tag removal
                let tag_regex = regex::Regex::new(r"<[^>]+>").unwrap();
                let text = tag_regex.replace_all(&content, " ").to_string();
                Ok(text.trim().to_string())
            }

            // RTF files
            "text/rtf" | "application/rtf" => {
                let content = fs::read_to_string(file_path).await?;
                // Basic RTF extraction - remove control words and groups
                let control_regex = regex::Regex::new(r"\\[a-z]+[\-0-9]*[ ]?").unwrap();
                let group_regex = regex::Regex::new(r"[\{\}]").unwrap();

                let mut text = control_regex.replace_all(&content, " ").to_string();
                text = group_regex.replace_all(&text, "").to_string();

                Ok(text.trim().to_string())
            }

            _ => {
                log::warn!("Unsupported file type for indexing: {}", mime_type);
                Ok(String::new())
            }
        }
    }

    /// Determine if file should be indexed based on type
    pub fn should_index(mime_type: &str, file_size: u64) -> bool {
        // Skip very large files (> 10MB)
        if file_size > 10 * 1024 * 1024 {
            return false;
        }

        // Index text-based files
        matches!(
            mime_type,
            "text/plain"
                | "text/markdown"
                | "text/csv"
                | "text/html"
                | "application/json"
                | "text/x-python"
                | "text/x-rust"
                | "text/javascript"
                | "text/x-java"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_document_creation() {
        let file = FileDocument {
            id: "test-123".to_string(),
            file_path: "/test/file.txt".to_string(),
            file_name: "file.txt".to_string(),
            file_type: "text".to_string(),
            file_size: 1024,
            bucket: "test-bucket".to_string(),
            content_text: "Test file content".to_string(),
            content_summary: Some("Summary".to_string()),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            indexed_at: Utc::now(),
            mime_type: Some("text/plain".to_string()),
            tags: vec!["test".to_string()],
        };

        assert_eq!(file.id, "test-123");
        assert_eq!(file.file_name, "file.txt");
    }

    #[test]
    fn test_should_index() {
        assert!(FileContentExtractor::should_index("text/plain", 1024));
        assert!(FileContentExtractor::should_index("text/markdown", 5000));
        assert!(!FileContentExtractor::should_index(
            "text/plain",
            20 * 1024 * 1024
        ));
        assert!(!FileContentExtractor::should_index("video/mp4", 1024));
    }

    #[tokio::test]
    async fn test_user_drive_vectordb_creation() {
        let temp_dir = std::env::temp_dir().join("test_drive_vectordb");
        let db = UserDriveVectorDB::new(Uuid::new_v4(), Uuid::new_v4(), temp_dir);

        assert!(db._collection_name.starts_with("drive_"));
    }
}
