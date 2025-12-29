use anyhow::Result;
use calamine::Reader;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[cfg(feature = "vectordb")]
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

#[cfg(feature = "vectordb")]
use qdrant_client::{
    qdrant::{Distance, PointStruct, VectorParams},
    Qdrant,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDocument {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub bucket: String,
    pub content_text: String,
    pub content_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    pub mime_type: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchQuery {
    pub query_text: String,
    pub bucket: Option<String>,
    pub file_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub file: FileDocument,
    pub score: f32,
    pub snippet: String,
    pub highlights: Vec<String>,
}

pub struct UserDriveVectorDB {
    user_id: Uuid,
    bot_id: Uuid,
    collection_name: String,
    db_path: PathBuf,
    #[cfg(feature = "vectordb")]
    client: Option<Arc<Qdrant>>,
}

impl UserDriveVectorDB {
    pub fn new(user_id: Uuid, bot_id: Uuid, db_path: PathBuf) -> Self {
        let collection_name = format!("drive_{}_{}", bot_id, user_id);

        Self {
            user_id,
            bot_id,
            collection_name,
            db_path,
            #[cfg(feature = "vectordb")]
            client: None,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn bot_id(&self) -> Uuid {
        self.bot_id
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    #[cfg(feature = "vectordb")]
    pub async fn initialize(&mut self, qdrant_url: &str) -> Result<()> {
        log::trace!(
            "Initializing vectordb, fallback path: {}",
            self.db_path.display()
        );
        let client = Qdrant::from_url(qdrant_url).build()?;

        let collections = client.list_collections().await?;
        let exists = {
            let collections_guard = collections.collections;
            collections_guard
                .iter()
                .any(|c| c.name == self.collection_name)
        };

        if !exists {
            client
                .create_collection(
                    qdrant_client::qdrant::CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParams {
                            size: 1536,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        }),
                )
                .await?;

            log::info!("Initialized vector DB collection: {}", self.collection_name);
        }

        self.client = Some(Arc::new(client));
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn initialize(&mut self, _qdrant_url: &str) -> Result<()> {
        log::warn!("Vector DB feature not enabled, using fallback storage");
        fs::create_dir_all(&self.db_path).await?;
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn index_file(&self, file: &FileDocument, embedding: Vec<f32>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let payload: qdrant_client::Payload = serde_json::to_value(file)?
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, qdrant_client::qdrant::Value::from(v.to_string())))
            .collect::<std::collections::HashMap<_, _>>()
            .into();

        let point = PointStruct::new(file.id.clone(), embedding, payload);

        client
            .upsert_points(qdrant_client::qdrant::UpsertPointsBuilder::new(
                &self.collection_name,
                vec![point],
            ))
            .await?;

        log::debug!("Indexed file: {} - {}", file.id, file.file_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn index_file(&self, file: &FileDocument, _embedding: Vec<f32>) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", file.id));
        let json = serde_json::to_string_pretty(file)?;
        fs::write(file_path, json).await?;
        Ok(())
    }

    pub async fn index_files_batch(&self, files: &[(FileDocument, Vec<f32>)]) -> Result<()> {
        #[cfg(feature = "vectordb")]
        {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

            let points: Vec<PointStruct> = files
                .iter()
                .filter_map(|(file, embedding)| {
                    serde_json::to_value(file).ok().and_then(|v| {
                        v.as_object().map(|m| {
                            let payload: qdrant_client::Payload = m
                                .clone()
                                .into_iter()
                                .map(|(k, v)| {
                                    (k, qdrant_client::qdrant::Value::from(v.to_string()))
                                })
                                .collect::<std::collections::HashMap<_, _>>()
                                .into();
                            PointStruct::new(file.id.clone(), embedding.clone(), payload)
                        })
                    })
                })
                .collect();

            if !points.is_empty() {
                client
                    .upsert_points(qdrant_client::qdrant::UpsertPointsBuilder::new(
                        &self.collection_name,
                        points,
                    ))
                    .await?;
            }
        }

        #[cfg(not(feature = "vectordb"))]
        {
            for (file, embedding) in files {
                self.index_file(file, embedding.clone()).await?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn search(
        &self,
        query: &FileSearchQuery,
        query_embedding: Vec<f32>,
    ) -> Result<Vec<FileSearchResult>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let filter =
            if query.bucket.is_some() || query.file_type.is_some() || !query.tags.is_empty() {
                let mut conditions = Vec::new();

                if let Some(bucket) = &query.bucket {
                    conditions.push(qdrant_client::qdrant::Condition::matches(
                        "bucket",
                        bucket.clone(),
                    ));
                }

                if let Some(file_type) = &query.file_type {
                    conditions.push(qdrant_client::qdrant::Condition::matches(
                        "file_type",
                        file_type.clone(),
                    ));
                }

                for tag in &query.tags {
                    conditions.push(qdrant_client::qdrant::Condition::matches(
                        "tags",
                        tag.clone(),
                    ));
                }

                if conditions.is_empty() {
                    None
                } else {
                    Some(qdrant_client::qdrant::Filter::must(conditions))
                }
            } else {
                None
            };

        let mut search_builder = qdrant_client::qdrant::SearchPointsBuilder::new(
            &self.collection_name,
            query_embedding,
            query.limit as u64,
        )
        .with_payload(true);

        if let Some(f) = filter {
            search_builder = search_builder.filter(f);
        }

        let search_result = client.search_points(search_builder).await?;

        let mut results = Vec::new();
        for point in search_result.result {
            let payload = &point.payload;
            if !payload.is_empty() {
                let get_str = |key: &str| -> String {
                    payload
                        .get(key)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                };

                let file = FileDocument {
                    id: get_str("id"),
                    file_path: get_str("file_path"),
                    file_name: get_str("file_name"),
                    file_type: get_str("file_type"),
                    file_size: payload
                        .get("file_size")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(0) as u64,
                    bucket: get_str("bucket"),
                    content_text: get_str("content_text"),
                    content_summary: payload
                        .get("content_summary")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    created_at: chrono::Utc::now(),
                    modified_at: chrono::Utc::now(),
                    indexed_at: chrono::Utc::now(),
                    mime_type: payload
                        .get("mime_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    tags: vec![],
                };

                let snippet = Self::create_snippet(&file.content_text, &query.query_text, 200);
                let highlights = Self::extract_highlights(&file.content_text, &query.query_text, 3);

                results.push(FileSearchResult {
                    file,
                    score: point.score,
                    snippet,
                    highlights,
                });
            }
        }

        Ok(results)
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn search(
        &self,
        query: &FileSearchQuery,
        _query_embedding: Vec<f32>,
    ) -> Result<Vec<FileSearchResult>> {
        let mut results = Vec::new();
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(file) = serde_json::from_str::<FileDocument>(&content) {
                    if let Some(bucket) = &query.bucket {
                        if &file.bucket != bucket {
                            continue;
                        }
                    }

                    if let Some(file_type) = &query.file_type {
                        if &file.file_type != file_type {
                            continue;
                        }
                    }

                    let query_lower = query.query_text.to_lowercase();
                    if file.file_name.to_lowercase().contains(&query_lower)
                        || file.content_text.to_lowercase().contains(&query_lower)
                        || file
                            .content_summary
                            .as_ref()
                            .is_some_and(|s| s.to_lowercase().contains(&query_lower))
                    {
                        let snippet =
                            Self::create_snippet(&file.content_text, &query.query_text, 200);
                        let highlights =
                            Self::extract_highlights(&file.content_text, &query.query_text, 3);

                        results.push(FileSearchResult {
                            file,
                            score: 1.0,
                            snippet,
                            highlights,
                        });
                    }
                }

                if results.len() >= query.limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn create_snippet(content: &str, query: &str, max_length: usize) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();

        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(max_length / 2);
            let end = (pos + query.len() + max_length / 2).min(content.len());
            let snippet = &content[start..end];

            if start > 0 && end < content.len() {
                format!("...{}...", snippet)
            } else if start > 0 {
                format!("...{}", snippet)
            } else if end < content.len() {
                format!("{}...", snippet)
            } else {
                snippet.to_string()
            }
        } else if content.len() > max_length {
            format!("{}...", &content[..max_length])
        } else {
            content.to_string()
        }
    }

    fn extract_highlights(content: &str, query: &str, max_highlights: usize) -> Vec<String> {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        let mut highlights = Vec::new();
        let mut pos = 0;

        while let Some(found_pos) = content_lower[pos..].find(&query_lower) {
            let actual_pos = pos + found_pos;
            let start = actual_pos.saturating_sub(40);
            let end = (actual_pos + query.len() + 40).min(content.len());

            highlights.push(content[start..end].to_string());

            if highlights.len() >= max_highlights {
                break;
            }

            pos = actual_pos + query.len();
        }

        highlights
    }

    #[cfg(feature = "vectordb")]
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client
            .delete_points(
                qdrant_client::qdrant::DeletePointsBuilder::new(&self.collection_name).points(
                    vec![qdrant_client::qdrant::PointId::from(file_id.to_string())],
                ),
            )
            .await?;

        log::debug!("Deleted file from index: {}", file_id);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", file_id));
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn get_count(&self) -> Result<u64> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let info = client.collection_info(self.collection_name.clone()).await?;

        Ok(info
            .result
            .ok_or_else(|| anyhow::anyhow!("No result from collection info"))?
            .points_count
            .unwrap_or(0))
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn get_count(&self) -> Result<u64> {
        let mut count = 0;
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }

        Ok(count)
    }

    pub async fn update_file_metadata(&self, file_id: &str, tags: Vec<String>) -> Result<()> {
        #[cfg(not(feature = "vectordb"))]
        {
            let file_path = self.db_path.join(format!("{}.json", file_id));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path).await?;
                let mut file: FileDocument = serde_json::from_str(&content)?;
                file.tags = tags;
                let json = serde_json::to_string_pretty(&file)?;
                fs::write(file_path, json).await?;
            }
        }

        #[cfg(feature = "vectordb")]
        {
            let _ = (file_id, tags);
            log::warn!("Metadata update not yet implemented for Qdrant backend");
        }

        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn clear(&self) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client.delete_collection(&self.collection_name).await?;

        client
            .create_collection(
                qdrant_client::qdrant::CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    }),
            )
            .await?;

        log::info!("Cleared drive vector collection: {}", self.collection_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn clear(&self) -> Result<()> {
        if self.db_path.exists() {
            fs::remove_dir_all(&self.db_path).await?;
            fs::create_dir_all(&self.db_path).await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileContentExtractor;

impl FileContentExtractor {
    pub async fn extract_text(file_path: &PathBuf, mime_type: &str) -> Result<String> {
        match mime_type {
            "text/plain" | "text/markdown" | "text/csv" => {
                let content = fs::read_to_string(file_path).await?;
                Ok(content)
            }

            t if t.starts_with("text/") => {
                let content = fs::read_to_string(file_path).await?;
                Ok(content)
            }

            "application/pdf" => {
                log::info!("PDF extraction for {}", file_path.display());
                Self::extract_pdf_text(file_path).await
            }

            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/msword" => {
                log::info!("Word document extraction for {}", file_path.display());
                Self::extract_docx_text(file_path).await
            }

            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel" => {
                log::info!("Spreadsheet extraction for {}", file_path.display());
                Self::extract_xlsx_text(file_path).await
            }

            "application/json" => {
                let content = fs::read_to_string(file_path).await?;

                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => Ok(serde_json::to_string_pretty(&json)?),
                    Err(_) => Ok(content),
                }
            }

            "text/xml" | "application/xml" | "text/html" => {
                let content = fs::read_to_string(file_path).await?;

                let tag_regex = regex::Regex::new(r"<[^>]+>").expect("valid regex");
                let text = tag_regex.replace_all(&content, " ").to_string();
                Ok(text.trim().to_string())
            }

            "text/rtf" | "application/rtf" => {
                let content = fs::read_to_string(file_path).await?;

                let control_regex = regex::Regex::new(r"\\[a-z]+[\-0-9]*[ ]?").expect("valid regex");
                let group_regex = regex::Regex::new(r"[\{\}]").expect("valid regex");

                let mut text = control_regex.replace_all(&content, " ").to_string();
                text = group_regex.replace_all(&text, "").to_string();

                Ok(text.trim().to_string())
            }

            _ => {
                log::warn!("Unsupported file type for indexing: {}", mime_type);
                Ok(String::new())
            }
        }
    }

    async fn extract_pdf_text(file_path: &PathBuf) -> Result<String> {
        let bytes = fs::read(file_path).await?;

        match pdf_extract::extract_text_from_mem(&bytes) {
            Ok(text) => {
                let cleaned = text
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(cleaned)
            }
            Err(e) => {
                log::warn!("PDF extraction failed for {}: {}", file_path.display(), e);
                Ok(String::new())
            }
        }
    }

    async fn extract_docx_text(file_path: &Path) -> Result<String> {
        let path = file_path.to_path_buf();

        let result = tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            let mut content = String::new();

            if let Ok(mut document) = archive.by_name("word/document.xml") {
                let mut xml_content = String::new();
                std::io::Read::read_to_string(&mut document, &mut xml_content)?;

                let text_regex = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").expect("valid regex");

                content = text_regex
                    .captures_iter(&xml_content)
                    .filter_map(|c| c.get(1).map(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join("");

                content = content.split("</w:p>").collect::<Vec<_>>().join("\n");
            }

            Ok::<String, anyhow::Error>(content)
        })
        .await?;

        match result {
            Ok(text) => Ok(text),
            Err(e) => {
                log::warn!("DOCX extraction failed for {}: {}", file_path.display(), e);
                Ok(String::new())
            }
        }
    }

    async fn extract_xlsx_text(file_path: &Path) -> Result<String> {
        let path = file_path.to_path_buf();

        let result = tokio::task::spawn_blocking(move || {
            let mut workbook: calamine::Xlsx<_> = calamine::open_workbook(&path)?;
            let mut content = String::new();

            for sheet_name in workbook.sheet_names() {
                if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                    use std::fmt::Write;
                    let _ = writeln!(&mut content, "=== {} ===", sheet_name);

                    for row in range.rows() {
                        let row_text: Vec<String> = row
                            .iter()
                            .map(|cell| match cell {
                                calamine::Data::Empty => String::new(),
                                calamine::Data::String(s)
                                | calamine::Data::DateTimeIso(s)
                                | calamine::Data::DurationIso(s) => s.clone(),
                                calamine::Data::Float(f) => f.to_string(),
                                calamine::Data::Int(i) => i.to_string(),
                                calamine::Data::Bool(b) => b.to_string(),
                                calamine::Data::Error(e) => format!("{e:?}"),
                                calamine::Data::DateTime(dt) => dt.to_string(),
                            })
                            .collect();

                        let line = row_text.join("\t");
                        if !line.trim().is_empty() {
                            content.push_str(&line);
                            content.push('\n');
                        }
                    }
                    content.push('\n');
                }
            }

            Ok::<String, anyhow::Error>(content)
        })
        .await?;

        match result {
            Ok(text) => Ok(text),
            Err(e) => {
                log::warn!("XLSX extraction failed for {}: {}", file_path.display(), e);
                Ok(String::new())
            }
        }
    }

    pub fn should_index(mime_type: &str, file_size: u64) -> bool {
        if file_size > 10 * 1024 * 1024 {
            return false;
        }

        matches!(
            mime_type,
            "text/plain"
                | "text/markdown"
                | "text/csv"
                | "text/html"
                | "application/json"
                | "text/x-python"
                | "text/x-rust"
                | "text/javascript"
                | "text/x-java"
        )
    }
}
