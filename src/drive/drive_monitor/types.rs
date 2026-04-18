#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::KnowledgeBaseManager;
use crate::core::shared::state::AppState;
#[cfg(not(any(feature = "research", feature = "llm")))]
use std::collections::HashMap;
#[cfg(any(feature = "research", feature = "llm"))]
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
#[cfg(any(feature = "research", feature = "llm"))]
use tokio::sync::RwLock as TokioRwLock;

use crate::drive::drive_files::DriveFileRepository;

#[cfg(any(feature = "research", feature = "llm"))]
static LLM_STREAMING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(any(feature = "research", feature = "llm"))]
pub fn set_llm_streaming(streaming: bool) {
    LLM_STREAMING.store(streaming, Ordering::SeqCst);
}

#[cfg(any(feature = "research", feature = "llm"))]
pub fn is_llm_streaming() -> bool {
    LLM_STREAMING.load(Ordering::SeqCst)
}

const MAX_BACKOFF_SECS: u64 = 300;
const INITIAL_BACKOFF_SECS: u64 = 30;
const RETRY_BACKOFF_SECS: i64 = 3600;
const MAX_FAIL_COUNT: i32 = 3;

pub fn normalize_etag(etag: &str) -> String {
    etag.trim_matches('"').to_string()
}

pub fn normalize_config_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        trimmed.to_string()
    }
}

impl DriveMonitor {
    /// Start monitoring the drive bucket for changes
    /// This is a placeholder that will be implemented with the actual monitoring logic
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("DriveMonitor monitoring started for bucket: {}", self.bucket_name);
        // The actual monitoring logic is handled by LocalFileMonitor
        // This method is kept for backward compatibility
        Ok(())
    }
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
    pub files_being_indexed: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(any(feature = "research", feature = "llm"))]
    pub pending_kb_index: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(any(feature = "research", feature = "llm"))]
    pub kb_indexed_folders: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(not(any(feature = "research", feature = "llm")))]
    pub _pending_kb_index: Arc<TokioRwLock<HashSet<String>>>,
    pub file_repo: Arc<DriveFileRepository>,
    #[allow(dead_code)]
    pub pending_changes: Arc<TokioRwLock<Vec<String>>>,
    #[allow(dead_code)]
    pub last_etag_snapshot: Arc<TokioRwLock<std::collections::HashMap<String, String>>>,
}

impl DriveMonitor {
    fn normalize_config_value(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            String::new()
        } else {
            trimmed.to_string()
        }
    }

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
            files_being_indexed: Arc::new(TokioRwLock::new(HashSet::new())),
            #[cfg(any(feature = "research", feature = "llm"))]
            pending_kb_index: Arc::new(TokioRwLock::new(HashSet::new())),
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_indexed_folders: Arc::new(TokioRwLock::new(HashSet::new())),
            #[cfg(not(any(feature = "research", feature = "llm")))]
            _pending_kb_index: Arc::new(TokioRwLock::new(HashSet::new())),
            file_repo,
            pending_changes: Arc::new(TokioRwLock::new(Vec::new())),
            last_etag_snapshot: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }
}
