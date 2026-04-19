pub mod document_processor;
pub mod embedding_generator;
pub mod kb_indexer;
pub mod permissions;
pub mod web_crawler;
pub mod website_crawler_service;

pub use document_processor::{DocumentFormat, DocumentProcessor, TextChunk};
pub use embedding_generator::{
    EmailEmbeddingGenerator, EmbeddingConfig, EmbeddingGenerator, KbEmbeddingGenerator,
};
pub use kb_indexer::{CollectionInfo, IndexingResult, KbFolderMonitor, KbIndexer, QdrantConfig, SearchResult};
pub use web_crawler::{WebCrawler, WebPage, WebsiteCrawlConfig};
pub use website_crawler_service::{ensure_crawler_service_running, WebsiteCrawlerService};

use anyhow::Result;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::shared::utils::DbPool;

#[derive(Debug)]
pub struct KnowledgeBaseManager {
    indexer: Arc<KbIndexer>,
    processor: Arc<DocumentProcessor>,
    monitor: Arc<RwLock<KbFolderMonitor>>,
}

impl KnowledgeBaseManager {
    pub fn new(work_root: impl Into<std::path::PathBuf>) -> Self {
        Self::with_default_config(work_root)
    }

    pub fn with_default_config(work_root: impl Into<std::path::PathBuf>) -> Self {
        let work_root = work_root.into();
        let embedding_config = EmbeddingConfig::from_env();
        let qdrant_config = QdrantConfig::default();

        let indexer = Arc::new(KbIndexer::new(embedding_config.clone(), qdrant_config));
        let processor = Arc::new(DocumentProcessor::default());
        let monitor = Arc::new(RwLock::new(KbFolderMonitor::new(
            work_root,
            embedding_config,
        )));

        Self {
            indexer,
            processor,
            monitor,
        }
    }

    pub fn with_bot_config(work_root: impl Into<std::path::PathBuf>, pool: DbPool, bot_id: Uuid) -> Self {
        let work_root = work_root.into();
        let embedding_config = EmbeddingConfig::from_bot_config(&pool, &bot_id);
        info!("KB Manager using embedding config from bot {}: url={}, model={}", 
              bot_id, embedding_config.embedding_url, embedding_config.embedding_model);
        let qdrant_config = QdrantConfig::from_config(pool.clone(), &bot_id);

        let indexer = Arc::new(KbIndexer::new_with_pool(embedding_config.clone(), qdrant_config, pool));
        let processor = Arc::new(DocumentProcessor::default());
        let monitor = Arc::new(RwLock::new(KbFolderMonitor::new(
            work_root,
            embedding_config,
        )));

        Self {
            indexer,
            processor,
            monitor,
        }
    }

    pub async fn index_kb_folder(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        kb_name: &str,
        kb_path: &Path,
    ) -> Result<()> {
        info!(
            "Indexing knowledge base: {} for bot {} from path: {}",
            kb_name,
            bot_name,
            kb_path.display()
        );

        let result = self
            .indexer
            .index_kb_folder(bot_id, bot_name, kb_name, kb_path)
            .await?;

        info!(
            "Successfully indexed {} documents with {} chunks into collection {}",
            result.documents_processed, result.chunks_indexed, result.collection_name
        );

        Ok(())
    }

    pub async fn index_single_file(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        kb_name: &str,
        file_path: &Path,
    ) -> Result<kb_indexer::IndexingResult> {
        info!(
            "Indexing single file: {} into KB {} for bot {}",
            file_path.display(),
            kb_name,
            bot_name
        );

        let result = self
            .indexer
            .index_single_file(bot_id, bot_name, kb_name, file_path)
            .await?;

        info!(
            "Successfully indexed {} chunks from {} into collection {}",
            result.chunks_indexed,
            file_path.display(),
            result.collection_name
        );

        Ok(result)
    }

    pub async fn search(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        kb_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
        let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);
        
        // Use from_bot_config with state connection if available
        if let Some(pool) = self.indexer.get_db_pool() {
            let embedding_config = EmbeddingConfig::from_bot_config(pool, &bot_id);
            self.indexer.search_with_config(&collection_name, query, limit, &embedding_config).await
        } else {
            // Fallback to default config
            self.indexer.search(&collection_name, query, limit).await
        }
    }

    pub async fn search_collection(
        &self,
        collection_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.indexer.search(collection_name, query, limit).await
    }

    pub async fn process_document(&self, file_path: &Path) -> Result<Vec<TextChunk>> {
        self.processor.process_document(file_path).await
    }

    pub async fn handle_gbkb_change(&self, bot_id: Uuid, bot_name: &str, kb_folder: &Path) -> Result<()> {
        info!(
            "Handling .gbkb folder change for bot {} at {}",
            bot_name,
            kb_folder.display()
        );

        let kb_name = kb_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        #[derive(diesel::QueryableByName)]
        struct KbDocCount {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            document_count: Option<i32>,
        }

        if let Some(pool) = self.indexer.get_db_pool() {
            if let Ok(mut conn) = pool.get() {
                let existing: Option<KbDocCount> = diesel::sql_query(
                    "SELECT document_count FROM kb_collections WHERE bot_id = $1 AND name = $2"
                )
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(kb_name)
                .get_result(&mut conn)
                .ok();

                if let Some(row) = existing {
                    if let Some(count) = row.document_count {
                        if count > 0 {
                            info!(
                                "KB {} for bot {}/{} already indexed with {} docs, skipping re-index",
                                kb_name, bot_name, bot_id, count
                            );
                            return Ok(());
                        }
                    }
                }
            }
        }

        let monitor = self.monitor.read().await;
        let result = monitor.process_gbkb_folder(bot_id, bot_name, kb_folder).await?;

        let kb_name = kb_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let collection_name = result.collection_name.clone();
        let folder_path = kb_folder.to_string_lossy().to_string();
        let doc_count = result.documents_processed;

        if let Some(pool) = self.indexer.get_db_pool() {
            if let Ok(mut conn) = pool.get() {
                diesel::sql_query(
                    "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (bot_id, name) DO UPDATE SET
                        folder_path = EXCLUDED.folder_path,
                        qdrant_collection = EXCLUDED.qdrant_collection,
                        document_count = EXCLUDED.document_count,
                        updated_at = NOW()"
                )
                .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(kb_name)
                .bind::<diesel::sql_types::Text, _>(&folder_path)
                .bind::<diesel::sql_types::Text, _>(&collection_name)
                .bind::<diesel::sql_types::Integer, _>(doc_count as i32)
                .execute(&mut conn)
                .map_err(|e| {
                    error!("Failed to upsert kb_collections for {}/{}: {}", bot_name, kb_name, e);
                    e
                })?;
                info!(
                    "Upserted kb_collections: bot={}/{}, kb={}, collection={}, docs={}",
                    bot_name, bot_id, kb_name, collection_name, doc_count
                );
            } else {
                warn!("No DB connection available to upsert kb_collections for {}/{}", bot_name, kb_name);
            }
        } else {
            warn!("No DB pool available to upsert kb_collections for {}/{}", bot_name, kb_name);
        }

        Ok(())
    }

    pub async fn clear_kb(&self, bot_id: Uuid, bot_name: &str, kb_name: &str) -> Result<()> {
        let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
        let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);

        warn!("Clearing knowledge base collection: {}", collection_name);

        match self.indexer.delete_collection(&collection_name).await {
            Ok(_) => {
                info!("Successfully cleared collection: {}", collection_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to clear collection {}: {}", collection_name, e);
                Err(e)
            }
        }
    }

    pub async fn delete_file_from_kb(&self, bot_id: Uuid, bot_name: &str, kb_name: &str, file_path: &str) -> Result<()> {
        let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
        let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);

        // Use the relative path within the gbkb folder (e.g., "cartas/file.pdf")
        let relative_path = file_path
            .strip_prefix(&format!("{}/", kb_name))
            .unwrap_or(file_path);

        info!("Deleting vectors for file {} from collection {}", relative_path, collection_name);

        match self.indexer.delete_file_points(&collection_name, relative_path).await {
            Ok(_) => {
                info!("Successfully deleted vectors for file {} from {}", relative_path, collection_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete vectors for file {} from {}: {}", relative_path, collection_name, e);
                Err(e)
            }
        }
    }

    pub async fn get_kb_stats(&self, bot_id: Uuid, bot_name: &str, kb_name: &str) -> Result<KbStatistics> {
        let bot_id_short = bot_id.to_string().chars().take(8).collect::<String>();
        let collection_name = format!("{}_{}_{}", bot_name, bot_id_short, kb_name);

        let collection_info = self.indexer.get_collection_info(&collection_name).await?;

        let estimated_doc_count = if collection_info.points_count > 0 {
            std::cmp::max(1, collection_info.points_count / 10)
        } else {
            0
        };

        let estimated_size = collection_info.points_count * 1024;

        Ok(KbStatistics {
            collection_name,
            document_count: estimated_doc_count,
            chunk_count: collection_info.points_count,
            total_size_bytes: estimated_size,
            status: collection_info.status,
        })
    }
}

#[derive(Debug, Clone)]
pub struct KbStatistics {
    pub collection_name: String,
    pub document_count: usize,
    pub chunk_count: usize,
    pub total_size_bytes: usize,
    pub status: String,
}

#[derive(Debug)]
pub struct DriveMonitorIntegration {
    kb_manager: Arc<KnowledgeBaseManager>,
}

impl DriveMonitorIntegration {
    pub fn new(kb_manager: Arc<KnowledgeBaseManager>) -> Self {
        Self { kb_manager }
    }

    pub async fn on_gbkb_folder_changed(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        folder_path: &Path,
        change_type: ChangeType,
    ) -> Result<()> {
        match change_type {
            ChangeType::Created | ChangeType::Modified => {
                info!(
                    "Drive monitor detected {:?} in .gbkb folder: {}",
                    change_type,
                    folder_path.display()
                );
                self.kb_manager
                    .handle_gbkb_change(bot_id, bot_name, folder_path)
                    .await
            }
            ChangeType::Deleted => {
                if let Some(kb_name) = folder_path.file_name().and_then(|n| n.to_str()) {
                    self.kb_manager.clear_kb(bot_id, bot_name, kb_name).await
                } else {
                    Err(anyhow::anyhow!("Invalid KB folder path"))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}
