//! Resolve stable ids to bounded, citable snippets (#2451).

use super::model::{Citation, CiteError, Document};

/// Find a focus phrase and return its character offsets in `text`.
fn find_focus(text: &str, focus: &str) -> Option<(usize, usize)> {
    let byte_start = text.find(focus)?;
    let start = text.get(..byte_start)?.chars().count();
    Some((start, start.saturating_add(focus.chars().count())))
}

/// Resolve a stable id to a citation. When `focus` is supplied and found, the
/// snippet is a window centred on it; otherwise it starts at the first char.
pub fn cite(
    document: &Document,
    id: &str,
    focus: Option<&str>,
    max_chars: usize,
) -> Result<Citation, CiteError> {
    let block = document
        .blocks
        .iter()
        .find(|block| block.id == id)
        .ok_or_else(|| CiteError::UnknownId { id: id.to_string() })?;
    if block.text.is_empty() {
        return Err(CiteError::EmptyText { id: id.to_string() });
    }

    let chars: Vec<char> = block.text.chars().collect();
    let window = max_chars.max(1);
    let (start, end) = match focus.and_then(|phrase| find_focus(&block.text, phrase)) {
        Some((focus_start, _)) => {
            let half = window / 2;
            let start = focus_start
                .saturating_sub(half)
                .min(chars.len().saturating_sub(window));
            (start, start.saturating_add(window).min(chars.len()))
        }
        None => (0, window.min(chars.len())),
    };
    let snippet: String = chars[start..end].iter().collect();

    Ok(Citation {
        id: id.to_string(),
        kind: block.kind,
        page: block.page,
        text: block.text.clone(),
        snippet,
        start,
        end,
    })
}
