use crate::basic::compiler::BasicCompiler;
use crate::kb::embeddings;
use crate::kb::qdrant_client;
use crate::shared::state::AppState;
use log::{debug, error, info, warn};
use opendal::Operator;
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
}

impl DriveMonitor {
    pub fn new(state: Arc<AppState>, bucket_name: String) -> Self {
        Self {
            state,
            bucket_name,
            file_states: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Drive Monitor service started for bucket: {}", self.bucket_name);
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
        let op = match &self.state.s3_operator {
            Some(op) => op,
            None => {
                return Ok(());
            }
        };

        self.check_gbdialog_changes(op).await?;
        self.check_gbkb_changes(op).await?;
        
        if let Err(e) = self.check_default_gbot(op).await {
            error!("Error checking default bot config: {}", e);
        }

        Ok(())
    }

    async fn check_gbdialog_changes(
        &self,
        op: &Operator,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = ".gbdialog/";
        
        let mut current_files = HashMap::new();
        
        let mut lister = op.lister_with(prefix).recursive(true).await?;
        while let Some(entry) = lister.try_next().await? {
            let path = entry.path().to_string();
            
            if path.ends_with('/') || !path.ends_with(".bas") {
                continue;
            }

            let meta = op.stat(&path).await?;
            let file_state = FileState {
                path: path.clone(),
                size: meta.content_length() as i64,
                etag: meta.etag().unwrap_or_default().to_string(),
                last_modified: meta.last_modified().map(|dt| dt.to_rfc3339()),
            };
            current_files.insert(path, file_state);
        }

        let mut file_states = self.file_states.write().await;
        for (path, current_state) in current_files.iter() {
            if let Some(previous_state) = file_states.get(path) {
                if current_state.etag != previous_state.etag {
                    if let Err(e) = self.compile_tool(op, path).await {
                        error!("Failed to compile tool {}: {}", path, e);
                    }
                }
            } else {
                if let Err(e) = self.compile_tool(op, path).await {
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
        op: &Operator,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = ".gbkb/";
        
        let mut current_files = HashMap::new();
        
        let mut lister = op.lister_with(prefix).recursive(true).await?;
        while let Some(entry) = lister.try_next().await? {
            let path = entry.path().to_string();
            
            if path.ends_with('/') {
                continue;
            }

            let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
            if !["pdf", "txt", "md", "docx"].contains(&ext.as_str()) {
                continue;
            }

            let meta = op.stat(&path).await?;
            let file_state = FileState {
                path: path.clone(),
                size: meta.content_length() as i64,
                etag: meta.etag().unwrap_or_default().to_string(),
                last_modified: meta.last_modified().map(|dt| dt.to_rfc3339()),
            };
            current_files.insert(path, file_state);
        }

        let mut file_states = self.file_states.write().await;
        for (path, current_state) in current_files.iter() {
            if let Some(previous_state) = file_states.get(path) {
                if current_state.etag != previous_state.etag {
                    if let Err(e) = self.index_document(op, path).await {
                        error!("Failed to index document {}: {}", path, e);
                    }
                }
            } else {
                if let Err(e) = self.index_document(op, path).await {
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

    async fn check_default_gbot(
        &self,
        op: &Operator,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let prefix = format!("{}default.gbot/", self.bucket_name);
        let config_key = format!("{}config.csv", prefix);
        
        match op.stat(&config_key).await {
            Ok(_) => {
                let content = op.read(&config_key).await?;
                let csv_content = String::from_utf8(content.to_vec())
                    .map_err(|e| format!("UTF-8 error in config.csv: {}", e))?;
                debug!("Found config.csv: {} bytes", csv_content.len());
                Ok(())
            }
            Err(e) => {
                debug!("Config file not found or inaccessible: {}", e);
                Ok(())
            }
        }
    }

    async fn compile_tool(
        &self,
        op: &Operator,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let content = op.read(file_path).await?;
        let source_content = String::from_utf8(content.to_vec())?;

        let tool_name = file_path
            .strip_prefix(".gbdialog/")
            .unwrap_or(file_path)
            .strip_suffix(".bas")
            .unwrap_or(file_path)
            .to_string();

        let bot_name = self
            .bucket_name
            .strip_suffix(".gbai")
            .unwrap_or(&self.bucket_name);
        let work_dir = format!("./work/{}.gbai/.gbdialog", bot_name);
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
        op: &Operator,
        file_path: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() < 3 {
            warn!("Invalid KB path structure: {}", file_path);
            return Ok(());
        }

        let collection_name = parts[1];
        let content = op.read(file_path).await?;
        let bytes = content.to_vec();
        
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
