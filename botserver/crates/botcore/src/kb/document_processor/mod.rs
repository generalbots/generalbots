mod ooxml_extract;
mod rtf;
mod types;

pub use types::{ChunkMetadata, DocumentFormat, DocumentMetadata, TextChunk};

use anyhow::Result;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncReadExt;
use botlib::security::command_guard::SafeCommand;


#[derive(Debug)]
pub struct DocumentProcessor {
    chunk_size: usize,
    chunk_overlap: usize,
}


mod document_processor;
mod chunk_size;
mod strip_html_tags;
mod fallback_text_extraction;
mod extract_text_from_word_document;

