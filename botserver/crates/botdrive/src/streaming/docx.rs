use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a DOCX file paragraph-by-paragraph, extracting text
/// from each `<w:p>` element without loading the full XML tree.
pub struct DocxStreamProcessor;

impl StreamProcessor for DocxStreamProcessor {
    fn process_stream(&mut self, _path: &Path) -> Result<u64, String> {
        // TODO(#493): Implement paragraph-by-paragraph DOCX extraction.
        // Use quick-xml streaming reader on word/document.xml.
        Ok(0)
    }
}
