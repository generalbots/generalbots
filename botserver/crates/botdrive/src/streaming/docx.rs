use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a DOCX file paragraph-by-paragraph, extracting text
/// from each `<w:p>` element without loading the full XML tree.
#[cfg(feature = "drive")]
pub struct DocxStreamProcessor;

#[cfg(feature = "drive")]
impl StreamProcessor for DocxStreamProcessor {
fn process_stream(&mut self, path: &Path) -> Result<String, String> {
crate::vectordb::extract_docx_text_sync(path)
.map_err(|e| format!("DOCX extraction error: {}", e))
}
}

#[cfg(not(feature = "drive"))]
pub struct DocxStreamProcessor;

#[cfg(not(feature = "drive"))]
impl StreamProcessor for DocxStreamProcessor {
fn process_stream(&mut self, _path: &Path) -> Result<String, String> {
Err("DOCX extraction requires 'drive' feature".to_string())
}
}
