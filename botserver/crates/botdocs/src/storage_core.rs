use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::DocState;
use crate::storage_docx::convert_docx_to_html;
use crate::types::Document;

/// Document id → (bytes, last modification time) of the in-memory cache.
type DocumentCache = Lazy<RwLock<HashMap<String, (Vec<u8>, DateTime<Utc>)>>>;

static DOCUMENT_CACHE: DocumentCache = Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL_SECS: i64 = 3600;

/// Normalises an owner id into a single Drive path segment.
///
/// The owner is always a server-verified id (#2135) — a directory UUID or the
/// per-install `local-…` id — so this only removes separators a future identity
/// source could smuggle in; it is not an authorisation check.
pub fn sanitize_owner(user_identifier: &str) -> String {
    user_identifier
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase()
}

/// Drive prefix owning every document of `user_identifier`.
///
/// Callers must pass a verified owner (`DocUser::id`); there is no default, so
/// no request can fall back to a namespace shared with other accounts.
pub fn get_user_docs_path(user_identifier: &str) -> String {
    format!("users/{}/docs", sanitize_owner(user_identifier))
}

pub fn generate_doc_id() -> String {
    Uuid::new_v4().to_string()
}

pub async fn cache_document_bytes(doc_id: &str, bytes: Vec<u8>) {
    let mut cache = DOCUMENT_CACHE.write().await;
    cache.insert(doc_id.to_string(), (bytes, Utc::now()));

    let now = Utc::now();
    cache.retain(|_, (_, modified)| (now - *modified).num_seconds() < CACHE_TTL_SECS);
}

pub async fn get_cached_document_bytes(doc_id: &str) -> Option<Vec<u8>> {
    let cache = DOCUMENT_CACHE.read().await;
    cache.get(doc_id).map(|(bytes, _)| bytes.clone())
}

pub async fn remove_from_cache(doc_id: &str) {
    let mut cache = DOCUMENT_CACHE.write().await;
    cache.remove(doc_id);
}

pub fn count_words(content: &str) -> usize {
    let plain_text = strip_html(content);
    plain_text
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .count()
}

pub fn create_new_document(owner_id: &str) -> Document {
    let doc_id = generate_doc_id();
    Document {
        id: doc_id,
        title: "Untitled Document".to_string(),
        content: "<p><br></p>".to_string(),
        owner_id: owner_id.to_string(),
        storage_path: String::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        collaborators: Vec::new(),
        version: 1,
        track_changes: None,
        comments: None,
        footnotes: None,
        endnotes: None,
        styles: None,
        toc: None,
        track_changes_enabled: false,
    }
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn paragraphs_to_html(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|p| format!("<p>{}</p>", escape_html(p)))
        .collect::<Vec<_>>()
        .join("")
}

pub fn html_to_paragraphs(html: &str) -> Vec<String> {
    parse_html_to_paragraphs(html)
        .into_iter()
        .map(|p| p.text)
        .collect()
}

#[derive(Default, Clone)]
pub struct ParagraphData {
    pub text: String,
    pub style: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

pub fn parse_html_to_paragraphs(html: &str) -> Vec<ParagraphData> {
    let mut paragraphs = Vec::new();
    let mut current = ParagraphData::default();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut is_closing = false;
    let mut text_buffer = String::new();

    let mut bold_stack: i32 = 0;
    let mut italic_stack: i32 = 0;
    let mut underline_stack: i32 = 0;
    // Any formatted run inside the paragraph marks the whole paragraph, so a
    // closed `<b>` run is not lost when `</p>` arrives.
    let mut run_bold = false;
    let mut run_italic = false;
    let mut run_underline = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
                is_closing = false;
            }
            '>' => {
                in_tag = false;
                let tag = tag_name.to_lowercase();
                let tag_trimmed = tag.split_whitespace().next().unwrap_or("");

                if is_closing {
                    match tag_trimmed {
                        "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li"
                        | "blockquote" | "pre" => {
                            if !text_buffer.is_empty() || !current.text.is_empty() {
                                current.text = format!(
                                    "{}{}",
                                    current.text,
                                    decode_html_entities(&text_buffer)
                                );
                                current.bold = current.bold || run_bold;
                                current.italic = current.italic || run_italic;
                                current.underline = current.underline || run_underline;
                                run_bold = false;
                                run_italic = false;
                                run_underline = false;
                                if !current.text.trim().is_empty() {
                                    paragraphs.push(current);
                                }
                                current = ParagraphData::default();
                                text_buffer.clear();
                            }
                        }
                        "b" | "strong" => bold_stack = bold_stack.saturating_sub(1),
                        "i" | "em" => italic_stack = italic_stack.saturating_sub(1),
                        "u" => underline_stack = underline_stack.saturating_sub(1),
                        _ => {}
                    }
                } else {
                    match tag_trimmed {
                        "br" => {
                            text_buffer.push('\n');
                        }
                        "p" | "div" => {
                            if !text_buffer.is_empty() {
                                current.text = format!(
                                    "{}{}",
                                    current.text,
                                    decode_html_entities(&text_buffer)
                                );
                                text_buffer.clear();
                            }
                            current.style = "p".to_string();
                            current.bold = bold_stack > 0;
                            current.italic = italic_stack > 0;
                            current.underline = underline_stack > 0;
                        }
                        "h1" => current.style = "h1".to_string(),
                        "h2" => current.style = "h2".to_string(),
                        "h3" => current.style = "h3".to_string(),
                        "li" => current.style = "li".to_string(),
                        "blockquote" => current.style = "blockquote".to_string(),
                        "pre" | "code" => current.style = "code".to_string(),
                        "b" | "strong" => {
                            bold_stack += 1;
                            run_bold = true;
                        }
                        "i" | "em" => {
                            italic_stack += 1;
                            run_italic = true;
                        }
                        "u" => {
                            underline_stack += 1;
                            run_underline = true;
                        }
                        _ => {}
                    }
                }
                tag_name.clear();
            }
            '/' if in_tag && tag_name.is_empty() => {
                is_closing = true;
            }
            _ if in_tag => {
                tag_name.push(ch);
            }
            _ => {
                text_buffer.push(ch);
            }
        }
    }

    if !text_buffer.is_empty() {
        current.text = format!("{}{}", current.text, decode_html_entities(&text_buffer));
    }
    current.bold = current.bold || run_bold;
    current.italic = current.italic || run_italic;
    current.underline = current.underline || run_underline;
    if !current.text.trim().is_empty() {
        paragraphs.push(current);
    }

    paragraphs
}

pub fn decode_html_entities(text: &str) -> String {
    // Numeric entities first, `&amp;` last: decoding `&amp;` early would turn
    // `&amp;lt;` into `<`, which is not what a single decode means.
    decode_numeric_entities(text)
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Decodes `&#233;` and `&#xE9;`; anything unrecognised is left untouched.
fn decode_numeric_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '&' {
            out.push(ch);
            continue;
        }
        let mut body = String::new();
        while let Some(&next) = chars.peek() {
            if next == ';' || body.len() >= 10 {
                break;
            }
            if !(next.is_ascii_alphanumeric() || next == '#') {
                break;
            }
            body.push(next);
            chars.next();
        }
        let terminated = chars.peek() == Some(&';');
        if terminated {
            chars.next();
        }
        let decoded = body
            .strip_prefix('#')
            .and_then(|digits| match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok(),
                None => digits.parse::<u32>().ok(),
            })
            .and_then(char::from_u32);
        match (decoded, terminated) {
            (Some(ch), true) => out.push(ch),
            _ => {
                out.push('&');
                out.push_str(&body);
                if terminated {
                    out.push(';');
                }
            }
        }
    }
    out
}

pub async fn load_docx_from_drive(
    state: &DocState,
    user_identifier: &str,
    file_path: &str,
) -> Result<Document, String> {
    let bytes = state
        .drive
        .get_object(&state.bucket_name, file_path)
        .await
        .map_err(|e| format!("Failed to load DOCX: {e}"))?;

    let doc = load_docx_from_bytes(&bytes, user_identifier, file_path).await?;
    // #2042 — a Drive-opened DOCX is persisted under the deck id so the first
    // save (and every save after a restart) patches the real package.
    if let Err(error) =
        crate::storage_docx_source::persist_original_docx(state, user_identifier, &doc.id, &bytes, &doc.content).await
    {
        log::warn!("docs: could not persist the opened DOCX: {error}");
    }
    Ok(doc)
}

pub async fn load_docx_from_bytes(
    bytes: &[u8],
    user_identifier: &str,
    file_path: &str,
) -> Result<Document, String> {
    let raw_name = file_path.split('/').next_back().unwrap_or("Untitled");
    let file_name = raw_name
        .strip_suffix(".docx")
        .or_else(|| raw_name.strip_suffix(".doc"))
        .unwrap_or(raw_name);

    let doc_id = generate_doc_id();

    cache_document_bytes(&doc_id, bytes.to_vec()).await;

    let html_content = match crate::ooxml::load_docx_preserving(bytes) {
        Ok(ooxml_doc) => {
            let texts: Vec<String> = ooxml_doc.paragraphs.iter().map(|p| p.text.clone()).collect();
            paragraphs_to_html(&texts)
        }
        Err(_) => convert_docx_to_html(bytes)?,
    };

    Ok(Document {
        id: doc_id,
        title: file_name.to_string(),
        content: html_content,
        owner_id: user_identifier.to_string(),
        storage_path: file_path.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        collaborators: Vec::new(),
        version: 1,
        track_changes: None,
        comments: None,
        footnotes: None,
        endnotes: None,
        styles: None,
        toc: None,
        track_changes_enabled: false,
    })
}

#[cfg(test)]
#[path = "storage_core_tests.rs"]
mod tests;
