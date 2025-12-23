use anyhow::Result;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncReadExt;


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
            Self::DOCX => 100 * 1024 * 1024,
            Self::XLSX => 100 * 1024 * 1024,
            Self::PPTX => 200 * 1024 * 1024,
            Self::TXT => 100 * 1024 * 1024,
            Self::MD => 10 * 1024 * 1024,
            Self::HTML => 50 * 1024 * 1024,
            Self::RTF => 50 * 1024 * 1024,
            Self::CSV => 1024 * 1024 * 1024,
            Self::JSON => 100 * 1024 * 1024,
            Self::XML => 100 * 1024 * 1024,
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


    pub async fn process_document(&self, file_path: &Path) -> Result<Vec<TextChunk>> {

        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {:?}", file_path));
        }


        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len() as usize;


        let format = DocumentFormat::from_extension(file_path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {:?}", file_path))?;


        if file_size > format.max_size() {
            return Err(anyhow::anyhow!(
                "File too large: {} bytes (max: {} bytes)",
                file_size,
                format.max_size()
            ));
        }

        info!(
            "Processing document: {:?} (format: {:?}, size: {} bytes)",
            file_path, format, file_size
        );


        let text = self.extract_text(file_path, format).await?;


        let cleaned_text = self.clean_text(&text);


        let chunks = self.create_chunks(&cleaned_text, file_path);

        info!(
            "Created {} chunks from document: {:?}",
            chunks.len(),
            file_path
        );

        Ok(chunks)
    }


    async fn extract_text(&self, file_path: &Path, format: DocumentFormat) -> Result<String> {
        match format {
            DocumentFormat::TXT | DocumentFormat::MD => {

                let mut file = tokio::fs::File::open(file_path).await?;
                let mut contents = String::new();
                file.read_to_string(&mut contents).await?;
                Ok(contents)
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


    async fn extract_pdf_text(&self, file_path: &Path) -> Result<String> {

        let output = tokio::process::Command::new("pdftotext")
            .arg("-layout")
            .arg(file_path)
            .arg("-")
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                info!("Successfully extracted PDF with pdftotext: {:?}", file_path);
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => {
                warn!(
                    "pdftotext failed for {:?}, trying library extraction",
                    file_path
                );
                self.extract_pdf_with_library(file_path).await
            }
        }
    }


    async fn extract_pdf_with_library(&self, file_path: &Path) -> Result<String> {
        use pdf_extract::extract_text;

        match extract_text(file_path) {
            Ok(text) => {
                info!("Successfully extracted PDF with library: {:?}", file_path);
                Ok(text)
            }
            Err(e) => {
                warn!("PDF library extraction failed: {}", e);

                self.extract_pdf_basic(file_path).await
            }
        }
    }


    async fn extract_pdf_basic(&self, file_path: &Path) -> Result<String> {

        match pdf_extract::extract_text(file_path) {
            Ok(text) if !text.is_empty() => Ok(text),
            _ => {

                Err(anyhow::anyhow!(
                    "Could not extract text from PDF. Please ensure pdftotext is installed."
                ))
            }
        }
    }


    async fn extract_docx_text(&self, file_path: &Path) -> Result<String> {


        let output = tokio::process::Command::new("pandoc")
            .arg("-f")
            .arg("docx")
            .arg("-t")
            .arg("plain")
            .arg(file_path)
            .output()
            .await;

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
            Ok(self.extract_json_strings(&json))
        } else {
            Ok(contents)
        }
    }


    fn extract_json_strings(&self, value: &serde_json::Value) -> String {
        let mut result = String::new();

        match value {
            serde_json::Value::String(s) => {
                result.push_str(s);
                result.push(' ');
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    result.push_str(&self.extract_json_strings(item));
                }
            }
            serde_json::Value::Object(map) => {
                for (_key, val) in map {
                    result.push_str(&self.extract_json_strings(val));
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


    fn clean_text(&self, text: &str) -> String {

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
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut chunk_index = 0;


        let step_size = self.chunk_size.saturating_sub(self.chunk_overlap);
        let total_chunks = if step_size > 0 {
            (total_chars + step_size - 1) / step_size
        } else {
            1
        };

        while start < total_chars {
            let end = std::cmp::min(start + self.chunk_size, total_chars);


            let mut chunk_end = end;
            if end < total_chars {

                for i in (start..end).rev() {
                    if chars[i].is_whitespace() {
                        chunk_end = i + 1;
                        break;
                    }
                }
            }

            let chunk_content: String = chars[start..chunk_end].iter().collect();

            chunks.push(TextChunk {
                content: chunk_content,
                metadata: ChunkMetadata {
                    document_path: file_path.to_string_lossy().to_string(),
                    document_title: file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                    chunk_index,
                    total_chunks,
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

        chunks
    }


    pub async fn process_kb_folder(
        &self,
        kb_path: &Path,
    ) -> Result<HashMap<String, Vec<TextChunk>>> {
        let mut results = HashMap::new();

        if !kb_path.exists() {
            return Err(anyhow::anyhow!(
                "Knowledge base folder not found: {:?}",
                kb_path
            ));
        }

        info!("Processing knowledge base folder: {:?}", kb_path);


        self.process_directory_recursive(kb_path, &mut results)
            .await?;

        info!("Processed {} documents in knowledge base", results.len());

        Ok(results)
    }


    fn process_directory_recursive<'a>(
        &'a self,
        dir: &'a Path,
        results: &'a mut HashMap<String, Vec<TextChunk>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;

                if metadata.is_dir() {

                    self.process_directory_recursive(&path, results).await?;
                } else if metadata.is_file() {

                    if DocumentFormat::from_extension(&path).is_some() {
                        match self.process_document(&path).await {
                            Ok(chunks) => {
                                let key = path.to_string_lossy().to_string();
                                results.insert(key, chunks);
                            }
                            Err(e) => {
                                error!("Failed to process document {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }

            Ok(())
        })
    }
}
