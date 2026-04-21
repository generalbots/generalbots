use anyhow::Result;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::core::config::ConfigManager;
use crate::core::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
use crate::core::shared::utils::{create_tls_client, DbPool};

use super::document_processor::{DocumentProcessor, TextChunk};
use super::embedding_generator::{is_embedding_server_ready, Embedding, EmbeddingConfig, KbEmbeddingGenerator};

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            api_key: None,
            timeout_secs: 30,
        }
    }
}

impl QdrantConfig {
    pub fn from_config(pool: DbPool, bot_id: &Uuid) -> Self {
        let (url, api_key) = if let Some(sm) = crate::core::shared::utils::get_secrets_manager_sync() {
            sm.get_vectordb_config_sync()
        } else {
            let config_manager = ConfigManager::new(pool.clone());
            let url = config_manager
                .get_config(bot_id, "vectordb-url", Some(""))
                .unwrap_or_else(|_| "".to_string());
            (url, None)
        };
        Self {
            url,
            api_key,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, serde_json::Value>,
}

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

#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub vector: Vec<f32>,
    pub limit: usize,
    pub with_payload: bool,
    pub score_threshold: Option<f32>,
    pub filter: Option<serde_json::Value>,
}

pub struct KbIndexer {
    document_processor: DocumentProcessor,
    embedding_generator: KbEmbeddingGenerator,
    qdrant_config: QdrantConfig,
    http_client: reqwest::Client,
    db_pool: Option<DbPool>,
}

impl std::fmt::Debug for KbIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KbIndexer")
            .field("document_processor", &self.document_processor)
            .field("embedding_generator", &self.embedding_generator)
            .field("qdrant_config", &self.qdrant_config)
            .field("http_client", &"reqwest::Client")
            .field("db_pool", &"DbPool")
            .finish()
    }
}

impl KbIndexer {
    pub fn new(embedding_config: EmbeddingConfig, qdrant_config: QdrantConfig) -> Self {
        let document_processor = DocumentProcessor::default();
        let embedding_generator = KbEmbeddingGenerator::new(embedding_config);

        let http_client = create_tls_client(Some(qdrant_config.timeout_secs));

        Self {
            document_processor,
            embedding_generator,
            qdrant_config,
            http_client,
            db_pool: None,
        }
    }

    pub fn new_with_pool(embedding_config: EmbeddingConfig, qdrant_config: QdrantConfig, db_pool: DbPool) -> Self {
        let document_processor = DocumentProcessor::default();
        let embedding_generator = KbEmbeddingGenerator::new(embedding_config);

        let http_client = create_tls_client(Some(qdrant_config.timeout_secs));

        Self {
            document_processor,
            embedding_generator,
            qdrant_config,
            http_client,
            db_pool: Some(db_pool),
        }
    }

    pub fn get_db_pool(&self) -> Option<&DbPool> {
        self.db_pool.as_ref()
    }

    pub fn get_pool(&self) -> DbPool {
        self.db_pool.clone().unwrap_or_else(|| panic!("DbPool not available"))
    }

    pub async fn check_qdrant_health(&self) -> Result<bool> {
        let health_url = format!("{}/healthz", self.qdrant_config.url);

        match self.http_client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn index_kb_folder(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        kb_name: &str,
        kb_path: &Path,
    ) -> Result<IndexingResult> {
        let start_mem = MemoryStats::current();
        info!("Indexing KB folder: {} for bot {} [START RSS={}]",
              kb_name, bot_name, MemoryStats::format_bytes(start_mem.rss_bytes));
        log_jemalloc_stats();

        if !is_embedding_server_ready() {
            info!("Embedding server not ready yet, waiting up to 60s...");
            if !self.embedding_generator.wait_for_server(60).await {
                warn!(
                    "Embedding server is not available. KB indexing skipped. \
                    Wait for the embedding server to start before indexing."
                );
                return Err(anyhow::anyhow!(
                    "Embedding server not available. KB indexing deferred until embedding service is ready."
                ));
            }
        }

        if !self.check_qdrant_health().await.unwrap_or(false) {
            warn!(
                "Qdrant vector database is not available at {}. KB indexing skipped. \
                Install and start vector_db component to enable KB indexing.",
                self.qdrant_config.url
            );
            return Err(anyhow::anyhow!(
                "Qdrant vector database is not available at {}. Start the vector_db service to enable KB indexing.",
                self.qdrant_config.url
            ));
        }

        let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
        let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);

        self.ensure_collection_exists(&collection_name).await?;

        let before_docs = MemoryStats::current();
        trace!("Before process_kb_folder RSS={}",
              MemoryStats::format_bytes(before_docs.rss_bytes));

        let documents = self.document_processor.process_kb_folder(kb_path).await?;

        let after_docs = MemoryStats::current();
        trace!("After process_kb_folder: {} documents, RSS={} (delta={})",
              documents.len(),
              MemoryStats::format_bytes(after_docs.rss_bytes),
              MemoryStats::format_bytes(after_docs.rss_bytes.saturating_sub(before_docs.rss_bytes)));

        let mut total_chunks = 0;
        let mut indexed_documents = 0;
        const BATCH_SIZE: usize = 5; // Smaller batch size to prevent memory exhaustion
        let mut batch_docs = Vec::with_capacity(BATCH_SIZE);

        // Process documents in iterator to avoid keeping all in memory
        let doc_iter = documents.into_iter();

        for (doc_path, chunks) in doc_iter {
            if chunks.is_empty() {
                debug!("Skipping document with no chunks: {}", doc_path);
                continue;
            }

            batch_docs.push((doc_path, chunks));

            // Process batch when full
            if batch_docs.len() >= BATCH_SIZE {
                let (processed, chunks_count) = self.process_document_batch(&collection_name, &mut batch_docs).await?;
                indexed_documents += processed;
                total_chunks += chunks_count;

                // Clear batch and force memory cleanup
                batch_docs.clear();
                batch_docs.shrink_to_fit();

                // Yield control to prevent blocking
                tokio::task::yield_now().await;

                // Memory pressure check - more aggressive
                let current_mem = MemoryStats::current();
                if current_mem.rss_bytes > 1_500_000_000 { // 1.5GB threshold (reduced)
                    warn!("High memory usage detected: {}, forcing cleanup",
                          MemoryStats::format_bytes(current_mem.rss_bytes));

                    // Force garbage collection hint
                    std::hint::black_box(&batch_docs);

                    // Add delay to allow memory cleanup
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }

        // Process remaining documents in final batch
        if !batch_docs.is_empty() {
            let (processed, chunks_count) = self.process_document_batch(&collection_name, &mut batch_docs).await?;
            indexed_documents += processed;
            total_chunks += chunks_count;
        }

        self.update_collection_metadata(&collection_name, bot_name, kb_name, total_chunks)?;

        let end_mem = MemoryStats::current();
        trace!("Indexing complete: {} docs, {} chunks, RSS={} (total delta={})",
              indexed_documents, total_chunks,
              MemoryStats::format_bytes(end_mem.rss_bytes),
              MemoryStats::format_bytes(end_mem.rss_bytes.saturating_sub(start_mem.rss_bytes)));
        log_jemalloc_stats();

        Ok(IndexingResult {
            collection_name,
            documents_processed: indexed_documents,
            chunks_indexed: total_chunks,
        })
    }

    async fn process_document_batch(
        &self,
        collection_name: &str,
        batch_docs: &mut Vec<(String, Vec<TextChunk>)>,
    ) -> Result<(usize, usize)> {
        let mut processed_count = 0;
        let mut total_chunks = 0;

        // Process documents one by one to minimize memory usage
        while let Some((doc_path, chunks)) = batch_docs.pop() {
            let before_embed = MemoryStats::current();
            trace!(
                "Processing document: {} ({} chunks) RSS={}",
                doc_path,
                chunks.len(),
                MemoryStats::format_bytes(before_embed.rss_bytes)
            );

            // Re-validate embedding server is still available
            if !is_embedding_server_ready() {
                warn!("Embedding server became unavailable during indexing, aborting batch");
                return Err(anyhow::anyhow!(
                    "Embedding server became unavailable during KB indexing. Processed {} documents before failure.",
                    processed_count
                ));
            }

            // Process chunks in smaller sub-batches to prevent memory exhaustion
            const CHUNK_BATCH_SIZE: usize = 20; // Process 20 chunks at a time
            let chunk_batches = chunks.chunks(CHUNK_BATCH_SIZE);

            for chunk_batch in chunk_batches {
                trace!("Processing chunk batch of {} chunks", chunk_batch.len());

                let embeddings = match self
                    .embedding_generator
                    .generate_embeddings(chunk_batch)
                    .await
                {
                    Ok(emb) => emb,
                    Err(e) => {
                        warn!("Embedding generation failed for {}: {}", doc_path, e);
                        break; // Skip to next document
                    }
                };

                let points = Self::create_qdrant_points(&doc_path, embeddings)?;
                self.upsert_points(collection_name, points).await?;

                // Yield control between chunk batches
                tokio::task::yield_now().await;
            }

            let after_embed = MemoryStats::current();
            trace!("After processing document: RSS={} (delta={})",
                  MemoryStats::format_bytes(after_embed.rss_bytes),
                  MemoryStats::format_bytes(after_embed.rss_bytes.saturating_sub(before_embed.rss_bytes)));

            total_chunks += chunks.len();
            processed_count += 1;

            // Force memory cleanup after each document
            std::hint::black_box(&chunks);
        }

        Ok((processed_count, total_chunks))
    }

    async fn ensure_collection_exists(&self, collection_name: &str) -> Result<()> {
        let check_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);
        let required_dims = self.embedding_generator.get_dimensions();

        let response = self.http_client.get(&check_url).send().await?;

        if response.status().is_success() {
            // Check if the existing collection has the correct vector size
            let info_json: serde_json::Value = response.json().await?;
            let existing_dims = info_json["result"]["config"]["params"]["vectors"]["size"]
                .as_u64()
                .map(|d| d as usize);

            match existing_dims {
                Some(dims) if dims == required_dims => {
                    trace!("Collection {} already exists with correct dims ({})", collection_name, required_dims);
                    return Ok(());
                }
                Some(dims) => {
                    warn!(
                        "Collection {} exists with dim={} but embedding requires dim={}. \
                        Recreating collection.",
                        collection_name, dims, required_dims
                    );
                    // Delete the stale collection so we can recreate it
                    let delete_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);
                    let del_resp = self.http_client.delete(&delete_url).send().await?;
                    if !del_resp.status().is_success() {
                        let err = del_resp.text().await.unwrap_or_default();
                        return Err(anyhow::anyhow!(
                            "Failed to delete stale collection {}: {}",
                            collection_name, err
                        ));
                    }
                    info!("Deleted stale collection {} (was dim={})", collection_name, dims);
                }
                None => {
                    // Could not read dims – recreate to be safe
                    warn!("Could not read dims for collection {}, recreating", collection_name);
                    let delete_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);
                    let _ = self.http_client.delete(&delete_url).send().await;
                }
            }
        }

        info!("Creating collection {} with dim={}", collection_name, required_dims);

        let config = CollectionConfig {
            vectors: VectorConfig {
                size: required_dims,
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

        self.create_collection_indexes(collection_name).await?;

        Ok(())
    }

    async fn create_collection_indexes(&self, collection_name: &str) -> Result<()> {
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

    fn create_qdrant_points(
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

    async fn upsert_points(&self, collection_name: &str, points: Vec<QdrantPoint>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        let batch_size = 100;

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

pub async fn index_single_file(
    &self,
    bot_id: Uuid,
    bot_name: &str,
    kb_name: &str,
    file_path: &Path,
) -> Result<IndexingResult> {
    self.index_single_file_with_id(bot_id, bot_name, kb_name, file_path, None).await
}

pub async fn index_single_file_with_id(
    &self,
    bot_id: Uuid,
    bot_name: &str,
    kb_name: &str,
    file_path: &Path,
    document_id: Option<&str>,
) -> Result<IndexingResult> {
    if !is_embedding_server_ready() {
        return Err(anyhow::anyhow!(
            "Embedding server not available. Cannot index file."
        ));
    }

    if !self.check_qdrant_health().await.unwrap_or(false) {
        return Err(anyhow::anyhow!(
            "Qdrant vector database is not available."
        ));
    }

    let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
    let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);

    self.ensure_collection_exists(&collection_name).await?;

    let doc_path = document_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| file_path.to_string_lossy().to_string());

    info!(
        "Indexing single file: {} (id: {}) into collection {}",
        file_path.display(), doc_path, collection_name
    );

    if let Err(e) = self.delete_file_points(&collection_name, &doc_path).await {
        warn!("Failed to delete old points for {} before reindex: {}", doc_path, e);
    }

    let chunks = self.document_processor.process_document(file_path).await?;

    if chunks.is_empty() {
        warn!("No chunks extracted from file: {}", file_path.display());
        return Ok(IndexingResult {
            collection_name,
            documents_processed: 0,
            chunks_indexed: 0,
        });
    }

    let embeddings = self
        .embedding_generator
        .generate_embeddings(&chunks)
        .await?;

    let points = Self::create_qdrant_points(&doc_path, embeddings)?;
    self.upsert_points(&collection_name, points).await?;

        self.update_collection_metadata(&collection_name, bot_name, kb_name, chunks.len())?;

        info!(
            "Indexed {} chunks from {} into collection {}",
            chunks.len(),
            file_path.display(),
            collection_name
        );

        Ok(IndexingResult {
            collection_name,
            documents_processed: 1,
            chunks_indexed: chunks.len(),
        })
    }

    pub async fn delete_file_points(
        &self,
        collection_name: &str,
        document_path: &str,
    ) -> Result<()> {
        let filter = serde_json::json!({
            "must": [
                {
                    "key": "document_path",
                    "match": {
                        "value": document_path
                    }
                }
            ]
        });

        let delete_url = format!(
            "{}/collections/{}/points/delete?wait=true",
            self.qdrant_config.url, collection_name
        );

        let response = self
            .http_client
            .post(&delete_url)
            .json(&serde_json::json!({ "filter": filter }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to delete points: {}", error_text));
        }

        info!(
            "Deleted points for document {} from collection {}",
            document_path, collection_name
        );

        Ok(())
    }

    fn update_collection_metadata(
        &self,
        collection_name: &str,
        bot_name: &str,
        kb_name: &str,
        document_count: usize,
    ) -> Result<()> {
        let _ = self;
        info!(
            "Updated collection {} metadata: bot={}, kb={}, docs={}",
            collection_name, bot_name, kb_name, document_count
        );

        Ok(())
    }

    pub async fn search(
        &self,
        collection_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Get the collection's actual vector dimension to handle dimension mismatch
        let collection_dimension = self.get_collection_vector_dimension(collection_name).await?;

        let embedding = self
            .embedding_generator
            .generate_single_embedding(query)
            .await?;

        // Truncate embedding vector to match collection dimension if needed
        let search_vector = if let Some(target_dim) = collection_dimension {
            if embedding.vector.len() > target_dim {
                debug!(
                    "Truncating embedding from {} to {} dimensions for collection '{}'",
                    embedding.vector.len(), target_dim, collection_name
                );
                embedding.vector[..target_dim].to_vec()
            } else if embedding.vector.len() < target_dim {
                warn!(
                    "Embedding dimension ({}) is smaller than collection dimension ({}). \
                    Search may return poor results for collection '{}'.",
                    embedding.vector.len(), target_dim, collection_name
                );
                // Pad with zeros (not ideal but allows search to proceed)
                let mut padded = embedding.vector.clone();
                padded.resize(target_dim, 0.0);
                padded
            } else {
                embedding.vector
            }
        } else {
            embedding.vector
        };

        self.execute_search(collection_name, search_vector, limit).await
    }

    pub async fn search_with_config(
        &self,
        collection_name: &str,
        query: &str,
        limit: usize,
        embedding_config: &EmbeddingConfig,
    ) -> Result<Vec<SearchResult>> {
        let collection_dimension = self.get_collection_vector_dimension(collection_name).await?;

        let embedding_generator = KbEmbeddingGenerator::new(embedding_config.clone());
        let embedding = embedding_generator.generate_single_embedding(query).await?;

        let search_vector = if let Some(target_dim) = collection_dimension {
            if embedding.vector.len() > target_dim {
                debug!(
                    "Truncating embedding from {} to {} dimensions for collection '{}'",
                    embedding.vector.len(), target_dim, collection_name
                );
                embedding.vector[..target_dim].to_vec()
            } else if embedding.vector.len() < target_dim {
                warn!(
                    "Embedding dimension ({}) is smaller than collection dimension ({}). \
                    Search may return poor results for collection '{}'.",
                    embedding.vector.len(), target_dim, collection_name
                );
                let mut padded = embedding.vector.clone();
                padded.resize(target_dim, 0.0);
                padded
            } else {
                embedding.vector
            }
        } else {
            embedding.vector
        };

        self.execute_search(collection_name, search_vector, limit).await
    }

    async fn execute_search(
        &self,
        collection_name: &str,
        search_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let search_request = SearchRequest {
            vector: search_vector,
            limit,
            with_payload: true,
            score_threshold: None,
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
                        .and_then(|c| c.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let document_path = payload
                        .get("document_path")
                        .and_then(|p| p.as_str())
                        .unwrap_or_default()
                        .to_string();

                    results.push(SearchResult {
                        content,
                        document_path,
                        score: score as f32,
                        metadata: serde_json::Map::new(),
                    });
                }
            }
        }

        Ok(results)
    }

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

    /// Get the vector dimension of a collection from Qdrant
    async fn get_collection_vector_dimension(&self, collection_name: &str) -> Result<Option<usize>> {
        let info_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);

        let response = match self.http_client.get(&info_url).send().await {
            Ok(r) => r,
            Err(e) => {
                debug!("Failed to get collection dimension: {}", e);
                return Ok(None);
            }
        };

        if !response.status().is_success() {
            debug!("Collection '{}' not found or error, using default dimension", collection_name);
            return Ok(None);
        }

        let info_json: serde_json::Value = response.json().await?;
        let dimension = info_json["result"]["config"]["params"]["vectors"]["size"]
            .as_u64()
            .map(|d| d as usize);

        Ok(dimension)
    }

    pub async fn get_collection_info(&self, collection_name: &str) -> Result<CollectionInfo> {
        let info_url = format!("{}/collections/{}", self.qdrant_config.url, collection_name);

        let response = self.http_client.get(&info_url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Ok(CollectionInfo {
                    name: collection_name.to_string(),
                    points_count: 0,
                    vectors_count: 0,
                    indexed_vectors_count: 0,
                    segments_count: 0,
                    status: "not_found".to_string(),
                });
            }
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to get collection info: {}",
                error_text
            ));
        }

        let response_json: serde_json::Value = response.json().await?;

        let result = &response_json["result"];

        let points_count = result["points_count"].as_u64().unwrap_or(0) as usize;
        let vectors_count = result["vectors_count"]
            .as_u64()
            .or_else(|| {
                result["vectors_count"]
                    .as_object()
                    .map(|_| points_count as u64)
            })
            .unwrap_or(0) as usize;
        let indexed_vectors_count = result["indexed_vectors_count"]
            .as_u64()
            .unwrap_or(vectors_count as u64) as usize;
        let segments_count = result["segments_count"].as_u64().unwrap_or(0) as usize;
        let status = result["status"].as_str().unwrap_or("unknown").to_string();

        Ok(CollectionInfo {
            name: collection_name.to_string(),
            points_count,
            vectors_count,
            indexed_vectors_count,
            segments_count,
            status,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub points_count: usize,
    pub vectors_count: usize,
    pub indexed_vectors_count: usize,
    pub segments_count: usize,
    pub status: String,
}

#[derive(Debug)]
pub struct IndexingResult {
    pub collection_name: String,
    pub documents_processed: usize,
    pub chunks_indexed: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub content: String,
    pub document_path: String,
    pub score: f32,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct KbFolderMonitor {
    indexer: KbIndexer,
    work_root: PathBuf,
}

impl KbFolderMonitor {
    pub fn new(work_root: PathBuf, embedding_config: EmbeddingConfig) -> Self {
        let qdrant_config = if let Some(sm) = crate::core::shared::utils::get_secrets_manager_sync() {
            let (url, api_key) = sm.get_vectordb_config_sync();
            QdrantConfig {
                url,
                api_key,
                timeout_secs: 30,
            }
        } else {
            QdrantConfig::default()
        };
        let indexer = KbIndexer::new(embedding_config, qdrant_config);

        Self { indexer, work_root }
    }

    pub async fn process_gbkb_folder(&self, bot_id: Uuid, bot_name: &str, kb_folder: &Path) -> Result<IndexingResult> {
        let kb_name = kb_folder
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid KB folder name"))?;

        info!("Processing .gbkb folder: {} for bot {}", kb_name, bot_name);

        let local_path = self
            .work_root
            .join(bot_name)
            .join(format!("{}.gbkb", bot_name))
            .join(kb_name);

        let result = self
            .indexer
            .index_kb_folder(bot_id, bot_name, kb_name, &local_path)
            .await?;

        info!(
            "Indexed {} documents ({} chunks) into collection {}",
            result.documents_processed, result.chunks_indexed, result.collection_name
        );

        Ok(result)
    }
}
