mod ooxml_extract;
mod rtf;
mod types;

pub use types::{ChunkMetadata, DocumentFormat, DocumentMetadata, TextChunk};

use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use tokio::io::AsyncReadExt;

use crate::security::command_guard::SafeCommand;

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
            debug!("Skipping empty file (0 bytes): {}", file_path.display());
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
                if file_size > 10 * 1024 * 1024 {
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
            DocumentFormat::PPTX => self.extract_pptx_text(file_path).await,
            DocumentFormat::XLSX => self.extract_xlsx_text(file_path).await,
            DocumentFormat::HTML => self.extract_html_text(file_path).await,
            DocumentFormat::CSV => self.extract_csv_text(file_path).await,
            DocumentFormat::JSON => self.extract_json_text(file_path).await,
            DocumentFormat::XML => self.extract_xml_text(file_path).await,
            DocumentFormat::RTF => self.extract_rtf_text(file_path).await,
        }
    }

    async fn extract_large_text_file(&self, file_path: &Path) -> Result<String> {
        use tokio::io::AsyncBufReadExt;

        let file = tokio::fs::File::open(file_path).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut content = String::new();
        let mut line_count = 0;
        const MAX_LINES: usize = 100_000;

        while let Some(line) = lines.next_line().await? {
            if line_count >= MAX_LINES {
                warn!("Truncating large file at {} lines: {}", MAX_LINES, file_path.display());
                break;
            }
            content.push_str(&line);
            content.push('\n');
            line_count += 1;

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
                info!("Successfully extracted PDF with pdftotext: {}", file_path.display());
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            _ => {
                warn!("pdftotext failed for {}, trying library extraction", file_path.display());
                self.extract_pdf_with_library(file_path)
            }
        }
    }

    fn extract_pdf_with_library(&self, file_path: &Path) -> Result<String> {
        let _ = self;
        #[cfg(feature = "drive")]
        {
            use pdf_extract::extract_text;
            match extract_text(file_path) {
                Ok(text) => {
                    info!("Successfully extracted PDF with library: {}", file_path.display());
                    return Ok(text);
                }
                Err(e) => {
                    warn!("PDF library extraction failed: {}", e);
                }
            }
        }
        #[cfg(not(feature = "drive"))]
        let _ = file_path;
        Self::extract_pdf_basic_sync(file_path)
    }

    #[cfg(feature = "drive")]
    fn extract_pdf_basic_sync(file_path: &Path) -> Result<String> {
        if let Ok(text) = pdf_extract::extract_text(file_path) {
            if !text.is_empty() {
                return Ok(text);
            }
        }
        Err(anyhow::anyhow!("Could not extract text from PDF"))
    }

    #[cfg(not(feature = "drive"))]
    fn extract_pdf_basic_sync(_file_path: &Path) -> Result<String> {
        Err(anyhow::anyhow!("PDF extraction requires 'drive' feature"))
    }

    async fn extract_docx_text(&self, file_path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(file_path).await?;
        let path_display = file_path.display().to_string();
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            match ooxml_extract::extract_docx_text_from_zip(&bytes) {
                Ok(text) if !text.trim().is_empty() => {
                    log::info!("Extracted DOCX text from ZIP: {path_display}");
                    return Ok(text);
                }
                Ok(_) => log::warn!("DOCX ZIP extraction returned empty text: {path_display}"),
                Err(e) => log::warn!("DOCX ZIP extraction failed for {path_display}: {e}"),
            }

            #[cfg(feature = "docs")]
            match crate::docs::ooxml::load_docx_preserving(&bytes) {
                Ok(doc) => {
                    let text: String = doc.paragraphs.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("\n");
                    if !text.trim().is_empty() {
                        log::info!("Extracted DOCX with ooxmlsdk: {path_display}");
                        return Ok(text);
                    }
                    log::warn!("ooxmlsdk DOCX returned empty: {path_display}");
                }
                Err(e) => log::warn!("ooxmlsdk DOCX failed for {path_display}: {e}"),
            }

            Err(anyhow::anyhow!("All DOCX extraction methods failed for {path_display}"))
        })
        .await??;

        Ok(result)
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

    async fn extract_pptx_text(&self, file_path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(file_path).await?;
        let path_display = file_path.display().to_string();
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            match ooxml_extract::extract_pptx_text_from_zip(&bytes) {
                Ok(text) if !text.trim().is_empty() => {
                    log::info!("Extracted PPTX text from ZIP: {path_display}");
                    return Ok(text);
                }
                Ok(_) => log::warn!("PPTX ZIP extraction returned empty text: {path_display}"),
                Err(e) => log::warn!("PPTX ZIP extraction failed for {path_display}: {e}"),
            }

            #[cfg(feature = "slides")]
            match crate::slides::ooxml::load_pptx_preserving(&bytes) {
                Ok(pptx) => {
                    let mut text = String::new();
                    for slide in &pptx.slides {
                        for slide_text in &slide.texts {
                            if !text.is_empty() {
                                text.push('\n');
                            }
                            text.push_str(slide_text);
                        }
                    }
                    if !text.trim().is_empty() {
                        log::info!("Extracted PPTX with ooxmlsdk: {path_display}");
                        return Ok(text);
                    }
                    log::warn!("ooxmlsdk PPTX returned empty: {path_display}");
                }
                Err(e) => log::warn!("ooxmlsdk PPTX failed for {path_display}: {e}"),
            }

            Err(anyhow::anyhow!("All PPTX extraction methods failed for {path_display}"))
        })
        .await??;

        Ok(result)
    }

    #[cfg(feature = "kb-extraction")]
    async fn extract_xlsx_text(&self, file_path: &Path) -> Result<String> {
        let path = file_path.to_path_buf();
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            use calamine::{open_workbook_from_rs, Reader, Xlsx};
            use std::io::Read;

            let mut file = std::fs::File::open(&path)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let cursor = Cursor::new(bytes.as_slice());
            let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
                .map_err(|e| anyhow::anyhow!("Failed to open XLSX: {e}"))?;

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

            Ok(content)
        })
        .await??;

        if result.trim().is_empty() {
            warn!("XLSX extraction produced empty text: {}", file_path.display());
        } else {
            info!("Extracted XLSX with calamine library: {}", file_path.display());
        }

        Ok(result)
    }

    #[cfg(not(feature = "kb-extraction"))]
    async fn extract_xlsx_text(&self, file_path: &Path) -> Result<String> {
        self.fallback_text_extraction(file_path).await
    }

    async fn extract_xml_text(&self, file_path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(file_path).await?;
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            use quick_xml::events::Event;
            use quick_xml::Reader;

            let mut reader = Reader::from_reader(bytes.as_slice());
            let mut text = String::new();
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Text(t)) => {
                        if let Ok(s) = t.unescape() {
                            let s = s.trim();
                            if !s.is_empty() {
                                if !text.is_empty() {
                                    text.push(' ');
                                }
                                text.push_str(s);
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "XML parsing error at position {}: {e}",
                            reader.error_position()
                        ));
                    }
                    _ => {}
                }
                buf.clear();
            }

            Ok(text)
        })
        .await??;

        if result.trim().is_empty() {
            warn!("XML extraction produced empty text: {}", file_path.display());
            return self.fallback_text_extraction(file_path).await;
        }

        info!("Extracted XML with quick-xml: {}", file_path.display());
        Ok(result)
    }

    async fn extract_rtf_text(&self, file_path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(file_path).await?;
        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            let content = String::from_utf8_lossy(&bytes);
            let text = rtf::strip_rtf_commands(&content);
            Ok(text)
        })
        .await??;

        if result.trim().is_empty() {
            warn!("RTF extraction produced empty text: {}", file_path.display());
            return self.fallback_text_extraction(file_path).await;
        }

        info!("Extracted RTF text: {}", file_path.display());
        Ok(result)
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

        const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;
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

        const MAX_CHUNKS: usize = 1000;
        let max_chunks_to_create = std::cmp::min(total_chunks, MAX_CHUNKS);

        while start < total_chars && chunk_index < max_chunks_to_create {
            let end = std::cmp::min(start + self.chunk_size, total_chars);

            let mut chunk_end = end;
            if end < total_chars {
                let search_start = std::cmp::max(start, end.saturating_sub(100));
                for i in (search_start..end).rev() {
                    if chars[i].is_whitespace() {
                        chunk_end = i + 1;
                        break;
                    }
                }
            }

            let chunk_content: String = chars[start..chunk_end].iter().collect();

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

        let mut results = HashMap::new();
        const BATCH_SIZE: usize = 10;

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

                tokio::task::yield_now().await;
            }

            results.extend(batch_results);

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
