use std::io::Cursor;

use docx_rs::*;

use crate::storage_core::{escape_html, parse_html_to_paragraphs};

/// Renders one run's text with its emphasis and line-break children. Used by
/// both paragraphs and hyperlinks because docx-rs nests runs inside links.
fn run_html(run: &Run) -> String {
    let mut run_text = String::new();
    let is_bold = run.run_property.bold.is_some();
    let is_italic = run.run_property.italic.is_some();
    let is_underline = run.run_property.underline.is_some();
    for child in &run.children {
        match child {
            RunChild::Text(text) => run_text.push_str(&escape_html(&text.text)),
            RunChild::Break(_) => run_text.push_str("<br>"),
            RunChild::Tab(_) => run_text.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            _ => {}
        }
    }
    if run_text.is_empty() {
        return run_text;
    }
    if is_bold {
        run_text = format!("<strong>{run_text}</strong>");
    }
    if is_italic {
        run_text = format!("<em>{run_text}</em>");
    }
    if is_underline {
        run_text = format!("<u>{run_text}</u>");
    }
    run_text
}

pub fn convert_docx_to_html(bytes: &[u8]) -> Result<String, String> {
    let docx = docx_rs::read_docx(bytes).map_err(|e| format!("Failed to parse DOCX: {e}"))?;

    let mut html = String::new();

    for child in docx.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
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
                    match content {
                        ParagraphChild::Run(run) => para_html.push_str(&run_html(run)),
                        // Hyperlink labels are user text: dropping them deleted
                        // link text from opened/imported documents.
                        ParagraphChild::Hyperlink(link) => {
                            for child in &link.children {
                                if let ParagraphChild::Run(run) = child {
                                    para_html.push_str(&run_html(run));
                                }
                            }
                        }
                        _ => {}
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
            DocumentChild::Table(table) => {
                html.push_str("<table style=\"border-collapse:collapse;width:100%\">");
                for row in &table.rows {
                    let TableChild::TableRow(tr) = row;
                    html.push_str("<tr>");
                    for cell in &tr.cells {
                        let TableRowChild::TableCell(tc) = cell;
                        html.push_str("<td style=\"border:1px solid #ccc;padding:8px\">");
                        for para in &tc.children {
                            if let TableCellContent::Paragraph(p) = para {
                                for content in &p.children {
                                    match content {
                                        ParagraphChild::Run(run) => html.push_str(&run_html(run)),
                                        ParagraphChild::Hyperlink(link) => {
                                            for child in &link.children {
                                                if let ParagraphChild::Run(run) = child {
                                                    html.push_str(&run_html(run));
                                                }
                                            }
                                        }
                                        _ => {}
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
            // A generated table of contents is plain text for the editor.
            DocumentChild::TableOfContents(toc) => {
                html.push_str("<div class=\"gb-docx-toc\">");
                for item in &toc.items {
                    if !item.text.trim().is_empty() {
                        html.push_str(&format!("<p>{}</p>", escape_html(&item.text)));
                    }
                }
                html.push_str("</div>");
            }
            _ => {}
        }
    }

    Ok(html)
}

pub fn convert_html_to_docx(title: &str, html_content: &str) -> Result<Vec<u8>, String> {
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
