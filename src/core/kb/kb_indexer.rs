use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::document_processor::{DocumentProcessor, TextChunk};
use super::embedding_generator::{Embedding, EmbeddingConfig, KbEmbeddingGenerator};

/// Qdrant client configuration
#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6333".to_string(),
            api_key: None,
            timeout_secs: 30,
        }
    }
}

/// Point structure for Qdrant
#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, serde_json::Value>,
}

/// Collection configuration for Qdrant
#[derive(Debug, Serialize)]
pub struct CollectionConfig {
    pub vectors: VectorConfig,
    pub replication_factor: u32,
    pub shard_number: u32,
}

#[derive(Debug, Serialize)]
pub struct VectorConfig {
    pub size: usize,
    pub distance: String,
}

/// Search request structure
#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    pub limit: usize,
    pub with_payload: bool,
    pub score_threshold: Option<f32>,
    pub filter: Option<serde_json::Value>,
}

/// Knowledge Base Indexer for Qdrant
pub struct KbIndexer {
    document_processor: DocumentProcessor,
    embedding_generator: KbEmbeddingGenerator,
    qdrant_config: QdrantConfig,
    http_client: reqwest::Client,
}

impl std::fmt::Debug for KbIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KbIndexer")
            .field("document_processor", &self.document_processor)
            .field("embedding_generator", &self.embedding_generator)
            .field("qdrant_config", &self.qdrant_config)
            .field("http_client", &"reqwest::Client")
            .finish()
    }
}

impl KbIndexer {
    pub fn new(embedding_config: EmbeddingConfig, qdrant_config: QdrantConfig) -> Self {
        let document_processor = DocumentProcessor::default();
        let embedding_generator = KbEmbeddingGenerator::new(embedding_config);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(qdrant_config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            document_processor,
            embedding_generator,
            qdrant_config,
            http_client,
        }
    }

    /// Index a knowledge base folder
    pub async fn index_kb_folder(
        &self,
        bot_name: &str,
        kb_name: &str,
        kb_path: &Path,
    ) -> Result<IndexingResult> {
        info!("Indexing KB folder: {} for bot {}", kb_name, bot_name);

        // Create collection name
        let collection_name = format!("{}_{}", bot_name, kb_name);

        // Ensure collection exists
        self.ensure_collection_exists(&collection_name).await?;

        // Process all documents in the folder
        let documents = self.document_processor.process_kb_folder(kb_path).await?;

        let mut total_chunks = 0;
        let mut indexed_documents = 0;

        for (doc_path, chunks) in documents {
            if chunks.is_empty() {
                continue;
            }

            info!(
                "Processing document: {} ({} chunks)",
                doc_path,
                chunks.len()
            );

            // Generate embeddings for chunks
            let embeddings = self
                .embedding_generator
                .generate_embeddings(&chunks)
                .await?;

            // Create points for Qdrant
            let points = self.create_qdrant_points(&doc_path, embeddings)?;

            // Upsert points to collection
            self.upsert_points(&collection_name, points).await?;

            total_chunks += chunks.len();
            indexed_documents += 1;
        }

        // Update collection info in database
        self.update_collection_metadata(&collection_name, bot_name, kb_name, total_chunks)
            .await?;

        Ok(IndexingResult {
            collection_name,
            documents_processed: indexed_documents,
            chunks_indexed: total_chunks,
        })
    }

    /// Ensure Qdrant collection exists
    async fn ensure_collection_exists(&self, collection_name: &str) -> Result<()> {
        // Check if collection exists
        let check_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);

        let response = self.http_client.get(&check_url).send().await?;

        if response.status().is_success() {
            info!("Collection {} already exists", collection_name);
            return Ok(());
        }

        // Create collection
        info!("Creating collection: {}", collection_name);

        let config = CollectionConfig {
            vectors: VectorConfig {
                size: 384, // Default for bge-small, should be configurable
                distance: "Cosine".to_string(),
            },
            replication_factor: 1,
            shard_number: 1,
        };

        let create_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);

        let response = self
            .http_client
            .put(&create_url)
            .json(&config)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to create collection: {}",
                error_text
            ));
        }

        // Create indexes for better performance
        self.create_collection_indexes(collection_name).await?;

        Ok(())
    }

    /// Create indexes for collection
    async fn create_collection_indexes(&self, collection_name: &str) -> Result<()> {
        // Create HNSW index for vector search
        let index_config = serde_json::json!({
            "hnsw_config": {
                "m": 16,
                "ef_construct": 200,
                "full_scan_threshold": 10000
            }
        });

        let index_url = format!(
            "{}/collections/{}/index",
            self.qdrant_config.url, collection_name
        );

        let response = self
            .http_client
            .put(&index_url)
            .json(&index_config)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to create index, using defaults");
        }

        Ok(())
    }

    /// Create Qdrant points from chunks and embeddings
    fn create_qdrant_points(
        &self,
        doc_path: &str,
        embeddings: Vec<(TextChunk, Embedding)>,
    ) -> Result<Vec<QdrantPoint>> {
        let mut points = Vec::new();

        for (chunk, embedding) in embeddings {
            let point_id = Uuid::new_v4().to_string();

            let mut payload = HashMap::new();
            payload.insert(
                "content".to_string(),
                serde_json::Value::String(chunk.content),
            );
            payload.insert(
                "document_path".to_string(),
                serde_json::Value::String(doc_path.to_string()),
            );
            payload.insert(
                "chunk_index".to_string(),
                serde_json::Value::Number(chunk.metadata.chunk_index.into()),
            );
            payload.insert(
                "total_chunks".to_string(),
                serde_json::Value::Number(chunk.metadata.total_chunks.into()),
            );
            payload.insert(
                "start_char".to_string(),
                serde_json::Value::Number(chunk.metadata.start_char.into()),
            );
            payload.insert(
                "end_char".to_string(),
                serde_json::Value::Number(chunk.metadata.end_char.into()),
            );

            if let Some(title) = chunk.metadata.document_title {
                payload.insert(
                    "document_title".to_string(),
                    serde_json::Value::String(title),
                );
            }

            points.push(QdrantPoint {
                id: point_id,
                vector: embedding.vector,
                payload,
            });
        }

        Ok(points)
    }

    /// Upsert points to Qdrant collection
    async fn upsert_points(&self, collection_name: &str, points: Vec<QdrantPoint>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        let batch_size = 100; // Qdrant recommended batch size

        for batch in points.chunks(batch_size) {
            let upsert_request = serde_json::json!({
                "points": batch
            });

            let upsert_url = format!(
                "{}/collections/{}/points?wait=true",
                self.qdrant_config.url, collection_name
            );

            let response = self
                .http_client
                .put(&upsert_url)
                .json(&upsert_request)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Failed to upsert points: {}", error_text));
            }
        }

        debug!(
            "Upserted {} points to collection {}",
            points.len(),
            collection_name
        );

        Ok(())
    }

    /// Update collection metadata in database
    async fn update_collection_metadata(
        &self,
        collection_name: &str,
        bot_name: &str,
        kb_name: &str,
        document_count: usize,
    ) -> Result<()> {
        // This would update the kb_collections table
        // For now, just log the information
        info!(
            "Updated collection {} metadata: bot={}, kb={}, docs={}",
            collection_name, bot_name, kb_name, document_count
        );

        Ok(())
    }

    /// Search for similar chunks in a collection
    pub async fn search(
        &self,
        collection_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Generate embedding for query
        let embedding = self
            .embedding_generator
            .generate_single_embedding(query)
            .await?;

        // Create search request
        let search_request = SearchRequest {
            vector: embedding.vector,
            limit,
            with_payload: true,
            score_threshold: Some(0.5), // Minimum similarity threshold
            filter: None,
        };

        let search_url = format!(
            "{}/collections/{}/points/search",
            self.qdrant_config.url, collection_name
        );

        let response = self
            .http_client
            .post(&search_url)
            .json(&search_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Search failed: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let mut results = Vec::new();

        if let Some(result_array) = response_json["result"].as_array() {
            for item in result_array {
                if let (Some(score), Some(payload)) =
                    (item["score"].as_f64(), item["payload"].as_object())
                {
                    let content = payload
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let document_path = payload
                        .get("document_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    results.push(SearchResult {
                        content,
                        document_path,
                        score: score as f32,
                        metadata: payload.clone(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_name: &str) -> Result<()> {
        let delete_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);

        let response = self.http_client.delete(&delete_url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            warn!(
                "Failed to delete collection {}: {}",
                collection_name, error_text
            );
        }

        Ok(())
    }
}

/// Result of indexing operation
#[derive(Debug)]
pub struct IndexingResult {
    pub collection_name: String,
    pub documents_processed: usize,
    pub chunks_indexed: usize,
}

/// Search result from vector database
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub content: String,
    pub document_path: String,
    pub score: f32,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

/// Monitor for .gbkb folder changes and trigger indexing
#[derive(Debug)]
pub struct KbFolderMonitor {
    indexer: KbIndexer,
    work_root: PathBuf,
}

impl KbFolderMonitor {
    pub fn new(work_root: PathBuf, embedding_config: EmbeddingConfig) -> Self {
        let qdrant_config = QdrantConfig::default();
        let indexer = KbIndexer::new(embedding_config, qdrant_config);

        Self { indexer, work_root }
    }

    /// Process a .gbkb folder that was detected by drive monitor
    pub async fn process_gbkb_folder(&self, bot_name: &str, kb_folder: &Path) -> Result<()> {
        // Extract KB name from folder path
        let kb_name = kb_folder
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid KB folder name"))?;

        info!("Processing .gbkb folder: {} for bot {}", kb_name, bot_name);

        // Build local work path
        let local_path = self
            .work_root
            .join(bot_name)
            .join(format!("{}.gbkb", bot_name))
            .join(kb_name);

        // Index the folder
        let result = self
            .indexer
            .index_kb_folder(bot_name, kb_name, &local_path)
            .await?;

        info!(
            "Indexed {} documents ({} chunks) into collection {}",
            result.documents_processed, result.chunks_indexed, result.collection_name
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_name_generation() {
        let bot_name = "mybot";
        let kb_name = "docs";
        let collection_name = format!("{}_{}", bot_name, kb_name);
        assert_eq!(collection_name, "mybot_docs");
    }

    #[test]
    fn test_qdrant_point_creation() {
        let chunk = TextChunk {
            content: "Test content".to_string(),
            metadata: super::super::document_processor::ChunkMetadata {
                document_path: "test.txt".to_string(),
                document_title: Some("Test".to_string()),
                chunk_index: 0,
                total_chunks: 1,
                start_char: 0,
                end_char: 12,
                page_number: None,
            },
        };

        let embedding = Embedding {
            vector: vec![0.1, 0.2, 0.3],
            dimensions: 3,
            model: "test".to_string(),
            tokens_used: None,
        };

        let indexer = KbIndexer::new(EmbeddingConfig::default(), QdrantConfig::default());

        let points = indexer
            .create_qdrant_points("test.txt", vec![(chunk, embedding)])
            .unwrap();

        assert_eq!(points.len(), 1);
        assert_eq!(points[0].vector.len(), 3);
        assert!(points[0].payload.contains_key("content"));
    }
}
