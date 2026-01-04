use crate::basic::compiler::BasicCompiler;
use crate::config::ConfigManager;
use crate::core::kb::embedding_generator::is_embedding_server_ready;
use crate::core::kb::KnowledgeBaseManager;
use crate::core::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
use crate::shared::message_types::MessageType;
use crate::shared::state::AppState;
use aws_sdk_s3::Client;
use log::{debug, error, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::Duration;

const KB_INDEXING_TIMEOUT_SECS: u64 = 60;
const MAX_BACKOFF_SECS: u64 = 300;
const INITIAL_BACKOFF_SECS: u64 = 30;
#[derive(Debug, Clone)]
pub struct FileState {
    pub etag: String,
}
#[derive(Debug, Clone)]
pub struct DriveMonitor {
    state: Arc<AppState>,
    bucket_name: String,
    file_states: Arc<tokio::sync::RwLock<HashMap<String, FileState>>>,
    bot_id: uuid::Uuid,
    kb_manager: Arc<KnowledgeBaseManager>,
    work_root: PathBuf,
    is_processing: Arc<AtomicBool>,
    consecutive_failures: Arc<AtomicU32>,
    /// Track KB folders currently being indexed to prevent duplicate tasks
    kb_indexing_in_progress: Arc<TokioRwLock<HashSet<String>>>,
}
impl DriveMonitor {
    pub fn new(state: Arc<AppState>, bucket_name: String, bot_id: uuid::Uuid) -> Self {
        let work_root = PathBuf::from("work");
        let kb_manager = Arc::new(KnowledgeBaseManager::new(work_root.clone()));

        Self {
            state,
            bucket_name,
            file_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            bot_id,
            kb_manager,
            work_root,
            is_processing: Arc::new(AtomicBool::new(false)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            kb_indexing_in_progress: Arc::new(TokioRwLock::new(HashSet::new())),
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
        trace!("[PROFILE] start_monitoring ENTER");
        let start_mem = MemoryStats::current();
        trace!("[DRIVE_MONITOR] Starting DriveMonitor for bot {}, RSS={}",
              self.bot_id, MemoryStats::format_bytes(start_mem.rss_bytes));

        if !self.check_drive_health().await {
            warn!("[DRIVE_MONITOR] S3/MinIO not available for bucket {}, will retry with backoff",
                  self.bucket_name);
        }

        self.is_processing
            .store(true, std::sync::atomic::Ordering::SeqCst);

        trace!("[PROFILE] start_monitoring: calling check_for_changes...");
        info!("[DRIVE_MONITOR] Calling initial check_for_changes...");

        match self.check_for_changes().await {
            Ok(_) => {
                self.consecutive_failures.store(0, Ordering::Relaxed);
            }
            Err(e) => {
                warn!("[DRIVE_MONITOR] Initial check failed (will retry): {}", e);
                self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
            }
        }
        trace!("[PROFILE] start_monitoring: check_for_changes returned");

        let after_initial = MemoryStats::current();
        trace!("[DRIVE_MONITOR] After initial check, RSS={} (delta={})",
              MemoryStats::format_bytes(after_initial.rss_bytes),
              MemoryStats::format_bytes(after_initial.rss_bytes.saturating_sub(start_mem.rss_bytes)));

        let self_clone = Arc::new(self.clone());
        tokio::spawn(async move {
            while self_clone
                .is_processing
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                let backoff = self_clone.calculate_backoff();
                tokio::time::sleep(backoff).await;

                if !self_clone.check_drive_health().await {
                    let failures = self_clone.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                    if failures % 10 == 1 {
                        warn!("[DRIVE_MONITOR] S3/MinIO unavailable for bucket {} (failures: {}), backing off to {:?}",
                              self_clone.bucket_name, failures, self_clone.calculate_backoff());
                    }
                    continue;
                }

                match self_clone.check_for_changes().await {
                    Ok(_) => {
                        let prev_failures = self_clone.consecutive_failures.swap(0, Ordering::Relaxed);
                        if prev_failures > 0 {
                            info!("[DRIVE_MONITOR] S3/MinIO recovered for bucket {} after {} failures",
                                  self_clone.bucket_name, prev_failures);
                        }
                    }
                    Err(e) => {
                        self_clone.consecutive_failures.fetch_add(1, Ordering::Relaxed);
                        error!("Error during sync for bot {}: {}", self_clone.bot_id, e);
                    }
                }
            }
        });

        info!("DriveMonitor started for bot {}", self.bot_id);
        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping DriveMonitor for bot {}", self.bot_id);

        self.is_processing
            .store(false, std::sync::atomic::Ordering::SeqCst);

        self.file_states.write().await.clear();
        self.consecutive_failures.store(0, Ordering::Relaxed);

        info!("DriveMonitor stopped for bot {}", self.bot_id);
        Ok(())
    }
    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
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
                        warn!("[DRIVE_MONITOR] S3/MinIO unavailable for bucket {} (failures: {}), backing off to {:?}",
                              self.bucket_name, failures, self.calculate_backoff());
                    }
                    continue;
                }

                self.is_processing.store(true, Ordering::Release);

                match self.check_for_changes().await {
                    Ok(_) => {
                        let prev_failures = self.consecutive_failures.swap(0, Ordering::Relaxed);
                        if prev_failures > 0 {
                            info!("[DRIVE_MONITOR] S3/MinIO recovered for bucket {} after {} failures",
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
        trace!("[PROFILE] check_for_changes ENTER");
        let start_mem = MemoryStats::current();
        trace!("[DRIVE_MONITOR] check_for_changes START, RSS={}",
              MemoryStats::format_bytes(start_mem.rss_bytes));

        let Some(client) = &self.state.drive else {
            trace!("[PROFILE] check_for_changes: no drive client, returning");
            return Ok(());
        };

        trace!("[PROFILE] check_for_changes: calling check_gbdialog_changes...");
        trace!("[DRIVE_MONITOR] Checking gbdialog...");
        self.check_gbdialog_changes(client).await?;
        trace!("[PROFILE] check_for_changes: check_gbdialog_changes done");
        let after_dialog = MemoryStats::current();
        trace!("[DRIVE_MONITOR] After gbdialog, RSS={} (delta={})",
              MemoryStats::format_bytes(after_dialog.rss_bytes),
              MemoryStats::format_bytes(after_dialog.rss_bytes.saturating_sub(start_mem.rss_bytes)));

        trace!("[PROFILE] check_for_changes: calling check_gbot...");
        trace!("[DRIVE_MONITOR] Checking gbot...");
        self.check_gbot(client).await?;
        trace!("[PROFILE] check_for_changes: check_gbot done");
        let after_gbot = MemoryStats::current();
        trace!("[DRIVE_MONITOR] After gbot, RSS={} (delta={})",
              MemoryStats::format_bytes(after_gbot.rss_bytes),
              MemoryStats::format_bytes(after_gbot.rss_bytes.saturating_sub(after_dialog.rss_bytes)));

        trace!("[PROFILE] check_for_changes: calling check_gbkb_changes...");
        trace!("[DRIVE_MONITOR] Checking gbkb...");
        self.check_gbkb_changes(client).await?;
        trace!("[PROFILE] check_for_changes: check_gbkb_changes done");
        let after_gbkb = MemoryStats::current();
        trace!("[DRIVE_MONITOR] After gbkb, RSS={} (delta={})",
              MemoryStats::format_bytes(after_gbkb.rss_bytes),
              MemoryStats::format_bytes(after_gbkb.rss_bytes.saturating_sub(after_gbot.rss_bytes)));

        log_jemalloc_stats();

        let total_delta = after_gbkb.rss_bytes.saturating_sub(start_mem.rss_bytes);
        if total_delta > 50 * 1024 * 1024 {
            warn!("[DRIVE_MONITOR] check_for_changes grew by {} - potential leak!",
                  MemoryStats::format_bytes(total_delta));
        }

        trace!("[PROFILE] check_for_changes EXIT");
        Ok(())
    }
    async fn check_gbdialog_changes(
        &self,
        client: &Client,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = ".gbdialog/";
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
                if path_parts.len() < 2 || !path_parts[0].ends_with(".gbdialog") {
                    continue;
                }
                if path.ends_with('/') || !path.to_ascii_lowercase().ends_with(".bas") {
                    continue;
                }
                let file_state = FileState {
                    etag: obj.e_tag().unwrap_or_default().to_string(),
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
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        for path in previous_paths {
            if !current_files.contains_key(&path) {
                file_states.remove(&path);
            }
        }
        for (path, state) in current_files {
            file_states.insert(path, state);
        }
        Ok(())
    }
    async fn check_gbot(&self, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("[PROFILE] check_gbot ENTER");
        let config_manager = ConfigManager::new(self.state.conn.clone());
        debug!("check_gbot: Checking bucket {} for config.csv changes", self.bucket_name);
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
                    error!("check_gbot: Failed to list objects in bucket {}: {}", self.bucket_name, e);
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

                if !is_config_csv {
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
                            use crate::llm::local::ensure_llama_servers_running;
                            use crate::llm::DynamicLLMProvider;
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
                                    match config_manager.get_config(&self.bot_id, key, None) {
                                        Ok(old_value) => {
                                            if old_value != new_value {
                                                info!(
                                                    "Detected change in {} (old: {}, new: {})",
                                                    key, old_value, new_value
                                                );
                                                restart_needed = true;
                                                if key == "llm-url" || key == "llm-model" {
                                                    llm_url_changed = true;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            restart_needed = true;
                                            if key == "llm-url" || key == "llm-model" {
                                                llm_url_changed = true;
                                            }
                                        }
                                    }
                                }
                            }
                            let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);
                            if restart_needed {
                                if let Err(e) =
                                    ensure_llama_servers_running(Arc::clone(&self.state)).await
                                {
                                    log::error!("Failed to restart LLaMA servers after llm- config change: {}", e);
                                }
                            }
                            if llm_url_changed {
                                info!("check_gbot: LLM config changed, updating provider...");
                                let effective_url = if new_llm_url.is_empty() {
                                    config_manager.get_config(&self.bot_id, "llm-url", None).unwrap_or_default()
                                } else {
                                    new_llm_url
                                };
                                info!("check_gbot: Effective LLM URL: {}", effective_url);
                                if !effective_url.is_empty() {
                                    if let Some(dynamic_provider) = self.state.extensions.get::<Arc<DynamicLLMProvider>>().await {
                                        let model = if new_llm_model.is_empty() { None } else { Some(new_llm_model.clone()) };
                                        dynamic_provider.update_from_config(&effective_url, model).await;
                                        info!("Updated LLM provider to use URL: {}, model: {:?}", effective_url, new_llm_model);
                                    } else {
                                        error!("DynamicLLMProvider not found in extensions, LLM provider cannot be updated dynamically");
                                    }
                                } else {
                                    error!("check_gbot: No llm-url found in config, cannot update provider");
                                }
                            } else {
                                debug!("check_gbot: No LLM config changes detected");
                            }
                        }
                        if csv_content.lines().any(|line| line.starts_with("theme-")) {
                            self.broadcast_theme_change(&csv_content).await?;
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
        trace!("[PROFILE] check_gbot EXIT");
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
            let theme_response = crate::shared::models::BotResponse {
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
        info!(
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
                info!(
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
        let work_dir = format!("./work/{}.gbai/{}.gbdialog", bot_name, bot_name);
        let state_clone = Arc::clone(&self.state);
        let work_dir_clone = work_dir.clone();
        let tool_name_clone = tool_name.clone();
        let source_content_clone = source_content.clone();
        let bot_id = self.bot_id;
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&work_dir_clone)?;
            let local_source_path = format!("{}/{}.bas", work_dir_clone, tool_name_clone);
            std::fs::write(&local_source_path, &source_content_clone)?;
            let mut compiler = BasicCompiler::new(state_clone, bot_id);
            let result = compiler.compile_file(&local_source_path, &work_dir_clone)?;
            if let Some(mcp_tool) = result.mcp_tool {
                info!(
                    "MCP tool definition generated with {} parameters",
                    mcp_tool.input_schema.properties.len()
                );
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        })
        .await??;
        Ok(())
    }

    async fn check_gbkb_changes(
        &self,
        client: &Client,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("[PROFILE] check_gbkb_changes ENTER");
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
                if path.to_lowercase().ends_with(".pdf") {
                    pdf_files_found += 1;
                    info!(
                        "Detected {} PDF in .gbkb: {} (will extract text for vectordb)",
                        if is_new { "new" } else { "changed" },
                        path
                    );
                } else {
                    info!(
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

                    let kb_indexing_disabled = std::env::var("DISABLE_KB_INDEXING")
                        .map(|v| v == "true" || v == "1")
                        .unwrap_or(false);

                    if kb_indexing_disabled {
                        debug!("KB indexing disabled via DISABLE_KB_INDEXING, skipping {}", kb_folder_path.display());
                        continue;
                    }

                    if !is_embedding_server_ready() {
                        info!("[DRIVE_MONITOR] Embedding server not ready, deferring KB indexing for {}", kb_folder_path.display());
                        continue;
                    }

                    // Create a unique key for this KB folder to track indexing state
                    let kb_key = format!("{}_{}", bot_name, kb_name);

                    // Check if this KB folder is already being indexed
                    {
                        let indexing_set = self.kb_indexing_in_progress.read().await;
                        if indexing_set.contains(&kb_key) {
                            debug!("[DRIVE_MONITOR] KB folder {} already being indexed, skipping duplicate task", kb_key);
                            continue;
                        }
                    }

                    // Mark this KB folder as being indexed
                    {
                        let mut indexing_set = self.kb_indexing_in_progress.write().await;
                        indexing_set.insert(kb_key.clone());
                    }

                    let kb_manager = Arc::clone(&self.kb_manager);
                    let bot_name_owned = bot_name.to_string();
                    let kb_name_owned = kb_name.to_string();
                    let kb_folder_owned = kb_folder_path.clone();
                    let indexing_tracker = Arc::clone(&self.kb_indexing_in_progress);
                    let kb_key_owned = kb_key.clone();

                    tokio::spawn(async move {
                        info!(
                            "Triggering KB indexing for folder: {} (PDF text extraction enabled)",
                            kb_folder_owned.display()
                        );

                        let result = tokio::time::timeout(
                            Duration::from_secs(KB_INDEXING_TIMEOUT_SECS),
                            kb_manager.handle_gbkb_change(&bot_name_owned, &kb_folder_owned)
                        ).await;

                        // Always remove from tracking set when done, regardless of outcome
                        {
                            let mut indexing_set = indexing_tracker.write().await;
                            indexing_set.remove(&kb_key_owned);
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
                                    "KB indexing timed out after {}s for {}/{}",
                                    KB_INDEXING_TIMEOUT_SECS,
                                    bot_name_owned,
                                    kb_name_owned
                                );
                            }
                        }
                    });
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
            info!(
                "Processed {} .gbkb files (including {} PDFs for text extraction)",
                files_processed, pdf_files_found
            );
        }

        for (path, state) in current_files {
            file_states.insert(path, state);
        }

        for path in paths_to_remove {
            info!("Detected deletion in .gbkb: {}", path);
            file_states.remove(&path);

            let path_parts: Vec<&str> = path.split('/').collect();
            if path_parts.len() >= 2 {
                let kb_name = path_parts[1];

                let kb_prefix = format!("{}{}/", gbkb_prefix, kb_name);
                if !file_states.keys().any(|k| k.starts_with(&kb_prefix)) {
                    if let Err(e) = self.kb_manager.clear_kb(bot_name, kb_name).await {
                        log::error!("Failed to clear KB {}: {}", kb_name, e);
                    }
                }
            }
        }

        trace!("[PROFILE] check_gbkb_changes EXIT");
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

        info!(
            "Downloaded .gbkb file {} to {}",
            file_path,
            local_path.display()
        );

        Ok(())
    }
}
