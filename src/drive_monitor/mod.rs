use crate::basic::compiler::BasicCompiler;
use crate::config::ConfigManager;
use crate::shared::state::AppState;
use aws_sdk_s3::Client;
use log::{info};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct FileState {
    pub etag: String,
}

pub struct DriveMonitor {
    state: Arc<AppState>,
    bucket_name: String,
    file_states: Arc<tokio::sync::RwLock<HashMap<String, FileState>>>,
    bot_id: uuid::Uuid,
}

impl DriveMonitor {
    pub fn new(state: Arc<AppState>, bucket_name: String, bot_id: uuid::Uuid) -> Self {
        Self {
            state,
            bucket_name,
            file_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            bot_id,
        }
    }

    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Drive Monitor service started for bucket: {}", self.bucket_name);


            let mut tick = interval(Duration::from_secs(30));

            loop {
                tick.tick().await;
                if let Err(e) = self.check_for_changes().await {
                    log::error!("Error checking for drive changes: {}", e);
                }
            }
        })
    }

    async fn check_for_changes(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = match &self.state.drive {
            Some(client) => client,
            None => return Ok(()),
        };

        self.check_gbdialog_changes(client).await?;
        self.check_gbot(client).await?;

        Ok(())
    }

    async fn check_gbdialog_changes(&self, client: &Client) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = ".gbdialog/";
        let mut current_files = HashMap::new();
        let mut continuation_token = None;

        loop {
            let list_objects = client
                .list_objects_v2()
                .bucket(&self.bucket_name.to_lowercase())
                .set_continuation_token(continuation_token)
                .send()
                .await?;

            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();
                let path_parts: Vec<&str> = path.split('/').collect();

                if path_parts.len() < 2 || !path_parts[0].ends_with(".gbdialog") {
                    continue;
                }

                if path.ends_with('/') || !path.ends_with(".bas") {
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
            } else {
                if let Err(e) = self.compile_tool(client, path).await {
                    log::error!("Failed to compile tool {}: {}", path, e);
                }
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
        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
        let mut continuation_token = None;

        loop {
            let list_objects = client
                .list_objects_v2()
                .bucket(&self.bucket_name.to_lowercase())
                .set_continuation_token(continuation_token)
                .send()
                .await?;

            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();
                let path_parts: Vec<&str> = path.split('/').collect();

                if path_parts.len() < 2 || !path_parts[0].ends_with(".gbot") {
                    continue;
                }

                if !path.ends_with("config.csv") {
                    continue;
                }

                match client.head_object().bucket(&self.bucket_name).key(&path).send().await {
                    Ok(_head_res) => {
                        let response = client.get_object().bucket(&self.bucket_name).key(&path).send().await?;
                        let bytes = response.body.collect().await?.into_bytes();
                        let csv_content = String::from_utf8(bytes.to_vec())
                            .map_err(|e| format!("UTF-8 error in {}: {}", path, e))?;

                        let llm_lines: Vec<_> = csv_content
                            .lines()
                            .filter(|line| line.trim_start().starts_with("llm-"))
                            .collect();

                        if !llm_lines.is_empty() {
                            use crate::llm::local::ensure_llama_servers_running;
                            let mut restart_needed = false;

                            for line in llm_lines {
                                let parts: Vec<&str> = line.split(',').collect();
                                if parts.len() >= 2 {
                                    let key = parts[0].trim();
                                    let new_value = parts[1].trim();

                                    match config_manager.get_config(&self.bot_id, key, None) {
                                        Ok(old_value) => {
                                            if old_value != new_value {
                                                info!("Detected change in {} (old: {}, new: {})", key, old_value, new_value);
                                                restart_needed = true;
                                            }
                                        }
                                        Err(_) => {
                                            restart_needed = true;
                                        }
                                    }
                                }
                            }

                            let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);

                            if restart_needed {
                                if let Err(e) = ensure_llama_servers_running(&self.state).await {
                                    log::error!("Failed to restart LLaMA servers after llm- config change: {}", e);
                                }
                            }
                        } else {
                            let _ = config_manager.sync_gbot_config(&self.bot_id, &csv_content);
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

        Ok(())
    }

    async fn broadcast_theme_change(&self, csv_content: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
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
                    "theme-color1" => theme_data["data"]["color1"] = serde_json::Value::String(value.to_string()),
                    "theme-color2" => theme_data["data"]["color2"] = serde_json::Value::String(value.to_string()),
                    "theme-logo" => theme_data["data"]["logo_url"] = serde_json::Value::String(value.to_string()),
                    "theme-title" => theme_data["data"]["title"] = serde_json::Value::String(value.to_string()),
                    "theme-logo-text" => theme_data["data"]["logo_text"] = serde_json::Value::String(value.to_string()),
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
                message_type: 2,
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

    async fn compile_tool(&self, client: &Client, file_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Fetching object from S3: bucket={}, key={}", &self.bucket_name, file_path);

        let response = match client.get_object().bucket(&self.bucket_name).key(file_path).send().await {
            Ok(res) => {
                info!("Successfully fetched object from S3: bucket={}, key={}, size={}", 
                    &self.bucket_name, file_path, res.content_length().unwrap_or(0));
                res
            }
            Err(e) => {
                log::error!("Failed to fetch object from S3: bucket={}, key={}, error={:?}", 
                    &self.bucket_name, file_path, e);
                return Err(e.into());
            }
        };

        let bytes = response.body.collect().await?.into_bytes();
        let source_content = String::from_utf8(bytes.to_vec())?;

        let tool_name = file_path
            .split('/')
            .last()
            .unwrap_or(file_path)
            .strip_suffix(".bas")
            .unwrap_or(file_path)
            .to_string();

        let bot_name = self.bucket_name.strip_suffix(".gbai").unwrap_or(&self.bucket_name);
        let work_dir = format!("./work/{}.gbai/{}.gbdialog", bot_name, bot_name);

        std::fs::create_dir_all(&work_dir)?;

        let local_source_path = format!("{}/{}.bas", work_dir, tool_name);
        std::fs::write(&local_source_path, &source_content)?;

        let mut compiler = BasicCompiler::new(Arc::clone(&self.state), self.bot_id);
        let result = compiler.compile_file(&local_source_path, &work_dir)?;

        if let Some(mcp_tool) = result.mcp_tool {
            info!("MCP tool definition generated with {} parameters", 
                mcp_tool.input_schema.properties.len());
        }

        Ok(())
    }
}
