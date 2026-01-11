use crate::docs::ooxml::{load_docx_preserving, update_docx_text};
use crate::docs::types::{Document, DocumentMetadata};
use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

static DOCUMENT_CACHE: once_cell::sync::Lazy<RwLock<HashMap<String, (Vec<u8>, DateTime<Utc>)>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL_SECS: i64 = 3600;

pub fn get_user_docs_path(user_identifier: &str) -> String {
    let safe_id = user_identifier
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase();
    format!("users/{safe_id}/docs")
}

pub fn get_current_user_id() -> String {
    "default-user".to_string()
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

pub async fn load_docx_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    file_path: &str,
) -> Result<Document, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let result = s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(file_path)
        .send()
        .await
        .map_err(|e| format!("Failed to load DOCX: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read DOCX: {e}"))?
        .into_bytes()
        .to_vec();

    load_docx_from_bytes(&bytes, user_identifier, file_path).await
}

pub async fn load_docx_from_bytes(
    bytes: &[u8],
    user_identifier: &str,
    file_path: &str,
) -> Result<Document, String> {
    let file_name = file_path
        .split('/')
        .last()
        .unwrap_or("Untitled")
        .trim_end_matches(".docx")
        .trim_end_matches(".doc");

    let doc_id = generate_doc_id();

    cache_document_bytes(&doc_id, bytes.to_vec()).await;

    let html_content = match load_docx_preserving(bytes) {
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

pub fn convert_docx_to_html(bytes: &[u8]) -> Result<String, String> {
    let docx = docx_rs::read_docx(bytes).map_err(|e| format!("Failed to parse DOCX: {e}"))?;

    let mut html = String::new();

    for child in docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(para) => {
                let mut para_html = String::new();
                let mut is_heading = false;
                let mut heading_level = 0u8;

                if let Some(style) = &para.property.style {
                    let style_id = style.val.to_lowercase();
                    if style_id.starts_with("heading") || style_id.starts_with("title") {
                        is_heading = true;
                        heading_level = style_id
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse()
                            .unwrap_or(1);
                        if heading_level == 0 {
                            heading_level = 1;
                        }
                    }
                }

                for content in &para.children {
                    if let docx_rs::ParagraphChild::Run(run) = content {
                        let mut run_text = String::new();
                        let is_bold = run.run_property.bold.is_some();
                        let is_italic = run.run_property.italic.is_some();
                        let is_underline = run.run_property.underline.is_some();

                        for child in &run.children {
                            match child {
                                docx_rs::RunChild::Text(text) => {
                                    run_text.push_str(&escape_html(&text.text));
                                }
                                docx_rs::RunChild::Break(_) => {
                                    run_text.push_str("<br>");
                                }
                                docx_rs::RunChild::Tab(_) => {
                                    run_text.push_str("&nbsp;&nbsp;&nbsp;&nbsp;");
                                }
                                _ => {}
                            }
                        }

                        if !run_text.is_empty() {
                            if is_bold {
                                run_text = format!("<strong>{run_text}</strong>");
                            }
                            if is_italic {
                                run_text = format!("<em>{run_text}</em>");
                            }
                            if is_underline {
                                run_text = format!("<u>{run_text}</u>");
                            }
                            para_html.push_str(&run_text);
                        }
                    }
                }

                if !para_html.is_empty() {
                    if is_heading && heading_level > 0 && heading_level <= 6 {
                        html.push_str(&format!("<h{heading_level}>{para_html}</h{heading_level}>"));
                    } else {
                        html.push_str(&format!("<p>{para_html}</p>"));
                    }
                } else {
                    html.push_str("<p><br></p>");
                }
            }
            docx_rs::DocumentChild::Table(table) => {
                html.push_str("<table style=\"border-collapse:collapse;width:100%\">");
                for row in &table.rows {
                    let docx_rs::TableChild::TableRow(tr) = row;
                    html.push_str("<tr>");
                    for cell in &tr.cells {
                        let docx_rs::TableRowChild::TableCell(tc) = cell;
                        html.push_str("<td style=\"border:1px solid #ccc;padding:8px\">");
                        for para in &tc.children {
                            if let docx_rs::TableCellContent::Paragraph(p) = para {
                                for content in &p.children {
                                    if let docx_rs::ParagraphChild::Run(run) = content {
                                        for child in &run.children {
                                            if let docx_rs::RunChild::Text(text) = child {
                                                html.push_str(&escape_html(&text.text));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        html.push_str("</td>");
                    }
                    html.push_str("</tr>");
                }
                html.push_str("</table>");
            }
            _ => {}
        }
    }

    Ok(html)
}

pub async fn save_document_as_docx(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<Vec<u8>, String> {
    let docx_bytes = if let Some(original_bytes) = get_cached_document_bytes(doc_id).await {
        let paragraphs = html_to_paragraphs(content);
        update_docx_text(&original_bytes, &paragraphs).unwrap_or_else(|_| {
            convert_html_to_docx(title, content).unwrap_or_default()
        })
    } else {
        convert_html_to_docx(title, content)?
    };

    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;
    let base_path = get_user_docs_path(user_identifier);
    let docx_path = format!("{base_path}/{doc_id}.docx");

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&docx_path)
        .body(ByteStream::from(docx_bytes.clone()))
        .content_type("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        .send()
        .await
        .map_err(|e| format!("Failed to save DOCX: {e}"))?;

    cache_document_bytes(doc_id, docx_bytes.clone()).await;

    Ok(docx_bytes)
}

pub fn convert_html_to_docx(title: &str, html_content: &str) -> Result<Vec<u8>, String> {
    use docx_rs::*;

    let mut docx = Docx::new();

    if !title.is_empty() {
        let title_para = Paragraph::new().add_run(Run::new().add_text(title).bold().size(48));
        docx = docx.add_paragraph(title_para);
        docx = docx.add_paragraph(Paragraph::new());
    }

    let paragraphs = parse_html_to_paragraphs(html_content);
    for para_data in paragraphs {
        let mut paragraph = Paragraph::new();

        match para_data.style.as_str() {
            "h1" => {
                paragraph =
                    paragraph.add_run(Run::new().add_text(&para_data.text).bold().size(32));
            }
            "h2" => {
                paragraph =
                    paragraph.add_run(Run::new().add_text(&para_data.text).bold().size(28));
            }
            "h3" => {
                paragraph =
                    paragraph.add_run(Run::new().add_text(&para_data.text).bold().size(24));
            }
            "li" => {
                paragraph = paragraph
                    .add_run(Run::new().add_text("• "))
                    .add_run(Run::new().add_text(&para_data.text));
            }
            "blockquote" => {
                paragraph = paragraph
                    .indent(Some(720), None, None, None)
                    .add_run(Run::new().add_text(&para_data.text).italic());
            }
            "code" => {
                paragraph = paragraph.add_run(
                    Run::new()
                        .add_text(&para_data.text)
                        .fonts(RunFonts::new().ascii("Courier New")),
                );
            }
            _ => {
                let mut run = Run::new().add_text(&para_data.text);
                if para_data.bold {
                    run = run.bold();
                }
                if para_data.italic {
                    run = run.italic();
                }
                if para_data.underline {
                    run = run.underline("single");
                }
                paragraph = paragraph.add_run(run);
            }
        }

        docx = docx.add_paragraph(paragraph);
    }

    let mut buf = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buf)
        .map_err(|e| format!("Failed to build DOCX: {e}"))?;

    Ok(buf.into_inner())
}

pub async fn save_document_to_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<String, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{base_path}/{doc_id}.html");
    let meta_path = format!("{base_path}/{doc_id}.meta.json");

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .body(ByteStream::from(content.as_bytes().to_vec()))
        .content_type("text/html")
        .send()
        .await
        .map_err(|e| format!("Failed to save document: {e}"))?;

    let word_count = count_words(content);

    let metadata = serde_json::json!({
        "id": doc_id,
        "title": title,
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339(),
        "word_count": word_count,
        "version": 1
    });

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&meta_path)
        .body(ByteStream::from(metadata.to_string().into_bytes()))
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save metadata: {e}"))?;

    Ok(doc_path)
}

pub async fn save_document(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc: &Document,
) -> Result<String, String> {
    save_document_to_drive(state, user_identifier, &doc.id, &doc.title, &doc.content).await
}

pub async fn load_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<Option<Document>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{base_path}/{doc_id}.html");
    let meta_path = format!("{base_path}/{doc_id}.meta.json");

    let content = match s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .send()
        .await
    {
        Ok(result) => {
            let bytes = result
                .body
                .collect()
                .await
                .map_err(|e| e.to_string())?
                .into_bytes();
            String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?
        }
        Err(_) => return Ok(None),
    };

    let (title, created_at, updated_at) = match s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&meta_path)
        .send()
        .await
    {
        Ok(result) => {
            let bytes = result
                .body
                .collect()
                .await
                .map_err(|e| e.to_string())?
                .into_bytes();
            let meta_str = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
            let meta: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or_default();
            (
                meta["title"].as_str().unwrap_or("Untitled").to_string(),
                meta["created_at"]
                    .as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                meta["updated_at"]
                    .as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            )
        }
        Err(_) => ("Untitled".to_string(), Utc::now(), Utc::now()),
    };

    Ok(Some(Document {
        id: doc_id.to_string(),
        title,
        content,
        owner_id: user_identifier.to_string(),
        storage_path: doc_path,
        created_at,
        updated_at,
        collaborators: Vec::new(),
        version: 1,
        track_changes: None,
        comments: None,
        footnotes: None,
        endnotes: None,
        styles: None,
        toc: None,
        track_changes_enabled: false,
    }))
}

pub async fn list_documents_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
) -> Result<Vec<DocumentMetadata>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);
    let prefix = format!("{base_path}/");
    let mut documents = Vec::new();

    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                if key.ends_with(".meta.json") {
                    if let Ok(meta_result) = s3_client
                        .get_object()
                        .bucket(&state.bucket_name)
                        .key(key)
                        .send()
                        .await
                    {
                        if let Ok(bytes) = meta_result.body.collect().await {
                            if let Ok(meta_str) = String::from_utf8(bytes.into_bytes().to_vec()) {
                                if let Ok(meta) =
                                    serde_json::from_str::<serde_json::Value>(&meta_str)
                                {
                                    let doc_meta = DocumentMetadata {
                                        id: meta["id"].as_str().unwrap_or_default().to_string(),
                                        title: meta["title"]
                                            .as_str()
                                            .unwrap_or("Untitled")
                                            .to_string(),
                                        owner_id: user_identifier.to_string(),
                                        created_at: meta["created_at"]
                                            .as_str()
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|d| d.with_timezone(&Utc))
                                            .unwrap_or_else(Utc::now),
                                        updated_at: meta["updated_at"]
                                            .as_str()
                                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                            .map(|d| d.with_timezone(&Utc))
                                            .unwrap_or_else(Utc::now),
                                        word_count: meta["word_count"].as_u64().unwrap_or(0)
                                            as usize,
                                        storage_type: "html".to_string(),
                                    };
                                    documents.push(doc_meta);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(documents)
}

pub async fn delete_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<(), String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_docs_path(user_identifier);

    for ext in &[".html", ".docx", ".meta.json"] {
        let path = format!("{base_path}/{doc_id}{ext}");
        let _ = s3_client
            .delete_object()
            .bucket(&state.bucket_name)
            .key(&path)
            .send()
            .await;
    }

    remove_from_cache(doc_id).await;

    Ok(())
}

pub fn create_new_document() -> Document {
    let doc_id = generate_doc_id();
    Document {
        id: doc_id,
        title: "Untitled Document".to_string(),
        content: "<p><br></p>".to_string(),
        owner_id: get_current_user_id(),
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

pub fn count_words(content: &str) -> usize {
    let plain_text = strip_html(content);
    plain_text
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .count()
}

fn strip_html(html: &str) -> String {
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
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn paragraphs_to_html(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|p| format!("<p>{}</p>", escape_html(p)))
        .collect::<Vec<_>>()
        .join("")
}

fn html_to_paragraphs(html: &str) -> Vec<String> {
    parse_html_to_paragraphs(html)
        .into_iter()
        .map(|p| p.text)
        .collect()
}

#[derive(Default, Clone)]
struct ParagraphData {
    text: String,
    style: String,
    bold: bool,
    italic: bool,
    underline: bool,
}

fn parse_html_to_paragraphs(html: &str) -> Vec<ParagraphData> {
    let mut paragraphs = Vec::new();
    let mut current = ParagraphData::default();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut is_closing = false;
    let mut text_buffer = String::new();

    let mut bold_stack: i32 = 0;
    let mut italic_stack: i32 = 0;
    let mut underline_stack: i32 = 0;

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
                        "b" | "strong" => bold_stack += 1,
                        "i" | "em" => italic_stack += 1,
                        "u" => underline_stack += 1,
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
    if !current.text.trim().is_empty() {
        paragraphs.push(current);
    }

    paragraphs
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}
