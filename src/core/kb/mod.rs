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
pub use kb_indexer::{CollectionInfo, KbFolderMonitor, KbIndexer, QdrantConfig, SearchResult};
pub use web_crawler::{WebCrawler, WebPage, WebsiteCrawlConfig};
pub use website_crawler_service::{ensure_crawler_service_running, WebsiteCrawlerService};

use anyhow::Result;
use log::{error, info, warn};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct KnowledgeBaseManager {
    indexer: Arc<KbIndexer>,
    processor: Arc<DocumentProcessor>,
    monitor: Arc<RwLock<KbFolderMonitor>>,
}

impl KnowledgeBaseManager {
    pub fn new(work_root: impl Into<std::path::PathBuf>) -> Self {
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

    pub async fn index_kb_folder(
        &self,
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
            .index_kb_folder(bot_name, kb_name, kb_path)
            .await?;

        info!(
            "Successfully indexed {} documents with {} chunks into collection {}",
            result.documents_processed, result.chunks_indexed, result.collection_name
        );

        Ok(())
    }

    pub async fn search(
        &self,
        bot_name: &str,
        kb_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let collection_name = format!("{}_{}", bot_name, kb_name);
        self.indexer.search(&collection_name, query, limit).await
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

    pub async fn handle_gbkb_change(&self, bot_name: &str, kb_folder: &Path) -> Result<()> {
        info!(
            "Handling .gbkb folder change for bot {} at {}",
            bot_name,
            kb_folder.display()
        );

        let monitor = self.monitor.read().await;
        monitor.process_gbkb_folder(bot_name, kb_folder).await
    }

    pub async fn clear_kb(&self, bot_name: &str, kb_name: &str) -> Result<()> {
        let collection_name = format!("{}_{}", bot_name, kb_name);

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

    pub async fn get_kb_stats(&self, bot_name: &str, kb_name: &str) -> Result<KbStatistics> {
        let collection_name = format!("{}_{}", bot_name, kb_name);

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
                    .handle_gbkb_change(bot_name, folder_path)
                    .await
            }
            ChangeType::Deleted => {
                if let Some(kb_name) = folder_path.file_name().and_then(|n| n.to_str()) {
                    self.kb_manager.clear_kb(bot_name, kb_name).await
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
