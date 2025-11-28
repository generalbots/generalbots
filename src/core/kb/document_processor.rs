#![allow(dead_code)]

use anyhow::Result;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncReadExt;

/// Supported document formats for knowledge base
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
    /// Detect format from file extension
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

    /// Get maximum file size for this format (in bytes)
    pub fn max_size(&self) -> usize {
        match self {
            Self::PDF => 500 * 1024 * 1024,  // 500MB
            Self::DOCX => 100 * 1024 * 1024, // 100MB
            Self::XLSX => 100 * 1024 * 1024, // 100MB
            Self::PPTX => 200 * 1024 * 1024, // 200MB
            Self::TXT => 100 * 1024 * 1024,  // 100MB
            Self::MD => 10 * 1024 * 1024,    // 10MB
            Self::HTML => 50 * 1024 * 1024,  // 50MB
            Self::RTF => 50 * 1024 * 1024,   // 50MB
            Self::CSV => 1024 * 1024 * 1024, // 1GB
            Self::JSON => 100 * 1024 * 1024, // 100MB
            Self::XML => 100 * 1024 * 1024,  // 100MB
        }
    }
}

/// Document metadata extracted during processing
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

/// A text chunk ready for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub content: String,
    pub metadata: ChunkMetadata,
}

/// Metadata for a text chunk
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

/// Main document processor for knowledge base
#[derive(Debug)]
pub struct DocumentProcessor {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl Default for DocumentProcessor {
    fn default() -> Self {
        Self {
            chunk_size: 1000,   // 1000 characters as per docs
            chunk_overlap: 200, // 200 character overlap as per docs
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

    /// Get the chunk size
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Get the chunk overlap
    pub fn chunk_overlap(&self) -> usize {
        self.chunk_overlap
    }

    /// Process a document file and return extracted text chunks
    pub async fn process_document(&self, file_path: &Path) -> Result<Vec<TextChunk>> {
        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {:?}", file_path));
        }

        // Get file size
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len() as usize;

        // Detect format
        let format = DocumentFormat::from_extension(file_path)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {:?}", file_path))?;

        // Check file size
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

        // Extract text based on format
        let text = self.extract_text(file_path, format).await?;

        // Clean and normalize text
        let cleaned_text = self.clean_text(&text);

        // Generate chunks
        let chunks = self.create_chunks(&cleaned_text, file_path);

        info!(
            "Created {} chunks from document: {:?}",
            chunks.len(),
            file_path
        );

        Ok(chunks)
    }

    /// Extract text from document based on format
    async fn extract_text(&self, file_path: &Path, format: DocumentFormat) -> Result<String> {
        match format {
            DocumentFormat::TXT | DocumentFormat::MD => {
                // Direct text file reading
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

    /// Extract text from PDF files
    async fn extract_pdf_text(&self, file_path: &Path) -> Result<String> {
        // Try system pdftotext first (fastest and most reliable)
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

    /// Extract PDF using poppler-utils
    #[allow(dead_code)]
    async fn extract_pdf_with_poppler(&self, file_path: &Path) -> Result<String> {
        let output = tokio::process::Command::new("pdftotext")
            .arg(file_path)
            .arg("-")
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Fallback to library extraction
            self.extract_pdf_with_library(file_path).await
        }
    }

    /// Extract PDF using rust library (fallback)
    async fn extract_pdf_with_library(&self, file_path: &Path) -> Result<String> {
        use pdf_extract::extract_text;

        match extract_text(file_path) {
            Ok(text) => {
                info!("Successfully extracted PDF with library: {:?}", file_path);
                Ok(text)
            }
            Err(e) => {
                warn!("PDF library extraction failed: {}", e);
                // Last resort: try to get any text we can
                self.extract_pdf_basic(file_path).await
            }
        }
    }

    /// Basic PDF extraction using rust library (minimal approach)
    async fn extract_pdf_basic(&self, file_path: &Path) -> Result<String> {
        // Try using pdf-extract as final fallback
        match pdf_extract::extract_text(file_path) {
            Ok(text) if !text.is_empty() => Ok(text),
            _ => {
                // Last resort: return error message
                Err(anyhow::anyhow!(
                    "Could not extract text from PDF. Please ensure pdftotext is installed."
                ))
            }
        }
    }

    /// Extract text from DOCX files
    async fn extract_docx_text(&self, file_path: &Path) -> Result<String> {
        // Use docx-rs or similar crate
        // For now, use pandoc as fallback
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

    /// Extract text from HTML files
    async fn extract_html_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        // Simple HTML tag removal (production should use html parser)
        let text = contents
            .split('<')
            .flat_map(|s| s.split('>').skip(1))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(text)
    }

    /// Extract text from CSV files
    async fn extract_csv_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        // Convert CSV rows to text
        let mut text = String::new();
        for line in contents.lines() {
            text.push_str(line);
            text.push('\n');
        }

        Ok(text)
    }

    /// Extract text from JSON files
    async fn extract_json_text(&self, file_path: &Path) -> Result<String> {
        let contents = tokio::fs::read_to_string(file_path).await?;

        // Parse JSON and extract all string values
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(self.extract_json_strings(&json))
        } else {
            Ok(contents)
        }
    }

    /// Recursively extract string values from JSON
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

    /// Fallback text extraction for unsupported formats
    async fn fallback_text_extraction(&self, file_path: &Path) -> Result<String> {
        // Try to read as UTF-8 text
        match tokio::fs::read_to_string(file_path).await {
            Ok(contents) => Ok(contents),
            Err(_) => {
                // If not UTF-8, try with lossy conversion
                let bytes = tokio::fs::read(file_path).await?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
        }
    }

    /// Clean and normalize extracted text
    fn clean_text(&self, text: &str) -> String {
        // Remove multiple spaces and normalize whitespace
        let cleaned = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Remove control characters
        cleaned
            .chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Create overlapping chunks from text
    fn create_chunks(&self, text: &str, file_path: &Path) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            return chunks;
        }

        let mut start = 0;
        let mut chunk_index = 0;

        // Calculate total number of chunks for metadata
        let step_size = self.chunk_size.saturating_sub(self.chunk_overlap);
        let total_chunks = if step_size > 0 {
            (total_chars + step_size - 1) / step_size
        } else {
            1
        };

        while start < total_chars {
            let end = std::cmp::min(start + self.chunk_size, total_chars);

            // Find word boundary for clean cuts
            let mut chunk_end = end;
            if end < total_chars {
                // Look for word boundary
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
                    page_number: None, // Would be set for PDFs with page info
                },
            });

            chunk_index += 1;

            // Move forward by chunk_size - overlap
            start = if chunk_end >= self.chunk_overlap {
                chunk_end - self.chunk_overlap
            } else {
                chunk_end
            };

            // Prevent infinite loop
            if start >= total_chars {
                break;
            }
        }

        chunks
    }

    /// Process all documents in a knowledge base folder
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

        // Recursively process all files
        self.process_directory_recursive(kb_path, &mut results)
            .await?;

        info!("Processed {} documents in knowledge base", results.len());

        Ok(results)
    }

    /// Recursively process directory
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
                    // Recurse into subdirectory
                    self.process_directory_recursive(&path, results).await?;
                } else if metadata.is_file() {
                    // Check if this is a supported format
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let processor = DocumentProcessor::default();
        let text = "This is a test document with some content that needs to be chunked properly. "
            .repeat(20);
        let chunks = processor.create_chunks(&text, Path::new("test.txt"));

        // Verify chunks are created
        assert!(!chunks.is_empty());

        // Verify chunk size
        for chunk in &chunks {
            assert!(chunk.content.len() <= processor.chunk_size);
        }

        // Verify overlap exists
        if chunks.len() > 1 {
            let first_end = &chunks[0].content[chunks[0].content.len().saturating_sub(100)..];
            let second_start = &chunks[1].content[..100.min(chunks[1].content.len())];

            // There should be some overlap
            assert!(first_end.chars().any(|c| second_start.contains(c)));
        }
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            DocumentFormat::from_extension(Path::new("test.pdf")),
            Some(DocumentFormat::PDF)
        );
        assert_eq!(
            DocumentFormat::from_extension(Path::new("test.docx")),
            Some(DocumentFormat::DOCX)
        );
        assert_eq!(
            DocumentFormat::from_extension(Path::new("test.txt")),
            Some(DocumentFormat::TXT)
        );
        assert_eq!(
            DocumentFormat::from_extension(Path::new("test.md")),
            Some(DocumentFormat::MD)
        );
        assert_eq!(
            DocumentFormat::from_extension(Path::new("test.unknown")),
            None
        );
    }

    #[test]
    fn test_text_cleaning() {
        let processor = DocumentProcessor::default();
        let dirty_text = "  This   is\n\n\na    test\r\nwith  multiple    spaces  ";
        let cleaned = processor.clean_text(dirty_text);
        assert_eq!(cleaned, "This is a test with multiple spaces");
    }
}
