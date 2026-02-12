// Config file watcher - monitors config.csv files and reloads them when changed
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use crate::core::shared::state::AppState;

/// Tracks file state to detect changes
#[derive(Debug, Clone)]
struct FileState {
    modified: SystemTime,
    size: u64,
}

/// Config file watcher - monitors config.csv files in data directory
pub struct ConfigWatcher {
    data_dir: PathBuf,
    file_states: Arc<RwLock<HashMap<PathBuf, FileState>>>,
    state: Arc<AppState>,
}

impl ConfigWatcher {
    pub fn new(data_dir: PathBuf, state: Arc<AppState>) -> Self {
        Self {
            data_dir,
            file_states: Arc::new(RwLock::new(HashMap::new())),
            state,
        }
    }

    /// Start watching for config.csv changes
    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Starting config file watcher for: {}", self.data_dir.display());

            // Initial scan
            if let Err(e) = self.scan_configs().await {
                error!("Initial config scan failed: {}", e);
            }

            // Set up periodic polling (every 5 seconds)
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;
                if let Err(e) = self.scan_configs().await {
                    error!("Config scan failed: {}", e);
                }
            }
        })
    }

    /// Scan all config.csv files in the data directory and reload changed files
    async fn scan_configs(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Pattern: data_dir/*.gbai/*.gbot/config.csv
        let entries = match tokio::fs::read_dir(&self.data_dir).await {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read data directory {}: {}", self.data_dir.display(), e);
                return Err(e.into());
            }
        };

        let mut entries = entries;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Check if it's a .gbai directory
            if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("gbai") {
                let bot_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                // Look for *.gbot/config.csv
                let gbot_pattern = path.join(format!("{}.gbot", bot_name));
                let config_path = gbot_pattern.join("config.csv");

                if config_path.exists() {
                    if let Err(e) = self.check_and_reload_config(&config_path, bot_name).await {
                        error!("Failed to check config {:?}: {}", config_path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a config file has changed and reload it
    async fn check_and_reload_config(
        &self,
        config_path: &Path,
        bot_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = tokio::fs::metadata(config_path).await?;
        let modified = metadata.modified()?;
        let size = metadata.len();

        let mut states = self.file_states.write().await;

        // Check if file has changed
        let has_changed = match states.get(config_path) {
            Some(state) => state.modified != modified || state.size != size,
            None => true,
        };

        if has_changed {
            info!("Config file changed: {:?}", config_path);

            // Reload the config
            match tokio::fs::read_to_string(config_path).await {
                Ok(content) => {
                    let conn = self.state.conn.clone();
                    let bot_name_owned = bot_name.to_string();
                    let bot_name_for_log = bot_name_owned.clone();
                    let bot_name_for_llm = bot_name_owned.clone();
                    let content_clone = content.clone();

                    // Sync to database
                    let sync_result = tokio::task::spawn_blocking(move || {
                        let mut db_conn = conn.get()
                            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

                        // Get bot_id by name
                        let bot_id = crate::core::bot::get_bot_id_by_name(&mut db_conn, &bot_name_owned)
                            .map_err(|e| format!("Failed to get bot_id for '{}': {}", bot_name_owned, e))?;

                        // Use ConfigManager's sync_gbot_config (public method)
                        crate::core::config::ConfigManager::new(conn)
                            .sync_gbot_config(&bot_id, &content_clone)
                    }).await;

                    match sync_result {
                        Ok(Ok(updated)) => {
                            info!("Reloaded config for bot '{}' ({} entries updated)", bot_name_for_log, updated);

                            // Trigger immediate LLM config refresh
                            if let Some(dynamic_llm) = &self.state.dynamic_llm_provider {
                                // Get the updated config values
                                let pool = self.state.conn.clone();
                                let llm_config = tokio::task::spawn_blocking(move || {
                                    let mut db_conn = pool.get()
                                        .map_err(|e| format!("DB connection error: {}", e))?;

                                    let bot_id = crate::core::bot::get_bot_id_by_name(&mut db_conn, &bot_name_for_llm)
                                        .map_err(|e| format!("Get bot_id error: {}", e))?;

                                    let config_manager = crate::core::config::ConfigManager::new(pool);
                                    let llm_server = config_manager.get_config(&bot_id, "llm-server", None)
                                        .unwrap_or_default();
                                    let llm_model = config_manager.get_config(&bot_id, "llm-model", None)
                                        .unwrap_or_default();
                                    let llm_key = config_manager.get_config(&bot_id, "llm-key", None)
                                        .unwrap_or_default();

                                    Ok::<_, String>((llm_server, llm_model, llm_key))
                                }).await;

                                if let Ok(Ok((llm_server, llm_model, _llm_key))) = llm_config {
                                    if !llm_server.is_empty() {
                                        // Handle both local embedded (llm-server=true) and external API endpoints
                                        if llm_server.eq_ignore_ascii_case("true") {
                                            // Local embedded LLM server - trigger local LLM initialization
                                            info!("ConfigWatcher: Local LLM server enabled for bot '{}', model={}", bot_name_for_log, llm_model);
                                            // The local LLM will be initialized by LocalFileMonitor on next check
                                            // Just trigger a config refresh to notify components
                                        } else {
                                            // External LLM API endpoint - parse URL and endpoint path
                                            let (base_url, endpoint_path) = if llm_server.contains("/chat/completions") || llm_server.contains("/v1/") {
                                                // Extract base URL up to the path
                                                if let Some(pos) = llm_server.find("/v1/chat/completions") {
                                                    (&llm_server[..pos], Some(&llm_server[pos..]))
                                                } else if let Some(pos) = llm_server.find("/chat/completions") {
                                                    (&llm_server[..pos], Some(&llm_server[pos..]))
                                                } else {
                                                    (llm_server.as_str(), None)
                                                }
                                            } else {
                                                (llm_server.as_str(), None)
                                            };

                                            info!("ConfigWatcher: Refreshing LLM provider with URL={}, model={}, endpoint={:?}", base_url, llm_model, endpoint_path);
                                            dynamic_llm.update_from_config(base_url, Some(llm_model), endpoint_path.map(|s| s.to_string())).await;
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Failed to reload config for bot '{}': {}", bot_name_for_log, e);
                        }
                        Err(e) => {
                            error!("Task failed for bot '{}': {}", bot_name_for_log, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read config file {:?}: {}", config_path, e);
                    return Err(e.into());
                }
            }

            // Update state
            states.insert(config_path.to_path_buf(), FileState { modified, size });
        }

        Ok(())
    }
}
