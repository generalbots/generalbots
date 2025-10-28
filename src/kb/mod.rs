use crate::shared::models::KBCollection;
use crate::shared::state::AppState;
use log::{ error, info, warn};
use tokio_stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub mod embeddings;
pub mod minio_handler;
pub mod qdrant_client;

#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    Created(String),
    Modified(String),
    Deleted(String),
}

pub struct KBManager {
    state: Arc<AppState>,
    watched_collections: Arc<tokio::sync::RwLock<HashMap<String, KBCollection>>>,
}

impl KBManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            watched_collections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_collection(
        &self,
        bot_id: String,
        user_id: String,
        collection_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let folder_path = format!(".gbkb/{}", collection_name);
        let qdrant_collection = format!("kb_{}_{}", bot_id, collection_name);
        
        info!(
            "Adding KB collection: {} -> {}",
            collection_name, qdrant_collection
        );

        qdrant_client::ensure_collection_exists(&self.state, &qdrant_collection).await?;

        let now = chrono::Utc::now().to_rfc3339();
        let collection = KBCollection {
            id: uuid::Uuid::new_v4().to_string(),
            bot_id,
            user_id,
            name: collection_name.to_string(),
            folder_path: folder_path.clone(),
            qdrant_collection: qdrant_collection.clone(),
            document_count: 0,
            is_active: 1,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut collections = self.watched_collections.write().await;
        collections.insert(collection_name.to_string(), collection);

        Ok(())
    }

    pub async fn remove_collection(
        &self,
        collection_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut collections = self.watched_collections.write().await;
        collections.remove(collection_name);
        Ok(())
    }

    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(30));
            loop {
                tick.tick().await;
                let collections = self.watched_collections.read().await;
                for (name, collection) in collections.iter() {
                    if let Err(e) = self.check_collection_updates(collection).await {
                        error!("Error checking collection {}: {}", name, e);
                    }
                }
            }
        })
    }

    async fn check_collection_updates(
        &self,
        collection: &KBCollection,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let op = match &self.state.s3_operator {
            Some(op) => op,
            None => {
                warn!("S3 operator not configured");
                return Ok(());
            }
        };

        let mut lister = op.lister_with(&collection.folder_path).recursive(true).await?;
        while let Some(entry) = lister.try_next().await? {
            let path = entry.path().to_string();
            
            if path.ends_with('/') {
                continue;
            }

            let meta = op.stat(&path).await?;
            if let Err(e) = self
                .process_file(
                    &collection,
                    &path,
                    meta.content_length() as i64,
                    meta.last_modified().map(|dt| dt.to_rfc3339()),
                )
                .await
            {
                error!("Error processing file {}: {}", path, e);
            }
        }

        Ok(())
    }

    async fn process_file(
        &self,
        collection: &KBCollection,
        file_path: &str,
        file_size: i64,
        _last_modified: Option<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let content = self.get_file_content(file_path).await?;
        let file_hash = if content.len() > 100 {
            format!(
                "{:x}_{:x}_{}",
                content.len(),
                content[0] as u32 * 256 + content[1] as u32,
                content[content.len() - 1] as u32 * 256 + content[content.len() - 2] as u32
            )
        } else {
            format!("{:x}", content.len())
        };

        if self
            .is_file_indexed(collection.bot_id.clone(), file_path, &file_hash)
            .await?
        {
            return Ok(());
        }

        info!("Indexing file: {} to collection {}", file_path, collection.name);
        let text_content = self.extract_text(file_path, &content).await?;
        
        embeddings::index_document(
            &self.state,
            &collection.qdrant_collection,
            file_path,
            &text_content,
        )
        .await?;

        let metadata = serde_json::json!({
            "file_type": self.get_file_type(file_path),
            "last_modified": _last_modified,
        });

        self.save_document_metadata(
            collection.bot_id.clone(),
            &collection.name,
            file_path,
            file_size,
            &file_hash,
            metadata,
        )
        .await?;

        Ok(())
    }

    async fn get_file_content(
        &self,
        file_path: &str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let op = self
            .state
            .s3_operator
            .as_ref()
            .ok_or("S3 operator not configured")?;

        let content = op.read(file_path).await?;
        Ok(content.to_vec())
    }

    async fn extract_text(
        &self,
        file_path: &str,
        content: &[u8],
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let path_lower = file_path.to_ascii_lowercase();
        if path_lower.ends_with(".pdf") {
            match pdf_extract::extract_text_from_mem(content) {
                Ok(text) => Ok(text),
                Err(e) => {
                    error!("PDF extraction failed for {}: {}", file_path, e);
                    Err(format!("PDF extraction failed: {}", e).into())
                }
            }
        } else if path_lower.ends_with(".txt") || path_lower.ends_with(".md") {
            String::from_utf8(content.to_vec())
                .map_err(|e| format!("UTF-8 decoding failed: {}", e).into())
        } else if path_lower.ends_with(".docx") {
            warn!("DOCX format not yet supported: {}", file_path);
            Err("DOCX format not supported".into())
        } else {
            String::from_utf8(content.to_vec())
                .map_err(|e| format!("Unsupported file format or UTF-8 error: {}", e).into())
        }
    }

    async fn is_file_indexed(
        &self,
        _bot_id: String,
        _file_path: &str,
        _file_hash: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(false)
    }

    async fn save_document_metadata(
        &self,
        _bot_id: String,
        _collection_name: &str,
        file_path: &str,
        file_size: i64,
        file_hash: &str,
        _metadata: serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(
            "Saving metadata for {}: size={}, hash={}",
            file_path, file_size, file_hash
        );
        Ok(())
    }

    fn get_file_type(&self, file_path: &str) -> String {
        file_path
            .rsplit('.')
            .next()
            .unwrap_or("unknown")
            .to_lowercase()
    }
}
