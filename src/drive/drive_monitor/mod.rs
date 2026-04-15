use crate::basic::compiler::BasicCompiler;
use crate::core::config::ConfigManager;
#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::KnowledgeBaseManager;
use crate::core::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
use crate::core::shared::message_types::MessageType;
use crate::core::shared::state::AppState;

#[cfg(feature = "drive")]
use aws_sdk_s3::Client;
use chrono::{DateTime, Utc};
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
#[cfg(any(feature = "research", feature = "llm"))]
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
#[cfg(any(feature = "research", feature = "llm"))]
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::Duration;

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

fn normalize_etag(etag: &str) -> String {
    etag.trim_matches('"').to_string()
}

#[derive(Debug, Clone)]
pub struct DriveMonitor {
    state: Arc<AppState>,
    bucket_name: String,
    bot_id: uuid::Uuid,
    #[cfg(any(feature = "research", feature = "llm"))]
    kb_manager: Arc<KnowledgeBaseManager>,
    work_root: PathBuf,
    is_processing: Arc<AtomicBool>,
    scanning: Arc<AtomicBool>,
    consecutive_failures: Arc<AtomicU32>,
    #[cfg(any(feature = "research", feature = "llm"))]
    files_being_indexed: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(any(feature = "research", feature = "llm"))]
    pending_kb_index: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(any(feature = "research", feature = "llm"))]
    kb_indexed_folders: Arc<TokioRwLock<HashSet<String>>>,
    #[cfg(not(any(feature = "research", feature = "llm")))]
    _pending_kb_index: Arc<TokioRwLock<HashSet<String>>>,
    // Database-backed file state repository (replaces JSON file_states)
    file_repo: Arc<DriveFileRepository>,
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
        let kb_manager = Arc::new(KnowledgeBaseManager::with_bot_config(work_root.clone(), state.conn.clone(), bot_id));

        // Initialize DB-backed file state repository
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
        }
    }

    async fn check_drive_health(&self) -> bool {
        let Some(client) = &self.state.drive else {
            return false;
        };

        match tokio::time::timeout(
            Duration::from_secs(5),
            client.head_bucket().bucket(&self.bucket_name).send(),
        )
        .await
        {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                debug!("Health check failed: {}", e);
                false
            }
            Err(_) => {
                debug!("Health check timed out");
                false
            }
        }
    }

    fn calculate_backoff(&self) -> Duration {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures == 0 {
            return Duration::from_secs(INITIAL_BACKOFF_SECS);
        }
        let backoff_secs = INITIAL_BACKOFF_SECS * (1u64 << failures.min(4));
        Duration::from_secs(backoff_secs.min(MAX_BACKOFF_SECS))
    }

    /// Start a long-running background KB processor that handles pending indexing requests
    /// Only one instance runs per bot - this is spawned once from start_monitoring
    #[cfg(any(feature = "research", feature = "llm"))]
    pub fn start_kb_processor(&self) {
        Self::start_kb_processor_inner(
            Arc::clone(&self.kb_manager),
            self.bot_id,
            self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name).to_string(),
            self.work_root.clone(),
            Arc::clone(&self.pending_kb_index),
            Arc::clone(&self.files_being_indexed),
            Arc::clone(&self.kb_indexed_folders),
            Arc::clone(&self.file_repo),
            Arc::clone(&self.is_processing),
        );
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    fn start_kb_processor_inner(
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
      // Keep running as long as the DriveMonitor is active
      while is_processing.load(std::sync::atomic::Ordering::SeqCst) {
        // Get one pending KB folder from the queue
        let kb_key = {
          let pending = pending_kb_index.write().await;
          pending.iter().next().cloned()
        };

                let Some(kb_key) = kb_key else {
                    // Nothing pending, wait and retry
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                };

                // Parse KB key to get folder name
                let parts: Vec<&str> = kb_key.splitn(2, '_').collect();
                if parts.len() < 2 {
                    let mut pending = pending_kb_index.write().await;
                    pending.remove(&kb_key);
                    continue;
                }

                let kb_folder_name = parts[1];
                let kb_folder_path = work_root.join(&bot_name).join(format!("{}.gbkb/", bot_name)).join(kb_folder_name);

                // Check if already being indexed
                {
                    let indexing = files_being_indexed.read().await;
                    if indexing.contains(&kb_key) {
                        // Already processing, move to next
                        let mut pending = pending_kb_index.write().await;
                        pending.remove(&kb_key);
                        continue;
                    }
                }

                // Mark as being indexed
                {
                    let mut indexing = files_being_indexed.write().await;
                    indexing.insert(kb_key.clone());
                }

                trace!("Indexing KB: {} for bot: {}", kb_key, bot_name);

                // Perform the actual KB indexing
                let result = tokio::time::timeout(
                    Duration::from_secs(120),
                    kb_manager.handle_gbkb_change(bot_id, &bot_name, kb_folder_path.as_path()),
                ).await;

                // Remove from being indexed
                {
                    let mut indexing = files_being_indexed.write().await;
                    indexing.remove(&kb_key);
                }

                // Remove from pending queue
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

    /// Stub for production builds without llm/research features
    #[cfg(not(any(feature = "research", feature = "llm")))]
    pub fn start_kb_processor(&self) {
        // KB indexing not available in this build
    }

  pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start_mem = MemoryStats::current();
    trace!(
      "Starting DriveMonitor for bot {}",
      self.bot_id
    );

        // Check if already processing to prevent duplicate monitoring
        if self.is_processing.load(std::sync::atomic::Ordering::Acquire) {
            warn!("Already processing for bot {}, skipping", self.bot_id);
            return Ok(());
        }

        // File states are now loaded from DB on demand - no need to load from disk

        if !self.check_drive_health().await {
            warn!(
                "S3/MinIO not available for bucket {}, will retry with backoff",
                self.bucket_name
            );
        }

        self.is_processing
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Start the background KB processor - one instance per bot
        #[cfg(any(feature = "research", feature = "llm"))]
        self.start_kb_processor();

        trace!("start_monitoring: calling check_for_changes...");
        trace!("Calling initial check_for_changes...");

        match tokio::time::timeout(Duration::from_secs(12), self.check_for_changes()).await {
            Ok(Ok(_)) => {
                self.consecutive_failures.store(0, Ordering::Relaxed);
            }
            Ok(Err(e)) => {
                warn!("Initial check failed (will retry): {}", e);
                self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                error!("Initial check timed out after 5 minutes");
                self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
            }
        }
        trace!("start_monitoring: check_for_changes returned");

        let after_initial = MemoryStats::current();
        trace!(
            "After initial check, RSS={} (delta={})",
            MemoryStats::format_bytes(after_initial.rss_bytes),
            MemoryStats::format_bytes(after_initial.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );

        // Force enable periodic monitoring regardless of initial check result
        self.is_processing.store(true, std::sync::atomic::Ordering::SeqCst);
        trace!("Forced is_processing to true for periodic monitoring");

    let self_clone = self.clone();
    tokio::spawn(async move {
      let mut consecutive_processing_failures = 0;

      while self_clone
        .is_processing
        .load(std::sync::atomic::Ordering::SeqCst)
      {
                
        // Smart sleep based on fail_count - prevent excessive retries
        {
          let max_fail_count = self_clone.file_repo.get_max_fail_count(self_clone.bot_id);

          let base_sleep = if max_fail_count >= 3 {
            3600
          } else if max_fail_count >= 2 {
            900
          } else if max_fail_count >= 1 {
            300
          } else {
            60
          };

          if base_sleep > 10 {
            debug!("Sleep {}s based on fail_count={}", base_sleep, max_fail_count);
          }

          tokio::time::sleep(Duration::from_secs(base_sleep as u64)).await;
        }

        // Skip drive health check - just proceed with monitoring
                if false {
                    let failures = self_clone
                        .consecutive_failures
                        .fetch_add(1, Ordering::Relaxed)
                        + 1;
                    if failures % 10 == 1 {
                        warn!("S3/MinIO unavailable for bucket {} (failures: {}), backing off to {:?}",
                              self_clone.bucket_name, failures, self_clone.calculate_backoff());
                    }
                    continue;
                }

        // Add timeout to prevent hanging
        match tokio::time::timeout(Duration::from_secs(12), self_clone.check_for_changes()).await {
                    Ok(Ok(_)) => {
                        let prev_failures =
                            self_clone.consecutive_failures.swap(0, Ordering::Relaxed);
                        consecutive_processing_failures = 0;
                        if prev_failures > 0 {
                            trace!("S3/MinIO recovered for bucket {} after {} failures",
                                  self_clone.bucket_name, prev_failures);
                        }
                    }
                    Ok(Err(e)) => {
                        self_clone
                            .consecutive_failures
                            .fetch_add(1, Ordering::Relaxed);
                        consecutive_processing_failures += 1;
                        error!("Error during sync for bot {}: {}", self_clone.bot_id, e);

                        // If too many consecutive failures, stop processing temporarily
                        if consecutive_processing_failures > 10 {
                            error!("Too many consecutive failures ({}), stopping processing for bot {}",
                                   consecutive_processing_failures, self_clone.bot_id);
                            self_clone.is_processing.store(false, std::sync::atomic::Ordering::SeqCst);
                            break;
                        }
                    }
                    Err(_) => {
                        error!("check_for_changes timed out for bot {}", self_clone.bot_id);
                        consecutive_processing_failures += 1;

                        if consecutive_processing_failures > 5 {
                            error!("Too many timeouts, stopping processing for bot {}", self_clone.bot_id);
                            self_clone.is_processing.store(false, std::sync::atomic::Ordering::SeqCst);
                            break;
                        }
                    }
                }
            }

            trace!("Monitoring loop ended for bot {}", self_clone.bot_id);
        });

        trace!("DriveMonitor started for bot {}", self.bot_id);
        Ok(())
    }

  pub async fn stop_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    self.is_processing
      .store(false, std::sync::atomic::Ordering::SeqCst);

    self.consecutive_failures.store(0, Ordering::Relaxed);

    Ok(())
  }
    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            trace!(
                "Drive Monitor service started for bucket: {}",
                self.bucket_name
            );
            loop {
                let backoff = self.calculate_backoff();
                tokio::time::sleep(backoff).await;

                if self.is_processing.load(Ordering::Acquire) {
                    log::warn!(
                        "Drive monitor is still processing previous changes, skipping this tick"
                    );
                    continue;
                }

                if !self.check_drive_health().await {
                    let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                    if failures % 10 == 1 {
                        warn!("S3/MinIO unavailable for bucket {} (failures: {}), backing off to {:?}",
                              self.bucket_name, failures, self.calculate_backoff());
                    }
                    continue;
                }

                self.is_processing.store(true, Ordering::Release);

                match self.check_for_changes().await {
                    Ok(_) => {
                        let prev_failures = self.consecutive_failures.swap(0, Ordering::Relaxed);
                        if prev_failures > 0 {
                            trace!("S3/MinIO recovered for bucket {} after {} failures",
                                  self.bucket_name, prev_failures);
                        }
                    }
                    Err(e) => {
                        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
                        log::error!("Error checking for drive changes: {}", e);
                    }
                }

                self.is_processing.store(false, Ordering::Release);
            }
        })
    }
    async fn check_for_changes(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("check_for_changes ENTER");
        let start_mem = MemoryStats::current();
        trace!(
            "check_for_changes START, RSS={}",
            MemoryStats::format_bytes(start_mem.rss_bytes)
        );

        let Some(client) = &self.state.drive else {
            warn!("No drive client available for bot {}, skipping file monitoring", self.bot_id);
            return Ok(());
        };

        trace!("check_for_changes: calling check_gbdialog_changes...");
        trace!("Checking gbdialog...");
        self.check_gbdialog_changes(client).await?;
        trace!("check_for_changes: check_gbdialog_changes done");
        let after_dialog = MemoryStats::current();
        trace!(
            "After gbdialog, RSS={} (delta={})",
            MemoryStats::format_bytes(after_dialog.rss_bytes),
            MemoryStats::format_bytes(after_dialog.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );

        trace!("check_for_changes: calling check_gbot...");
        trace!("Checking gbot...");
        self.check_gbot(client).await?;
        trace!("check_for_changes: check_gbot done");
        let after_gbot = MemoryStats::current();
        trace!(
            "After gbot, RSS={} (delta={})",
            MemoryStats::format_bytes(after_gbot.rss_bytes),
            MemoryStats::format_bytes(after_gbot.rss_bytes.saturating_sub(after_dialog.rss_bytes))
        );

        trace!("check_for_changes: calling check_gbkb_changes...");
        trace!("Checking gbkb...");
        self.check_gbkb_changes(client).await?;
        trace!("check_for_changes: check_gbkb_changes done");
        let after_gbkb = MemoryStats::current();
        trace!(
            "After gbkb, RSS={} (delta={})",
            MemoryStats::format_bytes(after_gbkb.rss_bytes),
            MemoryStats::format_bytes(after_gbkb.rss_bytes.saturating_sub(after_gbot.rss_bytes))
        );

        log_jemalloc_stats();

        let total_delta = after_gbkb.rss_bytes.saturating_sub(start_mem.rss_bytes);
        if total_delta > 50 * 1024 * 1024 {
            warn!(
                "check_for_changes grew by {} - potential leak!",
                MemoryStats::format_bytes(total_delta)
            );
        }

        trace!("check_for_changes EXIT");
        Ok(())
    }
    async fn check_gbdialog_changes(
        &self,
        client: &Client,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // No prefix filter - list all and filter by *.gbdialog pattern below
        let mut current_files = HashMap::new();
        let mut continuation_token = None;
        loop {
            let list_objects = match tokio::time::timeout(
                Duration::from_secs(30),
                client
                    .list_objects_v2()
                    .bucket(self.bucket_name.to_lowercase())
                    .set_continuation_token(continuation_token)
                    .send(),
            )
            .await
            {
                Ok(Ok(list)) => list,
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => {
                    log::error!("Timeout listing objects in bucket {}", self.bucket_name);
                    return Ok(());
                }
            };
            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();
                let path_parts: Vec<&str> = path.split('/').collect();
                // Filter for paths matching *.gbdialog/*.bas pattern
                if path_parts.len() < 2 || !path_parts[0].ends_with(".gbdialog") {
                    continue;
                }
                if path.ends_with('/') || !path.to_ascii_lowercase().ends_with(".bas") {
                    continue;
                }
                let etag = normalize_etag(obj.e_tag().unwrap_or_default());
                let last_modified = obj.last_modified().and_then(|dt| {
                    DateTime::parse_from_rfc3339(&dt.to_string()).ok().map(|d| d.with_timezone(&Utc))
                });
                current_files.insert(path, (etag, last_modified));
            }
            if !list_objects.is_truncated.unwrap_or(false) {
                break;
            }
            continuation_token = list_objects.next_continuation_token;
        }
    // First pass: identify which files need compilation
    let files_to_compile: Vec<String> = {
        current_files
            .iter()
            .filter_map(|(path, (current_etag, _))| {
                if let Some(prev) = self.file_repo.get_file_state(self.bot_id, path) {
                    if prev.etag.as_deref() != Some(current_etag.as_str()) {
                        Some(path.clone())
                    } else {
                        None
                    }
                } else {
                    Some(path.clone()) // New file
                }
            })
            .collect()
    };

    // Compile files that need it - compile_tool acquires its own write lock
    // Track successful compilations to preserve indexed status
    let mut successful_compilations: std::collections::HashSet<String> = std::collections::HashSet::new();
    for path in &files_to_compile {
        match self.compile_tool(client, path).await {
            Ok(_) => {
                successful_compilations.insert(path.clone());
            }
            Err(e) => {
                log::error!("Failed to compile tool {}: {}", path, e);
            }
        }
    }

    // Remove files that no longer exist (deleted from MinIO)
    let previous_files = self.file_repo.get_files_by_type(self.bot_id, "gbdialog");
    for prev_file in &previous_files {
        if !current_files.contains_key(&prev_file.file_path) {
            // Delete the compiled .ast file from disk
            let bot_name = self
                .bucket_name
                .strip_suffix(".gbai")
                .unwrap_or(&self.bucket_name);
            let ast_path = self.work_root
                .join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name))
                .join(PathBuf::from(&prev_file.file_path).file_name().unwrap_or_default().to_str().unwrap_or(""))
                .with_extension("ast");
            
            if ast_path.exists() {
                if let Err(e) = std::fs::remove_file(&ast_path) {
                    warn!("Failed to delete orphaned .ast file {}: {}", ast_path.display(), e);
                } else {
                    info!("Deleted orphaned .ast file: {}", ast_path.display());
                }
            }
            
            // Also delete .bas, .mcp.json, .tool.json files
            let bas_path = ast_path.with_extension("bas");
            let mcp_path = ast_path.with_extension("mcp.json");
            let tool_path = ast_path.with_extension("tool.json");
            
            for file_to_delete in [&bas_path, &mcp_path, &tool_path] {
                if file_to_delete.exists() {
                    let _ = std::fs::remove_file(file_to_delete);
                }
            }
            
            if let Err(e) = self.file_repo.delete_file(self.bot_id, &prev_file.file_path) {
                warn!("Failed to delete file state for {}: {}", prev_file.file_path, e);
            }
        }
    }

    // Merge current_files into DB via file_repo
    // For each file in current_files:
    // - If compilation succeeded: set indexed=true
    // - If compilation failed: preserve previous indexed status, increment fail_count
    // - If unchanged: preserve existing state (including indexed status)
    // - If new and not compiled: add with default state (indexed=false)
    for (path, (etag, last_modified)) in &current_files {
        if successful_compilations.contains(path) {
            // Compilation succeeded - mark as indexed
            if let Err(e) = self.file_repo.upsert_file_full(
                self.bot_id, path, "gbdialog",
                Some(etag.clone()), *last_modified,
                true, 0, None,
            ) {
                warn!("Failed to upsert file {}: {}", path, e);
            }
        } else if let Some(prev) = self.file_repo.get_file_state(self.bot_id, path) {
            let etag_unchanged = prev.etag.as_deref() == Some(etag.as_str());
            if etag_unchanged {
                // File unchanged - preserve all previous state (already in DB)
            } else {
                // File changed but compilation failed - increment fail_count
                if let Err(e) = self.file_repo.upsert_file_full(
                    self.bot_id, path, "gbdialog",
                    Some(etag.clone()), *last_modified,
                    prev.indexed, prev.fail_count + 1, Some(Utc::now()),
                ) {
                    warn!("Failed to upsert file {}: {}", path, e);
                }
            }
        } else {
            // New file where compilation failed: indexed=false
            if let Err(e) = self.file_repo.upsert_file_full(
                self.bot_id, path, "gbdialog",
                Some(etag.clone()), *last_modified,
                false, 0, None,
            ) {
                warn!("Failed to upsert file {}: {}", path, e);
            }
        }
    }
        Ok(())
    }
    async fn check_gbot(&self, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("check_gbot ENTER");
        let config_manager = ConfigManager::new(self.state.conn.clone());
        let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);
        let gbot_prefix = format!("{}.gbot/", bot_name);
        debug!(
            "check_gbot: Checking bucket {} for config.csv changes (prefix: {})",
            self.bucket_name, gbot_prefix
        );
        let mut continuation_token = None;
        loop {
            let list_objects = match tokio::time::timeout(
                Duration::from_secs(30),
                client
                    .list_objects_v2()
                    .bucket(self.bucket_name.to_lowercase())
                    .prefix(&gbot_prefix)
                    .set_continuation_token(continuation_token)
                    .send(),
            )
            .await
            {
                Ok(Ok(list)) => list,
                Ok(Err(e)) => {
                    error!(
                        "check_gbot: Failed to list objects in bucket {}: {}",
                        self.bucket_name, e
                    );
                    return Err(e.into());
                }
                Err(_) => {
                    error!("Timeout listing objects in bucket {}", self.bucket_name);
                    return Ok(());
                }
            };
            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();
                let path_lower = path.to_ascii_lowercase();

                let is_config_csv = path_lower.ends_with("/config.csv")
                    || path_lower == "config.csv";

                let is_prompt_file = path_lower.ends_with("prompt.md")
                    || path_lower.ends_with("prompt.txt");

                debug!("check_gbot: Checking path: {} (is_config_csv: {}, is_prompt: {})", path, is_config_csv, is_prompt_file);

                if !is_config_csv && !is_prompt_file {
                    continue;
                }

                if is_prompt_file {
                    // Check etag to avoid re-downloading unchanged prompt files
                    let etag = normalize_etag(obj.e_tag().unwrap_or_default());
                    let prompt_state_key = format!("__prompt__{}", path);
                    let should_download = match self.file_repo.get_file_state(self.bot_id, &prompt_state_key) {
                        Some(prev) => prev.etag.as_deref() != Some(&etag),
                        None => true,
                    };
                    if should_download {
                        match client.get_object().bucket(&self.bucket_name).key(&path).send().await {
                            Ok(response) => {
                                let bytes = response.body.collect().await?.into_bytes();
                                let content = String::from_utf8(bytes.to_vec())
                                    .map_err(|e| format!("UTF-8 error in {}: {}", path, e))?;
                                let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);
                                let gbot_dir = self.work_root.join(format!("{}.gbai/{}.gbot", bot_name, bot_name));
                                let path_buf = PathBuf::from(&path);
                                let file_name = path_buf.file_name()
                                    .and_then(|n| n.to_str()).unwrap_or("PROMPT.md");
                                if let Err(e) = tokio::task::spawn_blocking({
                                    let gbot_dir_str = gbot_dir.to_string_lossy().to_string();
                                    let file_name_owned = file_name.to_string();
                                    let content_owned = content.clone();
                                    move || {
                                        std::fs::create_dir_all(&gbot_dir_str)?;
                                        std::fs::write(format!("{}/{}", gbot_dir_str, file_name_owned), &content_owned)?;
                                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                                    }
                                }).await {
                                    log::error!("Failed to save prompt file: {}", e);
                                } else {
                                    log::info!("Downloaded prompt file {} to work directory", path);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to download prompt file {}: {}", path, e);
                            }
                        }
                        if let Err(e) = self.file_repo.upsert_file_full(
                            self.bot_id, &prompt_state_key, "gbot-prompt",
                            Some(etag), None, false, 0, None,
                        ) {
                            warn!("Failed to save prompt file state: {}", e);
                        }
                    } else {
                        trace!("Prompt file {} unchanged (etag match), skipping download", path);
                    }
                    continue;
                }

                debug!("check_gbot: Found config.csv at path: {}", path);
                let etag = normalize_etag(obj.e_tag().unwrap_or_default());
                let last_modified = obj.last_modified().map(|dt| dt.to_string());
                let config_state_key = format!("__config__{}", path);
                let should_sync = match self.file_repo.get_file_state(self.bot_id, &config_state_key) {
                    Some(prev) => {
                        let etag_changed = prev.etag.as_deref() != Some(&etag);
                        let mod_changed = match (&prev.last_modified, &last_modified) {
                            (Some(prev_dt), Some(new_dt)) => prev_dt.to_string() != new_dt.to_string(),
                            (None, Some(_)) => true,
                            _ => false,
                        };
                        etag_changed || mod_changed
                    }
                    None => true,
                };
                debug!("check_gbot: config.csv should_sync={} (etag={}, last_modified={:?})", should_sync, etag, last_modified);
                if should_sync {
                    match client
                        .head_object()
                        .bucket(&self.bucket_name)
                        .key(&path)
                        .send()
                        .await
                    {
                        Ok(_head_res) => {
                            let response = client
                                .get_object()
                                .bucket(&self.bucket_name)
                                .key(&path)
                                .send()
                                .await?;
                        let bytes = response.body.collect().await?.into_bytes();
                        let csv_content = String::from_utf8(bytes.to_vec())
                            .map_err(|e| format!("UTF-8 error in {}: {}", path, e))?;
                        let llm_lines: Vec<_> = csv_content
                            .lines()
                            .filter(|line| line.trim_start().starts_with("llm-"))
                            .collect();
                        if llm_lines.is_empty() {
                            let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);
                        } else {
                            #[cfg(feature = "llm")]
                            {
                                use crate::llm::local::ensure_llama_servers_running;
                                let mut restart_needed = false;
                                let mut llm_url_changed = false;
                                let mut new_llm_url = String::new();
                                let mut new_llm_model = String::new();
                                for line in &llm_lines {
                                    let parts: Vec<&str> = line.split(',').collect();
                                    if parts.len() >= 2 {
                                        let key = parts[0].trim();
                                        let new_value = parts[1].trim();
                                        if key == "llm-url" {
                                            new_llm_url = new_value.to_string();
                                        }
                                        if key == "llm-model" {
                                            new_llm_model = new_value.to_string();
                                        }
                                        let normalized_old_value = match config_manager.get_config(&self.bot_id, key, None) {
                                            Ok(val) => Self::normalize_config_value(&val),
                                            Err(_) => String::new(),
                                        };
                                        let normalized_new_value = Self::normalize_config_value(new_value);

                                        if normalized_old_value != normalized_new_value {
                                            trace!(
                                                "Detected change in {} (old: {}, new: {})",
                                                key, normalized_old_value, normalized_new_value
                                            );
                                            restart_needed = true;
                                            if key == "llm-url" || key == "llm-model" {
                                                llm_url_changed = true;
                                            }
                                        }
                                    }
                                }

                                let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);

                                if restart_needed {
                                    if let Err(e) =
                                        ensure_llama_servers_running(Arc::clone(&self.state)).await
                                    {
                                        warn!("Refreshed LLM servers but with errors: {}", e);
                                    }

                                    if llm_url_changed {
                                        trace!("Broadcasting LLM configuration refresh");
                                        let effective_url = if !new_llm_url.is_empty() {
                                            new_llm_url
                                        } else {
                                            config_manager
                                                .get_config(&self.bot_id, "llm-url", None)
                                                .unwrap_or_default()
                                        };
                                        let effective_model = if !new_llm_model.is_empty() {
                                            new_llm_model
                                        } else {
                                            config_manager
                                                .get_config(&self.bot_id, "llm-model", None)
                                                .unwrap_or_default()
                                        };

                                        trace!(
                                            "LLM configuration changed to: URL={}, Model={}",
                                            effective_url, effective_model
                                        );

                                        // Read the llm-endpoint-path config
                                        let effective_endpoint_path = config_manager
                                            .get_config(
                                                &self.bot_id,
                                                "llm-endpoint-path",
                                                Some("/v1/chat/completions"),
                                            )
                                            .unwrap_or_else(|_| "/v1/chat/completions".to_string());

                                        // Update the DynamicLLMProvider with the new configuration
                                        #[cfg(feature = "llm")]
                                        if let Some(dynamic_llm) = &self.state.dynamic_llm_provider
                                        {
                                            dynamic_llm
                                                .update_from_config(
                                                    &effective_url,
                                                    Some(effective_model),
                                                    Some(effective_endpoint_path),
                                                    None,
                                                )
                                                .await;
                                            trace!("Dynamic LLM provider updated with new configuration");
                                        } else {
                                            warn!("Dynamic LLM provider not available - config change ignored");
                                        }
                                    }
                                }
                            }

                            #[cfg(not(feature = "llm"))]
                            {
                                let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);
                            }
                        }
                        if csv_content.lines().any(|line| line.starts_with("theme-")) {
                            self.broadcast_theme_change(&csv_content).await?;
                        }

                        // Update file state in DB for config.csv
                        let last_mod_dt = last_modified.as_ref().and_then(|s| {
                            DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                        });
                        if let Err(e) = self.file_repo.upsert_file_full(
                            self.bot_id, &config_state_key, "gbot-config",
                            Some(etag), last_mod_dt, false, 0, None,
                        ) {
                            warn!("Failed to save config file state: {}", e);
                        }

                        // Check for system-prompt-file and download it
                        let prompt_file_line = csv_content
                            .lines()
                            .find(|l| l.trim().starts_with("system-prompt-file,"));
                        if let Some(line) = prompt_file_line {
                            let parts: Vec<&str> = line.split(',').collect();
                            if parts.len() >= 2 {
                                let prompt_filename = parts[1].trim();
                                if !prompt_filename.is_empty() {
                                    let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);
                                    let prompt_key = format!("{}.gbot/{}", bot_name, prompt_filename);
                                    if let Ok(prompt_response) = client
                                        .get_object()
                                        .bucket(&self.bucket_name)
                                        .key(&prompt_key)
                                        .send()
                                        .await
                                    {
                                        let prompt_bytes = prompt_response.body.collect().await?.into_bytes();
                let prompt_content = String::from_utf8(prompt_bytes.to_vec())
                    .map_err(|_e| format!("UTF-8 error in {}", prompt_filename))?;

                // Save to work directory
                let gbot_dir = self.work_root.join(format!("{}.gbai/{}.gbot", bot_name, bot_name));

                if let Err(e) = tokio::task::spawn_blocking({
                                            let gbot_dir_str = gbot_dir.to_string_lossy().to_string();
                                            let prompt_filename_owned = prompt_filename.to_string();
                                            move || {
                                                std::fs::create_dir_all(&gbot_dir_str)?;
                                                std::fs::write(format!("{}/{}", gbot_dir_str, prompt_filename_owned), &prompt_content)?;
                                                Ok::<(), Box<dyn Error + Send + Sync>>(())
                                            }
                                        }).await {
                                            log::error!("Failed to save prompt file: {}", e);
                                        } else {
                                            log::info!("Downloaded {} to work directory", prompt_filename);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Config file {} not found or inaccessible: {}", path, e);
                    }
                }
                }
            }
            if !list_objects.is_truncated.unwrap_or(false) {
                break;
            }
            continuation_token = list_objects.next_continuation_token;
        }
        trace!("check_gbot EXIT");
        Ok(())
    }
  async fn broadcast_theme_change(
    &self,
    csv_content: &str,
  ) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut theme_data = serde_json::json!({
      "event": "change_theme",
      "data": {}
    });
    for line in csv_content.lines() {
      let parts: Vec<&str> = line.split(',').collect();
      if parts.len() >= 2 {
        let key = parts[0].trim();
        let value = parts[1].trim();
        match key {
          "theme-color1" => {
            theme_data["data"]["color1"] = serde_json::Value::String(value.to_string());
          }
          "theme-color2" => {
            theme_data["data"]["color2"] = serde_json::Value::String(value.to_string());
          }
          "theme-logo" => {
            theme_data["data"]["logo_url"] =
              serde_json::Value::String(value.to_string());
          }
          "theme-title" => {
            theme_data["data"]["title"] = serde_json::Value::String(value.to_string());
          }
          "theme-logo-text" => {
            theme_data["data"]["logo_text"] =
              serde_json::Value::String(value.to_string());
          }
          _ => {}
        }
      }
    }
    // Clone channels to avoid holding lock while sending
    let channels: Vec<_> = {
      let response_channels = self.state.response_channels.lock().await;
      response_channels.iter().map(|(id, tx)| (id.clone(), tx.clone())).collect()
    };
    for (session_id, tx) in channels {
      let theme_response = crate::core::shared::models::BotResponse {
        bot_id: self.bot_id.to_string(),
        user_id: "system".to_string(),
        session_id,
        channel: "web".to_string(),
        content: serde_json::to_string(&theme_data)?,
        message_type: MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
      };
      let _ = tx.try_send(theme_response);
    }
    Ok(())
  }
    async fn compile_tool(
        &self,
        client: &Client,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!(
            "Fetching object from Drive: bucket={}, key={}",
            &self.bucket_name, file_path
        );
        let response = match client
            .get_object()
            .bucket(&self.bucket_name)
            .key(file_path)
            .send()
            .await
        {
            Ok(res) => {
                trace!(
                    "Successfully fetched object from Drive: bucket={}, key={}, size={}",
                    &self.bucket_name,
                    file_path,
                    res.content_length().unwrap_or(0)
                );
                res
            }
            Err(e) => {
                log::error!(
                    "Failed to fetch object from Drive: bucket={}, key={}, error={:?}",
                    &self.bucket_name,
                    file_path,
                    e
                );
                return Err(e.into());
            }
        };
        let bytes = response.body.collect().await?.into_bytes();
        let source_content = String::from_utf8(bytes.to_vec())?;
        let tool_name = file_path
            .rsplit('/')
            .next()
            .unwrap_or(file_path)
            .strip_suffix(".bas")
            .unwrap_or(file_path)
            .to_string();
        let bot_name = self
            .bucket_name
            .strip_suffix(".gbai")
            .unwrap_or(&self.bucket_name);
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        let work_dir_str = work_dir.to_string_lossy().to_string();
        info!("Compiling tool '{}' to work_dir: {}", tool_name, work_dir_str);
        let state_clone = Arc::clone(&self.state);
        let work_dir_clone = work_dir_str.clone();
        let tool_name_clone = tool_name.clone();
        let source_content_clone = source_content.clone();
        let bot_id = self.bot_id;
        let elapsed_ms = tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&work_dir_clone)?;
            let local_source_path = format!("{}/{}.bas", work_dir_clone, tool_name_clone);
            std::fs::write(&local_source_path, &source_content_clone)?;
            let mut compiler = BasicCompiler::new(state_clone, bot_id);
            let start_time = std::time::Instant::now();
            let result = compiler.compile_file(&local_source_path, &work_dir_str)?;
            let elapsed = start_time.elapsed().as_millis();
            if let Some(mcp_tool) = result.mcp_tool {
                trace!(
                    "MCP tool definition generated with {} parameters",
                    mcp_tool.input_schema.properties.len()
                );
            }
            Ok::<u128, Box<dyn Error + Send + Sync>>(elapsed)
        })
        .await??;

    info!("Successfully compiled {} in {} ms", tool_name, elapsed_ms);
    // Note: indexed status is set by check_gbdialog_changes based on compile_tool result

        // Check for USE WEBSITE commands and trigger immediate crawling
        if source_content.contains("USE WEBSITE") {
            self.trigger_immediate_website_crawl(&source_content).await?;
        }

        Ok(())
    }

    async fn trigger_immediate_website_crawl(
        &self,
        source_content: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use regex::Regex;
        use std::collections::HashSet;
        use diesel::prelude::*;

        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let re = Regex::new(r#"USE\s+WEBSITE\s+"([^"]+)"(?:\s+REFRESH\s+"([^"]+)")?"#)?;
        let mut processed_urls = HashSet::new();

        for cap in re.captures_iter(source_content) {
            if let Some(url) = cap.get(1) {
                let url_str = url.as_str();

                // Prevent duplicate processing of same URL in single batch
                if processed_urls.contains(url_str) {
                    trace!("Skipping duplicate URL in batch: {}", url_str);
                    continue;
                }
                processed_urls.insert(url_str.to_string());

                let refresh_str = cap.get(2).map(|m| m.as_str()).unwrap_or("1m");

                trace!("Found USE WEBSITE command for {}, checking if crawl needed", url_str);

                // Check if crawl is already in progress or recently completed
                let mut conn = self.state.conn.get()
                    .map_err(|e| format!("Failed to get database connection: {}", e))?;

                // Check if crawl is already running or recently completed (within last 5 minutes)
                let recent_crawl: Result<i64, _> = diesel::sql_query(
                    "SELECT COUNT(*) as count FROM website_crawls
                     WHERE bot_id = $1 AND url = $2
                     AND (crawl_status = 2 OR (last_crawled > NOW() - INTERVAL '5 minutes'))"
                )
                .bind::<diesel::sql_types::Uuid, _>(&self.bot_id)
                .bind::<diesel::sql_types::Text, _>(url_str)
                .get_result::<CountResult>(&mut conn)
                .map(|r| r.count);

                if recent_crawl.unwrap_or(0) > 0 {
                    trace!("Skipping crawl for {} - already in progress or recently completed", url_str);
                    continue;
                }

                crate::basic::keywords::use_website::register_website_for_crawling_with_refresh(
                    &mut conn, &self.bot_id, url_str, refresh_str
                )?;

                // Use a semaphore to limit concurrent crawls
                static CRAWL_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1); // Reduced to 1

                let kb_manager = self.state.kb_manager.clone();
                let db_pool = self.state.conn.clone();
                let bot_id = self.bot_id;
                let url_owned = url_str.to_string();

                // Don't spawn if semaphore is full
                if let Ok(_permit) = CRAWL_SEMAPHORE.try_acquire() {
                    tokio::spawn(async move {
                        if let Err(e) = Self::crawl_website_immediately(url_owned, bot_id, kb_manager, db_pool).await {
                            error!("Failed to immediately crawl website: {}", e);
                        }
                        // Permit is automatically dropped here
                    });
                } else {
                    warn!("Crawl semaphore full, skipping immediate crawl for {}", url_str);
                }
            }
        }

        Ok(())
    }

    async fn crawl_website_immediately(
        url: String,
        _bot_id: uuid::Uuid,
        _kb_manager: Option<Arc<crate::core::kb::KnowledgeBaseManager>>,
        _db_pool: crate::core::shared::DbPool,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        #[cfg(feature = "crawler")]
        {
            use crate::core::kb::website_crawler_service::WebsiteCrawlerService;
            use diesel::prelude::*;

            if _kb_manager.is_none() {
                warn!("Knowledge base manager not available, skipping website crawl");
                return Ok(());
            }

            let mut conn = _db_pool.get()?;

            // Get the website record
            #[derive(diesel::QueryableByName)]
            struct WebsiteRecord {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: uuid::Uuid,
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                bot_id: uuid::Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                url: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                expires_policy: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                refresh_policy: String,
                #[diesel(sql_type = diesel::sql_types::Integer)]
                max_depth: i32,
                #[diesel(sql_type = diesel::sql_types::Integer)]
                max_pages: i32,
            }

            let website: WebsiteRecord = diesel::sql_query(
                "SELECT id, bot_id, url, expires_policy, refresh_policy, max_depth, max_pages
                 FROM website_crawls
                 WHERE bot_id = $1 AND url = $2"
            )
            .bind::<diesel::sql_types::Uuid, _>(&_bot_id)
            .bind::<diesel::sql_types::Text, _>(&url)
            .get_result(&mut conn)?;

            // Convert to WebsiteCrawlRecord format expected by crawl_website
            let website_record = crate::core::kb::website_crawler_service::WebsiteCrawlRecord {
                id: website.id,
                bot_id: website.bot_id,
                url: website.url,
                expires_policy: website.expires_policy,
                refresh_policy: Some(website.refresh_policy),
                max_depth: website.max_depth,
                max_pages: website.max_pages,
                next_crawl: None,
                crawl_status: Some(0),
            };

            // Create a temporary crawler service to use its crawl_website method
            let crawler_service = WebsiteCrawlerService::new(_db_pool.clone());
            match crawler_service.crawl_single_website(website_record).await {
                Ok(_) => {},
                Err(e) => return Err(format!("Website crawl failed: {}", e).into()),
            }
        }
        #[cfg(not(feature = "crawler"))]
        {
            warn!("Crawler feature not enabled, skipping website crawl for {}", url);
        }

        Ok(())
    }

    async fn check_gbkb_changes(
        &self,
        client: &Client,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Prevent concurrent scans - if already scanning, skip this tick
        if self
            .scanning
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            trace!("Scan already in progress for bot {}, skipping", self.bot_id);
            return Ok(());
        }
        
    let bot_name = self
      .bucket_name
      .strip_suffix(".gbai")
      .unwrap_or(&self.bucket_name);

    let gbkb_prefix = format!("{}.gbkb/", bot_name);
    let mut current_files = HashMap::new();
    let mut continuation_token = None;

    let mut files_processed = 0;
    let mut files_to_process = Vec::new();
    let mut pdf_files_found = 0;

    loop {
      let list_objects = match tokio::time::timeout(
        Duration::from_secs(30),
        client
          .list_objects_v2()
          .bucket(self.bucket_name.to_lowercase())
          .prefix(&gbkb_prefix)
          .set_continuation_token(continuation_token)
          .send(),
      )
      .await
      {
        Ok(Ok(list)) => list,
        Ok(Err(e)) => {
          debug!("Error listing objects: {}", e);
          return Err(e.into());
        }
        Err(_) => {
          log::error!(
            "Timeout listing .gbkb objects in bucket {}",
            self.bucket_name
          );
          return Ok(());
        }
      };

            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();

                if path.ends_with('/') {
                    continue;
                }

                let size = obj.size().unwrap_or(0);
                if size == 0 {
                    trace!("Skipping 0-byte file in .gbkb: {}", path);
                    continue;
                }

                let etag = normalize_etag(obj.e_tag().unwrap_or_default());
                let last_modified = obj.last_modified().and_then(|dt| {
                    DateTime::parse_from_rfc3339(&dt.to_string()).ok().map(|d| d.with_timezone(&Utc))
                });
                current_files.insert(path.clone(), (etag, last_modified));
            }

            if !list_objects.is_truncated.unwrap_or(false) {
                break;
            }
            continuation_token = list_objects.next_continuation_token;
        }

    

        // Check if ALL KBs for this bot are already indexed in Qdrant
        // If so, only scan for NEW files - skip re-indexing existing ones
        let mut kb_folders: HashSet<String> = HashSet::new();
        for (path, _) in current_files.iter() {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 3 && parts[0].ends_with(".gbkb") {
                kb_folders.insert(parts[1].to_string());
            }
        }

        let mut all_indexed = true;
        for kb_name in &kb_folders {
            let kb_key = format!("{}_{}", bot_name, kb_name);
            let indexed = {
                let indexed_folders = self.kb_indexed_folders.read().await;
                indexed_folders.contains(&kb_key)
            };
            if !indexed {
                all_indexed = false;
                break;
            }
        }

        // Build set of already-indexed KB folder names for quick lookup
        let indexed_kb_names: HashSet<String> = {
            let indexed = self.kb_indexed_folders.read().await;
            kb_folders.iter()
                .filter(|kb| indexed.contains(&format!("{}_{}", bot_name, kb)))
                .cloned()
                .collect()
        };

    for (path, (_, current_last_modified)) in current_files.iter() {
      let prev_state = self.file_repo.get_file_state(self.bot_id, path);
      let is_new = prev_state.is_none();

            // Skip files from already-indexed KB folders that are not new
            // This prevents re-download loop when DB is loaded fresh
            let kb_name_from_path = path.split('/').nth(1).map(|s| s.to_string());
            if all_indexed && !is_new {
                trace!("Skipping already indexed file: {}", path);
                continue;
            }
            // Extra safety: if the KB folder is indexed, skip non-new files
            if all_indexed {
                if let Some(kb) = &kb_name_from_path {
                    if indexed_kb_names.contains(kb) {
                        trace!("Skipping file from indexed KB: {}", path);
                        continue;
                    }
                }
            }

      // Use only last_modified for change detection - more reliable than ETag
      let is_modified = if let Some(prev) = &prev_state {
        prev.last_modified != *current_last_modified
      } else {
        false
      };

      if is_new || is_modified {
        #[cfg(any(feature = "research", feature = "llm"))]
        {
          // Only remove from indexed_folders if KB is actually being re-indexed
          let path_parts: Vec<&str> = path.split('/').collect();
      if path_parts.len() >= 2 {
        let kb_name = path_parts[1];
        let kb_key = format!("{}_{}", bot_name, kb_name);

        // Check and remove in one atomic operation
        let should_remove = {
          let indexed_folders = self.kb_indexed_folders.read().await;
          indexed_folders.contains(&kb_key)
        };

        // Only remove if NOT already indexed
        if !should_remove {
          let mut indexed_folders = self.kb_indexed_folders.write().await;
          indexed_folders.remove(&kb_key);
        }
      }
        }
        if let Some(prev) = &prev_state {
          if prev.fail_count >= MAX_FAIL_COUNT {
            let elapsed = Utc::now()
              .signed_duration_since(prev.last_failed_at.unwrap_or(Utc::now()));
            if elapsed.num_seconds() < RETRY_BACKOFF_SECS {
              continue;
            }
          }
        }

        if path.to_lowercase().ends_with(".pdf") {
          pdf_files_found += 1;
        }

        files_to_process.push(path.clone());
        files_processed += 1;

                // REMOVED: Skip downloads if LLM is actively streaming - was causing deadlocks
                // #[cfg(any(feature = "research", feature = "llm"))]
                // if is_llm_streaming() {
                //     debug!("Skipping download - LLM is streaming, will retry later");
                //     files_to_process.clear();
                //     break;
                // }

        if files_to_process.len() >= 10 {
          for file_path in std::mem::take(&mut files_to_process) {
            if let Err(e) = self.download_gbkb_file(client, &file_path).await {
              log::error!("Failed to download .gbkb file {}: {}", file_path, e);
            }
          }
          tokio::time::sleep(Duration::from_millis(100)).await;
        }

                // Queue KB folder for indexing - only if not already indexed and no files changed
                let path_parts: Vec<&str> = path.split('/').collect();
                if path_parts.len() >= 3 {
                    let kb_name = path_parts[1];
                    let kb_key = format!("{}_{}", bot_name, kb_name);

                    #[cfg(any(feature = "research", feature = "llm"))]
                    {
                        let indexing_set = self.files_being_indexed.read().await;
                        let already_indexing = indexing_set.contains(&kb_key);
                        drop(indexing_set);

                        if !already_indexing {
                            let already_indexed = {
                                let indexed_folders = self.kb_indexed_folders.read().await;
                                indexed_folders.contains(&kb_key)
                            };

        if !already_indexed {
              let mut pending = self.pending_kb_index.write().await;
              pending.insert(kb_key.clone());
            }
                        }
                    }

                    #[cfg(not(any(feature = "research", feature = "llm")))]
                    {
                        let _ = kb_name;
                        debug!("KB indexing disabled (research/llm features not enabled)");
                    }
                }
            }
        }

        // Download remaining files (less than 10)
        if !files_to_process.is_empty() {
    for file_path in std::mem::take(&mut files_to_process) {
      if let Err(e) = self.download_gbkb_file(client, &file_path).await {
        log::error!("Failed to download .gbkb file {}: {}", file_path, e);
      }
    }
    }

        // Find files deleted from MinIO
        let previous_gbkb = self.file_repo.get_files_by_prefix(self.bot_id, &gbkb_prefix);
        let paths_to_remove: Vec<String> = previous_gbkb.iter()
            .filter(|f| !current_files.contains_key(&f.file_path))
            .map(|f| f.file_path.clone())
            .collect();

        if files_processed > 0 {
            trace!(
                "Processed {} .gbkb files (including {} PDFs for text extraction)",
                files_processed, pdf_files_found
            );
        }
        // Persist each current file to the DB, preserving state when unchanged
        for (path, (etag, last_modified)) in &current_files {
            if let Some(previous) = self.file_repo.get_file_state(self.bot_id, path) {
                let content_unchanged = previous.last_modified == *last_modified
                    || (previous.etag.as_deref() == Some(etag.as_str())
                        && previous.last_modified.is_some()
                        && last_modified.is_some());

                if content_unchanged {
                    // Unchanged - leave existing DB row as-is
                    continue;
                }
            }
            // New or changed file — upsert with default state (indexed=false)
            if let Err(e) = self.file_repo.upsert_file_full(
                self.bot_id, path, "gbkb",
                Some(etag.clone()), *last_modified,
                false, 0, None,
            ) {
                warn!("Failed to upsert gbkb file {}: {}", path, e);
            }
        }

        for path in paths_to_remove {
            trace!("Detected deletion in .gbkb: {}", path);
            if let Err(e) = self.file_repo.delete_file(self.bot_id, &path) {
                warn!("Failed to delete gbkb file state {}: {}", path, e);
            }

        // Delete the downloaded file from disk
        let bot_name = self
            .bucket_name
            .strip_suffix(".gbai")
            .unwrap_or(&self.bucket_name);
        let local_path = self.work_root.join(bot_name).join(&path);

        if local_path.exists() {
            if let Err(e) = std::fs::remove_file(&local_path) {
                warn!("Failed to delete orphaned .gbkb file {}: {}", local_path.display(), e);
            } else {
                info!("Deleted orphaned .gbkb file: {}", local_path.display());
            }
        }

        // Delete vectors for this specific file from Qdrant
        let path_parts: Vec<&str> = path.split('/').collect();
        if path_parts.len() >= 2 {
            let kb_name = path_parts[1];

            #[cfg(any(feature = "research", feature = "llm"))]
            {
                if let Err(e) = self.kb_manager.delete_file_from_kb(self.bot_id, bot_name, kb_name, &path).await {
                    log::error!("Failed to delete vectors for file {}: {}", path, e);
                }
            }
            #[cfg(not(any(feature = "research", feature = "llm")))]
            {
                let _ = (bot_name, kb_name);
                debug!("Bypassing vector delete because research/llm features are not enabled");
            }

            let kb_prefix = format!("{}{}/", gbkb_prefix, kb_name);
            if !self.file_repo.has_files_with_prefix(self.bot_id, &kb_prefix) {
                #[cfg(any(feature = "research", feature = "llm"))]
                {
                    if let Err(e) = self.kb_manager.clear_kb(self.bot_id, bot_name, kb_name).await {
                        log::error!("Failed to clear KB {}: {}", kb_name, e);
                    }
                    let mut indexed_folders = self.kb_indexed_folders.write().await;
                    let kb_key = format!("{}_{}", bot_name, kb_name);
                    indexed_folders.remove(&kb_key);
                }

                // Remove the empty KB folder from disk
                let kb_folder = self.work_root.join(bot_name).join(&kb_prefix);
                if kb_folder.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&kb_folder) {
                        warn!("Failed to remove KB folder {}: {}", kb_folder.display(), e);
                    } else {
                        info!("Removed empty KB folder: {}", kb_folder.display());
                    }
                }

                #[cfg(not(any(feature = "research", feature = "llm")))]
                {
                    let _ = (bot_name, kb_name);
                    debug!("Bypassing KB clear because research/llm features are not enabled");
                }
            }
        }
    }

    self.scanning.store(false, Ordering::Release);
    Ok(())
  }

    async fn download_gbkb_file(
        &self,
        client: &Client,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bot_name = self
            .bucket_name
            .strip_suffix(".gbai")
            .unwrap_or(&self.bucket_name);

        let local_path = self.work_root.join(bot_name).join(file_path);

        if file_path.to_lowercase().ends_with(".pdf") {
            debug!("Downloading PDF file for text extraction: {}", file_path);
        }

        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let response = client
            .get_object()
            .bucket(&self.bucket_name)
            .key(file_path)
            .send()
            .await?;

        let bytes = response.body.collect().await?.into_bytes();
        tokio::fs::write(&local_path, bytes).await?;

        trace!(
            "Downloaded .gbkb file {} to {}",
            file_path,
            local_path.display()
        );

        Ok(())
    }
}
