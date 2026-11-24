//! Ordered change sets between two semantic document versions (#2451).

use super::model::{Change, Document};

/// Compute an ordered, human-readable diff.
///
/// Additions and modifications are emitted in the order they appear in `to`;
/// removals are emitted at the point where a gap appears in `from`. The result
/// is deterministic and suitable for a preview before applying a patch.
pub fn diff(from: &Document, to: &Document) -> Vec<Change> {
    let mut changes = Vec::new();
    let mut old_cursor = 0usize;

    for (new_position, new_block) in to.blocks.iter().enumerate() {
        let after = if new_position == 0 {
            None
        } else {
            to.blocks
                .get(new_position.saturating_sub(1))
                .map(|block| block.id.clone())
        };

        match from
            .blocks
            .iter()
            .position(|block| block.id == new_block.id)
        {
            Some(old_position) => {
                // A reordered block (old_position < cursor) is reported as a
                // modification only; never slice with an inverted range.
                if old_position >= old_cursor {
                    for skipped in &from.blocks[old_cursor..old_position] {
                        changes.push(Change::Removed {
                            id: skipped.id.clone(),
                            kind: skipped.kind,
                            text: skipped.text.clone(),
                        });
                    }
                    old_cursor = old_position.saturating_add(1);
                }
                let old_block = &from.blocks[old_position];
                if old_block.kind != new_block.kind || old_block.text != new_block.text {
                    changes.push(Change::Modified {
                        id: new_block.id.clone(),
                        before: old_block.text.clone(),
                        after: new_block.text.clone(),
                    });
                }
            }
            None => changes.push(Change::Added {
                after,
                id: new_block.id.clone(),
                kind: new_block.kind,
                text: new_block.text.clone(),
            }),
        }
    }

    for skipped in from.blocks.iter().skip(old_cursor) {
        changes.push(Change::Removed {
            id: skipped.id.clone(),
            kind: skipped.kind,
            text: skipped.text.clone(),
        });
    }

    changes
}
