//! Knowledge-base indexing writes into the vector database. When the product
//! file switches that service off there is nothing to write into, so every KB
//! file is skipped with one info line per process instead of an error per file.

use std::sync::atomic::{AtomicBool, Ordering};

static NOTICE_GIVEN: AtomicBool = AtomicBool::new(false);

/// Whether KB files may be indexed into the vector database in this run.
pub(super) fn kb_vector_indexing_enabled() -> bool {
    let off = botcore::product::is_service_off("vector_db");
    if off && first_notice(&NOTICE_GIVEN) {
        log::info!("Vector database is switched off for this edition; knowledge-base files are not indexed");
    }
    !off
}

/// True exactly once per flag, so the skip is announced a single time.
fn first_notice(given: &AtomicBool) -> bool {
    !given.swap(true, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_skip_is_announced_only_once() {
        let given = AtomicBool::new(false);
        assert!(first_notice(&given));
        assert!(!first_notice(&given));
        assert!(!first_notice(&given));
    }
}
