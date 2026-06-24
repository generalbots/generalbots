//! Streaming file processor for large documents (Issue #493).
//! Processes files in chunks to keep peak RAM < 256MB.

use std::io::{BufReader, Read};
use std::path::Path;

/// Maximum chunk size for streaming reads (16MB).
const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Trait for format-specific streaming processors.
pub trait StreamProcessor {
    /// Process the file and return extracted text content.
    /// Returns the full extracted text for downstream chunking and indexing.
    fn process_stream(&mut self, path: &Path) -> Result<String, String>;
}

/// Reads a file in chunks, calling a callback for each chunk.
/// This avoids loading the entire file into memory.
pub fn read_in_chunks<F>(path: &Path, mut callback: F) -> Result<u64, String>
where
    F: FnMut(&[u8]) -> Result<(), String>,
{
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_chunks = 0u64;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        callback(&buffer[..bytes_read])?;
        total_chunks += 1;
    }

    Ok(total_chunks)
}

/// Reads a file in chunks and accumulates text, respecting line boundaries.
/// Returns the concatenated text (useful for known-small files where chunking
/// is used only as a safety net).
pub fn read_file_streaming(path: &Path) -> Result<String, String> {
    let mut output = String::new();
    let mut buffer = String::with_capacity(CHUNK_SIZE);
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let mut reader = BufReader::new(file);

    use std::io::BufRead;
    loop {
        buffer.clear();
        let bytes_read = reader
            .read_line(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        output.push_str(&buffer);
    }

    Ok(output)
}
