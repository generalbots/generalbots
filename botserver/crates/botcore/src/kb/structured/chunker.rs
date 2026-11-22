use super::citation::SourceRef;
use super::chunk::{stable_id, AclMetadata, StructuredChunk};

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

        for (index, content) in Self::window_chunks(
            &text,
            self.options.max_chars,
            self.options.overlap_chars,
        )
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
            start = end.saturating_sub(overlap).max(start + step.min(end - start));
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
    use super::*;

    fn acl() -> AclMetadata {
        AclMetadata::private("tenant-a")
    }

    #[test]
    fn clause_chunk_carries_section_and_page() {
        let text = [
            "# Employment",
            "## Termination",
            "[[PAGE:12]]",
            "7.3 Termination Notice",
            "The termination notice period is ninety days.",
        ]
        .join("\n");
        let chunks = StructureAwareChunker::default().chunk_document(
            "contract", "tenant-a", "contract.docx", "3", &text, &acl(),
        );
        let clause = chunks
            .iter()
            .find(|chunk| chunk.source.locator == "7.3");
        assert!(clause.is_some());
        if let Some(chunk) = clause {
            assert_eq!(chunk.source.page, Some(12));
            assert_eq!(chunk.source.kind, super::super::SourceKind::Clause);
            assert!(chunk
                .heading_path
                .iter()
                .any(|heading| heading.contains("Termination")));
            assert_eq!(chunk.citation().file, "contract.docx");
            assert_eq!(chunk.citation().version, "3");
        }
    }

    #[test]
    fn slide_and_cell_markers_are_addressed() {
        let text = [
            "[[SLIDE:5]]",
            "Quarterly revenue grew by twelve percent.",
            "[[SHEET:Q1]]",
            "[[CELL:B2]]",
            "Revenue total for the quarter.",
        ]
        .join("\n");
        let chunks = StructureAwareChunker::default().chunk_document(
            "deck", "tenant-a", "deck.pptx", "1", &text, &acl(),
        );
        assert!(chunks.iter().any(|c| c.source.locator == "5"));
        assert!(chunks.iter().any(|c| c.source.locator == "B2"));
    }

    #[test]
    fn stable_ids_repeat_and_change_on_edit() {
        let chunker = StructureAwareChunker::default();
        let text = "7.3 Notice\nThe notice period is ninety days from the effective date.";
        let first = chunker.chunk_document("doc", "t", "f", "1", text, &acl());
        let second = chunker.chunk_document("doc", "t", "f", "1", text, &acl());
        assert_eq!(first.len(), second.len());
        assert_eq!(first[0].id, second[0].id);

        let edited = chunker.chunk_document(
            "doc", "t", "f", "2",
            "7.3 Notice\nThe notice period is thirty days from the effective date.",
            &acl(),
        );
        assert_ne!(first[0].id, edited[0].id);
    }

    #[test]
    fn long_sections_are_windowed_within_max_chars() {
        let options = ChunkOptions {
            max_chars: 120,
            overlap_chars: 20,
        };
        let chunker = StructureAwareChunker::new(options);
        let text = "Section text repeats. ".repeat(40);
        let chunks = chunker.chunk_document("doc", "t", "f", "1", &text, &acl());
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| chunk.content.chars().count() <= 120));
    }
}
