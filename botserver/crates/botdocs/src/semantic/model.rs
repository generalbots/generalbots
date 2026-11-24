//! Value types for the semantic document layer (#2451).
//!
//! Everything in this module is a plain data type. The whole layer is pure and
//! can be unit-tested without a server, a database or a running bot.

use serde::{Deserialize, Serialize};

/// Wire schema version. Bump when the shape of the structured view changes.
pub const SEMANTIC_SCHEMA_VERSION: u32 = 1;

/// Block categories the semantic layer understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Paragraph,
    ListItem,
    TableCell,
    Quote,
    Code,
}

impl BlockKind {
    /// Stable wire name (kept in sync with the Docs AST `kind` strings).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Heading1 => "heading1",
            Self::Heading2 => "heading2",
            Self::Heading3 => "heading3",
            Self::Heading4 => "heading4",
            Self::Heading5 => "heading5",
            Self::Heading6 => "heading6",
            Self::Paragraph => "paragraph",
            Self::ListItem => "list-item",
            Self::TableCell => "table-cell",
            Self::Quote => "quote",
            Self::Code => "code",
        }
    }

    /// Canonical HTML tag used when rendering a block back to a document.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Heading1 => "h1",
            Self::Heading2 => "h2",
            Self::Heading3 => "h3",
            Self::Heading4 => "h4",
            Self::Heading5 => "h5",
            Self::Heading6 => "h6",
            Self::Paragraph => "p",
            Self::ListItem => "li",
            Self::TableCell => "td",
            Self::Quote => "blockquote",
            Self::Code => "pre",
        }
    }
}

/// One addressable block. `id` is an opaque, deterministic handle assigned at
/// parse time; it never changes when unrelated blocks are edited or inserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    /// 1-based section address (a new `h1` starts a new page).
    pub page: u32,
    /// Plain text only — inline markup and attributes are never stored.
    pub text: String,
}

/// A parsed document. No owner, author or other PII is represented here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// Options for a bounded structured read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOptions {
    /// Return only this 1-based page/section when set.
    pub page: Option<u32>,
    /// Absolute block index to start from (cursor from a previous read).
    pub from: usize,
    /// Approximate token budget for the returned window.
    pub budget_tokens: usize,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            page: None,
            from: 0,
            budget_tokens: 8_000,
        }
    }
}

/// A block as returned by a structured read, including its document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockView {
    pub id: String,
    pub kind: BlockKind,
    pub page: u32,
    /// 0-based document order, stable cursor for paging.
    pub index: usize,
    pub text: String,
    pub tokens: usize,
}

/// Compact, token-budgeted read result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadView {
    pub schema_version: u32,
    pub total_blocks: usize,
    pub total_pages: u32,
    pub returned: usize,
    pub used_tokens: usize,
    pub truncated: bool,
    pub next_index: Option<usize>,
    pub blocks: Vec<BlockView>,
}

/// Typed edit operations. Callers never supply raw HTML — only plain text that
/// is escaped on render. Every target/anchor is a stable block id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Replace the plain text of the block with `id`.
    Replace { id: String, text: String },
    /// Insert a block after the anchor id, or at document start when `None`.
    Insert {
        after: Option<String>,
        kind: BlockKind,
        text: String,
    },
    /// Delete the block with `id`.
    Delete { id: String },
}

/// One operation as applied, with the id of the block it produced when known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedOp {
    pub operation: Operation,
    pub resulting_id: Option<String>,
}

/// Explicit, typed failure reasons for a patch (maps to HTTP conflict/4xx).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchError {
    /// Caller edited a stale version (`If-Match` mismatch).
    Conflict { expected: u64, actual: u64 },
    /// An `Insert` anchor was not found.
    UnknownAnchor { id: String },
    /// A `Replace`/`Delete` target was not found.
    UnknownTarget { id: String },
    /// No operations were supplied.
    EmptyOperations,
}

/// Result of an all-or-nothing patch. `inverse` reverts the change in one
/// action when fed back to [`crate::semantic::patch`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchResult {
    pub previous_version: u64,
    pub new_version: u64,
    pub document: Document,
    pub applied: Vec<AppliedOp>,
    pub inverse: Vec<Operation>,
}

/// An ordered, human-readable change set between two document versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Added {
        after: Option<String>,
        id: String,
        kind: BlockKind,
        text: String,
    },
    Removed {
        id: String,
        kind: BlockKind,
        text: String,
    },
    Modified {
        id: String,
        before: String,
        after: String,
    },
}

/// A resolved citation: a bounded snippet plus offsets into the full text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub kind: BlockKind,
    pub page: u32,
    /// Full plain text of the addressed block.
    pub text: String,
    /// Excerpt returned to the model / highlighted in the UI.
    pub snippet: String,
    /// Character offsets of `snippet` inside `text`.
    pub start: usize,
    pub end: usize,
}

/// Explicit failure reasons for citation resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CiteError {
    UnknownId { id: String },
    EmptyText { id: String },
}

/// Cheap, pure token estimate for already-extracted plain text (~4 chars/token).
pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    chars / 4 + usize::from(chars % 4 != 0)
}
