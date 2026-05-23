use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a CSV file line-by-line using a buffered reader.
/// Processes each row individually to avoid loading the whole
/// file into memory.
pub struct CsvStreamProcessor;

impl StreamProcessor for CsvStreamProcessor {
    fn process_stream(&mut self, path: &Path) -> Result<u64, String> {
        let file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        let reader = BufReader::new(file);
        let mut total_rows = 0u64;

        for line in reader.lines() {
            let _line = line.map_err(|e| format!("CSV read error: {}", e))?;
            // TODO(#493): Yield row to caller or accumulator.
            total_rows += 1;
        }

        Ok(total_rows)
    }
}
