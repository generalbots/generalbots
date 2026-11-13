//! HTML → block model for PDF export.
//!
//! The browser editor stores rich HTML. Rendering it as one flat string of
//! text (the old `strip_html` route) loses every structural cue, so headings,
//! list numbering and page breaks must be recovered here as discrete blocks
//! before pagination (#2250, #2251).

use crate::storage_core::decode_html_entities;

/// A stretch of text sharing one emphasis combination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub mono: bool,
}

/// Structural role of a block, carried into layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    /// `indent` is the nesting depth; `marker` is the rendered label (`1.`).
    ListItem {
        indent: u8,
        marker: String,
    },
    Blockquote,
    Code,
    /// Explicit page break (`<div class="gb-doc-page-break">` or `<hr>`).
    PageBreak,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub runs: Vec<InlineRun>,
    /// Heading `id` attribute: becomes an internal PDF destination so a TOC
    /// entry can jump to it (#2252).
    pub anchor: Option<String>,
    /// First same-document `#fragment` link in the block: becomes a clickable
    /// internal PDF link over the block's lines (#2252).
    pub link: Option<String>,
}

impl Block {
    /// Plain text of the block, used by tests and by future text exports.
    pub fn text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }
}

struct ListContext {
    ordered: bool,
    counter: usize,
}

#[derive(Default)]
struct Builder {
    runs: Vec<InlineRun>,
    text: String,
    bold: u32,
    italic: u32,
    mono: u32,
    kind: Option<BlockKind>,
    owner: Option<String>,
    lists: Vec<ListContext>,
    anchor: Option<String>,
    link: Option<String>,
}

impl Builder {
    fn flush_run(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.text);
        self.runs.push(InlineRun {
            text,
            bold: self.bold > 0,
            italic: self.italic > 0,
            mono: self.mono > 0,
        });
    }

    fn push_text(&mut self, raw: &str) {
        let decoded = decode_html_entities(raw);
        for ch in decoded.chars() {
            if ch.is_whitespace() {
                // Source indentation is not document content: collapse it so
                // the layout only sees the spaces the author typed.
                self.text.push(' ');
            } else {
                self.text.push(ch);
            }
        }
    }

    fn push_literal(&mut self, literal: &str) {
        self.flush_run();
        self.runs.push(InlineRun {
            text: literal.to_string(),
            bold: self.bold > 0,
            italic: self.italic > 0,
            mono: self.mono > 0,
        });
    }

    /// Starts a block owning tag, emitting whatever was still open. Editors
    /// occasionally leave a paragraph unclosed, so this never drops content.
    fn start_block(
        &mut self,
        kind: BlockKind,
        owner: &str,
        anchor: Option<String>,
        out: &mut Vec<Block>,
    ) {
        self.finish_block(out);
        self.kind = Some(kind);
        self.owner = Some(owner.to_string());
        self.anchor = anchor;
    }

    fn finish_block(&mut self, out: &mut Vec<Block>) {
        self.flush_run();
        // Loose text outside any block (a raw HTML snippet) is still a
        // paragraph: dropping it would silently empty the exported PDF.
        let kind = self.kind.take().unwrap_or(BlockKind::Paragraph);
        self.owner = None;
        let anchor = self.anchor.take();
        let link = self.link.take();
        let mut runs = std::mem::take(&mut self.runs);
        trim_runs(&mut runs);
        if !runs.is_empty() {
            out.push(Block {
                kind,
                runs,
                anchor,
                link,
            });
        }
    }

    fn push_page_break(&mut self, out: &mut Vec<Block>) {
        self.finish_block(out);
        out.push(Block {
            kind: BlockKind::PageBreak,
            runs: Vec::new(),
            anchor: None,
            link: None,
        });
    }

    fn open(&mut self, name: &str, tag: &str, out: &mut Vec<Block>) {
        match name {
            "br" => {
                self.flush_run();
                self.runs.push(InlineRun {
                    text: "\n".to_string(),
                    bold: false,
                    italic: false,
                    mono: false,
                });
            }
            "strong" | "b" => {
                self.flush_run();
                self.bold += 1;
            }
            "em" | "i" => {
                self.flush_run();
                self.italic += 1;
            }
            "code" => {
                self.flush_run();
                self.mono += 1;
            }
            "p" | "div" => {
                if is_page_break(tag) {
                    self.push_page_break(out);
                } else if self.kind.is_none() {
                    self.start_block(BlockKind::Paragraph, name, None, out);
                }
            }
            "hr" => {
                if is_page_break(tag) {
                    self.push_page_break(out);
                } else {
                    self.finish_block(out);
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = name[1..].parse::<u8>().unwrap_or(1);
                // `id="sec-intro"` (editor anchors inserted by the Docs TOC,
                // #2252) is the destination a TOC link points at.
                let anchor = attribute_value(tag, "id").filter(|value| !value.is_empty());
                self.start_block(BlockKind::Heading(level), name, anchor, out);
            }
            // Only same-document links (`href="#anchor"`) can be resolved
            // inside the PDF; external URLs keep plain text.
            "a" => {
                if self.link.is_none() {
                    self.link = attribute_value(tag, "href")
                        .filter(|value| value.starts_with('#') && value.len() > 1)
                        .map(|value| value[1..].to_string());
                }
            }
            "ul" | "ol" => {
                self.finish_block(out);
                let start = if name == "ol" { parse_start(tag) } else { 1 };
                self.lists.push(ListContext {
                    ordered: name == "ol",
                    counter: start.saturating_sub(1),
                });
            }
            "li" => {
                self.finish_block(out);
                let indent = self.lists.len().saturating_sub(1).min(u8::MAX as usize) as u8;
                let marker = match self.lists.last_mut() {
                    Some(context) if context.ordered => {
                        context.counter += 1;
                        format!("{}.", context.counter)
                    }
                    _ => "-".to_string(),
                };
                self.start_block(BlockKind::ListItem { indent, marker }, "li", None, out);
            }
            "blockquote" => {
                self.start_block(BlockKind::Blockquote, name, None, out);
            }
            "pre" => {
                self.finish_block(out);
                self.start_block(BlockKind::Code, "pre", None, out);
            }
            "tr" => {
                self.finish_block(out);
                self.start_block(BlockKind::Paragraph, "tr", None, out);
            }
            "td" | "th" => {
                if !self.runs.is_empty() || !self.text.is_empty() {
                    self.push_literal("  |  ");
                }
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str, out: &mut Vec<Block>) {
        match name {
            "strong" | "b" => {
                self.flush_run();
                self.bold = self.bold.saturating_sub(1);
            }
            "em" | "i" => {
                self.flush_run();
                self.italic = self.italic.saturating_sub(1);
            }
            "code" => {
                self.flush_run();
                self.mono = self.mono.saturating_sub(1);
            }
            "ul" | "ol" => {
                self.finish_block(out);
                self.lists.pop();
            }
            _ => {
                if self.owner.as_deref() == Some(name) {
                    self.finish_block(out);
                }
            }
        }
    }
}

fn trim_runs(runs: &mut Vec<InlineRun>) {
    if let Some(first) = runs.first_mut() {
        first.text = first.text.trim_start().to_string();
    }
    if let Some(last) = runs.last_mut() {
        last.text = last.text.trim_end().to_string();
    }
    runs.retain(|run| !run.text.trim().is_empty());
}

fn tag_name(tag: &str) -> &str {
    tag.split_whitespace().next().unwrap_or("")
}

/// Reads `name="value"` (or `name='value'`) from a raw tag body.
///
/// Used for `id=` (heading destinations) and `href=#...` (TOC links), the two
/// attributes the PDF TOC navigation needs (#2252).
fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let mut search_from = 0;
    while let Some(offset) = lower[search_from..].find(&needle) {
        let absolute = search_from + offset;
        // `data-id=` must not satisfy a lookup for `id=`.
        let glued_to_other_name = absolute > 0
            && lower[..absolute]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
        if glued_to_other_name {
            search_from = absolute + needle.len();
            continue;
        }
        let rest = tag[absolute + needle.len()..].trim_start();
        let value = match rest.chars().next() {
            Some(quote @ ('"' | '\'')) => rest[1..].split(quote).next().unwrap_or(""),
            _ => rest
                .split(|ch: char| ch.is_whitespace() || ch == '>')
                .next()
                .unwrap_or(""),
        };
        return Some(decode_html_entities(value).trim().to_string());
    }
    None
}

/// `class="gb-doc-page-break"` (DOCX converter) and `class="gb-page-break"`
/// (Ctrl+Enter in the editor) both mean "start a new page".
fn is_page_break(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    let name = tag_name(&lower);
    matches!(name, "div" | "hr" | "p") && lower.contains("page-break")
}

/// `<ol start="3">` continues numbering from three.
fn parse_start(tag: &str) -> usize {
    let lower = tag.to_ascii_lowercase();
    let Some(position) = lower.find("start=") else {
        return 1;
    };
    let rest = &lower[position + "start=".len()..];
    let digits: String = rest
        .trim_start_matches(['"', '\''])
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse::<usize>().unwrap_or(1).max(1)
}

/// Converts editor HTML into the block model used by layout.
///
/// Handles the tags the editor and the DOCX importer emit: headings,
/// paragraphs/divs, ordered and unordered lists, blockquotes, code, table
/// rows, inline `<strong>`/`<em>` and the page-break markers.
pub fn parse_blocks(html: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut builder = Builder::default();
    let mut rest = html;
    let mut skip_until: Option<String> = None;

    while !rest.is_empty() {
        if let Some(tag) = skip_until.as_deref() {
            let needle = format!("</{tag}");
            let lowered = rest.to_ascii_lowercase();
            match lowered.find(&needle) {
                Some(position) => {
                    let after = &rest[position..];
                    match after.find('>') {
                        Some(end) => {
                            rest = &after[end + 1..];
                            skip_until = None;
                        }
                        None => break,
                    }
                }
                None => break,
            }
            continue;
        }

        let Some(open) = rest.find('<') else {
            builder.push_text(rest);
            break;
        };
        if open > 0 {
            builder.push_text(&rest[..open]);
        }
        let after = &rest[open..];

        if after.starts_with("<!--") {
            match after.find("-->") {
                Some(end) => rest = &after[end + 3..],
                None => break,
            }
            continue;
        }

        let Some(close) = after.find('>') else {
            builder.push_text(after);
            break;
        };
        let tag = after[1..close].trim();
        rest = &after[close + 1..];
        if tag.is_empty() || tag.starts_with(['!', '?']) {
            continue;
        }

        let closing = tag.starts_with('/');
        let body = if closing { tag[1..].trim() } else { tag };
        let self_closing = body.ends_with('/');
        let body = body.trim_end_matches('/').trim();
        let name = tag_name(body);
        if name.is_empty() {
            continue;
        }

        if closing {
            if matches!(name, "style" | "script" | "head") {
                continue;
            }
            builder.close(name, &mut out);
        } else {
            if matches!(name, "style" | "script" | "head") {
                skip_until = Some(name.to_string());
                continue;
            }
            builder.open(name, body, &mut out);
            if self_closing {
                builder.close(name, &mut out);
            }
        }
    }

    builder.finish_block(&mut out);
    out
}
