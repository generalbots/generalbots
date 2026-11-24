//! Bounded, structured read over a semantic document (#2451).

use super::model::{
    BlockView, Document, ReadOptions, ReadView, SEMANTIC_SCHEMA_VERSION, estimate_tokens,
};

/// Produce a compact structured view with page addressing and a token budget.
///
/// Blocks are taken in document order from `from`. An oversized first block is
/// still returned once so a caller always makes forward progress; `truncated`
/// and `next_index` let it page through the rest.
pub fn read_blocks(document: &Document, options: &ReadOptions) -> ReadView {
    let total_blocks = document.blocks.len();
    let total_pages = document
        .blocks
        .iter()
        .map(|block| block.page)
        .max()
        .unwrap_or(1);

    let mut included = Vec::new();
    let mut used_tokens = 0usize;
    let mut next_index = None;

    for (index, block) in document.blocks.iter().enumerate() {
        if index < options.from {
            continue;
        }
        if let Some(page) = options.page {
            if page >= 1 && block.page != page {
                continue;
            }
        }
        let tokens = estimate_tokens(&block.text);
        if !included.is_empty() && used_tokens.saturating_add(tokens) > options.budget_tokens {
            next_index = Some(index);
            break;
        }
        used_tokens = used_tokens.saturating_add(tokens);
        included.push(BlockView {
            id: block.id.clone(),
            kind: block.kind,
            page: block.page,
            index,
            text: block.text.clone(),
            tokens,
        });
    }

    ReadView {
        schema_version: SEMANTIC_SCHEMA_VERSION,
        total_blocks,
        total_pages,
        returned: included.len(),
        used_tokens,
        truncated: next_index.is_some(),
        next_index,
        blocks: included,
    }
}
