use crate::basic::compiler::BasicCompiler;
use crate::shared::state::AppState;
use log::{debug, error, info, warn};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalFileState {
    modified: SystemTime,
    size: u64,
}

pub struct LocalFileMonitor {
    state: Arc<AppState>,
    data_dir: PathBuf,
    file_states: Arc<RwLock<HashMap<String, LocalFileState>>>,
    is_processing: Arc<AtomicBool>,
}

impl LocalFileMonitor {
    pub fn new(state: Arc<AppState>) -> Self {
        // Use ~/data as the base directory
        let data_dir = PathBuf::from(std::env::var("HOME")
            .unwrap_or_else(|_| ".".to_string()))
            .join("data");

        info!("[LOCAL_MONITOR] Initializing with data_dir: {:?}", data_dir);

        Self {
            state,
            data_dir,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start_monitoring(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("[LOCAL_MONITOR] Starting local file monitor for ~/data/*.gbai directories");

        // Create data directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&self.data_dir).await {
            warn!("[LOCAL_MONITOR] Failed to create data directory: {}", e);
        }

        // Initial scan of all .gbai directories
        self.scan_and_compile_all().await?;

        self.is_processing.store(true, Ordering::SeqCst);

        // Spawn the monitoring loop
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.monitoring_loop().await;
        });

        info!("[LOCAL_MONITOR] Local file monitor started");
        Ok(())
    }

    async fn monitoring_loop(&self) {
        info!("[LOCAL_MONITOR] Starting monitoring loop");

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
                error!("[LOCAL_MONITOR] Failed to create watcher: {}. Falling back to polling.", e);
                // Fall back to polling if watcher creation fails
                self.polling_loop().await;
                return;
            }
        };

        // Watch the data directory
        if let Err(e) = watcher.watch(&self.data_dir, RecursiveMode::Recursive) {
            warn!("[LOCAL_MONITOR] Failed to watch directory {:?}: {}. Using polling fallback.", self.data_dir, e);
            drop(watcher);
            self.polling_loop().await;
            return;
        }

        info!("[LOCAL_MONITOR] Watching directory: {:?}", self.data_dir);

        while self.is_processing.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Process events from the watcher
            while let Ok(event) = rx.try_recv() {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any => {
                        for path in &event.paths {
                            if self.is_gbdialog_file(path) {
                                info!("[LOCAL_MONITOR] Detected change: {:?}", path);
                                if let Err(e) = self.compile_local_file(path).await {
                                    error!("[LOCAL_MONITOR] Failed to compile {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in &event.paths {
                            if self.is_gbdialog_file(path) {
                                info!("[LOCAL_MONITOR] File removed: {:?}", path);
                                self.remove_file_state(path).await;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Periodic scan to catch any missed changes
            if let Err(e) = self.scan_and_compile_all().await {
                error!("[LOCAL_MONITOR] Scan failed: {}", e);
            }
        }

        info!("[LOCAL_MONITOR] Monitoring loop ended");
    }

    async fn polling_loop(&self) {
        info!("[LOCAL_MONITOR] Using polling fallback (checking every 10s)");

        while self.is_processing.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(10)).await;

            if let Err(e) = self.scan_and_compile_all().await {
                error!("[LOCAL_MONITOR] Scan failed: {}", e);
            }
        }
    }

    fn is_gbdialog_file(&self, path: &Path) -> bool {
        // Check if path is something like ~/data/*.gbai/.gbdialog/*.bas
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("bas"))
            .unwrap_or(false)
            && path.ancestors()
                .any(|p| p.ends_with(".gbdialog"))
    }

    async fn scan_and_compile_all(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug!("[LOCAL_MONITOR] Scanning ~/data for .gbai directories");

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

                // Look for .gbdialog folder inside
                let gbdialog_path = path.join(".gbdialog");
                if gbdialog_path.exists() {
                    self.compile_gbdialog(&bot_name, &gbdialog_path).await?;
                }
            }
        }

        Ok(())
    }

    async fn compile_gbdialog(&self, bot_name: &str, gbdialog_path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug!("[LOCAL_MONITOR] Processing .gbdialog for bot: {}", bot_name);

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
                    info!("[LOCAL_MONITOR] Compiling: {:?}", path);
                    if let Err(e) = self.compile_local_file(&path).await {
                        error!("[LOCAL_MONITOR] Failed to compile {:?}: {}", path, e);
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

        // Extract bot name from path like ~/data/cristo.gbai/.gbdialog/file.bas
        let bot_name = file_path
            .ancestors()
            .find(|p| p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("gbai")).unwrap_or(false))
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Create work directory structure
        let work_dir = self.data_dir.join(format!("{}.gbai", bot_name));

        // Read the file content
        let source_content = tokio::fs::read_to_string(file_path).await?;

        // Compile the file
        let state_clone = Arc::clone(&self.state);
        let work_dir_clone = work_dir.clone();
        let tool_name_clone = tool_name.to_string();
        let source_content_clone = source_content.clone();
        let bot_id = uuid::Uuid::new_v4(); // Generate a bot ID or get from somewhere

        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&work_dir_clone)?;
            let local_source_path = work_dir_clone.join(format!("{}.bas", tool_name_clone));
            std::fs::write(&local_source_path, &source_content_clone)?;
            let mut compiler = BasicCompiler::new(state_clone, bot_id);
            let result = compiler.compile_file(local_source_path.to_str().unwrap(), work_dir_clone.to_str().unwrap())?;
            if let Some(mcp_tool) = result.mcp_tool {
                info!(
                    "[LOCAL_MONITOR] MCP tool generated with {} parameters",
                    mcp_tool.input_schema.properties.len()
                );
            }
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        })
        .await??;

        info!("[LOCAL_MONITOR] Successfully compiled: {:?}", file_path);
        Ok(())
    }

    async fn remove_file_state(&self, path: &Path) {
        let file_key = path.to_string_lossy().to_string();
        let mut states = self.file_states.write().await;
        states.remove(&file_key);
    }

    pub async fn stop_monitoring(&self) {
        info!("[LOCAL_MONITOR] Stopping local file monitor");
        self.is_processing.store(false, Ordering::SeqCst);
        self.file_states.write().await.clear();
    }
}

impl Clone for LocalFileMonitor {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            data_dir: self.data_dir.clone(),
            file_states: Arc::clone(&self.file_states),
            is_processing: Arc::clone(&self.is_processing),
        }
    }
}
