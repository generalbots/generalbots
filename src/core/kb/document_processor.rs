use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncReadExt;
use crate::security::command_guard::SafeCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    PDF,
    DOCX,
    XLSX,
    PPTX,
    TXT,
    MD,
    HTML,
    RTF,
    CSV,
    JSON,
    XML,
}

impl DocumentFormat {
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "pdf" => Some(Self::PDF),
            "docx" => Some(Self::DOCX),
            "xlsx" => Some(Self::XLSX),
            "pptx" => Some(Self::PPTX),
            "txt" => Some(Self::TXT),
            "md" | "markdown" => Some(Self::MD),
            "html" | "htm" => Some(Self::HTML),
            "rtf" => Some(Self::RTF),
            "csv" => Some(Self::CSV),
            "json" => Some(Self::JSON),
            "xml" => Some(Self::XML),
            _ => None,
        }
    }

    pub fn max_size(&self) -> usize {
        match self {
            Self::PDF => 500 * 1024 * 1024,
            Self::PPTX => 200 * 1024 * 1024,
            Self::DOCX | Self::XLSX | Self::TXT | Self::JSON | Self::XML => 100 * 1024 * 1024,
            Self::HTML | Self::RTF => 50 * 1024 * 1024,
            Self::MD => 10 * 1024 * 1024,
            Self::CSV => 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_count: Option<usize>,
    pub word_count: Option<usize>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub document_path: String,
    pub document_title: Option<String>,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub page_number: Option<usize>,
}

#[derive(Debug)]
pub struct DocumentProcessor {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl Default for DocumentProcessor {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
        }
    }
}

impl DocumentProcessor {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn chunk_overlap(&self) -> usize {
        self.chunk_overlap
    }

    pub fn is_supported_file(&self, path: &Path) -> bool {
        DocumentFormat::from_extension(path).is_some()
    }

    pub async fn process_document(&self, file_path: &Path) -> Result<Vec<TextChunk>> {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
        }

        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len() as usize;

        if file_size == 0 {
            debug!(
                "Skipping empty file (0 bytes): {}",
                file_path.display()
            );
            return Ok(Vec::new());
        }

        let format = DocumentFormat::from_extension(file_path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", file_path.display()))?;

        if file_size > format.max_size() {
            return Err(anyhow::anyhow!(
                "File too large: {} bytes (max: {} bytes)",
                file_size,
                format.max_size()
            ));
        }

        info!(
            "Processing document: {} (format: {:?}, size: {} bytes)",
            file_path.display(),
            format,
            file_size
        );

        let text = self.extract_text(file_path, format).await?;

        let cleaned_text = Self::clean_text(&text);

        let chunks = self.create_chunks(&cleaned_text, file_path);

        info!(
            "Created {} chunks from document: {}",
            chunks.len(),
            file_path.display()
        );

        Ok(chunks)
    }

    async fn extract_text(&self, file_path: &Path, format: DocumentFormat) -> Result<String> {
        // Check file size before processing to prevent memory exhaustion
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len() as usize;
        
        if file_size > format.max_size() {
            return Err(anyhow::anyhow!(
                "File too large: {} bytes (max: {} bytes)",
                file_size,
                format.max_size()
            ));
        }

        match format {
            DocumentFormat::TXT | DocumentFormat::MD => {
                // Use streaming read for large text files
                if file_size > 10 * 1024 * 1024 { // 10MB
                    self.extract_large_text_file(file_path).await
                } else {
                    let mut file = tokio::fs::File::open(file_path).await?;
                    let mut contents = String::with_capacity(std::cmp::min(file_size, 1024 * 1024));
                    file.read_to_string(&mut contents).await?;
                    Ok(contents)
                }
            }
            DocumentFormat::PDF => self.extract_pdf_text(file_path).await,
            DocumentFormat::DOCX => self.extract_docx_text(file_path).await,
            DocumentFormat::HTML => self.extract_html_text(file_path).await,
            DocumentFormat::CSV => self.extract_csv_text(file_path).await,
            DocumentFormat::JSON => self.extract_json_text(file_path).await,
            _ => {
                warn!(
                    "Format {:?} extraction not yet implemented, using fallback",
                    format
                );
                self.fallback_text_extraction(file_path).await
            }
        }
    }

    async fn extract_large_text_file(&self, file_path: &Path) -> Result<String> {
        use tokio::io::AsyncBufReadExt;
        
        let file = tokio::fs::File::open(file_path).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut content = String::new();
        let mut line_count = 0;
        const MAX_LINES: usize = 100_000; // Limit lines to prevent memory exhaustion
        
        while let Some(line) = lines.next_line().await? {
            if line_count >= MAX_LINES {
                warn!("Truncating large file at {} lines: {}", MAX_LINES, file_path.display());
                break;
            }
            content.push_str(&line);
            content.push('\n');
            line_count += 1;
            
            // Yield control periodically
            if line_count % 1000 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        Ok(content)
    }

    async fn extract_pdf_text(&self, file_path: &Path) -> Result<String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        let cmd_result = SafeCommand::new("pdftotext")
            .and_then(|c| c.arg("-layout"))
            .and_then(|c| c.arg(&file_path_str))
            .and_then(|c| c.arg("-"));

        let output = match cmd_result {
            Ok(cmd) => cmd.execute_async().await,
            Err(e) => {
                warn!("Failed to build pdftotext command: {}", e);
                return self.extract_pdf_with_library(file_path);
            }
        };

        match output {
            Ok(output) if output.status.success() => {
                info!(
                    "Successfully extracted PDF with pdftotext: {}",
                    file_path.display()
                );
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => {
                warn!(
                    "pdftotext failed for {}, trying library extraction",
                    file_path.display()
                );
                self.extract_pdf_with_library(file_path)
            }
        }
    }

    fn extract_pdf_with_library(&self, file_path: &Path) -> Result<String> {
        let _ = self; // Suppress unused self warning
        #[cfg(feature = "drive")]
        {
            use pdf_extract::extract_text;

            match extract_text(file_path) {
                Ok(text) => {
                    info!(
                        "Successfully extracted PDF with library: {}",
                        file_path.display()
                    );
                    return Ok(text);
                }
                Err(e) => {
                    warn!("PDF library extraction failed: {}", e);
                }
            }
        }

        Self::extract_pdf_basic_sync(file_path)
    }

    fn extract_pdf_basic_sync(file_path: &Path) -> Result<String> {
        #[cfg(feature = "drive")]
        {
            if let Ok(text) = pdf_extract::extract_text(file_path) {
                if !text.is_empty() {
                    return Ok(text);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not extract text from PDF. Please ensure pdftotext is installed."
        ))
    }

    async fn extract_docx_text(&self, file_path: &Path) -> Result<String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        let cmd_result = SafeCommand::new("pandoc")
            .and_then(|c| c.arg("-f"))
            .and_then(|c| c.arg("docx"))
            .and_then(|c| c.arg("-t"))
            .and_then(|c| c.arg("plain"))
            .and_then(|c| c.arg(&file_path_str));

        let output = match cmd_result {
            Ok(cmd) => cmd.execute_async().await,
            Err(e) => {
                warn!("Failed to build pandoc command: {}", e);
                return self.fallback_text_extraction(file_path).await;
            }
        };

        match output {
            Ok(output) if output.status.success() => {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => {
                warn!("pandoc failed for DOCX, using fallback");
                self.fallback_text_extraction(file_path).await
            }
        }
    }

    async fn extract_html_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        let text = contents
            .split('<')
            .flat_map(|s| s.split('>').skip(1))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(text)
    }

    async fn extract_csv_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        let mut text = String::new();
        for line in contents.lines() {
            text.push_str(line);
            text.push('\n');
        }

        Ok(text)
    }

    async fn extract_json_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(Self::extract_json_strings(&json))
        } else {
            Ok(contents)
        }
    }

    fn extract_json_strings(value: &serde_json::Value) -> String {
        let mut result = String::new();

        match value {
            serde_json::Value::String(s) => {
                result.push_str(s);
                result.push(' ');
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    result.push_str(&Self::extract_json_strings(item));
                }
            }
            serde_json::Value::Object(map) => {
                for (_key, val) in map {
                    result.push_str(&Self::extract_json_strings(val));
                }
            }
            _ => {}
        }

        result
    }

    async fn fallback_text_extraction(&self, file_path: &Path) -> Result<String> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(contents) => Ok(contents),
            Err(_) => {
                let bytes = tokio::fs::read(file_path).await?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
        }
    }

    fn clean_text(text: &str) -> String {
        let cleaned = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        cleaned
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn create_chunks(&self, text: &str, file_path: &Path) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        
        // For very large texts, limit processing to prevent memory exhaustion
        const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB
        let text_to_process = if text.len() > MAX_TEXT_SIZE {
            warn!("Truncating large text to {} chars for chunking: {}", MAX_TEXT_SIZE, file_path.display());
            &text[..MAX_TEXT_SIZE]
        } else {
            text
        };
        
        let chars: Vec<char> = text_to_process.chars().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut chunk_index = 0;

        let step_size = self.chunk_size.saturating_sub(self.chunk_overlap);
        let total_chunks = if step_size > 0 {
            total_chars.div_ceil(step_size)
        } else {
            1
        };

        // Limit maximum number of chunks to prevent memory exhaustion
        const MAX_CHUNKS: usize = 1000;
        let max_chunks_to_create = std::cmp::min(total_chunks, MAX_CHUNKS);

        while start < total_chars && chunk_index < max_chunks_to_create {
            let end = std::cmp::min(start + self.chunk_size, total_chars);

            let mut chunk_end = end;
            if end < total_chars {
                // Find word boundary within reasonable distance
                let search_start = std::cmp::max(start, end.saturating_sub(100));
                for i in (search_start..end).rev() {
                    if chars[i].is_whitespace() {
                        chunk_end = i + 1;
                        break;
                    }
                }
            }

            let chunk_content: String = chars[start..chunk_end].iter().collect();

            // Skip empty or very small chunks
            if chunk_content.trim().len() < 10 {
                start = chunk_end;
                continue;
            }

            chunks.push(TextChunk {
                content: chunk_content,
                metadata: ChunkMetadata {
                    document_path: file_path.to_string_lossy().to_string(),
                    document_title: file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                    chunk_index,
                    total_chunks: max_chunks_to_create,
                    start_char: start,
                    end_char: chunk_end,
                    page_number: None,
                },
            });

            chunk_index += 1;

            start = if chunk_end >= self.chunk_overlap {
                chunk_end - self.chunk_overlap
            } else {
                chunk_end
            };

            if start >= total_chars {
                break;
            }
        }

        if chunk_index >= MAX_CHUNKS {
            warn!("Truncated chunking at {} chunks for: {}", MAX_CHUNKS, file_path.display());
        }

        chunks
    }

    pub async fn process_kb_folder(
        &self,
        kb_path: &Path,
    ) -> Result<HashMap<String, Vec<TextChunk>>> {
        if !kb_path.exists() {
            return Err(anyhow::anyhow!(
                "Knowledge base folder not found: {}",
                kb_path.display()
            ));
        }

        info!("Processing knowledge base folder: {}", kb_path.display());

        // Process files in small batches to prevent memory exhaustion
        let mut results = HashMap::new();
        const BATCH_SIZE: usize = 10; // Much smaller batch size
        
        let files = self.collect_supported_files(kb_path).await?;
        info!("Found {} supported files to process", files.len());
        
        for batch in files.chunks(BATCH_SIZE) {
            let mut batch_results = HashMap::new();
            
            for file_path in batch {
                match self.process_document(file_path).await {
                    Ok(chunks) => {
                        if !chunks.is_empty() {
                            batch_results.insert(file_path.to_string_lossy().to_string(), chunks);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to process document {}: {}", file_path.display(), e);
                    }
                }
                
                // Yield control after each file
                tokio::task::yield_now().await;
            }
            
            // Merge batch results and clear batch memory
            results.extend(batch_results);
            
            // Force memory cleanup between batches
            if results.len() % (BATCH_SIZE * 2) == 0 {
                results.shrink_to_fit();
            }
            
            info!("Processed batch, total documents: {}", results.len());
        }

        info!("Completed processing {} documents in knowledge base", results.len());
        Ok(results)
    }

    async fn collect_supported_files(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        self.collect_files_recursive(dir, &mut files, 0).await?;
        Ok(files)
    }

    async fn collect_files_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
        depth: usize,
    ) -> Result<()> {
        // Prevent excessive recursion
        if depth > 10 {
            warn!("Skipping deep directory to prevent stack overflow: {}", dir.display());
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_dir() {
                Box::pin(self.collect_files_recursive(&path, files, depth + 1)).await?;
            } else if self.is_supported_file(&path) {
                // Skip very large files
                if metadata.len() > 50 * 1024 * 1024 {
                    warn!("Skipping large file: {} ({})", path.display(), metadata.len());
                    continue;
                }
                files.push(path);
            }
        }

        Ok(())
    }
}
