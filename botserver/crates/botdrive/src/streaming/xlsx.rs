use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams an Excel file row-by-row, extracting text from each
/// worksheet without loading the entire workbook into memory.
pub struct XlsxStreamProcessor;

impl StreamProcessor for XlsxStreamProcessor {
    fn process_stream(&mut self, _path: &Path) -> Result<u64, String> {
        // TODO(#493): Implement row-by-row XLSX extraction.
        // Use calamine with worksheet/sheet iteration.
        Ok(0)
    }
}
