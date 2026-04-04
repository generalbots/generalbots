use crate::basic::compiler::BasicCompiler;
use crate::core::kb::{EmbeddingConfig, KnowledgeBaseManager};
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tokio::time::Duration;
use notify::{RecursiveMode, EventKind, RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalFileState {
    modified: SystemTime,
    size: u64,
}

/// Tracks state of a KB folder for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KbFolderState {
    /// Combined hash of all file mtimes and sizes in the folder tree
    content_hash: u64,
    /// Number of files indexed last time
    file_count: usize,
}

pub struct LocalFileMonitor {
    state: Arc<AppState>,
    data_dir: PathBuf,
    work_root: PathBuf,
    file_states: Arc<RwLock<HashMap<String, LocalFileState>>>,
    kb_states: Arc<RwLock<HashMap<String, KbFolderState>>>,
    is_processing: Arc<AtomicBool>,
    #[cfg(any(feature = "research", feature = "llm"))]
    kb_manager: Option<Arc<KnowledgeBaseManager>>,
}

impl LocalFileMonitor {
    pub fn new(state: Arc<AppState>) -> Self {
        // Use botserver-stack/data/system/work as the work directory
        let work_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("botserver-stack/data/system/work");

        // Use /opt/gbo/data as the base directory for source files
        let data_dir = PathBuf::from("/opt/gbo/data");

        #[cfg(any(feature = "research", feature = "llm"))]
        let kb_manager = match &state.kb_manager {
            Some(km) => Some(Arc::clone(km)),
            None => {
                debug!("KB manager not available in LocalFileMonitor");
                None
            }
        };

        trace!("Initializing with data_dir: {:?}, work_root: {:?}", data_dir, work_root);

        Self {
            state,
            data_dir,
            work_root,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            kb_states: Arc::new(RwLock::new(HashMap::new())),
            is_processing: Arc::new(AtomicBool::new(false)),
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_manager,
        }
    }

    pub async fn start_monitoring(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Local file monitor started - watching /opt/gbo/data/*.gbai directories");

        // Create data directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&self.data_dir).await {
            warn!("Failed to create data directory: {}", e);
        }

        // Load persisted file states from disk
        self.load_states().await;

        // Initial scan of all .gbai directories
        self.scan_and_compile_all().await?;

        // Persist states back to disk
        self.save_states().await;

        self.is_processing.store(true, Ordering::SeqCst);

        // Spawn the monitoring loop
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });

        debug!("Local file monitor successfully initialized");
        Ok(())
    }

    async fn monitoring_loop(&self) {
        trace!("Starting monitoring loop");

        // Try to create a file system watcher
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Use notify crate for file system watching
        let tx_clone = tx.clone();
        let mut watcher: RecommendedWatcher = match RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx_clone.try_send(event);
                }
            },
            notify::Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create watcher: {}. Falling back to polling.", e);
                // Fall back to polling if watcher creation fails
                self.polling_loop().await;
                return;
            }
        };

        // Watch the data directory
        if let Err(e) = watcher.watch(&self.data_dir, RecursiveMode::Recursive) {
            warn!("Failed to watch directory {:?}: {}. Using polling fallback.", self.data_dir, e);
            drop(watcher);
            self.polling_loop().await;
            return;
        }

        trace!("Watching directory: {:?}", self.data_dir);

        while self.is_processing.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(60)).await;

            // Process events from the watcher
            while let Ok(event) = rx.try_recv() {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any => {
                        for path in &event.paths {
                            if self.is_gbdialog_file(path) {
                                debug!("Detected change in: {:?}", path);
                                if let Err(e) = self.compile_local_file(path).await {
                                    error!("Failed to compile {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in &event.paths {
                            if self.is_gbdialog_file(path) {
                                debug!("File removed: {:?}", path);
                                self.remove_file_state(path).await;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        trace!("Monitoring loop ended");
    }

    async fn polling_loop(&self) {
        trace!("Using polling fallback (checking every 60s)");

        while self.is_processing.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(60)).await;

            if let Err(e) = self.scan_and_compile_all().await {
                error!("Scan failed: {}", e);
            }
        }
    }

    fn is_gbdialog_file(&self, path: &Path) -> bool {
        // Check if path is something like /opt/gbo/data/*.gbai/*.gbdialog/*.bas
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("bas"))
            .unwrap_or(false)
            && path.ancestors()
                .any(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(".gbdialog"))
                        .unwrap_or(false)
                })
    }

    async fn scan_and_compile_all(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        trace!("Scanning directory: {:?}", self.data_dir);

        let entries = match tokio::fs::read_dir(&self.data_dir).await {
            Ok(e) => e,
            Err(e) => {
                debug!("[LOCAL_MONITOR] Cannot read data directory: {}", e);
                return Ok(());
            }
        };

        let mut entries = entries;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Check if this is a .gbai directory
            if path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("gbai"))
                .unwrap_or(false)
            {
                let bot_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                // Look for <botname>.gbdialog folder inside (e.g., cristo.gbai/cristo.gbdialog)
                let gbdialog_path = path.join(format!("{}.gbdialog", bot_name));
                if gbdialog_path.exists() {
                    self.compile_gbdialog(bot_name, &gbdialog_path).await?;
                }

                // Index .gbkb folders
                #[cfg(any(feature = "research", feature = "llm"))]
                {
                    if let Some(ref kb_manager) = self.kb_manager {
                        let gbkb_path = path.join(format!("{}.gbkb", bot_name));
                        if gbkb_path.exists() {
                            if let Err(e) = self.index_gbkb_folder(bot_name, &gbkb_path, kb_manager).await {
                                error!("Failed to index .gbkb folder {:?}: {}", gbkb_path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    async fn index_gbkb_folder(
        &self,
        bot_name: &str,
        gbkb_path: &Path,
        _kb_manager: &Arc<KnowledgeBaseManager>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Get bot_id from database
        let bot_id = {
            use crate::core::shared::models::schema::bots::dsl::*;
            let mut conn = self.state.conn.get()
                .map_err(|e| format!("Failed to get DB connection: {}", e))?;

            bots.filter(name.eq(bot_name))
                .select(id)
                .first::<Uuid>(&mut *conn)
                .map_err(|e| format!("Failed to get bot_id for '{}': {}", bot_name, e))?
        };

        // Load bot-specific embedding config from database
        let embedding_config = EmbeddingConfig::from_bot_config(&self.state.conn, &bot_id);

        // Compute content hash of the entire .gbkb tree
        let (content_hash, file_count) = self.compute_gbkb_hash(gbkb_path).await?;

        // Index each KB folder inside .gbkb (e.g., carta, proc)
        let entries = tokio::fs::read_dir(gbkb_path).await?;
        let mut entries = entries;

        while let Some(entry) = entries.next_entry().await? {
            let kb_folder_path = entry.path();

            if kb_folder_path.is_dir() {
                if let Some(kb_name) = kb_folder_path.file_name().and_then(|n| n.to_str()) {
                    let kb_key = format!("{}:{}", bot_name, kb_name);

                    // Check if KB content changed since last index
                    let should_index = {
                        let states = self.kb_states.read().await;
                        states.get(&kb_key)
                            .map(|state| state.content_hash != content_hash || state.file_count != file_count)
                            .unwrap_or(true)
                    };

                    if !should_index {
                        debug!("KB '{}' for bot '{}' unchanged, skipping re-index", kb_name, bot_name);
                        continue;
                    }

                    info!("Indexing KB '{}' for bot '{}'", kb_name, bot_name);

                    // Create a temporary KbIndexer with the bot-specific config
                    let qdrant_config = crate::core::kb::QdrantConfig::default();
                    let indexer = crate::core::kb::KbIndexer::new(embedding_config.clone(), qdrant_config);

                    if let Err(e) = indexer.index_kb_folder(
                        bot_id,
                        bot_name,
                        kb_name,
                        &kb_folder_path,
                    ).await {
                        error!("Failed to index KB '{}' for bot '{}': {}", kb_name, bot_name, e);
                    }

                    // Update state to mark as indexed
                    let mut states = self.kb_states.write().await;
                    states.insert(kb_key, KbFolderState { content_hash, file_count });
                }
            }
        }

        Ok(())
    }

    /// Compute a simple hash over all file metadata in a folder tree
    #[cfg(any(feature = "research", feature = "llm"))]
    async fn compute_gbkb_hash(&self, root: &Path) -> Result<(u64, usize), Box<dyn Error + Send + Sync>> {
        let mut hash: u64 = 0;
        let mut file_count: usize = 0;

        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let Ok(meta) = tokio::fs::metadata(&path).await {
                    let mtime = meta.modified()
                        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
                        .unwrap_or(0);
                    let size = meta.len();
                    // Simple combinatorial hash
                    hash = hash.wrapping_mul(31).wrapping_add(mtime.wrapping_mul(37).wrapping_add(size));
                    file_count += 1;
                }
            }
        }

        Ok((hash, file_count))
    }

    async fn compile_gbdialog(&self, bot_name: &str, gbdialog_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = tokio::fs::read_dir(gbdialog_path).await?;
        let mut entries = entries;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("bas"))
                .unwrap_or(false)
            {
                let metadata = tokio::fs::metadata(&path).await?;
                let modified = metadata.modified()?;
                let size = metadata.len();

                let file_key = path.to_string_lossy().to_string();

                // Check if file changed
                let should_compile = {
                    let states = self.file_states.read().await;
                    states.get(&file_key)
                        .map(|state| state.modified != modified || state.size != size)
                        .unwrap_or(true)
                };

                if should_compile {
                    info!("Compiling bot: {}", bot_name);
                    debug!("Recompiling {:?} - modification detected", path);
                    if let Err(e) = self.compile_local_file(&path).await {
                        error!("Failed to compile {:?}: {}", path, e);
                    }

                    // Update state
                    let mut states = self.file_states.write().await;
                    states.insert(file_key, LocalFileState { modified, size });
                }
            }
        }

        Ok(())
    }

    async fn compile_local_file(&self, file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        let tool_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Extract bot name from path like /opt/gbo/data/cristo.gbai/.gbdialog/file.bas
        let bot_name = file_path
            .ancestors()
            .find(|p| p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("gbai")).unwrap_or(false))
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Create work directory structure in botserver/work (not in data/)
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));

        // Read the file content
        let source_content = tokio::fs::read_to_string(file_path).await?;

        // Compile the file
        let state_clone = Arc::clone(&self.state);
        let work_dir_clone = work_dir.clone();
        let tool_name_clone = tool_name.to_string();
        let source_content_clone = source_content.clone();
        let bot_name_clone = bot_name.to_string();

        // Get the actual bot_id from the database for this bot_name
        let bot_id = {
            use crate::core::shared::models::schema::bots::dsl::*;
            let mut conn = state_clone.conn.get()
                .map_err(|e| format!("Failed to get DB connection: {}", e))?;

            bots.filter(name.eq(&bot_name_clone))
                .select(id)
                .first::<Uuid>(&mut *conn)
                .map_err(|e| format!("Failed to get bot_id for '{}': {}", bot_name_clone, e))?
        };

        let elapsed_ms = tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&work_dir_clone)?;
            let local_source_path = work_dir_clone.join(format!("{}.bas", tool_name_clone));
            std::fs::write(&local_source_path, &source_content_clone)?;
            let mut compiler = BasicCompiler::new(state_clone, bot_id);
            let local_source_str = local_source_path.to_str()
                .ok_or_else(|| "Invalid UTF-8 in local source path".to_string())?;
            let work_dir_str = work_dir_clone.to_str()
                .ok_or_else(|| "Invalid UTF-8 in work directory path".to_string())?;
            let start_time = std::time::Instant::now();
            let result = compiler.compile_file(local_source_str, work_dir_str)?;
            let elapsed_ms = start_time.elapsed().as_millis();
            if let Some(mcp_tool) = result.mcp_tool {
                trace!(
                    "[LOCAL_MONITOR] MCP tool generated with {} parameters for bot {}",
                    mcp_tool.input_schema.properties.len(),
                    bot_name_clone
                );
            }
            Ok::<u128, Box<dyn Error + Send + Sync>>(elapsed_ms)
        })
        .await??;

        info!("Successfully compiled: {:?} in {} ms", file_path, elapsed_ms);
        Ok(())
    }

    async fn remove_file_state(&self, path: &Path) {
        let file_key = path.to_string_lossy().to_string();
        let mut states = self.file_states.write().await;
        states.remove(&file_key);
    }

    /// Persist file states and KB states to disk for survival across restarts
    async fn save_states(&self) {
        if let Err(e) = tokio::fs::create_dir_all(&self.work_root).await {
            warn!("Failed to create work directory: {}", e);
            return;
        }

        // Persist file states
        let file_states_file = self.work_root.join("local_file_states.json");
        {
            let states = self.file_states.read().await;
            match serde_json::to_string_pretty(&*states) {
                Ok(json) => {
                    if let Err(e) = tokio::fs::write(&file_states_file, json).await {
                        warn!("Failed to persist file states: {}", e);
                    } else {
                        debug!("Persisted {} file states to disk", states.len());
                    }
                }
                Err(e) => warn!("Failed to serialize file states: {}", e),
            }
        }

        // Persist KB states
        let kb_states_file = self.work_root.join("local_kb_states.json");
        {
            let states = self.kb_states.read().await;
            match serde_json::to_string_pretty(&*states) {
                Ok(json) => {
                    if let Err(e) = tokio::fs::write(&kb_states_file, json).await {
                        warn!("Failed to persist KB states: {}", e);
                    } else {
                        debug!("Persisted {} KB states to disk", states.len());
                    }
                }
                Err(e) => warn!("Failed to serialize KB states: {}", e),
            }
        }
    }

    /// Load file states and KB states from disk
    async fn load_states(&self) {
        if let Err(e) = tokio::fs::create_dir_all(&self.work_root).await {
            warn!("Failed to create work directory: {}", e);
        }

        // Load file states
        let file_states_file = self.work_root.join("local_file_states.json");
        match tokio::fs::read_to_string(&file_states_file).await {
            Ok(json) => {
                match serde_json::from_str::<HashMap<String, LocalFileState>>(&json) {
                    Ok(states) => {
                        let count = states.len();
                        *self.file_states.write().await = states;
                        info!("Loaded {} persisted file states from disk", count);
                    }
                    Err(e) => warn!("Failed to parse persisted file states: {}", e),
                }
            }
            Err(_) => {
                debug!("No persisted file states found, starting fresh");
            }
        }

        // Load KB states
        let kb_states_file = self.work_root.join("local_kb_states.json");
        match tokio::fs::read_to_string(&kb_states_file).await {
            Ok(json) => {
                match serde_json::from_str::<HashMap<String, KbFolderState>>(&json) {
                    Ok(states) => {
                        let count = states.len();
                        *self.kb_states.write().await = states;
                        info!("Loaded {} persisted KB states from disk", count);
                    }
                    Err(e) => warn!("Failed to parse persisted KB states: {}", e),
                }
            }
            Err(_) => {
                debug!("No persisted KB states found, starting fresh");
            }
        }
    }

    pub async fn stop_monitoring(&self) {
        trace!("Stopping local file monitor");
        self.is_processing.store(false, Ordering::SeqCst);
        self.save_states().await;
    }
}

impl Clone for LocalFileMonitor {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            data_dir: self.data_dir.clone(),
            work_root: self.work_root.clone(),
            file_states: Arc::clone(&self.file_states),
            kb_states: Arc::clone(&self.kb_states),
            is_processing: Arc::clone(&self.is_processing),
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_manager: self.kb_manager.clone(),
        }
    }
}
