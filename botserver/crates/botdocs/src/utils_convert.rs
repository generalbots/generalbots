use super::utils_format::{html_escape, strip_html};

pub use super::utils_markdown::{html_to_markdown, markdown_to_html};

pub fn rtf_to_html(rtf: &str) -> String {
    let mut html = String::new();
    let mut in_group = 0;
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut skip_chars = 0;
    let chars: Vec<char> = rtf.chars().collect();
    let mut i = 0;

    html.push_str("<div>");

    while i < chars.len() {
        if skip_chars > 0 {
            skip_chars -= 1;
            i += 1;
            continue;
        }

        let ch = chars[i];

        match ch {
            '{' => in_group += 1,
            '}' => in_group -= 1,
            '\\' => {
                let mut cmd = String::new();
                i += 1;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    cmd.push(chars[i]);
                    i += 1;
                }
                // A control word carries an optional numeric parameter (`\b0`,
                // `\fs24`, `\u233`); it must be consumed here or its digits
                // leak into the text.
                let mut param = String::new();
                if i < chars.len() && (chars[i] == '-' || chars[i].is_ascii_digit()) {
                    param.push(chars[i]);
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        param.push(chars[i]);
                        i += 1;
                    }
                }
                let enabled = param.is_empty() || param != "0";

                match cmd.as_str() {
                    "b" => {
                        if enabled && !bold {
                            html.push_str("<strong>");
                            bold = true;
                        } else if !enabled && bold {
                            html.push_str("</strong>");
                            bold = false;
                        }
                    }
                    "i" => {
                        if enabled && !italic {
                            html.push_str("<em>");
                            italic = true;
                        } else if !enabled && italic {
                            html.push_str("</em>");
                            italic = false;
                        }
                    }
                    "ul" => {
                        if enabled && !underline {
                            html.push_str("<u>");
                            underline = true;
                        }
                    }
                    "ulnone" => {
                        if underline {
                            html.push_str("</u>");
                            underline = false;
                        }
                    }
                    "par" | "line" => html.push_str("<br>"),
                    "tab" => html.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
                    "u" => {
                        // `\uN` is followed by a fallback character that is
                        // dropped; negative N means "add 65536".
                        if let Ok(code) = param.parse::<i64>() {
                            let code = if code < 0 { code + 65_536 } else { code };
                            if let Some(ch) = char::from_u32(code as u32) {
                                html.push(ch);
                            }
                        }
                        if i < chars.len() && !matches!(chars[i], '\\' | '{' | '}') {
                            i += 1;
                        }
                    }
                    _ => {}
                }

                if i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                continue;
            }
            '\n' | '\r' => {}
            _ => {
                if in_group <= 1 {
                    html.push(ch);
                }
            }
        }
        i += 1;
    }

    if underline {
        html.push_str("</u>");
    }
    if italic {
        html.push_str("</em>");
    }
    if bold {
        html.push_str("</strong>");
    }

    html.push_str("</div>");
    html
}

pub fn html_to_rtf(html: &str) -> String {
    let mut rtf = String::from("{\\rtf1\\ansi\\deff0\n");
    rtf.push_str("{\\fonttbl{\\f0 Arial;}}\n");
    rtf.push_str("\\f0\\fs24\n");

    let mut result = html.to_string();
    result = result.replace("<strong>", "\\b ");
    result = result.replace("</strong>", "\\b0 ");
    result = result.replace("<b>", "\\b ");
    result = result.replace("</b>", "\\b0 ");
    result = result.replace("<em>", "\\i ");
    result = result.replace("</em>", "\\i0 ");
    result = result.replace("<i>", "\\i ");
    result = result.replace("</i>", "\\i0 ");
    result = result.replace("<u>", "\\ul ");
    result = result.replace("</u>", "\\ulnone ");
    result = result.replace("<br>", "\\par\n");
    result = result.replace("<br/>", "\\par\n");
    result = result.replace("<br />", "\\par\n");
    result = result.replace("<p>", "");
    result = result.replace("</p>", "\\par\\par\n");
    result = result.replace("<h1>", "\\fs48\\b ");
    result = result.replace("</h1>", "\\b0\\fs24\\par\n");
    result = result.replace("<h2>", "\\fs36\\b ");
    result = result.replace("</h2>", "\\b0\\fs24\\par\n");
    result = result.replace("<h3>", "\\fs28\\b ");
    result = result.replace("</h3>", "\\b0\\fs24\\par\n");

    let stripped = strip_html(&result);
    rtf.push_str(&stripped);
    rtf.push('}');
    rtf
}

pub fn odt_content_to_html(odt_xml: &str) -> String {
    let mut html = String::from("<div>");

    let mut in_text = false;
    let mut span_tag: Option<&'static str> = None;
    let mut heading_level = '1';
    let mut current_text = String::new();
    let chars: Vec<char> = odt_xml.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            let mut tag = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '>' {
                tag.push(chars[i]);
                i += 1;
            }

            if tag.starts_with("text:p") {
                if !current_text.is_empty() {
                    html.push_str(&current_text);
                    current_text.clear();
                }
                html.push_str("<p>");
                in_text = true;
            } else if tag == "/text:p" {
                html.push_str(&current_text);
                current_text.clear();
                html.push_str("</p>");
                in_text = false;
            } else if tag.starts_with("text:span") {
                // Flush first, or the pending run lands inside the span.
                if !current_text.is_empty() {
                    html.push_str(&current_text);
                    current_text.clear();
                }
                span_tag = if tag.contains("Bold") {
                    html.push_str("<strong>");
                    Some("strong")
                } else if tag.contains("Italic") {
                    html.push_str("<em>");
                    Some("em")
                } else {
                    None
                };
            } else if tag == "/text:span" {
                html.push_str(&current_text);
                current_text.clear();
                if let Some(tag) = span_tag.take() {
                    html.push_str(&format!("</{tag}>"));
                }
            } else if tag.starts_with("text:h") {
                heading_level = tag.chars().find(|c| c.is_ascii_digit()).unwrap_or('1');
                html.push_str(&format!("<h{heading_level}>"));
                in_text = true;
            } else if tag.starts_with("/text:h") {
                html.push_str(&current_text);
                current_text.clear();
                html.push_str(&format!("</h{heading_level}>"));
                in_text = false;
            } else if tag == "text:line-break" || tag == "text:line-break/" {
                current_text.push_str("<br>");
            } else if tag == "text:tab" || tag == "text:tab/" {
                current_text.push_str("&nbsp;&nbsp;&nbsp;&nbsp;");
            }
        } else if in_text {
            current_text.push(chars[i]);
        }
        i += 1;
    }

    if !current_text.is_empty() {
        html.push_str(&current_text);
    }

    html.push_str("</div>");
    html
}

pub fn html_to_odt_content(html: &str) -> String {
    let mut odt = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
office:version="1.2">
<office:body>
<office:text>
"#,
    );

    let mut result = html.replace(['\n', '\r'], " ");
    result = result.replace("<p>", "<text:p>");
    result = result.replace("</p>", "</text:p>\n");
    result = result.replace("<br>", "<text:line-break/>");
    result = result.replace("<br/>", "<text:line-break/>");
    result = result.replace("<br />", "<text:line-break/>");
    result = result.replace("<strong>", "<text:span text:style-name=\"Bold\">");
    result = result.replace("</strong>", "</text:span>");
    result = result.replace("<b>", "<text:span text:style-name=\"Bold\">");
    result = result.replace("</b>", "</text:span>");
    result = result.replace("<em>", "<text:span text:style-name=\"Italic\">");
    result = result.replace("</em>", "</text:span>");
    result = result.replace("<i>", "<text:span text:style-name=\"Italic\">");
    result = result.replace("</i>", "</text:span>");
    result = result.replace("<h1>", "<text:h text:outline-level=\"1\">");
    result = result.replace("</h1>", "</text:h>\n");
    result = result.replace("<h2>", "<text:h text:outline-level=\"2\">");
    result = result.replace("</h2>", "</text:h>\n");
    result = result.replace("<h3>", "<text:h text:outline-level=\"3\">");
    result = result.replace("</h3>", "</text:h>\n");

    let cleaned = strip_non_odt_tags(&result);
    for para in cleaned.lines() {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("<text:p") || trimmed.starts_with("<text:h") {
            odt.push_str(trimmed);
        } else {
            odt.push_str(&format!("<text:p>{trimmed}</text:p>"));
        }
        odt.push('\n');
    }

    odt.push_str("</office:text>\n</office:body>\n</office:document-content>");
    odt
}

/// Drops HTML tags that have no ODF counterpart, keeping the `text:` elements
/// this module emits. `strip_html` would remove those too and lose the styles.
fn strip_non_odt_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut tag = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                if tag.starts_with("text:") || tag.starts_with("/text:") {
                    out.push('<');
                    out.push_str(&tag);
                    out.push('>');
                }
            }
            _ if in_tag => tag.push(ch),
            _ => out.push(ch),
        }
    }
    out
}

pub fn detect_document_format(content: &[u8]) -> &'static str {
    // #2254 — a PDF is recognised by its signature, not by a text marker.
    if crate::import_pdf::looks_like_pdf(content) {
        return "pdf";
    }
    if content.len() >= 4 {
        if &content[0..4] == b"PK\x03\x04" {
            if content.len() > 30 {
                let content_str = String::from_utf8_lossy(&content[0..100.min(content.len())]);
                if content_str.contains("word/") {
                    return "docx";
                } else if content_str.contains("content.xml") {
                    return "odt";
                }
            }
            return "zip";
        }
        if &content[0..4] == b"{\\rt" {
            return "rtf";
        }
        if content.len() >= 2 && content[0] == 0xD0 && content[1] == 0xCF {
            return "doc";
        }
    }

    let text = String::from_utf8_lossy(content);
    if text.trim_start().starts_with("<!DOCTYPE html") || text.trim_start().starts_with("<html") {
        return "html";
    }
    if text.trim_start().starts_with('#') || text.contains("\n# ") {
        return "markdown";
    }

    "txt"
}

pub fn convert_to_html(content: &[u8]) -> Result<String, String> {
    let format = detect_document_format(content);
    let text = String::from_utf8_lossy(content).to_string();

    match format {
        "pdf" => crate::import_pdf::pdf_to_editor_html(content),
        "rtf" => Ok(rtf_to_html(&text)),
        "html" => Ok(text),
        "markdown" => Ok(markdown_to_html(&text)),
        "txt" => Ok(format!(
            "<p>{}</p>",
            html_escape(&text).replace('\n', "</p><p>")
        )),
        _ => Err(format!("Unsupported format: {format}")),
    }
}

#[cfg(test)]
#[path = "utils_convert_tests.rs"]
mod tests;
