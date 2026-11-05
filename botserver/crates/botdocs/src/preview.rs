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
            format!(r#"<div class="gb-office-doc">{html}</div>"#)
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
</style>
<div class="gb-preview-name">{name}</div>
{body}
</div>"#,
        name = crate::ui_fragments::html_escape(file_name),
        body = body
    ))
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

fn presentation_html(bytes: &[u8]) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to parse PPTX: {error}"))?;
    let mut slides: Vec<(u32, Vec<String>)> = Vec::new();
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
        slides.push((number, slide_paragraphs(&xml)));
    }
    slides.sort_by_key(|(number, _)| *number);
    if slides.is_empty() {
        return Err("no slides found in presentation".to_string());
    }
    let mut html = String::new();
    for (number, paragraphs) in slides {
        html.push_str(&format!(
            r#"<div class="gb-slide"><h3>Slide {number}</h3>"#
        ));
        for paragraph in paragraphs {
            let text = paragraph.trim();
            if !text.is_empty() {
                html.push_str(&format!(
                    "<p>{}</p>",
                    crate::ui_fragments::html_escape(text)
                ));
            }
        }
        html.push_str("</div>");
    }
    Ok(html)
}

fn slide_paragraphs(xml: &str) -> Vec<String> {
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
            },
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:p" => {
                    in_text = false;
                    if in_paragraph {
                        paragraphs.push(std::mem::take(&mut current));
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
        let paragraphs = slide_paragraphs(xml);
        assert_eq!(paragraphs, vec!["Quarter results", "Revenue up"]);
    }

    #[test]
    fn preview_rejects_unknown_extensions() {
        assert!(preview_html("a.bin", "bin", b"x").is_err());
    }
}
