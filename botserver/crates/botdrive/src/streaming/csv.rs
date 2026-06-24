use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::stream_processor::StreamProcessor;

/// Streams a CSV file line-by-line using a buffered reader.
/// Parses headers and first 100 data rows into a text representation
/// suitable for downstream chunking and indexing.
pub struct CsvStreamProcessor;

impl StreamProcessor for CsvStreamProcessor {
    fn process_stream(&mut self, path: &Path) -> Result<String, String> {
        let file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        let reader = BufReader::new(file);
        let mut lines_iter = reader.lines();
        let mut output = String::new();

        // Parse header row
        let header = match lines_iter.next() {
            Some(Ok(h)) => h,
            Some(Err(e)) => return Err(format!("CSV header error: {}", e)),
            None => return Ok("(empty CSV)\n".to_string()),
        };
        output.push_str("Headers: ");
        output.push_str(&header);
        output.push('\n');

        // Process up to 100 data rows
        let mut row_count = 0u64;
        for line_result in lines_iter {
            let line = line_result.map_err(|e| format!("CSV read error: {}", e))?;
            if row_count >= 100 {
                output.push_str("... (truncated, showing first 100 rows)\n");
                break;
            }
            output.push_str(&format!("Row {}: ", row_count + 1));
            output.push_str(&line);
            output.push('\n');
            row_count += 1;
        }

        output.push_str(&format!("Total rows: {}\n", row_count));
        Ok(output)
    }
}
