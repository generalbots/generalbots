use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams an Excel file row-by-row, extracting text from each
/// worksheet without loading the entire workbook into memory.
pub struct XlsxStreamProcessor;

impl StreamProcessor for XlsxStreamProcessor {
    fn process_stream(&mut self, path: &Path) -> Result<String, String> {
        #[cfg(feature = "sheet")]
        {
            crate::vectordb::extract_xlsx_text_sync(path)
                .map_err(|e| format!("XLSX extraction error: {}", e))
        }
        #[cfg(not(feature = "sheet"))]
        {
            let _ = (path);
            Err("XLSX extraction requires 'sheet' feature".to_string())
        }
    }
}
