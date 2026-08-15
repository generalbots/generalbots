//! Media anchored to worksheets (E6).

use serde::{Deserialize, Serialize};

/// An image anchored to a worksheet. Row/col are 0-based, matching the `data`
/// keys. The image bytes themselves stay in `source_bytes` and are preserved
/// byte-for-byte by the passthrough save; this model only records the anchor
/// so the grid knows where to render it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetImage {
    pub id: String,
    pub name: String,
    pub row: u32,
    pub col: u32,
}
