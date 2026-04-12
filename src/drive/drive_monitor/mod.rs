use crate::basic::compiler::BasicCompiler;
use crate::core::config::ConfigManager;
#[cfg(any(feature = "research", feature = "llm"))]
use crate::core::kb::embedding_generator::is_embedding_server_ready;
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
use serde::{Deserialize, Serialize};
use tokio::fs as tokio_fs;

const MAX_BACKOFF_SECS: u64 = 300;
const INITIAL_BACKOFF_SECS: u64 = 30;
const RETRY_BACKOFF_SECS: i64 = 3600;
const MAX_FAIL_COUNT: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub etag: String,
    #[serde(default)]
    pub indexed: bool,
    #[serde(default)]
    pub last_failed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub fail_count: u32,
}

#[derive(Debug, Clone)]
pub struct DriveMonitor {
    state: Arc<AppState>,
    bucket_name: String,
    file_states: Arc<tokio::sync::RwLock<HashMap<String, FileState>>>,
    bot_id: uuid::Uuid,
    #[cfg(any(feature = "research", feature = "llm"))]
    kb_manager: Arc<KnowledgeBaseManager>,
    work_root: PathBuf,
    is_processing: Arc<AtomicBool>,
    consecutive_failures: Arc<AtomicU32>,
    #[cfg(any(feature = "research", feature = "llm"))]
    files_being_indexed: Arc<TokioRwLock<HashSet<String>>>,
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

        Self {
            state,
            bucket_name,
            file_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            bot_id,
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_manager,
            work_root,
            is_processing: Arc::new(AtomicBool::new(false)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            #[cfg(any(feature = "research", feature = "llm"))]
            files_being_indexed: Arc::new(TokioRwLock::new(HashSet::new())),
        }
    }

    /// Get the path to the file states JSON file for this bot
    fn file_state_path(&self) -> PathBuf {
        self.work_root
            .join(&self.bucket_name)
            .join("file_states.json")
    }

    /// Load file states from disk to avoid reprocessing unchanged files
    async fn load_file_states(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let path = self.file_state_path();
        if path.exists() {
            match tokio_fs::read_to_string(&path).await {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, FileState>>(&content) {
                        Ok(states) => {
                            let mut file_states = self.file_states.write().await;
                            let count = states.len();
                            *file_states = states;
                            trace!(
                                "[DRIVE_MONITOR] Loaded {} file states from disk for bot {}",
                                count,
                                self.bot_id
                            );
                        }
                        Err(e) => {
                            warn!(
                                "[DRIVE_MONITOR] Failed to parse file states from {}: {}. Starting with empty state.",
                                path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[DRIVE_MONITOR] Failed to read file states from {}: {}. Starting with empty state.",
                        path.display(),
                        e
                    );
                }
            }
        } else {
            debug!(
                "[DRIVE_MONITOR] No existing file states found at {} for bot {}. Starting fresh.",
                path.display(),
                self.bot_id
            );
        }
        Ok(())
    }

    /// Static helper to save file states (used by background tasks)
    async fn save_file_states_static(
        file_states: &Arc<tokio::sync::RwLock<HashMap<String, FileState>>>,
        work_root: &PathBuf,
        bucket_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let path = work_root
            .join(bucket_name)
            .join("file_states.json");

        if let Some(parent) = path.parent() {
            if let Err(e) = tokio_fs::create_dir_all(parent).await {
                warn!(
                    "[DRIVE_MONITOR] Failed to create directory for file states: {} - {}",
                    parent.display(),
                    e
                );
            }
        }

        let states = file_states.read().await;
        match serde_json::to_string_pretty(&*states) {
            Ok(content) => {
                if let Err(e) = tokio_fs::write(&path, content).await {
                    warn!(
                        "[DRIVE_MONITOR] Failed to save file states to {}: {}",
                        path.display(),
                        e
                    );
                } else {
                    debug!(
                        "[DRIVE_MONITOR] Saved {} file states to disk for bucket {}",
                        states.len(),
                        bucket_name
                    );
                }
            }
            Err(e) => {
                warn!(
                    "[DRIVE_MONITOR] Failed to serialize file states: {}",
                    e
                );
            }
        }
        Ok(())
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
                debug!("[DRIVE_MONITOR] Health check failed: {}", e);
                false
            }
            Err(_) => {
                debug!("[DRIVE_MONITOR] Health check timed out");
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

    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!("start_monitoring ENTER");
        let start_mem = MemoryStats::current();
        trace!(
            "[DRIVE_MONITOR] Starting DriveMonitor for bot {}, RSS={}",
            self.bot_id,
            MemoryStats::format_bytes(start_mem.rss_bytes)
        );

        // Check if already processing to prevent duplicate monitoring
        if self.is_processing.load(std::sync::atomic::Ordering::Acquire) {
            warn!("Already processing for bot {}, skipping", self.bot_id);
            return Ok(());
        }

        // Load file states from disk to avoid reprocessing unchanged files
        if let Err(e) = self.load_file_states().await {
            warn!(
                "[DRIVE_MONITOR] Failed to load file states for bot {}: {}",
                self.bot_id, e
            );
        }

        if !self.check_drive_health().await {
            warn!(
                "[DRIVE_MONITOR] S3/MinIO not available for bucket {}, will retry with backoff",
                self.bucket_name
            );
        }

        self.is_processing
            .store(true, std::sync::atomic::Ordering::SeqCst);

        trace!("start_monitoring: calling check_for_changes...");
        trace!("Calling initial check_for_changes...");

        match tokio::time::timeout(Duration::from_secs(300), self.check_for_changes()).await {
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
            "[DRIVE_MONITOR] After initial check, RSS={} (delta={})",
            MemoryStats::format_bytes(after_initial.rss_bytes),
            MemoryStats::format_bytes(after_initial.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );

        // Force enable periodic monitoring regardless of initial check result
        self.is_processing.store(true, std::sync::atomic::Ordering::SeqCst);
        trace!("Forced is_processing to true for periodic monitoring");

        let self_clone = self.clone(); // Don't wrap in Arc::new - that creates a copy
        tokio::spawn(async move {
            let mut consecutive_processing_failures = 0;
            trace!("Starting periodic monitoring loop for bot {}", self_clone.bot_id);

            let is_processing_state = self_clone.is_processing.load(std::sync::atomic::Ordering::SeqCst);
            trace!("is_processing state at loop start: {} for bot {}", is_processing_state, self_clone.bot_id);

            while self_clone
                .is_processing
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                debug!("[DRIVE_MONITOR] Inside monitoring loop for bot {}", self_clone.bot_id);
                debug!("[DRIVE_MONITOR] Periodic check starting for bot {}", self_clone.bot_id);
                // Use fixed 10 second interval instead of backoff calculation
                tokio::time::sleep(Duration::from_secs(10)).await;

                debug!("[DRIVE_MONITOR] Checking drive health for bot {}", self_clone.bot_id);
                // Skip drive health check - just proceed with monitoring
                // if !self_clone.check_drive_health().await {
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

                debug!("[DRIVE_MONITOR] About to call check_for_changes for bot {}", self_clone.bot_id);
                // Add timeout to prevent hanging
                match tokio::time::timeout(Duration::from_secs(300), self_clone.check_for_changes()).await {
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
        trace!("Stopping DriveMonitor for bot {}", self.bot_id);

        self.is_processing
            .store(false, std::sync::atomic::Ordering::SeqCst);

        self.file_states.write().await.clear();
        self.consecutive_failures.store(0, Ordering::Relaxed);

        trace!("DriveMonitor stopped for bot {}", self.bot_id);
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
            "[DRIVE_MONITOR] check_for_changes START, RSS={}",
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
            "[DRIVE_MONITOR] After gbdialog, RSS={} (delta={})",
            MemoryStats::format_bytes(after_dialog.rss_bytes),
            MemoryStats::format_bytes(after_dialog.rss_bytes.saturating_sub(start_mem.rss_bytes))
        );

        trace!("check_for_changes: calling check_gbot...");
        trace!("Checking gbot...");
        self.check_gbot(client).await?;
        trace!("check_for_changes: check_gbot done");
        let after_gbot = MemoryStats::current();
        trace!(
            "[DRIVE_MONITOR] After gbot, RSS={} (delta={})",
            MemoryStats::format_bytes(after_gbot.rss_bytes),
            MemoryStats::format_bytes(after_gbot.rss_bytes.saturating_sub(after_dialog.rss_bytes))
        );

        trace!("check_for_changes: calling check_gbkb_changes...");
        trace!("Checking gbkb...");
        self.check_gbkb_changes(client).await?;
        trace!("check_for_changes: check_gbkb_changes done");
        let after_gbkb = MemoryStats::current();
        trace!(
            "[DRIVE_MONITOR] After gbkb, RSS={} (delta={})",
            MemoryStats::format_bytes(after_gbkb.rss_bytes),
            MemoryStats::format_bytes(after_gbkb.rss_bytes.saturating_sub(after_gbot.rss_bytes))
        );

        log_jemalloc_stats();

        let total_delta = after_gbkb.rss_bytes.saturating_sub(start_mem.rss_bytes);
        if total_delta > 50 * 1024 * 1024 {
            warn!(
                "[DRIVE_MONITOR] check_for_changes grew by {} - potential leak!",
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
            let file_state = FileState {
                etag: obj.e_tag().unwrap_or_default().to_string(),
                indexed: false,
                last_failed_at: None,
                fail_count: 0,
            };
                current_files.insert(path, file_state);
            }
            if !list_objects.is_truncated.unwrap_or(false) {
                break;
            }
            continuation_token = list_objects.next_continuation_token;
        }
        let mut file_states = self.file_states.write().await;
        for (path, current_state) in current_files.iter() {
            if let Some(previous_state) = file_states.get(path) {
                if current_state.etag != previous_state.etag {
                    if let Err(e) = self.compile_tool(client, path).await {
                        log::error!("Failed to compile tool {}: {}", path, e);
                    }
                }
            } else if let Err(e) = self.compile_tool(client, path).await {
                log::error!("Failed to compile tool {}: {}", path, e);
            }
        }
        let previous_paths: Vec<String> = file_states
            .keys()
            .cloned()
            .collect();
        for path in previous_paths {
            if !current_files.contains_key(&path) {
                file_states.remove(&path);
            }
        }
        for (path, mut state) in current_files {
            if path.ends_with(".bas") {
                state.indexed = true;
            }
            file_states.insert(path, state);
        }
        // Save file states to disk in background to avoid blocking
        // Use static helper to avoid double Arc (fixes "dispatch failure" error)
        let file_states_clone = Arc::clone(&self.file_states);
        let work_root_clone = self.work_root.clone();
        let bucket_name_clone = self.bucket_name.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::save_file_states_static(&file_states_clone, &work_root_clone, &bucket_name_clone).await {
                warn!("Failed to save file states: {}", e);
            }
        });
        Ok(())
    }
    async fn check_gbot(&self, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("check_gbot ENTER");
        let config_manager = ConfigManager::new(self.state.conn.clone());
        debug!(
            "check_gbot: Checking bucket {} for config.csv changes",
            self.bucket_name
        );
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

                let is_config_csv = path_lower == "config.csv"
                    || path_lower.ends_with("/config.csv")
                    || path_lower.contains(".gbot/config.csv");

                let is_prompt_file = path_lower.ends_with("prompt.md")
                    || path_lower.ends_with("prompt.txt")
                    || path_lower.ends_with("PROMPT.MD")
                    || path_lower.ends_with("PROMPT.TXT");

                debug!("check_gbot: Checking path: {} (is_config_csv: {}, is_prompt: {})", path, is_config_csv, is_prompt_file);

                if !is_config_csv && !is_prompt_file {
                    continue;
                }

                if is_prompt_file {
                    // Check etag to avoid re-downloading unchanged prompt files
                    let etag = obj.e_tag().unwrap_or_default().to_string();
                    let prompt_state_key = format!("__prompt__{}", path);
                    let should_download = {
                        let states = self.file_states.read().await;
                        match states.get(&prompt_state_key) {
                            Some(prev) => prev.etag != etag,
                            None => true,
                        }
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
                        let mut states = self.file_states.write().await;
                        states.insert(prompt_state_key, FileState { etag, indexed: false, last_failed_at: None, fail_count: 0 });
                        drop(states);
                        let file_states_clone = Arc::clone(&self.file_states);
                        let work_root_clone = self.work_root.clone();
                        let bucket_name_clone = self.bucket_name.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::save_file_states_static(&file_states_clone, &work_root_clone, &bucket_name_clone).await {
                                warn!("Failed to save file states after prompt update: {}", e);
                            }
                        });
                    } else {
                        trace!("Prompt file {} unchanged (etag match), skipping download", path);
                    }
                    continue;
                }

                debug!("check_gbot: Found config.csv at path: {}", path);
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

                        // Check for system-prompt-file and download it
                        let prompt_file_line = csv_content
                            .lines()
                            .find(|l| l.trim().starts_with("system-prompt-file,"));
                        if let Some(line) = prompt_file_line {
                            let parts: Vec<&str> = line.split(',').collect();
                            if parts.len() >= 2 {
                                let prompt_filename = parts[1].trim();
                                if !prompt_filename.is_empty() {
                                    // Get prompt file from MinIO
                                    let prompt_key = format!("salesianos.gbot/{}", prompt_filename);
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
                let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);
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
        let response_channels = self.state.response_channels.lock().await;
        for (session_id, tx) in response_channels.iter() {
            let theme_response = crate::core::shared::models::BotResponse {
                bot_id: self.bot_id.to_string(),
                user_id: "system".to_string(),
                session_id: session_id.clone(),
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
        drop(response_channels);
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
        trace!("check_gbkb_changes ENTER");
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
                Ok(Err(e)) => return Err(e.into()),
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

            let file_state = FileState {
                etag: obj.e_tag().unwrap_or_default().to_string(),
                indexed: false,
                last_failed_at: None,
                fail_count: 0,
            };
                current_files.insert(path.clone(), file_state);
            }

            if !list_objects.is_truncated.unwrap_or(false) {
                break;
            }
            continuation_token = list_objects.next_continuation_token;
        }

        let mut file_states = self.file_states.write().await;

        for (path, current_state) in current_files.iter() {
            let is_new = !file_states.contains_key(path);
            let is_modified = file_states
                .get(path)
                .map(|prev| prev.etag != current_state.etag)
                .unwrap_or(false);

            if is_new || is_modified {
                if let Some(prev_state) = file_states.get(path) {
                    if prev_state.fail_count >= MAX_FAIL_COUNT {
                        let elapsed = Utc::now()
                            .signed_duration_since(prev_state.last_failed_at.unwrap_or(Utc::now()));
                        if elapsed.num_seconds() < RETRY_BACKOFF_SECS {
                            trace!(
                                "Skipping {} - fail_count={} (last failed {}s ago, max {}s backoff)",
                                path, prev_state.fail_count, elapsed.num_seconds(), RETRY_BACKOFF_SECS
                            );
                            continue;
                        }
                    }
                }

                if path.to_lowercase().ends_with(".pdf") {
                    pdf_files_found += 1;
                    trace!(
                        "Detected {} PDF in .gbkb: {} (will extract text for vectordb)",
                        if is_new { "new" } else { "changed" },
                        path
                    );
                } else {
                    trace!(
                        "Detected {} in .gbkb: {}",
                        if is_new { "new file" } else { "change" },
                        path
                    );
                }

                files_to_process.push(path.clone());
                files_processed += 1;

                if files_to_process.len() >= 10 {
                    for file_path in std::mem::take(&mut files_to_process) {
                        if let Err(e) = self.download_gbkb_file(client, &file_path).await {
                            log::error!("Failed to download .gbkb file {}: {}", file_path, e);
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                let path_parts: Vec<&str> = path.split('/').collect();
                if path_parts.len() >= 3 {
                    let kb_name = path_parts[1];
                    let kb_folder_path = self
                        .work_root
                        .join(bot_name)
                        .join(&gbkb_prefix)
                        .join(kb_name);

                    #[cfg(any(feature = "research", feature = "llm"))]
                    {
                        if !is_embedding_server_ready() {
                            info!("Embedding server not yet marked ready, KB indexing will wait for it");
                        }

                        // Create a unique key for this KB folder to track indexing state
                        let kb_key = format!("{}_{}", bot_name, kb_name);

            // Check fail_count for this KB folder - implement backoff
            {
                let states = self.file_states.read().await;
                let _kb_prefix = format!("{}/", gbkb_prefix);
                
                let max_fail_count = states.values()
                    .map(|s| s.fail_count)
                    .max()
                    .unwrap_or(0);
                
                // Backoff: wait longer based on fail count
                // fail_count 0: no wait, 1: 5min, 2: 15min, 3+: 1h
                if max_fail_count > 0 {
                    let wait_seconds = match max_fail_count {
                        1 => 300,   // 5 min
                        2 => 900,   // 15 min
                        _ => 3600,  // 1 hour
                    };
                    
                    if let Some(last_failed) = states.values()
                        .filter_map(|s| s.last_failed_at)
                        .max() 
                    {
                        let elapsed = chrono::Utc::now() - last_failed;
                        if elapsed.num_seconds() < wait_seconds {
                            trace!("[DRIVE_MONITOR] KB folder {} in backoff (fail_count={}, elapsed={}s < {}s), skipping", 
                                kb_key, max_fail_count, elapsed.num_seconds(), wait_seconds);
                            continue;
                        }
                    }
                }
            }

            // Check if this KB folder is already being indexed
            {
                let indexing_set = self.files_being_indexed.read().await;
                if indexing_set.contains(&kb_key) {
                    debug!("[DRIVE_MONITOR] KB folder {} already being indexed, skipping duplicate task", kb_key);
                    continue;
                }
            }

            // Mark this KB folder as being indexed
            {
                let mut indexing_set = self.files_being_indexed.write().await;
                indexing_set.insert(kb_key.clone());
            }

                        let kb_manager = Arc::clone(&self.kb_manager);
                        let bot_id = self.bot_id;
                        let bot_name_owned = bot_name.to_string();
                        let kb_name_owned = kb_name.to_string();
                        let kb_folder_owned = kb_folder_path.clone();
            let _files_being_indexed = Arc::clone(&self.files_being_indexed);
            let file_key = Arc::clone(&self.files_being_indexed);
            let kb_key_owned = kb_key.clone();
            let file_states = Arc::clone(&self.file_states);
            let work_root = self.work_root.clone();
            let bucket_name = self.bucket_name.clone();
            let gbkb_prefix_owned = gbkb_prefix.clone();
            let bot_name_for_spawn = bot_name.to_string();

            tokio::spawn(async move {
                trace!(
                    "Triggering KB indexing for folder: {} (PDF text extraction enabled)",
                    kb_folder_owned.display()
                );

                let result = tokio::time::timeout(
                    Duration::from_secs(60),
                    kb_manager.handle_gbkb_change(bot_id, &bot_name_owned, &kb_folder_owned),
                )
                .await;

                // Always remove from tracking set when done, regardless of outcome
                {
                    let mut indexing_set = file_key.write().await;
                    indexing_set.remove(&kb_key_owned);
                }

                let kb_prefix = format!("{}/", gbkb_prefix_owned);
                let kb_folder_name = kb_folder_owned
                    .strip_prefix(&work_root)
                    .ok()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .trim_start_matches(&format!("{}/", bot_name_for_spawn))
                    .trim_start_matches(&kb_prefix)
                    .to_string();

                {
                    let mut states = file_states.write().await;
                    for (path, state) in states.iter_mut() {
                        if path.starts_with(&format!("{}{}/", kb_prefix, kb_folder_name)) {
                            match &result {
                                Ok(Ok(_)) => {
                                    state.indexed = true;
                                    state.fail_count = 0;
                                    state.last_failed_at = None;
                                }
                                Ok(Err(_)) | Err(_) => {
                                    state.fail_count = state.fail_count.saturating_add(1);
                                    state.last_failed_at = Some(chrono::Utc::now());
                                }
                            }
                        }
                    }
                    let states_clone = Arc::clone(&file_states);
                    let work_root_clone = work_root.clone();
                    let bucket_name_clone = bucket_name.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::save_file_states_static(&states_clone, &work_root_clone, &bucket_name_clone).await {
                            warn!("Failed to save file states after indexing update: {}", e);
                        }
                    });
                }

                match result {
                    Ok(Ok(_)) => {
                        debug!(
                            "Successfully processed KB change for {}/{}",
                            bot_name_owned, kb_name_owned
                        );
                    }
                    Ok(Err(e)) => {
                        log::error!(
                            "Failed to process .gbkb change for {}/{}: {}",
                            bot_name_owned,
                            kb_name_owned,
                            e
                        );
                    }
                    Err(_) => {
                        log::error!(
                            "KB indexing timed out after 60s for {}/{}",
                            bot_name_owned, kb_name_owned
                        );
                    }
                }
            });
                    }

                    #[cfg(not(any(feature = "research", feature = "llm")))]
                    {
                        let _ = kb_folder_path;
                        debug!(
                            "KB indexing disabled because research/llm features are not enabled"
                        );
                    }
                }
            }
        }

        let paths_to_remove: Vec<String> = file_states
            .keys()
            .filter(|path| path.starts_with(&gbkb_prefix) && !current_files.contains_key(*path))
            .cloned()
            .collect();

        for file_path in files_to_process {
            if let Err(e) = self.download_gbkb_file(client, &file_path).await {
                log::error!("Failed to download .gbkb file {}: {}", file_path, e);
            }
        }

        if files_processed > 0 {
            trace!(
                "Processed {} .gbkb files (including {} PDFs for text extraction)",
                files_processed, pdf_files_found
            );
        }
        for (path, mut state) in current_files {
            if let Some(previous) = file_states.get(&path) {
                if state.etag == previous.etag {
                    state.indexed = previous.indexed;
                    state.fail_count = previous.fail_count;
                    state.last_failed_at = previous.last_failed_at;
                }
            }
            file_states.insert(path, state);
        }

        // Save file states to disk in background to avoid blocking
        // Use static helper to avoid double Arc (fixes "dispatch failure" error)
        let file_states_clone = Arc::clone(&self.file_states);
        let work_root_clone = self.work_root.clone();
        let bucket_name_clone = self.bucket_name.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::save_file_states_static(&file_states_clone, &work_root_clone, &bucket_name_clone).await {
                warn!("Failed to save file states: {}", e);
            }
        });

        for path in paths_to_remove {
            trace!("Detected deletion in .gbkb: {}", path);
            file_states.remove(&path);

            let path_parts: Vec<&str> = path.split('/').collect();
            if path_parts.len() >= 2 {
                let kb_name = path_parts[1];

                let kb_prefix = format!("{}{}/", gbkb_prefix, kb_name);
                if !file_states.keys().any(|k| k.starts_with(&kb_prefix)) {
                    #[cfg(any(feature = "research", feature = "llm"))]
                    if let Err(e) = self.kb_manager.clear_kb(self.bot_id, bot_name, kb_name).await {
                        log::error!("Failed to clear KB {}: {}", kb_name, e);
                    }

                    #[cfg(not(any(feature = "research", feature = "llm")))]
                    {
                        let _ = (bot_name, kb_name);
                        debug!("Bypassing KB clear because research/llm features are not enabled");
                    }
                }
            }
        }

        trace!("check_gbkb_changes EXIT");
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
