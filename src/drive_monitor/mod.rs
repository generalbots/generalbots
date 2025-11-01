use crate::basic::compiler::BasicCompiler;
use crate::config::ConfigManager;
use crate::kb::embeddings;
use crate::kb::qdrant_client;
use crate::shared::state::AppState;
use aws_sdk_s3::Client;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct FileState {
    pub path: String,
    pub size: i64,
    pub etag: String,
    pub last_modified: Option<String>,
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
            info!(
                "Drive Monitor service started for bucket: {}",
                self.bucket_name
            );
            let mut tick = interval(Duration::from_secs(30));
            loop {
                tick.tick().await;
                if let Err(e) = self.check_for_changes().await {
                    error!("Error checking for drive changes: {}", e);
                }
            }
        })
    }

    async fn check_for_changes(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = match &self.state.s3_client {
            Some(client) => client,
            None => {
                return Ok(());
            }
        };

        self.check_gbdialog_changes(client).await?;
        self.check_gbkb_changes(client).await?;

        if let Err(e) = self.check_gbot(client).await {
            error!("Error checking default bot config: {}", e);
        }

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
            let list_objects = client
                .list_objects_v2()
                .bucket(&self.bucket_name.to_lowercase())
                .set_continuation_token(continuation_token)
                .send()
                .await?;
            debug!("List objects result: {:?}", list_objects);

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
                    path: path.clone(),
                    size: obj.size().unwrap_or(0),
                    etag: obj.e_tag().unwrap_or_default().to_string(),
                    last_modified: obj.last_modified().map(|dt| dt.to_string()),
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
                        error!("Failed to compile tool {}: {}", path, e);
                    }
                }
            } else {
                if let Err(e) = self.compile_tool(client, path).await {
                    error!("Failed to compile tool {}: {}", path, e);
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

    async fn check_gbkb_changes(
        &self,
        client: &Client,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = ".gbkb/";

        let mut current_files = HashMap::new();

        let mut continuation_token = None;
        loop {
            let list_objects = client
                .list_objects_v2()
                .bucket(&self.bucket_name.to_lowercase())
                .prefix(prefix)
                .set_continuation_token(continuation_token)
                .send()
                .await?;
            debug!("List objects result: {:?}", list_objects);

            for obj in list_objects.contents.unwrap_or_default() {
                let path = obj.key().unwrap_or_default().to_string();

                let path_parts: Vec<&str> = path.split('/').collect();
                if path_parts.len() < 2 || !path_parts[0].ends_with(".gbkb") {
                    continue;
                }

                if path.ends_with('/') {
                    continue;
                }

                let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
                if !["pdf", "txt", "md", "docx"].contains(&ext.as_str()) {
                    continue;
                }

                let file_state = FileState {
                    path: path.clone(),
                    size: obj.size().unwrap_or(0),
                    etag: obj.e_tag().unwrap_or_default().to_string(),
                    last_modified: obj.last_modified().map(|dt| dt.to_string()),
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
                    if let Err(e) = self.index_document(client, path).await {
                        error!("Failed to index document {}: {}", path, e);
                    }
                }
            } else {
                if let Err(e) = self.index_document(client, path).await {
                    error!("Failed to index document {}: {}", path, e);
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

                debug!("Checking config file at path: {}", path);
                match client
                    .head_object()
                    .bucket(&self.bucket_name)
                    .key(&path)
                    .send()
                    .await
                {
                    Ok(head_res) => {
                        debug!(
                            "HeadObject successful for {}, metadata: {:?}",
                            path, head_res
                        );
                        let response = client
                            .get_object()
                            .bucket(&self.bucket_name)
                            .key(&path)
                            .send()
                            .await?; 
                        debug!(
                            "GetObject successful for {}, content length: {}",
                            path,
                            response.content_length().unwrap_or(0)
                        );


                        let bytes = response.body.collect().await?.into_bytes();
                        debug!("Collected {} bytes for {}", bytes.len(), path);
                        let csv_content = String::from_utf8(bytes.to_vec())
                            .map_err(|e| format!("UTF-8 error in {}: {}", path, e))?;
                        debug!("Found {}: {} bytes", path, csv_content.len());



                            // Restart LLaMA servers only if llm- properties changed
                            let llm_lines: Vec<_> = csv_content
                                .lines()
                                .filter(|line| line.trim_start().starts_with("llm-"))
                                .collect();

                            if !llm_lines.is_empty() {
                                use crate::llm_legacy::llm_local::ensure_llama_servers_running;
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
                                                info!("New llm- property detected: {}", key);
                                                restart_needed = true;
                                            }
                                        }
                                    }
                                }

                                if restart_needed {
                                    info!("Detected llm- configuration change, restarting LLaMA servers...");
                                    if let Err(e) = ensure_llama_servers_running(&self.state).await {
                                        error!("Failed to restart LLaMA servers after llm- config change: {}", e);
                                    }
                                } else {
                                    info!("No llm- property changes detected; skipping LLaMA server restart.");
                                }
                            config_manager.sync_gbot_config(&self.bot_id, &csv_content);
                        }
                    }
                    Err(e) => {
                        debug!("Config file {} not found or inaccessible: {}", path, e);
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

    async fn compile_tool(
        &self,
        client: &Client,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        debug!(
            "Fetching object from S3: bucket={}, key={}",
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
                debug!(
                    "Successfully fetched object from S3: bucket={}, key={}, size={}",
                    &self.bucket_name,
                    file_path,
                    res.content_length().unwrap_or(0)
                );
                res
            }
            Err(e) => {
                error!(
                    "Failed to fetch object from S3: bucket={}, key={}, error={:?}",
                    &self.bucket_name, file_path, e
                );
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

        let bot_name = self
            .bucket_name
            .strip_suffix(".gbai")
            .unwrap_or(&self.bucket_name);
        let work_dir = format!("./work/{}.gbai/{}.gbdialog", bot_name, bot_name);
        std::fs::create_dir_all(&work_dir)?;

        let local_source_path = format!("{}/{}.bas", work_dir, tool_name);
        std::fs::write(&local_source_path, &source_content)?;

        let compiler = BasicCompiler::new(Arc::clone(&self.state));
        let result = compiler.compile_file(&local_source_path, &work_dir)?;

        if let Some(mcp_tool) = result.mcp_tool {
            info!(
                "MCP tool definition generated with {} parameters",
                mcp_tool.input_schema.properties.len()
            );
        }

        if result.openai_tool.is_some() {
            debug!("OpenAI tool definition generated");
        }

        Ok(())
    }

    async fn index_document(
        &self,
        client: &Client,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() < 3 {
            warn!("Invalid KB path structure: {}", file_path);
            return Ok(());
        }

        let collection_name = parts[1];
        let response = client
            .get_object()
            .bucket(&self.bucket_name)
            .key(file_path)
            .send()
            .await?;
        let bytes = response.body.collect().await?.into_bytes();

        let text_content = self.extract_text(file_path, &bytes)?;
        if text_content.trim().is_empty() {
            warn!("No text extracted from: {}", file_path);
            return Ok(());
        }

        info!(
            "Extracted {} characters from {}",
            text_content.len(),
            file_path
        );

        let qdrant_collection = format!("kb_default_{}", collection_name);
        qdrant_client::ensure_collection_exists(&self.state, &qdrant_collection).await?;

        embeddings::index_document(&self.state, &qdrant_collection, file_path, &text_content)
            .await?;

        Ok(())
    }

    fn extract_text(
        &self,
        file_path: &str,
        content: &[u8],
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let path_lower = file_path.to_ascii_lowercase();
        if path_lower.ends_with(".pdf") {
            match pdf_extract::extract_text_from_mem(content) {
                Ok(text) => Ok(text),
                Err(e) => {
                    error!("PDF extraction failed for {}: {}", file_path, e);
                    Err(format!("PDF extraction failed: {}", e).into())
                }
            }
        } else if path_lower.ends_with(".txt") || path_lower.ends_with(".md") {
            String::from_utf8(content.to_vec())
                .map_err(|e| format!("UTF-8 decoding failed: {}", e).into())
        } else {
            String::from_utf8(content.to_vec())
                .map_err(|e| format!("Unsupported file format or UTF-8 error: {}", e).into())
        }
    }

    pub async fn clear_state(&self) {
        let mut states = self.file_states.write().await;
        states.clear();
    }
}
