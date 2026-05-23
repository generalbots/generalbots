use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a PDF file page-by-page, extracting text from each
/// separately to avoid loading the entire document into memory.
pub struct PdfStreamProcessor;

impl StreamProcessor for PdfStreamProcessor {
    fn process_stream(&mut self, _path: &Path) -> Result<u64, String> {
        // TODO(#493): Implement page-by-page PDF extraction.
        // Use lopdf or pdf-extract with page iteration.
        Ok(0)
    }
}
