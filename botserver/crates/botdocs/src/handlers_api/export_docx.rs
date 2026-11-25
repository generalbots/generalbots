//! Pure DOCX rendering helpers for the Docs export handlers.
//!
//! Split out of `handlers_api/export.rs` (#2452 follow-up) to keep the handler
//! file below the 450-line limit. Behaviour is unchanged: these functions only
//! turn already-redacted HTML into a DOCX package.

use crate::re_cache::cached_re;
use crate::storage::parse_html_to_paragraphs;
use crate::storage_core::{strip_html, ParagraphData};
use docx_rs::{AlignmentType, Docx, Paragraph, Run, RunFonts, Table, TableCell, TableRow};

enum HtmlBlock {
    Fragment(String),
    Table(Vec<Vec<String>>),
    Image(String),
}

pub(crate) fn html_to_docx_full(title: &str, html: &str) -> Result<Vec<u8>, String> {
    let (body, header, footer) = crate::storage_docx_hf::extract_header_footer_zones(html);
    let mut docx = Docx::new();

    if !title.is_empty() {
        let title_para = Paragraph::new()
            .add_run(Run::new().add_text(title).bold().size(48))
            .align(AlignmentType::Center);
        docx = docx.add_paragraph(title_para);
        docx = docx.add_paragraph(Paragraph::new());
    }

    for block in split_html_blocks(&body) {
        match block {
            HtmlBlock::Table(rows) => {
                docx = docx.add_table(build_docx_table(&rows));
            }
            HtmlBlock::Image(alt) => {
                let label = if alt.is_empty() {
                    "[Image]".to_string()
                } else {
                    format!("[Image: {alt}]")
                };
                docx = docx
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text(label).italic()));
            }
            HtmlBlock::Fragment(fragment) => {
                for p in parse_html_to_paragraphs(&fragment) {
                    docx = docx.add_paragraph(paragraph_from_data(&p));
                }
            }
        }
    }

    docx = crate::storage_docx_hf::apply_docx_header_footer(docx, header, footer);

    let mut buf = std::io::Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buf)
        .map_err(|e| format!("Failed to build DOCX: {e}"))?;
    Ok(buf.into_inner())
}

fn paragraph_from_data(p: &ParagraphData) -> Paragraph {
    match p.style.as_str() {
        "h1" => Paragraph::new()
            .style("Heading1")
            .add_run(Run::new().add_text(&p.text).bold().size(32)),
        "h2" => Paragraph::new()
            .style("Heading2")
            .add_run(Run::new().add_text(&p.text).bold().size(28)),
        "h3" => Paragraph::new()
            .style("Heading3")
            .add_run(Run::new().add_text(&p.text).bold().size(24)),
        "li" => Paragraph::new()
            .add_run(Run::new().add_text("• "))
            .add_run(Run::new().add_text(&p.text)),
        "blockquote" => Paragraph::new()
            .indent(Some(720), None, None, None)
            .add_run(Run::new().add_text(&p.text).italic()),
        "code" => Paragraph::new().add_run(
            Run::new()
                .add_text(&p.text)
                .fonts(RunFonts::new().ascii("Courier New")),
        ),
        _ => {
            let mut run = Run::new().add_text(&p.text);
            if p.bold {
                run = run.bold();
            }
            if p.italic {
                run = run.italic();
            }
            if p.underline {
                run = run.underline("single");
            }
            Paragraph::new().add_run(run)
        }
    }
}

fn split_html_blocks(html: &str) -> Vec<HtmlBlock> {
    let mut blocks = Vec::new();
    let mut pos = 0usize;

    while pos < html.len() {
        let table_start = html[pos..].find("<table").map(|r| pos + r);
        let img_start = html[pos..].find("<img").map(|r| pos + r);

        let next = match (table_start, img_start) {
            (Some(t), Some(i)) => Some((std::cmp::min(t, i), t <= i)),
            (Some(t), None) => Some((t, true)),
            (None, Some(i)) => Some((i, false)),
            (None, None) => None,
        };

        match next {
            None => {
                let rest = html[pos..].to_string();
                if !rest.trim().is_empty() {
                    blocks.push(HtmlBlock::Fragment(rest));
                }
                break;
            }
            Some((n, is_table)) => {
                let before = html[pos..n].to_string();
                if !before.trim().is_empty() {
                    blocks.push(HtmlBlock::Fragment(before));
                }

                if is_table {
                    let Some(end_rel) = html[n..].find("</table>") else {
                        blocks.push(HtmlBlock::Fragment(html[n..].to_string()));
                        break;
                    };
                    let end = n + end_rel + "</table>".len();
                    blocks.push(HtmlBlock::Table(parse_table_html(&html[n..end])));
                    pos = end;
                } else {
                    let Some(end_rel) = html[n..].find('>') else {
                        break;
                    };
                    let end = n + end_rel + 1;
                    blocks.push(HtmlBlock::Image(extract_img_alt(&html[n..end])));
                    pos = end;
                }
            }
        }
    }

    blocks
}

fn parse_table_html(table: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let Some(tr_re) = cached_re!(r"<tr[^>]*>(.*?)</tr>") else {
        return rows;
    };
    let Some(cell_re) = cached_re!(r"<t[dh][^>]*>(.*?)</t[dh]>") else {
        return rows;
    };

    for tr in tr_re.captures_iter(table) {
        let mut cells = Vec::new();
        for cell in cell_re.captures_iter(&tr[1]) {
            cells.push(strip_html(&cell[1]).trim().to_string());
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    rows
}

fn build_docx_table(rows: &[Vec<String>]) -> Table {
    let doc_rows: Vec<TableRow> = rows
        .iter()
        .map(|cells| {
            let doc_cells: Vec<TableCell> = cells
                .iter()
                .map(|c| {
                    TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(c)))
                })
                .collect();
            TableRow::new(doc_cells)
        })
        .collect();
    Table::new(doc_rows)
}

fn extract_img_alt(tag: &str) -> String {
    if let Some(pos) = tag.find("alt=\"") {
        let rest = &tag[pos + 5..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    String::new()
}
