use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a PDF file page-by-page, extracting text from each
/// separately to avoid loading the entire document into memory.
#[cfg(feature = "drive")]
pub struct PdfStreamProcessor;

#[cfg(feature = "drive")]
impl StreamProcessor for PdfStreamProcessor {
fn process_stream(&mut self, path: &Path) -> Result<String, String> {
crate::vectordb::extract_pdf_text_sync(path)
.map_err(|e| format!("PDF extraction error: {}", e))
}
}

#[cfg(not(feature = "drive"))]
pub struct PdfStreamProcessor;

#[cfg(not(feature = "drive"))]
impl StreamProcessor for PdfStreamProcessor {
fn process_stream(&mut self, _path: &Path) -> Result<String, String> {
Err("PDF extraction requires 'drive' feature".to_string())
}
}
