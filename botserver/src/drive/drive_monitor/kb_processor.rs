#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::KnowledgeBaseManager;
#[cfg(any(feature = "research", feature = "llm"))]
use log::{error, info, trace, warn};
#[cfg(any(feature = "research", feature = "llm"))]
use std::collections::HashSet;
#[cfg(any(feature = "research", feature = "llm"))]
use std::path::PathBuf;
#[cfg(any(feature = "research", feature = "llm"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(feature = "research", feature = "llm"))]
use std::sync::Arc;
#[cfg(any(feature = "research", feature = "llm"))]
use tokio::sync::RwLock as TokioRwLock;
#[cfg(any(feature = "research", feature = "llm"))]
use tokio::time::Duration;
#[cfg(any(feature = "research", feature = "llm"))]
use crate::drive::drive_files::DriveFileRepository;

#[cfg(any(feature = "research", feature = "llm"))]
pub fn start_kb_processor(
    kb_manager: Arc<KnowledgeBaseManager>,
    bot_id: uuid::Uuid,
    bot_name: String,
    work_root: PathBuf,
    pending_kb_index: Arc<TokioRwLock<HashSet<String>>>,
    files_being_indexed: Arc<TokioRwLock<HashSet<String>>>,
    kb_indexed_folders: Arc<TokioRwLock<HashSet<String>>>,
    file_repo: Arc<DriveFileRepository>,
    is_processing: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        while is_processing.load(Ordering::SeqCst) {
            let kb_key = {
                let pending = pending_kb_index.write().await;
                pending.iter().next().cloned()
            };

            let Some(kb_key) = kb_key else {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            };

            let parts: Vec<&str> = kb_key.splitn(2, '_').collect();
            if parts.len() < 2 {
                let mut pending = pending_kb_index.write().await;
                pending.remove(&kb_key);
                continue;
            }

            let kb_folder_name = parts[1];
            let kb_folder_path =
                work_root.join(&bot_name).join(format!("{}.gbkb/", bot_name)).join(kb_folder_name);

            {
                let indexing = files_being_indexed.read().await;
                if indexing.contains(&kb_key) {
                    let mut pending = pending_kb_index.write().await;
                    pending.remove(&kb_key);
                    continue;
                }
            }

            {
                let mut indexing = files_being_indexed.write().await;
                indexing.insert(kb_key.clone());
            }

            trace!("Indexing KB: {} for bot: {}", kb_key, bot_name);

            let result =
                tokio::time::timeout(Duration::from_secs(120), kb_manager.handle_gbkb_change(bot_id, &bot_name, kb_folder_path.as_path()))
                    .await;

            {
                let mut indexing = files_being_indexed.write().await;
                indexing.remove(&kb_key);
            }

            {
                let mut pending = pending_kb_index.write().await;
                pending.remove(&kb_key);
            }

            match result {
                Ok(Ok(_)) => {
                    info!("Successfully indexed KB: {}", kb_key);
                    {
                        let mut indexed = kb_indexed_folders.write().await;
                        indexed.insert(kb_key.clone());
                    }
                    let pattern = format!("{}/", kb_folder_name);
                    if let Err(e) = file_repo.mark_indexed_by_pattern(bot_id, &pattern) {
                        warn!("Failed to mark files indexed for {}: {}", kb_key, e);
                    }
                }
                Ok(Err(e)) => {
                    warn!("Failed to index KB {}: {}", kb_key, e);
                    let pattern = format!("{}/", kb_folder_name);
                    if let Err(e) = file_repo.mark_failed_by_pattern(bot_id, &pattern) {
                        warn!("Failed to mark files failed for {}: {}", kb_key, e);
                    }
                }
                Err(_) => {
                    error!("KB indexing timed out after 120s for {}", kb_key);
                    let pattern = format!("{}/", kb_folder_name);
                    if let Err(e) = file_repo.mark_failed_by_pattern(bot_id, &pattern) {
                        warn!("Failed to mark files failed for {}: {}", kb_key, e);
                    }
                }
            }
        }

        trace!("Stopping for bot {}", bot_name);
    });
}
