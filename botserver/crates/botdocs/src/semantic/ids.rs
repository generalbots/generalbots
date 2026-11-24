//! Deterministic, content-derived stable ids (#2451).
//!
//! An id depends only on a block's kind, normalized text and the number of
//! earlier identical siblings. It never depends on document order alone, so
//! editing or inserting an *unrelated* block cannot change an existing id.

use super::model::{Block, BlockKind};
use std::collections::HashMap;

/// Collapse whitespace so cosmetic reformatting does not change an id.
pub(crate) fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn mix(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

/// Build the stable id for a block instance. Identical siblings are
/// disambiguated by `occurrence` (their 0-based index among identical blocks).
pub fn stable_id(kind: BlockKind, text: &str, occurrence: usize) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    mix(&mut hash, kind.as_str().as_bytes());
    mix(&mut hash, b"\x1f");
    mix(&mut hash, normalize_text(text).as_bytes());
    mix(&mut hash, b"\x1f");
    mix(&mut hash, occurrence.to_string().as_bytes());
    if occurrence == 0 {
        format!("s-{hash:016x}")
    } else {
        format!("s-{hash:016x}-{occurrence}")
    }
}

/// Assign ids to every freshly parsed block, in document order.
pub(crate) fn assign_ids(blocks: &mut [Block]) {
    let mut seen: HashMap<(BlockKind, String), usize> = HashMap::new();
    for block in blocks.iter_mut() {
        let key = (block.kind, normalize_text(&block.text));
        let occurrence = *seen.get(&key).unwrap_or(&0);
        block.id = stable_id(block.kind, &block.text, occurrence);
        seen.insert(key, occurrence.saturating_add(1));
    }
}

/// Pick an unused id for a newly inserted block at parse-independent time.
pub(crate) fn next_id(blocks: &[Block], kind: BlockKind, text: &str) -> String {
    let mut occurrence = 0usize;
    loop {
        let candidate = stable_id(kind, text, occurrence);
        if !blocks.iter().any(|block| block.id == candidate) {
            return candidate;
        }
        occurrence = occurrence.saturating_add(1);
    }
}
