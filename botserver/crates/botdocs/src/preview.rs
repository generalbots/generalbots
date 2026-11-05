//! Read-only Office previews for the Drive modal.
//!
//! Chat/automation output often lands in Drive as a real `.docx`, `.xlsx` or
//! `.pptx`. Drive could download those bytes but had no renderer, so clicking
//! one fell through to a raw text dump. This module turns the three formats
//! into a safe HTML fragment:
//!
//! - DOCX/DOC/ODT/RTF: the existing `docx-rs` converter used by the editor;
//! - XLSX/XLS/XLSB/ODS: `calamine`, one HTML table per worksheet (bounded);
//! - PPTX/PPT: the slide XML text runs, one card per slide.
//!
//! Everything user-authored is escaped; the only HTML emitted is the wrapper.

use base64::Engine as _;
use std::io::{Cursor, Read};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use calamine::{open_workbook_from_rs, Data, Ods, Reader, Xls, Xlsb, Xlsx};
use serde::Deserialize;

use crate::state::DocState;

const MAX_ROWS: usize = 200;
const MAX_COLUMNS: usize = 40;

#[derive(Debug, Deserialize)]
pub struct OfficePreviewQuery {
    pub bucket: String,
    pub path: String,
}

/// GET /api/files/office-preview?bucket=&path=
pub async fn handle_office_preview(
    State(state): State<Arc<DocState>>,
    Query(query): Query<OfficePreviewQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    if query.bucket.trim().is_empty() || query.path.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "bucket and path are required".to_string(),
        ));
    }
    let bytes = state
        .drive
        .get_object(&query.bucket, &query.path)
        .await
        .map_err(|error| {
            log::warn!(
                "office preview: drive read of {}/{} failed: {error}",
                query.bucket,
                query.path
            );
            (StatusCode::NOT_FOUND, "file not found".to_string())
        })?;
    let extension = query
        .path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let file_name = query.path.rsplit('/').next().unwrap_or(&query.path);
    preview_html(file_name, &extension, &bytes)
        .map(Html)
        .map_err(|error| {
            log::warn!(
                "office preview: conversion of {} failed: {error}",
                query.path
            );
            (StatusCode::UNPROCESSABLE_ENTITY, error)
        })
}

/// Renders one Office document as an HTML fragment (escaped user content).
pub fn preview_html(file_name: &str, extension: &str, bytes: &[u8]) -> Result<String, String> {
    let body = match extension {
        "docx" | "doc" | "odt" | "rtf" => {
            let html = crate::storage::convert_docx_to_html(bytes)?;
            format!(
                r#"<div class="gb-office-doc">{html}</div>{}"#,
                media_gallery(bytes, "word/media/")
            )
        }
        "xlsx" | "xls" | "xlsb" | "ods" => spreadsheet_html(extension, bytes)?,
        "pptx" | "ppt" => presentation_html(bytes)?,
        _ => return Err(format!("preview not supported for .{extension}")),
    };
    Ok(format!(
        r#"<div class="gb-office-preview">
<style>
.gb-office-preview{{background:#fff;color:#111827;border-radius:8px;padding:24px;max-height:70vh;overflow:auto;width:100%;font-family:Georgia,serif;line-height:1.6;font-size:14px;}}
.gb-office-preview .gb-preview-name{{font:12px/1.4 sans-serif;color:#64748b;margin-bottom:14px;letter-spacing:.02em;}}
.gb-office-preview h1,.gb-office-preview h2,.gb-office-preview h3{{color:#0f172a;margin:16px 0 8px;}}
.gb-office-preview table{{border-collapse:collapse;margin:8px 0 20px;font-size:12px;font-family:sans-serif;}}
.gb-office-preview td,.gb-office-preview th{{border:1px solid #cbd5e1;padding:4px 8px;text-align:left;vertical-align:top;}}
.gb-office-preview tr:nth-child(even) td{{background:#f1f5f9;}}
.gb-office-preview .gb-sheet-name{{font:600 13px sans-serif;color:#334155;margin:18px 0 4px;}}
.gb-office-preview .gb-slide{{border:1px solid #cbd5e1;border-radius:8px;padding:16px 20px;margin:12px 0;background:#f8fafc;}}
.gb-office-preview .gb-slide h3{{margin:0 0 8px;font:600 12px sans-serif;text-transform:uppercase;letter-spacing:.08em;color:#64748b;}}
.gb-office-preview .gb-slide p{{margin:4px 0;}}
.gb-office-preview .gb-slide-notes{{margin-top:10px;padding-top:8px;border-top:1px dashed #cbd5e1;}}
.gb-office-preview .gb-slide-notes h4{{margin:0 0 4px;font:600 11px sans-serif;text-transform:uppercase;letter-spacing:.06em;color:#64748b;}}
.gb-office-preview .gb-slide-notes p{{font-size:12px;color:#475569;font-style:italic;}}
.gb-office-preview .gb-office-media{{display:flex;flex-wrap:wrap;gap:12px;margin-top:16px;}}
.gb-office-preview .gb-office-media img{{max-width:320px;max-height:240px;border:1px solid #cbd5e1;border-radius:6px;}}
.gb-office-preview .gb-office-media figcaption{{font:11px sans-serif;color:#64748b;}}
</style>
<div class="gb-preview-name">{name}</div>
{body}
</div>"#,
        name = crate::ui_fragments::html_escape(file_name),
        body = body
    ))
}

/// Inline images embedded in the package (docx `word/media/`, pptx
/// `ppt/media/`), appended as a bounded gallery: the converter does not
/// reproduce Office layout, but dropping the images entirely made image-heavy
/// documents preview as a blank page (#1714). EMF/WMF/TIFF are skipped (not
/// browser-renderable); each image is capped at 2 MiB, 12 per document.
fn media_gallery(bytes: &[u8], prefix: &str) -> String {
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return String::new();
    };
    let mut html = String::new();
    let mut shown = 0usize;
    for index in 0..archive.len() {
        if shown >= 12 {
            break;
        }
        let Ok(mut file) = archive.by_index(index) else {
            continue;
        };
        let name = match file.name() {
            Ok(value) => value.to_string(),
            Err(_) => continue,
        };
        if !name.starts_with(prefix) {
            continue;
        }
        let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
        let mime = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => continue,
        };
        let mut data = Vec::new();
        if file.read_to_end(&mut data).is_err() || data.len() > 2 * 1024 * 1024 {
            continue;
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        html.push_str(&format!(
            r#"<figure><img src="data:{mime};base64,{encoded}" alt="embedded image" loading="lazy"/><figcaption>{}</figcaption></figure>"#,
            crate::ui_fragments::html_escape(&name)
        ));
        shown += 1;
    }
    if shown == 0 {
        String::new()
    } else {
        format!(r#"<div class="gb-office-media">{html}</div>"#)
    }
}

/// One bounded HTML row set per worksheet. A macro keeps the three concrete
/// calamine readers without a generic bound the compiler cannot infer.
macro_rules! read_sheets {
    ($workbook:expr) => {{
        let mut sheets: Vec<(String, Vec<Vec<String>>)> = Vec::new();
        for name in $workbook.sheet_names().to_owned() {
            let Ok(range) = $workbook.worksheet_range(&name) else {
                continue;
            };
            let mut rows: Vec<Vec<String>> = Vec::new();
            for row in range.rows().take(MAX_ROWS) {
                rows.push(row.iter().take(MAX_COLUMNS).map(cell_text).collect());
            }
            sheets.push((name, rows));
        }
        sheets
    }};
}

fn spreadsheet_html(extension: &str, bytes: &[u8]) -> Result<String, String> {
    let cursor = Cursor::new(bytes);
    let sheets = match extension {
        "xls" => {
            let mut workbook: Xls<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLS: {error}"))?;
            read_sheets!(workbook)
        }
        "xlsb" => {
            let mut workbook: Xlsb<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLSB: {error}"))?;
            read_sheets!(workbook)
        }
        "ods" => {
            let mut workbook: Ods<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse ODS: {error}"))?;
            read_sheets!(workbook)
        }
        _ => {
            let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
                .map_err(|error| format!("Failed to parse XLSX: {error}"))?;
            read_sheets!(workbook)
        }
    };

    if sheets.is_empty() {
        return Ok("<p>(empty workbook)</p>".to_string());
    }

    let mut html = String::new();
    for (name, rows) in sheets {
        html.push_str(&format!(
            r#"<div class="gb-sheet-name">{}</div><table>"#,
            crate::ui_fragments::html_escape(&name)
        ));
        for row in rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str("<td>");
                html.push_str(&crate::ui_fragments::html_escape(&cell));
                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
        html.push_str("</table>");
    }
    Ok(html)
}

/// Cells are rendered as display strings; formulas/dates keep the value
/// calamine exposes (ISO date strings where the format provides one).
fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Int(value) => value.to_string(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Bool(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Error(error) => format!("#ERROR {error:?}"),
    }
}

#[derive(Default)]
struct SlidePreview {
    paragraphs: Vec<String>,
    tables: Vec<Vec<Vec<String>>>,
    notes: Vec<String>,
}

impl SlidePreview {
    /// Splits one slide XML into flow paragraphs and tables. Paragraphs inside
    /// `a:tbl` are cell content: rendering them as slide paragraphs as well
    /// duplicated every table cell above the table (#1714).
    fn parse(xml: &str) -> Self {
        use quick_xml::events::Event;
        let mut reader = quick_xml::Reader::from_str(xml);
        let mut preview = Self::default();
        let mut in_table = 0usize;
        let mut in_row = false;
        let mut in_cell = false;
        let mut in_paragraph = false;
        let mut in_text = false;
        let mut paragraph = String::new();
        let mut cell = String::new();
        let mut row: Vec<String> = Vec::new();
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => match event.name().as_ref() {
                    b"a:tbl" => in_table += 1,
                    b"a:tr" if in_table > 0 => {
                        in_row = true;
                        row.clear();
                    }
                    b"a:tc" if in_table > 0 => {
                        in_cell = true;
                        cell.clear();
                    }
                    b"a:p" if in_table == 0 => {
                        in_paragraph = true;
                        paragraph.clear();
                    }
                    b"a:t" => in_text = true,
                    _ => {}
                },
                Ok(Event::Text(text)) if in_text => {
                    if let Ok(decoded) = text.unescape() {
                        if in_cell {
                            cell.push_str(&decoded);
                        } else if in_paragraph {
                            paragraph.push_str(&decoded);
                        }
                    }
                }
                Ok(Event::End(event)) => match event.name().as_ref() {
                    b"a:t" => in_text = false,
                    b"a:tc" if in_cell => {
                        row.push(std::mem::take(&mut cell));
                        in_cell = false;
                    }
                    b"a:tr" if in_row => {
                        preview.tables.push(std::mem::take(&mut row));
                        in_row = false;
                    }
                    b"a:tbl" => in_table = in_table.saturating_sub(1),
                    b"a:p" if in_paragraph => {
                        preview.paragraphs.push(std::mem::take(&mut paragraph));
                        in_paragraph = false;
                    }
                    _ => {}
                },
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        preview
    }
}

fn presentation_html(bytes: &[u8]) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to parse PPTX: {error}"))?;
    let mut slides: Vec<(u32, SlidePreview)> = Vec::new();
    for index in 0..archive.len() {
        let name = archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX entry: {error}"))?
            .name()
            .map_err(|error| format!("Failed to read PPTX entry name: {error}"))?
            .to_string();
        let Some(rest) = name.strip_prefix("ppt/slides/slide") else {
            continue;
        };
        let Some(number) = rest
            .strip_suffix(".xml")
            .and_then(|value| value.parse::<u32>().ok())
        else {
            continue;
        };
        let mut xml = String::new();
        archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX slide: {error}"))?
            .read_to_string(&mut xml)
            .map_err(|error| format!("Failed to read PPTX slide XML: {error}"))?;
        let mut preview = SlidePreview::parse(&xml);
        preview.notes = slide_notes(&mut archive, number);
        slides.push((number, preview));
    }
    slides.sort_by_key(|(number, _)| *number);
    if slides.is_empty() {
        return Err("no slides found in presentation".to_string());
    }
    let mut html = String::new();
    for (number, slide) in slides {
        html.push_str(&format!(
            r#"<div class="gb-slide"><h3>Slide {number}</h3>"#
        ));
        for paragraph in slide.paragraphs {
            let text = paragraph.trim();
            if !text.is_empty() {
                html.push_str(&format!(
                    "<p>{}</p>",
                    crate::ui_fragments::html_escape(text)
                ));
            }
        }
        for table in slide.tables {
            html.push_str("<table>");
            for (row_index, row) in table.iter().enumerate() {
                html.push_str("<tr>");
                for cell in row {
                    let tag = if row_index == 0 { "th" } else { "td" };
                    html.push_str(&format!(
                        "<{tag}>{}</{tag}>",
                        crate::ui_fragments::html_escape(cell.trim())
                    ));
                }
                html.push_str("</tr>");
            }
            html.push_str("</table>");
        }
        if !slide.notes.is_empty() {
            html.push_str(r#"<div class="gb-slide-notes"><h4>Speaker notes</h4>"#);
            for note in slide.notes {
                html.push_str(&format!(
                    "<p>{}</p>",
                    crate::ui_fragments::html_escape(note.trim())
                ));
            }
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }
    html.push_str(&media_gallery(bytes, "ppt/media/"));
    Ok(html)
}

fn zip_entry_text(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|error| format!("{name}: {error}"))?;
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|error| format!("Failed to read {name}: {error}"))?;
    Ok(content)
}

/// Speaker notes for a slide. The note part is not guaranteed to share the
/// slide number, so the relationship target is resolved from the slide's
/// `.rels` file first (#1714).
fn slide_notes(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, slide_number: u32) -> Vec<String> {
    let rels = format!("ppt/slides/_rels/slide{slide_number}.xml.rels");
    let Ok(rels_xml) = zip_entry_text(archive, &rels) else {
        return Vec::new();
    };
    let Some(target) = notes_target(&rels_xml) else {
        return Vec::new();
    };
    let path = rels_target_path("ppt/slides", &target);
    let Ok(xml) = zip_entry_text(archive, &path) else {
        return Vec::new();
    };
    notes_paragraphs(&xml)
}

fn notes_target(rels_xml: &str) -> Option<String> {
    let mut remaining = rels_xml;
    while let Some(position) = remaining.find("Target=\"") {
        let after = &remaining[position + 8..];
        let end = after.find('"')?;
        let value = &after[..end];
        if value.contains("notesSlide") && value.ends_with(".xml") {
            return Some(value.to_string());
        }
        remaining = &after[end..];
    }
    None
}

/// Resolves an OOXML relationship target (`../notesSlides/notesSlide1.xml`)
/// against the part directory (`ppt/slides`).
fn rels_target_path(base_dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = base_dir.split('/').filter(|part| !part.is_empty()).collect();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.truncate(parts.len().saturating_sub(1));
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn notes_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"a:p" => {
                    in_paragraph = true;
                    current.clear();
                }
                b"a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    current.push_str(&decoded);
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:p" => {
                    in_text = false;
                    if in_paragraph {
                        let text = std::mem::take(&mut current);
                        let trimmed = text.trim();
                        // Notes parts repeat the slide number and empty
                        // placeholders; neither is speaker note content.
                        if !trimmed.is_empty()
                            && !trimmed.chars().all(|character| character.is_ascii_digit())
                        {
                            paragraphs.push(text);
                        }
                        in_paragraph = false;
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_paragraphs_collect_runs_per_paragraph() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:r><a:t>Quarter</a:t></a:r><a:r><a:t> results</a:t></a:r></a:p><a:p><a:t>Revenue up</a:t></a:p></p:sld>"#;
        let paragraphs = SlidePreview::parse(xml).paragraphs;
        assert_eq!(paragraphs, vec!["Quarter results", "Revenue up"]);
    }

    #[test]
    fn slide_tables_keep_rows_and_cells_out_of_paragraphs() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:t>Title</a:t></a:p><a:tbl><a:tr><a:tc><a:p><a:t>A</a:t></a:p></a:tc><a:tc><a:p><a:t>B</a:t></a:p></a:tc></a:tr><a:tr><a:tc><a:p><a:t>1</a:t></a:p></a:tc><a:tc><a:p><a:t>2</a:t></a:p></a:tc></a:tr></a:tbl></p:sld>"#;
        let slide = SlidePreview::parse(xml);
        assert_eq!(slide.paragraphs, vec!["Title"]);
        assert_eq!(
            slide.tables,
            vec![vec![vec!["A", "B"], vec!["1", "2"]]]
        );
    }

    #[test]
    fn notes_target_resolves_the_relationship() {
        let rels = r#"<Relationships><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide3.xml"/></Relationships>"#;
        let target = notes_target(rels).expect("notes relationship");
        assert_eq!(
            rels_target_path("ppt/slides", &target),
            "ppt/notesSlides/notesSlide3.xml"
        );
    }

    #[test]
    fn notes_paragraphs_skip_slide_number_placeholders() {
        let xml = r#"<p:notes xmlns:a="a"><a:p><a:t>Keep this note</a:t></a:p><a:p><a:t>7</a:t></a:p></p:notes>"#;
        assert_eq!(notes_paragraphs(xml), vec!["Keep this note"]);
    }

    #[test]
    fn preview_rejects_unknown_extensions() {
        assert!(preview_html("a.bin", "bin", b"x").is_err());
    }
}
