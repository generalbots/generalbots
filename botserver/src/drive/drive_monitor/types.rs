use crate::core::shared::state::AppState;
use crate::drive::drive_files::DriveFileRepository;
use crate::drive::drive_monitor::monitor::CHECK_INTERVAL_SECS;
#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::KnowledgeBaseManager;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

pub fn normalize_etag(etag: &str) -> String {
    etag.trim_matches('"').to_string()
}

impl DriveMonitor {
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("DriveMonitor monitoring started for bucket: {}", self.bucket_name);

        loop {
            // Reentrancy protection: skip if previous scan is still running
            if self.is_processing.load(Ordering::Relaxed) {
                log::debug!("DriveMonitor still processing, skipping iteration");
            } else {
                self.is_processing.store(true, Ordering::Relaxed);
                if let Err(e) = self.scan_bucket().await {
                    log::error!("Failed to scan bucket {}: {}", self.bucket_name, e);
                }
                self.is_processing.store(false, Ordering::Relaxed);
            }
            tokio::time::sleep(std::time::Duration::from_secs(CHECK_INTERVAL_SECS)).await;
        }
    }

    async fn scan_bucket(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("DriveMonitor: Starting scan of bucket {}", self.bucket_name);
        let start = std::time::Instant::now();

        if let Some(s3) = &self.state.drive {
            match s3.list_objects_with_metadata(&self.bucket_name, None).await {
                Ok(objects) => {
                    log::info!("Found {} objects in bucket {}", objects.len(), self.bucket_name);

                    let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);

                    let current_keys: Vec<String> = objects.iter().map(|o| o.key.clone()).collect();

        for obj in &objects {
            if obj.key.ends_with('/') {
                log::debug!("Skipping directory entry: {}", obj.key);
                continue;
            }

            let file_type = classify_file(&obj.key);
                        let full_key = format!("{}.gbai/{}", bot_name, obj.key);
                        let etag = obj.etag.as_deref().map(normalize_etag);

            let existing = self.file_repo.get_file_state(self.bot_id, &full_key);
            let needs_reindex = match &existing {
                Some(prev) if prev.indexed && prev.etag.as_deref() == etag.as_deref() => false,
                Some(prev) if prev.indexed && prev.etag.as_deref() != etag.as_deref() => {
                    log::info!("ETag changed for {}, will reindex", full_key);
                    true
                }
                Some(prev) if !prev.indexed && prev.etag.as_deref() == etag.as_deref() => {
                    log::debug!("{} unchanged but not yet indexed, will index", full_key);
                    true
                }
                Some(_) => true,
                None => true,
            };

            let etag_changed = existing.as_ref().map_or(true, |prev| prev.etag.as_deref() != etag.as_deref());

            if etag_changed || existing.is_none() {
                match self.file_repo.upsert_file(
                    self.bot_id,
                    &full_key,
                    file_type,
                    etag,
                    None,
                ) {
                    Ok(_) => log::info!("Added/updated drive_files for: {} ({})", full_key, file_type),
                    Err(e) => log::error!("Failed to upsert {}: {}", full_key, e),
                }

                if file_type == "bas" {
                    self.sync_bas_to_work(bot_name, &obj.key).await;
                }
            } else {
                log::debug!("{} unchanged, skipping upsert", full_key);
            }

            if needs_reindex && file_type == "kb" {
                    #[cfg(any(feature = "research", feature = "llm"))]
                    {
                        self.index_kb_file(bot_name, &full_key, &obj.key).await;
                    }
                }

                if file_type == "config" && needs_reindex {
                    self.sync_bot_config(bot_name, &obj.key).await;
                }
                    }

        self.handle_deleted_files(bot_name, &current_keys);
        }
        Err(e) => {
            log::error!("Failed to list objects in {}: {}", self.bucket_name, e);
        }
        }
        } else {
            log::warn!("S3 client not available for bucket scan");
        }

        let elapsed = start.elapsed();
        log::info!("DriveMonitor: Completed scan of {} in {:.2?}", self.bucket_name, elapsed);
        Ok(())
    }

    fn handle_deleted_files(&self, bot_name: &str, current_keys: &[String]) {
        let db_files = self.file_repo.get_all_files_for_bot(self.bot_id);
        for db_file in &db_files {
            let s3_key = match db_file.file_path.strip_prefix(&format!("{}.gbai/", bot_name)) {
                Some(k) => k,
                None => continue,
            };
            if !current_keys.iter().any(|k| k == s3_key) {
                log::info!("File deleted from S3: {} (was in DB)", db_file.file_path);

                if db_file.file_type == "kb" {
                    #[cfg(any(feature = "research", feature = "llm"))]
                    {
                        self.delete_kb_file_vectors(bot_name, &db_file.file_path, s3_key);
                    }
                }

                if let Err(e) = self.file_repo.delete_file(self.bot_id, &db_file.file_path) {
                    log::error!("Failed to delete drive_files entry for {}: {}", db_file.file_path, e);
                }
            }
        }
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    async fn index_kb_file(&self, bot_name: &str, full_key: &str, s3_key: &str) {
        let parsed = match parse_kb_path(s3_key) {
            Some(p) => p,
            None => {
                log::debug!("Not a KB file path: {}", s3_key);
                return;
            }
        };

        let mut being_indexed = self.files_being_indexed.write().await;
        if being_indexed.contains(full_key) {
            log::debug!("Already indexing {}, skipping", full_key);
            return;
        }
        being_indexed.insert(full_key.to_string());
        drop(being_indexed);

        let s3 = match &self.state.drive {
            Some(s3) => s3,
            None => {
                log::error!("S3 client not available for KB indexing of {}", full_key);
                self.files_being_indexed.write().await.remove(full_key);
                return;
            }
        };

        let data = match s3.get_object_direct(&self.bucket_name, s3_key).await {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to download KB file {}/{}: {}", self.bucket_name, s3_key, e);
                let _ = self.file_repo.mark_failed(self.bot_id, full_key);
                self.files_being_indexed.write().await.remove(full_key);
                return;
            }
        };

        let temp_path = std::env::temp_dir().join(format!("gb_kb_{}_{}", uuid::Uuid::new_v4(), parsed.file_name));

        if let Err(e) = std::fs::write(&temp_path, &data) {
            log::error!("Failed to write temp file {}: {}", temp_path.display(), e);
            self.files_being_indexed.write().await.remove(full_key);
            return;
        }

        log::info!("Indexing KB file {}/{} -> temp {}", bot_name, parsed.kb_name, temp_path.display());

        match self.kb_manager.index_single_file_with_id(
            self.bot_id,
            bot_name,
            &parsed.kb_name,
            &temp_path,
            Some(full_key),
        ).await {
            Ok(result) => {
                log::info!(
                    "Indexed {} chunks from {} into collection {}",
                    result.chunks_indexed,
                    full_key,
                    result.collection_name
                );
                let _ = self.file_repo.mark_indexed(self.bot_id, full_key);
                self.upsert_kb_collection(bot_name, &parsed.kb_name, &result.collection_name, result.documents_processed);
            }
            Err(e) => {
                log::error!("KB indexing failed for {}: {}", full_key, e);
                let _ = self.file_repo.mark_failed(self.bot_id, full_key);
            }
        }

        let _ = std::fs::remove_file(&temp_path);
        self.files_being_indexed.write().await.remove(full_key);
    }

    async fn sync_bot_config(&self, bot_name: &str, s3_key: &str) {
        let s3 = match &self.state.drive {
            Some(s3) => s3,
            None => {
                log::error!("S3 client not available for config sync");
                return;
            }
        };

        let data = match s3.get_object_direct(&self.bucket_name, s3_key).await {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to download config.csv from {}/{}: {}", self.bucket_name, s3_key, e);
                return;
            }
        };

        let content = match String::from_utf8(data) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to parse config.csv as UTF-8: {}", e);
                return;
            }
        };

        let config_manager = crate::core::config::ConfigManager::new(self.state.conn.clone());

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.to_lowercase().starts_with("key,") {
                continue;
            }
            if let Some((key, value)) = line.split_once(',') {
                let key = key.trim();
                let value = value.trim();
                if key.is_empty() {
                    continue;
                }
                if let Err(e) = config_manager.set_config(&self.bot_id, key, value) {
                    log::error!("Failed to set config {}={} for bot {}: {}", key, value, bot_name, e);
                } else {
                    log::info!("Synced config {}={} for bot {}", key, value, bot_name);
                }
            }
        }

        let full_key = format!("{}.gbai/{}", bot_name, s3_key);
        let _ = self.file_repo.mark_indexed(self.bot_id, &full_key);
    }

    async fn sync_bas_to_work(&self, bot_name: &str, s3_key: &str) {
        let s3 = match &self.state.drive {
            Some(s3) => s3,
            None => {
                log::error!("S3 client not available for .bas sync");
                return;
            }
        };

        let data = match s3.get_object_direct(&self.bucket_name, s3_key).await {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to download .bas from {}/{}: {}", self.bucket_name, s3_key, e);
                return;
            }
        };

        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        if let Err(e) = std::fs::create_dir_all(&work_dir) {
            log::error!("Failed to create work dir {}: {}", work_dir.display(), e);
            return;
        }

        let file_name = s3_key.split('/').next_back().unwrap_or(s3_key);
        let work_path = work_dir.join(file_name);

        match String::from_utf8(data) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&work_path, &content) {
                    log::error!("Failed to write {} to work dir: {}", work_path.display(), e);
                } else {
                    log::info!("Synced {} to work dir {}", s3_key, work_path.display());
                }
            }
            Err(e) => {
                log::error!("Failed to parse .bas as UTF-8: {}", e);
            }
        }
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    fn delete_kb_file_vectors(&self, bot_name: &str, _full_key: &str, s3_key: &str) {
        let parsed = match parse_kb_path(s3_key) {
            Some(p) => p,
            None => return,
        };

        let kb_manager = self.kb_manager.clone();
        let bot_id = self.bot_id;
        let bot_name = bot_name.to_string();
        let relative_path = parsed.relative_path.clone();

        tokio::spawn(async move {
            match kb_manager.delete_file_from_kb(bot_id, &bot_name, &parsed.kb_name, &relative_path).await {
                Ok(_) => log::info!("Deleted vectors for {} from {}/{}", relative_path, bot_name, parsed.kb_name),
                Err(e) => log::error!("Failed to delete vectors for {} from {}/{}: {}", relative_path, bot_name, parsed.kb_name, e),
            }
        });
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    fn upsert_kb_collection(&self, bot_name: &str, kb_name: &str, collection_name: &str, doc_count: usize) {
        use diesel::prelude::*;
        use uuid::Uuid;

        if let Ok(mut conn) = self.state.conn.get() {
            let folder_path = format!("{}.gbai/{}.gbkb/{}", bot_name, bot_name, kb_name);
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
            .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
            .bind::<diesel::sql_types::Text, _>(kb_name)
            .bind::<diesel::sql_types::Text, _>(&folder_path)
            .bind::<diesel::sql_types::Text, _>(collection_name)
            .bind::<diesel::sql_types::Integer, _>(doc_count as i32)
            .execute(&mut conn)
            .unwrap_or_else(|e| {
                log::error!("Failed to upsert kb_collections for {}/{}: {}", bot_name, kb_name, e);
                0
            });
        }
    }
}

fn classify_file(key: &str) -> &'static str {
    if key.ends_with(".bas") {
        "bas"
    } else if key.contains(".gbkb/") && is_kb_extension(key) {
        "kb"
    } else if key.contains(".gbot/") && key.ends_with("config.csv") {
        "config"
    } else {
        "other"
    }
}

fn is_kb_extension(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.ends_with(".txt")
        || lower.ends_with(".md")
        || lower.ends_with(".pdf")
        || lower.ends_with(".xlsx")
        || lower.ends_with(".xls")
        || lower.ends_with(".docx")
        || lower.ends_with(".doc")
        || lower.ends_with(".csv")
        || lower.ends_with(".pptx")
        || lower.ends_with(".ppt")
        || lower.ends_with(".html")
        || lower.ends_with(".htm")
        || lower.ends_with(".rtf")
        || lower.ends_with(".epub")
        || lower.ends_with(".xml")
        || lower.ends_with(".json")
        || lower.ends_with(".odt")
        || lower.ends_with(".ods")
        || lower.ends_with(".odp")
}

struct KbPathParts {
    kb_name: String,
    file_name: String,
    relative_path: String,
}

fn parse_kb_path(s3_key: &str) -> Option<KbPathParts> {
    let parts: Vec<&str> = s3_key.splitn(4, '/').collect();
    if parts.len() < 3 || !parts[0].ends_with(".gbkb") {
        return None;
    }
    let kb_name = parts[1].to_string();
    let file_name = parts[2..].join("/");
    let relative_path = format!("{}/{}", kb_name, file_name);
    Some(KbPathParts {
        kb_name,
        file_name,
        relative_path,
    })
}

#[derive(Debug, Clone)]
pub struct DriveMonitor {
    pub state: Arc<AppState>,
    pub bucket_name: String,
    pub bot_id: uuid::Uuid,
    #[cfg(any(feature = "research", feature = "llm"))]
    pub kb_manager: Arc<KnowledgeBaseManager>,
    pub work_root: PathBuf,
    pub is_processing: Arc<AtomicBool>,
    pub scanning: Arc<AtomicBool>,
    pub consecutive_failures: Arc<AtomicU32>,
    #[cfg(any(feature = "research", feature = "llm"))]
    pub files_being_indexed: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
    #[cfg(not(any(feature = "research", feature = "llm")))]
    pub _pending_kb_index: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
    pub file_repo: Arc<DriveFileRepository>,
}

impl DriveMonitor {
    pub fn new(state: Arc<AppState>, bucket_name: String, bot_id: uuid::Uuid) -> Self {
        let work_root = PathBuf::from(crate::core::shared::utils::get_work_path());
        #[cfg(any(feature = "research", feature = "llm"))]
        let kb_manager = Arc::new(KnowledgeBaseManager::with_bot_config(
            work_root.clone(),
            state.conn.clone(),
            bot_id,
        ));

        let file_repo = Arc::new(DriveFileRepository::new(state.conn.clone()));

        Self {
            state,
            bucket_name,
            bot_id,
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_manager,
            work_root,
            is_processing: Arc::new(AtomicBool::new(false)),
            scanning: Arc::new(AtomicBool::new(false)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            #[cfg(any(feature = "research", feature = "llm"))]
            files_being_indexed: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
            #[cfg(not(any(feature = "research", feature = "llm")))]
            _pending_kb_index: Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
            file_repo,
        }
    }
}
