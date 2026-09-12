//! Text extraction for Drive files read through the `GET` keyword.
//!
//! Plain text formats are decoded in memory and PDFs are decoded in memory by
//! `pdf-extract`. Every other document format is staged in a temporary file and
//! handed to `DocumentProcessor`, which already reads DOCX, DOC, XLSX, XLS,
//! PPTX, RTF and HTML. Before this module `GET` only understood PDF, so a
//! `.docx` or `.xlsx` in Drive failed with "not valid UTF-8 text".

use std::error::Error;
use std::path::Path;

use botcore::kb::document_processor::{DocumentFormat, DocumentProcessor};

/// Formats whose bytes are already text and need no conversion.
fn is_plain_text(format: DocumentFormat) -> bool {
    matches!(
        format,
        DocumentFormat::TXT
            | DocumentFormat::MD
            | DocumentFormat::CSV
            | DocumentFormat::JSON
            | DocumentFormat::XML
            | DocumentFormat::HTML
    )
}

/// Extracts the text of a Drive object.
///
/// Unknown extensions keep the historical behaviour of being read as UTF-8 so
/// scripts that store their own `.log`/`.ini` files keep working; a binary
/// payload under an unknown extension now names the extension instead of
/// reporting a bare UTF-8 failure.
pub async fn extract_document_text(
    file_path: &str,
    bytes: Vec<u8>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let Some(format) = DocumentFormat::from_extension(Path::new(file_path)) else {
        return decode_utf8_or_unsupported(file_path, bytes);
    };

    enforce_size_cap(file_path, format, bytes.len())?;

    if is_plain_text(format) {
        return decode_utf8(bytes);
    }

    if format == DocumentFormat::PDF {
        return pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| format!("PDF extraction failed: {e}").into());
    }

    extract_through_temp_file(file_path, format, bytes).await
}

/// Binary office formats are read by `DocumentProcessor`, which only accepts a
/// path: stage the object in the system temporary directory, process it and
/// remove it again.
async fn extract_through_temp_file(
    file_path: &str,
    format: DocumentFormat,
    bytes: Vec<u8>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let extension = Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("bin");

    let staged = std::env::temp_dir().join(format!("gbo-get-{}.{extension}", uuid::Uuid::new_v4()));

    tokio::fs::write(&staged, &bytes).await?;

    let processed = DocumentProcessor::default().process_document(&staged).await;

    if let Err(e) = tokio::fs::remove_file(&staged).await {
        log::warn!("failed to remove staged document {staged:?}: {e}");
    }

    let chunks = processed.map_err(|e| format!("{format:?} extraction failed: {e}"))?;

    let mut text = String::new();
    for chunk in chunks {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&chunk.content);
    }

    if text.is_empty() {
        return Err(format!("{file_path}: document produced no extractable text").into());
    }

    Ok(text)
}

/// Refuses a document before any conversion runs when it exceeds the limit the
/// format declares, and reports both sizes so the caller can tell why the file
/// was refused instead of receiving an extraction error later.
fn enforce_size_cap(
    file_path: &str,
    format: DocumentFormat,
    size: usize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let limit = format.max_size();

    if size > limit {
        return Err(format!(
            "{file_path}: {format:?} document is {size} bytes, above the {limit} byte limit"
        )
        .into());
    }

    Ok(())
}

fn decode_utf8(bytes: Vec<u8>) -> Result<String, Box<dyn Error + Send + Sync>> {
    String::from_utf8(bytes).map_err(|_| "File content is not valid UTF-8 text".into())
}

/// Unknown extension: text passes through unchanged, binary content produces an
/// error naming the extension — the only actionable information for a caller
/// that cannot know which format the object was meant to be.
fn decode_utf8_or_unsupported(
    file_path: &str,
    bytes: Vec<u8>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(_) => {
            let extension = Path::new(file_path)
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("(none)");

            Err(format!("{file_path}: unsupported document format '.{extension}'").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_formats_skip_conversion() {
        assert!(is_plain_text(DocumentFormat::TXT));
        assert!(is_plain_text(DocumentFormat::CSV));
        assert!(is_plain_text(DocumentFormat::JSON));
        assert!(!is_plain_text(DocumentFormat::DOCX));
        assert!(!is_plain_text(DocumentFormat::PDF));
    }

    #[tokio::test]
    async fn unknown_extension_is_read_as_utf8() {
        let result = extract_document_text("notes.log", b"linha".to_vec()).await;

        assert_eq!(
            result.map_err(|e| e.to_string()),
            Ok("linha".to_string())
        );
    }

    #[tokio::test]
    async fn binary_unknown_extension_names_the_extension() {
        let result = extract_document_text("blob.bin", vec![0xff, 0xfe]).await;

        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err("blob.bin: unsupported document format '.bin'".to_string())
        );
    }

    #[test]
    fn oversized_document_is_refused_before_conversion() {
        let above_pdf_limit = DocumentFormat::PDF.max_size() + 1;

        let refused = enforce_size_cap("huge.pdf", DocumentFormat::PDF, above_pdf_limit)
            .map_err(|e| e.to_string());

        assert_eq!(
            refused,
            Err(format!(
                "huge.pdf: PDF document is {above_pdf_limit} bytes, above the {} byte limit",
                DocumentFormat::PDF.max_size()
            ))
        );

        // A document at the limit still extracts.
        assert!(enforce_size_cap("ok.pdf", DocumentFormat::PDF, DocumentFormat::PDF.max_size()).is_ok());
    }
}
