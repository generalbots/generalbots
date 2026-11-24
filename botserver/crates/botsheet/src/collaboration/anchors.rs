//! Stable comment anchors that survive row/column shifts (#2447).
//!
//! Comments are stored by the literal `"row,col"` key of a cell. Insert a row
//! above a comment and the key still points at the old coordinates — the
//! thread visibly detaches from the sentence it refers to. This module gives
//! each thread a stable [`AnchorId`] bound to a logical [`GridPoint`], and
//! updates that point when rows or columns are inserted or deleted. The id
//! never changes, so clients can keep a comment attached to its cell through
//! arbitrary structural edits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Opaque, monotonic id for a comment thread anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnchorId(pub u64);

/// Coordinates of the cell an anchor currently points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPoint {
    pub worksheet: usize,
    pub row: u32,
    pub col: u32,
}

impl GridPoint {
    pub fn new(worksheet: usize, row: u32, col: u32) -> Self {
        Self {
            worksheet,
            row,
            col,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorStatus {
    /// The anchor resolves to a live cell.
    Live,
    /// Its cell was deleted; the id is retained for audit but no longer maps
    /// to a location, so clients can show a "comment on deleted content" state.
    Orphaned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Anchor {
    point: GridPoint,
    status: AnchorStatus,
}

/// Registry mapping stable anchor ids to shifted cell coordinates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnchorRegistry {
    next_id: u64,
    anchors: HashMap<AnchorId, Anchor>,
}

impl AnchorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, point: GridPoint) -> AnchorId {
        let id = AnchorId(self.next_id);
        self.next_id += 1;
        self.anchors.insert(
            id,
            Anchor {
                point,
                status: AnchorStatus::Live,
            },
        );
        id
    }

    /// Current cell for a live anchor, or `None` if orphaned/unknown.
    pub fn resolve(&self, id: &AnchorId) -> Option<GridPoint> {
        self.anchors
            .get(id)
            .filter(|anchor| anchor.status == AnchorStatus::Live)
            .map(|anchor| anchor.point)
    }

    pub fn status(&self, id: &AnchorId) -> Option<AnchorStatus> {
        self.anchors.get(id).map(|anchor| anchor.status)
    }

    /// Shift every live anchor in `worksheet` at or below `at` down by `count`.
    pub fn insert_rows(&mut self, worksheet: usize, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        for anchor in self.anchors.values_mut() {
            if anchor.point.worksheet == worksheet
                && anchor.status == AnchorStatus::Live
                && anchor.point.row >= at
            {
                anchor.point.row = anchor.point.row.saturating_add(count);
            }
        }
    }

    /// Shift every live anchor in `worksheet` at or right of `at` right by
    /// `count`.
    pub fn insert_columns(&mut self, worksheet: usize, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        for anchor in self.anchors.values_mut() {
            if anchor.point.worksheet == worksheet
                && anchor.status == AnchorStatus::Live
                && anchor.point.col >= at
            {
                anchor.point.col = anchor.point.col.saturating_add(count);
            }
        }
    }

    /// Delete rows `[at, at + count)`: anchors inside the removed range become
    /// orphaned, anchors below it shift up. Anchors on other worksheets are
    /// untouched.
    pub fn delete_rows(&mut self, worksheet: usize, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        let end = at.saturating_add(count);
        for anchor in self.anchors.values_mut() {
            if anchor.point.worksheet != worksheet || anchor.status != AnchorStatus::Live {
                continue;
            }
            if anchor.point.row >= at && anchor.point.row < end {
                anchor.status = AnchorStatus::Orphaned;
            } else if anchor.point.row >= end {
                anchor.point.row = anchor.point.row.saturating_sub(count);
            }
        }
    }

    /// Delete columns `[at, at + count)` with the same semantics as rows.
    pub fn delete_columns(&mut self, worksheet: usize, at: u32, count: u32) {
        if count == 0 {
            return;
        }
        let end = at.saturating_add(count);
        for anchor in self.anchors.values_mut() {
            if anchor.point.worksheet != worksheet || anchor.status != AnchorStatus::Live {
                continue;
            }
            if anchor.point.col >= at && anchor.point.col < end {
                anchor.status = AnchorStatus::Orphaned;
            } else if anchor.point.col >= end {
                anchor.point.col = anchor.point.col.saturating_sub(count);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{AnchorRegistry, AnchorStatus, GridPoint};

    /// Acceptance criterion: a comment stays on its sentence after 20
    /// insertions above it, and the anchor id is unchanged throughout.
    #[test]
    fn anchor_survives_twenty_insertions_above() {
        let mut registry = AnchorRegistry::new();
        let id = registry.create(GridPoint::new(0, 20, 3));
        for _ in 0..20 {
            registry.insert_rows(0, 0, 1);
        }
        assert_eq!(registry.resolve(&id), Some(GridPoint::new(0, 40, 3)));
        assert_eq!(registry.status(&id), Some(AnchorStatus::Live));
    }

    #[test]
    fn anchor_shifts_with_column_insertions() {
        let mut registry = AnchorRegistry::new();
        let id = registry.create(GridPoint::new(0, 5, 5));
        registry.insert_columns(0, 2, 3);
        assert_eq!(registry.resolve(&id), Some(GridPoint::new(0, 5, 8)));
        // A column inserted after the anchor does not move it.
        registry.insert_columns(0, 20, 1);
        assert_eq!(registry.resolve(&id), Some(GridPoint::new(0, 5, 8)));
    }

    #[test]
    fn deleted_range_orphans_anchor_and_shifts_others() {
        let mut registry = AnchorRegistry::new();
        let removed = registry.create(GridPoint::new(0, 3, 0));
        let below = registry.create(GridPoint::new(0, 10, 0));
        registry.delete_rows(0, 3, 2);

        assert_eq!(registry.status(&removed), Some(AnchorStatus::Orphaned));
        assert_eq!(registry.resolve(&removed), None);
        assert_eq!(registry.resolve(&below), Some(GridPoint::new(0, 8, 0)));
    }

    #[test]
    fn anchors_on_other_worksheets_are_unaffected() {
        let mut registry = AnchorRegistry::new();
        let other = registry.create(GridPoint::new(1, 5, 5));
        registry.insert_rows(0, 0, 4);
        registry.delete_columns(0, 0, 2);
        assert_eq!(registry.resolve(&other), Some(GridPoint::new(1, 5, 5)));
    }
}
