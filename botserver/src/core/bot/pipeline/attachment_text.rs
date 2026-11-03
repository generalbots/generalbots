//! #1641 — text extraction for chat attachments.
//!
//! The upload path used to hand the model a path hint, so a CSV/PDF/DOCX
//! attached to a message was invisible to the answer. This module turns the
//! bytes into a bounded excerpt that is injected as a separate system message:
//! the user's own text stays clean (and is what gets persisted), while the
//! model can actually read the file.

/// How much of the document the prompt carries. Large enough for a real
/// summary, small enough that a 200-page PDF cannot exhaust the context.
const MAX_EXCERPT_CHARS: usize = 12_000;

fn extension(name: &str) -> String {
    name.rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

fn is_text(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "csv"
            | "tsv"
            | "json"
            | "jsonl"
            | "html"
            | "htm"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "log"
            | "rs"
            | "js"
            | "ts"
            | "py"
            | "sh"
            | "bas"
            | "sql"
            | "css"
    )
}

/// True when the file is an image/video/audio container: the transcript must
/// say the content could not be read instead of implying the model saw it.
pub fn is_media(name: &str) -> bool {
    matches!(
        extension(name).as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "heic"
            | "mp4" | "mov" | "webm" | "avi" | "mkv"
            | "mp3" | "wav" | "ogg" | "m4a" | "flac"
    )
}

/// Extracts a bounded, whitespace-normalised excerpt from the attachment.
/// `None` means the format has no extractor in this build (the caller must be
/// honest about that, never invent content).
pub fn extract_excerpt(name: &str, bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let ext = extension(name);
    let raw = if is_text(&ext) {
        String::from_utf8_lossy(bytes).to_string()
    } else if ext == "pdf" {
        pdf_text(bytes)?
    } else if ext == "docx" {
        docx_text(bytes)?
    } else {
        return None;
    };
    let excerpt = normalize(&raw);
    if excerpt.is_empty() {
        None
    } else {
        Some(excerpt)
    }
}

fn normalize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(MAX_EXCERPT_CHARS));
    let mut last_blank = false;
    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            if last_blank {
                continue;
            }
            last_blank = true;
            out.push('\n');
        } else {
            last_blank = false;
            out.push_str(trimmed);
            out.push('\n');
        }
        if out.chars().count() >= MAX_EXCERPT_CHARS {
            let clipped: String = out.chars().take(MAX_EXCERPT_CHARS).collect();
            return format!("{clipped}\n[… excerpt truncated]");
        }
    }
    out.trim().to_string()
}

#[cfg(feature = "drive")]
fn pdf_text(bytes: &[u8]) -> Option<String> {
    pdf_extract::extract_text_from_mem(bytes).ok()
}

#[cfg(not(feature = "drive"))]
fn pdf_text(_bytes: &[u8]) -> Option<String> {
    None
}

/// Minimal DOCX reader: `word/document.xml` is a zip entry; text runs live in
/// `<w:t>` and paragraphs end at `</w:p>`. Avoids pulling a full OOXML parser
/// into the chat pipeline.
fn docx_text(bytes: &[u8]) -> Option<String> {
    use std::io::Read;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name("word/document.xml").ok()?;
    let mut xml = String::new();
    file.by_ref().take(4 * 1024 * 1024).read_to_string(&mut xml).ok()?;

    let mut out = String::new();
    let mut tag = String::new();
    let mut in_tag = false;
    let mut in_text = false;
    for ch in xml.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' => {
                in_tag = false;
                let closing = tag.starts_with('/');
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if name == "w:p" && closing {
                    out.push('\n');
                }
                in_text = !closing && name == "w:t";
            }
            _ if in_tag => tag.push(ch),
            _ if in_text => out.push(ch),
            _ => {}
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{extract_excerpt, is_media, normalize};

    #[test]
    fn text_file_yields_its_content() {
        let excerpt =
            extract_excerpt("notes.txt", b"Linha um\nLinha dois\n").expect("text must extract");
        assert!(excerpt.contains("Linha um"));
        assert!(excerpt.contains("Linha dois"));
    }

    #[test]
    fn image_is_not_text() {
        assert!(is_media("photo.jpg"));
        assert!(extract_excerpt("photo.jpg", &[0xff, 0xd8, 0xff]).is_none());
    }

    #[test]
    fn excerpt_is_bounded() {
        let big = "palavra ".repeat(10_000);
        let excerpt = extract_excerpt("big.md", big.as_bytes()).expect("markdown must extract");
        assert!(excerpt.chars().count() <= super::MAX_EXCERPT_CHARS + 32);
        assert!(excerpt.contains("truncated"));
    }

    #[test]
    fn blank_lines_collapse() {
        assert_eq!(normalize("a\n\n\n\nb\n"), "a\n\nb");
    }

    #[test]
    fn unknown_binary_format_has_no_extractor() {
        assert!(extract_excerpt("archive.zip", &[0x50, 0x4b, 0x03, 0x04]).is_none());
    }
}
