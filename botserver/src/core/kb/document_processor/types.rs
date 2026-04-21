use serde::{Deserialize, Serialize};

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
    pub fn from_extension(path: &std::path::Path) -> Option<Self> {
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
