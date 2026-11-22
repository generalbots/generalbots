use super::chunk::{stable_id, AclMetadata, StructuredChunk};
use super::citation::SourceRef;

/// Chunk sizing for structure-aware extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkOptions {
    pub max_chars: usize,
    pub overlap_chars: usize,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self {
            max_chars: 900,
            overlap_chars: 150,
        }
    }
}

#[derive(Debug)]
struct Identity {
    document_id: String,
    tenant_id: String,
    file: String,
    version: String,
}

impl Identity {
    fn new(document_id: &str, tenant_id: &str, file: &str, version: &str) -> Self {
        Self {
            document_id: document_id.to_string(),
            tenant_id: tenant_id.to_string(),
            file: file.to_string(),
            version: version.to_string(),
        }
    }
}

#[derive(Debug, Default)]
struct SectionState {
    page: Option<usize>,
    slide: Option<usize>,
    sheet: Option<String>,
    cell: Option<String>,
    clause: Option<String>,
    headings: Vec<(usize, String)>,
    lines: Vec<String>,
}

/// Splits extracted document text into addressed chunks.
///
/// Recognized inline markers (`[[PAGE:n]]`, `[[SLIDE:n]]`, `[[SHEET:name]]`,
/// `[[CELL:ref]]`) come from the office extractors; form feeds and `Page n`
/// lines cover PDFs. Markdown headings and numbered clauses (`7.3`, `Clause
/// 7.3`, `Section 7`) provide the logical address.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructureAwareChunker {
    options: ChunkOptions,
}

impl StructureAwareChunker {
    pub fn new(options: ChunkOptions) -> Self {
        Self { options }
    }

    pub fn chunk_document(
        &self,
        document_id: &str,
        tenant_id: &str,
        file: &str,
        version: &str,
        text: &str,
        acl: &AclMetadata,
    ) -> Vec<StructuredChunk> {
        let identity = Identity::new(document_id, tenant_id, file, version);
        let mut state = SectionState::default();
        let mut chunks = Vec::new();
        let mut page_number = 0usize;

        for page_text in text.split('\u{000c}') {
            page_number += 1;
            state.page = Some(page_number);
            for raw in page_text.lines() {
                let line = raw.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some((key, value)) = Self::marker(line) {
                    self.flush(&mut state, &identity, acl, &mut chunks);
                    match key {
                        "PAGE" => {
                            if let Ok(number) = value.parse::<usize>() {
                                state.page = Some(number);
                            }
                        }
                        "SLIDE" => {
                            if let Ok(number) = value.parse::<usize>() {
                                state.slide = Some(number);
                                state.cell = None;
                            }
                        }
                        "SHEET" => {
                            state.sheet = Some(value.to_string());
                            state.cell = None;
                        }
                        "CELL" => state.cell = Some(value.to_string()),
                        _ => {}
                    }
                    continue;
                }
                if let Some(number) = Self::page_hint(line) {
                    self.flush(&mut state, &identity, acl, &mut chunks);
                    state.page = Some(number);
                    continue;
                }
                if let Some((level, title)) = Self::markdown_heading(line) {
                    self.flush(&mut state, &identity, acl, &mut chunks);
                    Self::set_heading(&mut state.headings, level, title);
                    continue;
                }
                if let Some((level, number, title)) = Self::numbered_heading(line) {
                    self.flush(&mut state, &identity, acl, &mut chunks);
                    let label = if title.is_empty() {
                        number.clone()
                    } else {
                        title
                    };
                    Self::set_heading(&mut state.headings, level, label);
                    if number.contains('.') {
                        state.clause = Some(number);
                    }
                    continue;
                }
                state.lines.push(line.to_string());
            }
        }
        self.flush(&mut state, &identity, acl, &mut chunks);
        chunks
    }

    fn flush(
        &self,
        state: &mut SectionState,
        identity: &Identity,
        acl: &AclMetadata,
        out: &mut Vec<StructuredChunk>,
    ) {
        if state.lines.is_empty() {
            return;
        }
        let text = state.lines.join("\n");
        state.lines.clear();

        let source = Self::source_ref(state);
        let heading_path: Vec<String> = state
            .headings
            .iter()
            .map(|(_, title)| title.clone())
            .collect();
        let address = source.address();
        let path = heading_path.join("|");

        for (index, content) in
            Self::window_chunks(&text, self.options.max_chars, self.options.overlap_chars)
                .into_iter()
                .enumerate()
        {
            let index_text = index.to_string();
            let id = stable_id(&[
                &identity.tenant_id,
                &identity.document_id,
                &identity.version,
                &address,
                &path,
                &index_text,
                &content,
            ]);
            out.push(StructuredChunk {
                id,
                document_id: identity.document_id.clone(),
                file: identity.file.clone(),
                version: identity.version.clone(),
                tenant_id: identity.tenant_id.clone(),
                heading_path: heading_path.clone(),
                source: source.clone(),
                content,
                acl: acl.clone(),
            });
        }
    }

    fn source_ref(state: &SectionState) -> SourceRef {
        let mut source = if let Some(cell) = &state.cell {
            SourceRef::cell(cell.clone())
        } else if let Some(slide) = state.slide {
            SourceRef::slide(slide)
        } else if let Some(clause) = &state.clause {
            SourceRef::clause(clause.clone())
        } else if let Some(sheet) = &state.sheet {
            SourceRef::section(sheet.clone())
        } else if let Some(title) = state.headings.last().map(|(_, title)| title.clone()) {
            SourceRef::section(title)
        } else {
            SourceRef::page(state.page.unwrap_or(1))
        };
        if let Some(page) = state.page {
            source = source.with_page(page);
        }
        source
    }

    fn window_chunks(text: &str, max: usize, overlap: usize) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() <= max {
            return vec![text.to_string()];
        }
        let step = max.saturating_sub(overlap).max(1);
        let mut result = Vec::new();
        let mut start = 0usize;
        while start < chars.len() {
            let mut end = start.saturating_add(max).min(chars.len());
            if end < chars.len() {
                let lower = start.saturating_add(max / 2);
                if lower < end {
                    if let Some(offset) = chars[lower..end].iter().rposition(|c| c.is_whitespace())
                    {
                        end = lower + offset + 1;
                    }
                }
            }
            let piece: String = chars[start..end].iter().collect();
            if piece.trim().len() >= 10 {
                result.push(piece);
            }
            if end >= chars.len() {
                break;
            }
            start = end
                .saturating_sub(overlap)
                .max(start + step.min(end - start));
        }
        result
    }

    fn marker(line: &str) -> Option<(&str, &str)> {
        if !line.starts_with("[[") {
            return None;
        }
        let end = line.find("]]")?;
        let inner = &line[2..end];
        let colon = inner.find(':')?;
        Some((inner[..colon].trim(), inner[colon + 1..].trim()))
    }

    fn page_hint(line: &str) -> Option<usize> {
        let lower = line.to_lowercase();
        let rest = lower.strip_prefix("page ")?;
        rest.split_whitespace().next()?.parse::<usize>().ok()
    }

    fn markdown_heading(line: &str) -> Option<(usize, String)> {
        let level = line.chars().take_while(|c| *c == '#').count();
        if level == 0 || level > 6 {
            return None;
        }
        let title = line[level..].trim();
        if title.is_empty() {
            return None;
        }
        Some((level, title.to_string()))
    }

    fn numbered_heading(line: &str) -> Option<(usize, String, String)> {
        let upper = line.to_uppercase();
        let candidate = if let Some(rest) = upper.strip_prefix("CLAUSE ") {
            rest
        } else if let Some(rest) = upper.strip_prefix("SECTION ") {
            rest
        } else {
            line
        };
        let candidate = candidate.trim();
        let number_end = candidate
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(candidate.len());
        let number = candidate[..number_end].trim_end_matches('.');
        if number.is_empty() || !number.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }
        let title = candidate[number_end..]
            .trim_start_matches(|c: char| c == '.' || c == ':' || c.is_whitespace());
        let level = number.chars().filter(|c| *c == '.').count() + 1;
        Some((level, number.to_string(), title.to_string()))
    }

    fn set_heading(headings: &mut Vec<(usize, String)>, level: usize, title: String) {
        headings.retain(|(existing, _)| *existing < level);
        headings.push((level, title));
    }
}

#[cfg(test)]
mod tests {
    use super::super::SourceKind;
    use super::*;

    fn acl() -> AclMetadata {
        AclMetadata::private("tenant-a")
    }

    fn chunk_doc(doc: &str, file: &str, version: &str, text: &str) -> Vec<StructuredChunk> {
        StructureAwareChunker::default().chunk_document(doc, "t", file, version, text, &acl())
    }

    fn chunk(text: &str) -> Vec<StructuredChunk> {
        chunk_doc("d", "f", "1", text)
    }

    fn chunk_with(options: ChunkOptions, text: &str) -> Vec<StructuredChunk> {
        StructureAwareChunker::new(options).chunk_document("d", "t", "f", "1", text, &acl())
    }

    #[test]
    fn clause_chunk_carries_section_and_page() {
        let text =
            "# Employment\n## Termination\n[[PAGE:12]]\n7.3 Termination Notice\nThe notice period is ninety days.";
        let chunks = chunk_doc("contract", "contract.docx", "3", &text);
        let clause = chunks.iter().find(|c| c.source.locator == "7.3");
        assert!(clause.is_some());
        if let Some(c) = clause {
            assert_eq!(c.source.page, Some(12));
            assert_eq!(c.source.kind, SourceKind::Clause);
            assert!(c.heading_path.iter().any(|h| h.contains("Termination")));
            assert_eq!(c.citation().file, "contract.docx");
            assert_eq!(c.citation().version, "3");
        }
    }

    #[test]
    fn inline_markers_address_page_slide_and_cell() {
        let text =
            "[[PAGE:4]]\nPage four body.\n[[SLIDE:5]]\nQuarterly revenue.\n[[CELL:B2]]\nCell total body.";
        let chunks = chunk(text);
        let find = |locator: &str| {
            chunks
                .iter()
                .find(|c| c.source.locator == locator)
                .map(|c| (&c.source.kind, c.source.page))
        };
        assert_eq!(find("4"), Some((&SourceKind::Page, Some(4))));
        assert_eq!(find("5"), Some((&SourceKind::Slide, Some(4))));
        assert_eq!(find("B2"), Some((&SourceKind::Cell, Some(4))));
    }

    #[test]
    fn sheet_marker_creates_section_address() {
        let chunks = chunk("[[SHEET:Q1]]\nSummary body for the quarter.");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].source.kind, SourceKind::Section);
        assert_eq!(chunks[0].source.locator, "Q1");
        assert_eq!(chunks[0].source.page, Some(1));
    }

    #[test]
    fn form_feeds_advance_page_numbers() {
        let chunks = chunk("Alpha body.\n# Part A\u{000c}Beta body.\n# Part B");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].source.page, Some(1));
        assert_eq!(chunks[1].source.page, Some(2));
        assert_eq!(chunks[0].content, "Alpha body.");
        assert_eq!(chunks[1].content, "Beta body.");
    }

    #[test]
    fn markdown_headings_build_hierarchical_path() {
        let chunks = chunk("# Intro\n## Details\nBody under details.\n### Deep\nDeep body.");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].heading_path, ["Intro", "Details"]);
        assert_eq!(chunks[1].heading_path, ["Intro", "Details", "Deep"]);
    }

    #[test]
    fn stable_ids_repeat_and_change_on_edit() {
        let text = "7.3 Notice\nThe notice period is ninety days from the effective date.";
        let first = chunk(text);
        let second = chunk(text);
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].id, second[0].id);

        let edited = chunk_doc("d", "f", "2", text);
        assert_ne!(first[0].id, edited[0].id);
    }

    #[test]
    fn rechunking_same_text_is_deterministic() {
        let text =
            "# Guide\n7.3 Notice\nThe notice period is ninety days.\n[[SLIDE:2]]\nSlide body.";
        let first = chunk(text);
        let second = chunk(text);
        assert!(!first.is_empty());
        assert_eq!(first, second);
    }

    #[test]
    fn long_sections_are_windowed_within_max_chars() {
        let options = ChunkOptions {
            max_chars: 120,
            overlap_chars: 20,
        };
        let chunks = chunk_with(options, &"Section text repeats. ".repeat(40));
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.content.chars().count() <= 120));
    }

    #[test]
    fn windows_overlap_by_configured_amount() {
        let options = ChunkOptions {
            max_chars: 100,
            overlap_chars: 20,
        };
        let chunks = chunk_with(options, &"word ".repeat(60));
        assert!(chunks.len() >= 2);
        for pair in chunks.windows(2) {
            let prev: Vec<char> = pair[0].content.chars().collect();
            let next: Vec<char> = pair[1].content.chars().collect();
            let tail: String = prev[prev.len() - 20..].iter().collect();
            let head: String = next[..20].iter().collect();
            assert_eq!(tail, head);
        }
    }
}
