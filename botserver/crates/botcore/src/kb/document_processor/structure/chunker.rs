//! Pure chunking pass over parsed elements.
//!
//! Heading titles are kept on every chunk as a breadcrumb, locator hints are
//! inherited, table rows are packed atomically, and ids/citations are derived
//! deterministically from the structural address.

use super::types::{ChunkOptions, Element, StructuredChunk, StructuredDocument};
use std::collections::HashMap;

/// Turns parsed elements into bounded, citable, permission-tagged chunks.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuredChunker {
    options: ChunkOptions,
}

impl StructuredChunker {
    pub fn new(options: ChunkOptions) -> Self {
        Self { options }
    }

    pub fn options(&self) -> ChunkOptions {
        self.options
    }

    /// Chunks a document. Pure: no I/O, no clock, no globals.
    pub fn chunk(&self, doc: &StructuredDocument) -> Vec<StructuredChunk> {
        let mut headings: Vec<(usize, String)> = Vec::new();
        let mut page: Option<u32> = None;
        let mut clause: Option<String> = None;
        let mut sink = ChunkSink::new(doc, self.options.max_chars);

        for element in &doc.elements {
            match element {
                Element::Heading { level, text } => {
                    apply_heading(&mut headings, *level, text);
                }
                Element::Paragraph { text, locator } => {
                    inherit(locator, &mut page, &mut clause);
                    let path = section_path(&headings);
                    let budget = body_budget(&path, self.options.max_chars);
                    for part in split_text(text, budget) {
                        sink.push(part, &path, page, clause.as_deref());
                    }
                }
                Element::Table { rows, locator } => {
                    inherit(locator, &mut page, &mut clause);
                    let path = section_path(&headings);
                    let budget = body_budget(&path, self.options.max_chars);
                    pack_table(&mut sink, rows, &path, page, clause.as_deref(), budget);
                }
            }
        }
        sink.into_chunks()
    }
}

fn apply_heading(headings: &mut Vec<(usize, String)>, level: usize, text: &str) {
    let title = text.trim();
    if title.is_empty() {
        return;
    }
    while headings.last().is_some_and(|(seen, _)| *seen >= level) {
        headings.pop();
    }
    headings.push((level, title.to_string()));
}

fn section_path(headings: &[(usize, String)]) -> Vec<String> {
    headings.iter().map(|(_, title)| title.clone()).collect()
}

fn inherit(locator: &super::types::Locator, page: &mut Option<u32>, clause: &mut Option<String>) {
    if locator.page.is_some() {
        *page = locator.page;
    }
    if let Some(value) = &locator.clause {
        *clause = Some(value.clone());
    }
}

/// Body budget that leaves room for the heading breadcrumb inside `max_chars`.
fn body_budget(path: &[String], max_chars: usize) -> usize {
    let separators = path.len().saturating_sub(1).saturating_mul(3);
    let context: usize = path.iter().map(|title| title.chars().count()).sum();
    max_chars
        .saturating_sub(context)
        .saturating_sub(separators)
        .saturating_sub(1)
        .max(1)
}

fn compose_text(context: &str, body: &str, max_chars: usize) -> String {
    if context.is_empty() {
        return body.to_string();
    }
    let allowed = max_chars
        .saturating_sub(body.chars().count())
        .saturating_sub(1);
    if allowed == 0 {
        return body.to_string();
    }
    let clipped: String = context.chars().take(allowed).collect();
    if clipped.is_empty() {
        body.to_string()
    } else {
        format!("{clipped}\n{body}")
    }
}

/// Whitespace-aware windows of at most `max` characters, on char boundaries.
fn split_text(text: &str, max: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        return vec![text.to_string()];
    }
    let mut windows = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let mut end = start.saturating_add(max).min(chars.len());
        if end < chars.len() {
            if let Some(relative) = chars[start..end]
                .iter()
                .rposition(|character| character.is_whitespace())
            {
                let candidate = start + relative + 1;
                if candidate > start {
                    end = candidate;
                }
            }
        }
        if end <= start {
            end = start.saturating_add(1).min(chars.len());
        }
        windows.push(chars[start..end].iter().collect());
        start = end;
    }
    windows
}

/// Packs whole table rows into chunks; a row is never cut across chunks.
fn pack_table(
    sink: &mut ChunkSink<'_>,
    rows: &[Vec<String>],
    path: &[String],
    page: Option<u32>,
    clause: Option<&str>,
    budget: usize,
) {
    let mut group: Vec<String> = Vec::new();
    let mut used = 0usize;

    for row in rows {
        let rendered = row.join(" | ");
        let cost = rendered.chars().count();
        if !group.is_empty() && used.saturating_add(cost).saturating_add(1) > budget {
            sink.push(group.join("\n"), path, page, clause);
            group.clear();
            used = 0;
        }
        if !group.is_empty() {
            used = used.saturating_add(1);
        }
        used = used.saturating_add(cost);
        group.push(rendered);
    }

    if !group.is_empty() {
        sink.push(group.join("\n"), path, page, clause);
    }
}

/// Accumulates emitted chunks and the ordinal counter used by stable ids.
struct ChunkSink<'a> {
    doc: &'a StructuredDocument,
    max_chars: usize,
    chunks: Vec<StructuredChunk>,
    ordinals: HashMap<String, usize>,
}

impl<'a> ChunkSink<'a> {
    fn new(doc: &'a StructuredDocument, max_chars: usize) -> Self {
        Self {
            doc,
            max_chars,
            chunks: Vec::new(),
            ordinals: HashMap::new(),
        }
    }

    fn push(&mut self, body: String, path: &[String], page: Option<u32>, clause: Option<&str>) {
        let context = path.join(" > ");
        let text = compose_text(&context, &body, self.max_chars);
        let citation = format_citation(&self.doc.file, path, page, clause, &self.doc.version);
        let address = format!(
            "{}|{}|{}|{}|{}|{}",
            self.doc.id,
            self.doc.version,
            self.doc.file,
            path.join("/"),
            page.map_or_else(String::new, |number| format!("p{number}")),
            match clause {
                Some(value) => value,
                None => "",
            }
        );
        let ordinal = self.ordinals.entry(address.clone()).or_insert(0);
        let id = stable_id(&[
            &self.doc.id,
            &self.doc.version,
            &address,
            &ordinal.to_string(),
            &body,
        ]);
        *ordinal += 1;

        self.chunks.push(StructuredChunk {
            id,
            document_id: self.doc.id.clone(),
            file: self.doc.file.clone(),
            version: self.doc.version.clone(),
            heading_path: path.to_vec(),
            page,
            clause: clause.map(str::to_string),
            citation,
            text,
            body,
            permissions: self.doc.permissions.clone(),
        });
    }

    fn into_chunks(self) -> Vec<StructuredChunk> {
        self.chunks
    }
}

fn format_citation(
    file: &str,
    path: &[String],
    page: Option<u32>,
    clause: Option<&str>,
    version: &str,
) -> String {
    let mut citation = file.to_string();
    if !path.is_empty() {
        citation.push_str(" - ");
        citation.push_str(&path.join(" > "));
    }
    if let Some(value) = clause {
        citation.push_str(&format!(" [clause {value}]"));
    }
    if let Some(number) = page {
        citation.push_str(&format!(" (p.{number})"));
    }
    citation.push_str(&format!(" v{version}"));
    citation
}

/// Deterministic 64-bit FNV-1a over explicit components (no dependency).
fn stable_id(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for (position, part) in parts.iter().enumerate() {
        if position > 0 {
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("kb-{hash:016x}")
}
