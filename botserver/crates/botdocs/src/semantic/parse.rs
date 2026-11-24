//! Dependency-free HTML-ish parser and canonical renderer (#2451).
//!
//! The parser keeps only plain text of block elements; inline tags and all
//! attributes are dropped. Rendering re-escapes text, so no raw HTML supplied
//! by a caller is ever written back into a document.

use super::ids::{assign_ids, normalize_text};
use super::model::{Block, BlockKind, Document};
use crate::storage::{decode_html_entities, escape_html};

/// Map a block tag name to its typed kind. Container/inline tags return `None`.
fn block_kind(name: &str) -> Option<BlockKind> {
    match name {
        "h1" => Some(BlockKind::Heading1),
        "h2" => Some(BlockKind::Heading2),
        "h3" => Some(BlockKind::Heading3),
        "h4" => Some(BlockKind::Heading4),
        "h5" => Some(BlockKind::Heading5),
        "h6" => Some(BlockKind::Heading6),
        "p" => Some(BlockKind::Paragraph),
        "li" => Some(BlockKind::ListItem),
        "td" | "th" => Some(BlockKind::TableCell),
        "blockquote" => Some(BlockKind::Quote),
        "pre" => Some(BlockKind::Code),
        _ => None,
    }
}

/// Flush the accumulated text buffer as one block, if it is not empty.
fn push_block(blocks: &mut Vec<Block>, text: &mut String, kind: BlockKind, page: u32) {
    let decoded = decode_html_entities(text.trim());
    let plain = normalize_text(&decoded);
    text.clear();
    if plain.is_empty() {
        return;
    }
    blocks.push(Block {
        id: String::new(),
        kind,
        page,
        text: plain,
    });
}

/// Parse an HTML-ish document into flat, page-addressed, id-stable blocks.
pub fn parse(html: &str) -> Document {
    let mut blocks = Vec::new();
    let mut text = String::new();
    let mut open_kind: Option<BlockKind> = None;
    let mut in_tag = false;
    let mut tag = String::new();
    let mut page = 1u32;

    for ch in html.chars() {
        match ch {
            '<' if !in_tag => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let raw = tag.trim();
                let closing = raw.starts_with('/');
                let self_closing = raw.ends_with('/');
                let name = raw
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if let Some(kind) = block_kind(&name) {
                    if closing {
                        if open_kind.take().is_some() {
                            push_block(&mut blocks, &mut text, kind, page);
                        }
                    } else if !self_closing {
                        // A later `h1` starts a new section; its heading and
                        // body share the advanced page address.
                        if name == "h1" && !blocks.is_empty() {
                            page = page.saturating_add(1);
                        }
                        push_block(&mut blocks, &mut text, BlockKind::Paragraph, page);
                        open_kind = Some(kind);
                    }
                } else if !closing && (name == "br" || name == "br/") {
                    text.push(' ');
                }
            }
            '<' => tag.push(ch),
            _ if in_tag => tag.push(ch),
            _ => text.push(ch),
        }
    }
    push_block(
        &mut blocks,
        &mut text,
        open_kind.unwrap_or(BlockKind::Paragraph),
        page,
    );
    assign_ids(&mut blocks);
    Document { blocks }
}

/// Render blocks back to canonical HTML. Text is always escaped; there is no
/// path for a caller to inject raw markup through a patch operation.
pub fn render(document: &Document) -> String {
    let mut output = String::new();
    for block in &document.blocks {
        let tag = block.kind.tag();
        output.push('<');
        output.push_str(tag);
        output.push('>');
        output.push_str(&escape_html(&block.text));
        output.push_str("</");
        output.push_str(tag);
        output.push('>');
    }
    output
}
