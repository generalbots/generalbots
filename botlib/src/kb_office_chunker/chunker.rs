//! The pure chunking pass over parsed Office blocks.

use super::types::{BlockLocator, OfficeBlock, OfficeDocument, OfficeKbChunk, SourceAnchor};
use std::collections::HashMap;

/// Chunk sizing for long bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficeChunkOptions {
    pub max_chars: usize,
    pub overlap_chars: usize,
}

impl Default for OfficeChunkOptions {
    fn default() -> Self {
        Self {
            max_chars: 900,
            overlap_chars: 120,
        }
    }
}

/// Chunks parsed Office blocks without flattening their hierarchy.
#[derive(Debug, Clone, Copy, Default)]
pub struct OfficeKbChunker {
    options: OfficeChunkOptions,
}

impl OfficeKbChunker {
    pub fn new(options: OfficeChunkOptions) -> Self {
        Self { options }
    }

    pub fn chunk(&self, doc: &OfficeDocument) -> Vec<OfficeKbChunk> {
        let mut headings: Vec<(u32, String)> = Vec::new();
        let mut inherited = BlockLocator::default();
        let mut ordinals: HashMap<String, usize> = HashMap::new();
        let mut out = Vec::new();

        for block in &doc.blocks {
            self.apply_heading(&mut headings, block);
            self.inherit_locator(&mut inherited, &block.locator);

            let raw = block.text.trim();
            let is_heading = block.is_heading
                && block
                    .heading
                    .as_ref()
                    .is_some_and(|label| !label.trim().is_empty());
            let body = match (&block.heading, raw.is_empty(), is_heading) {
                (Some(label), true, true) => label.trim(),
                _ => raw,
            };
            if body.is_empty() {
                continue;
            }

            let section_path: Vec<String> =
                headings.iter().map(|(_, title)| title.clone()).collect();
            let ancestors: Vec<String> = if is_heading {
                section_path
                    .iter()
                    .take(section_path.len().saturating_sub(1))
                    .cloned()
                    .collect()
            } else {
                section_path.clone()
            };
            let anchor = SourceAnchor {
                file: doc.file.clone(),
                version: doc.version.clone(),
                page: inherited.page,
                slide: inherited.slide,
                sheet: inherited.sheet.clone(),
                cell: inherited.cell.clone(),
                clause: inherited.clause.clone(),
                section_path,
            };
            let address = anchor.address();
            let level = headings.last().map_or(0, |(value, _)| *value);

            for part in split_body(body, self.options.max_chars, self.options.overlap_chars) {
                let ordinal = ordinals.entry(address.clone()).or_insert(0);
                let id = stable_chunk_id(&[&doc.id, &doc.version, &address, &ordinal.to_string()]);
                *ordinal += 1;
                out.push(OfficeKbChunk {
                    id,
                    document_id: doc.id.clone(),
                    file: doc.file.clone(),
                    version: doc.version.clone(),
                    text: compose_text(&ancestors, &part),
                    body: part,
                    anchor: anchor.clone(),
                    heading_path: anchor.section_path.clone(),
                    level,
                });
            }
        }
        out
    }

    fn apply_heading(&self, headings: &mut Vec<(u32, String)>, block: &OfficeBlock) {
        let Some(label) = &block.heading else {
            return;
        };
        if !block.is_heading || label.trim().is_empty() {
            return;
        }
        while headings
            .last()
            .is_some_and(|(level, _)| *level >= block.level)
        {
            headings.pop();
        }
        headings.push((block.level, label.trim().to_string()));
    }

    fn inherit_locator(&self, inherited: &mut BlockLocator, locator: &BlockLocator) {
        if let Some(page) = locator.page {
            inherited.page = Some(page);
        }
        if let Some(slide) = locator.slide {
            inherited.slide = Some(slide);
            inherited.cell = None;
        }
        if let Some(sheet) = &locator.sheet {
            inherited.sheet = Some(sheet.clone());
            inherited.cell = None;
        }
        if let Some(cell) = &locator.cell {
            inherited.cell = Some(cell.clone());
        }
        if let Some(clause) = &locator.clause {
            inherited.clause = Some(clause.clone());
        }
    }
}

fn compose_text(ancestors: &[String], body: &str) -> String {
    if ancestors.is_empty() {
        body.to_string()
    } else {
        format!("{}\n{}", ancestors.join(" > "), body)
    }
}

/// Deterministic 64-bit FNV-1a over an explicit list of components.
fn stable_chunk_id(parts: &[&str]) -> String {
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

/// Split a body into overlapping windows on char boundaries, whitespace-aware.
fn split_body(body: &str, max_chars: usize, overlap_chars: usize) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    let max = max_chars.max(1);
    if chars.len() <= max {
        return vec![body.to_string()];
    }
    let overlap = overlap_chars.min(max.saturating_sub(1));
    let step = max.saturating_sub(overlap).max(1);
    let mut result = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let mut end = start.saturating_add(max).min(chars.len());
        if end < chars.len() {
            let lower = start.saturating_add(max / 2);
            if lower < end {
                if let Some(relative) = chars[lower..end].iter().rposition(|c| c.is_whitespace()) {
                    end = lower + relative + 1;
                }
            }
        }
        if end <= start {
            end = start.saturating_add(1).min(chars.len());
        }
        result.push(chars[start..end].iter().collect());
        if end >= chars.len() {
            break;
        }
        start = end
            .saturating_sub(overlap)
            .max(start.saturating_add(step.min(end - start)));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(text: &str) -> OfficeBlock {
        OfficeBlock::text(text, BlockLocator::default())
    }

    #[test]
    fn heading_nesting_is_preserved_and_popped() {
        let doc = OfficeDocument::new(
            "contract",
            "contract.docx",
            "3",
            vec![
                OfficeBlock::heading(1, "7 Termination", BlockLocator::default()),
                OfficeBlock::heading(2, "7.3 Notice", BlockLocator::clause("7.3")),
                body("Ninety days notice."),
                OfficeBlock::heading(1, "8 Governing law", BlockLocator::default()),
                body("Courts of Sao Paulo."),
            ],
        );
        let chunks = OfficeKbChunker::default().chunk(&doc);
        assert_eq!(chunks.len(), 5);
        assert_eq!(chunks[1].heading_path, ["7 Termination", "7.3 Notice"]);
        assert_eq!(chunks[1].level, 2);
        assert_eq!(chunks[2].body, "Ninety days notice.");
        assert!(chunks[2].text.starts_with("7 Termination > 7.3 Notice\n"));
        assert_eq!(chunks[3].heading_path, ["8 Governing law"]);
        assert!(!chunks[3].heading_path.iter().any(|h| h.contains("7.3")));
    }

    #[test]
    fn slides_and_sheets_keep_their_own_hierarchy() {
        let doc = OfficeDocument::new(
            "deck",
            "quarterly.pptx",
            "1",
            vec![
                OfficeBlock::slide(2),
                body("Revenue up 12%."),
                OfficeBlock::text("Speaker: compare Q1.", BlockLocator::default()),
            ],
        );
        let chunks = OfficeKbChunker::default().chunk(&doc);
        assert_eq!(chunks[0].anchor.slide, Some(2));
        assert_eq!(chunks[0].heading_path, ["Slide 2"]);
        assert_eq!(chunks[1].anchor.slide, Some(2));
        assert_eq!(chunks[1].heading_path, ["Slide 2"]);

        let sheet = OfficeDocument::new(
            "book",
            "fy26.xlsx",
            "2",
            vec![OfficeBlock::sheet("FY26"), OfficeBlock::cell("1450", "C14")],
        );
        let chunks = OfficeKbChunker::default().chunk(&sheet);
        assert_eq!(chunks[0].anchor.sheet.as_deref(), Some("FY26"));
        assert_eq!(chunks[0].heading_path, ["FY26"]);
        assert_eq!(chunks[1].anchor.sheet.as_deref(), Some("FY26"));
        assert_eq!(chunks[1].anchor.cell.as_deref(), Some("C14"));
        assert_eq!(chunks[1].heading_path, ["FY26"]);
    }

    #[test]
    fn source_anchors_are_deterministic_and_citable() {
        let doc = OfficeDocument::new(
            "contract",
            "contract.docx",
            "3",
            vec![
                OfficeBlock::heading(1, "Employment", BlockLocator::default()),
                OfficeBlock::heading(2, "Termination", BlockLocator::clause("7.3").with_page(12)),
                body("The notice period is ninety days."),
            ],
        );
        let chunker = OfficeKbChunker::default();
        let first = chunker.chunk(&doc);
        let second = chunker.chunk(&doc);
        assert_eq!(first, second);
        let chunk = &first[1];
        assert_eq!(chunk.anchor.file, "contract.docx");
        assert_eq!(chunk.anchor.page, Some(12));
        assert_eq!(chunk.anchor.clause.as_deref(), Some("7.3"));
        assert_eq!(chunk.anchor.section_path, ["Employment", "Termination"]);
        assert!(chunk.anchor.address().contains("p12"));
        assert!(chunk.anchor.address().contains("clause:7.3"));
        let citation = chunk.anchor.citation();
        assert!(citation.contains("contract.docx"));
        assert!(citation.contains("clause 7.3"));
        assert!(citation.contains("p.12"));
        assert!(citation.ends_with("v3"));

        let bumped = OfficeDocument {
            version: "4".to_string(),
            ..doc.clone()
        };
        assert_ne!(chunker.chunk(&bumped)[1].id, chunk.id);
    }

    #[test]
    fn empty_blocks_are_skipped_and_locators_inherit() {
        let doc = OfficeDocument::new(
            "d",
            "f.docx",
            "1",
            vec![
                body(""),
                OfficeBlock::text("Page marker", BlockLocator::page(4)),
                body("Still on page four."),
                OfficeBlock::text("Next page", BlockLocator::page(5)),
                body("Now page five."),
            ],
        );
        let chunks = OfficeKbChunker::default().chunk(&doc);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].anchor.page, Some(4));
        assert_eq!(chunks[1].anchor.page, Some(4));
        assert_eq!(chunks[2].anchor.page, Some(5));
        assert_eq!(chunks[3].anchor.page, Some(5));
    }

    #[test]
    fn long_bodies_are_windowed_with_distinct_ids() {
        let options = OfficeChunkOptions {
            max_chars: 60,
            overlap_chars: 10,
        };
        let doc = OfficeDocument::new("d", "f.txt", "1", vec![body(&"word ".repeat(80))]);
        let chunks = OfficeKbChunker::new(options).chunk(&doc);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.body.chars().count() <= 60));
        let mut ids: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), chunks.len());
    }
}
