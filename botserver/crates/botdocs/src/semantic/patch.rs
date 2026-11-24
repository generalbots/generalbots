//! Atomic, deterministic patch application by stable id (#2451).

use super::ids::next_id;
use super::model::{AppliedOp, Block, Document, Operation, PatchError, PatchResult};

fn find_position(blocks: &[Block], id: &str) -> Result<usize, PatchError> {
    blocks
        .iter()
        .position(|block| block.id == id)
        .ok_or_else(|| PatchError::UnknownTarget { id: id.to_string() })
}

/// Apply ordered operations to a copy of the document.
///
/// `requested_version` is the version the caller believes it is editing (the
/// `If-Match` value); `current_version` is the version actually stored. A
/// mismatch is a conflict and no operation is applied. The same patch applied
/// to the same version always yields the same result, and the returned
/// `inverse` reverts the change in one action.
pub fn patch(
    document: &Document,
    requested_version: u64,
    current_version: u64,
    operations: &[Operation],
) -> Result<PatchResult, PatchError> {
    if requested_version != current_version {
        return Err(PatchError::Conflict {
            expected: requested_version,
            actual: current_version,
        });
    }
    if operations.is_empty() {
        return Err(PatchError::EmptyOperations);
    }

    let mut blocks = document.blocks.clone();
    let mut applied: Vec<AppliedOp> = Vec::new();
    let mut inverse: Vec<Operation> = Vec::new();
    // (index into `applied`, id assigned to the inserted block)
    let mut inserted: Vec<(usize, String)> = Vec::new();

    for operation in operations {
        match operation {
            Operation::Replace { id, text } => {
                let position = find_position(&blocks, id)?;
                let before = blocks[position].text.clone();
                blocks[position].text = text.clone();
                applied.push(AppliedOp {
                    operation: operation.clone(),
                    resulting_id: Some(id.clone()),
                });
                inverse.push(Operation::Replace {
                    id: id.clone(),
                    text: before,
                });
            }
            Operation::Insert { after, kind, text } => {
                let position = match after {
                    Some(anchor) => {
                        let anchor_position = find_position(&blocks, anchor)
                            .map_err(|_| PatchError::UnknownAnchor { id: anchor.clone() })?;
                        anchor_position.saturating_add(1)
                    }
                    None => 0,
                };
                let new_id = next_id(&blocks, *kind, text);
                let page = if position == 0 {
                    1
                } else {
                    blocks[position.saturating_sub(1)].page
                };
                blocks.insert(
                    position,
                    Block {
                        id: new_id.clone(),
                        kind: *kind,
                        page,
                        text: text.clone(),
                    },
                );
                let applied_index = applied.len();
                applied.push(AppliedOp {
                    operation: operation.clone(),
                    resulting_id: None,
                });
                inverse.push(Operation::Delete { id: new_id.clone() });
                inserted.push((applied_index, new_id));
            }
            Operation::Delete { id } => {
                let position = find_position(&blocks, id)?;
                let removed = blocks.remove(position);
                let after = if position == 0 {
                    None
                } else {
                    blocks.get(position.saturating_sub(1)).map(|b| b.id.clone())
                };
                applied.push(AppliedOp {
                    operation: operation.clone(),
                    resulting_id: Some(id.clone()),
                });
                inverse.push(Operation::Insert {
                    after,
                    kind: removed.kind,
                    text: removed.text,
                });
            }
        }
    }

    // An inserted block may itself have been deleted later in the same patch;
    // in that case it has no resulting id to report.
    for (applied_index, new_id) in inserted {
        if blocks.iter().any(|block| block.id == new_id) {
            if let Some(entry) = applied.get_mut(applied_index) {
                entry.resulting_id = Some(new_id);
            }
        }
    }

    // Inverses were recorded alongside their forward op; undoing a sequence
    // requires replaying them in reverse so anchors still exist at each step.
    inverse.reverse();

    Ok(PatchResult {
        previous_version: requested_version,
        new_version: current_version.saturating_add(1),
        document: Document { blocks },
        applied,
        inverse,
    })
}
