//! Neutral data types for structure-aware document chunking.

use serde::{Deserialize, Serialize};

/// Locator hints that are inherited by following elements until a new marker
/// replaces them. Keeping them on the element (rather than repeating them in
/// text) lets every derived chunk carry a precise source citation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locator {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
}

impl Locator {
    pub fn page(number: u32) -> Self {
        Self {
            page: Some(number),
            ..Self::default()
        }
    }

    pub fn clause(value: impl Into<String>) -> Self {
        Self {
            clause: Some(value.into()),
            ..Self::default()
        }
    }

    pub fn with_clause(mut self, value: impl Into<String>) -> Self {
        self.clause = Some(value.into());
        self
    }
}

/// One parsed element of a structured document.
///
/// A heading opens a new section scope; paragraphs and tables are content in
/// the currently open scope. Tables carry whole rows so the chunker can pack
/// them without ever splitting a row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element {
    Heading { level: usize, text: String },
    Paragraph { text: String, locator: Locator },
    Table { rows: Vec<Vec<String>>, locator: Locator },
}

/// Visibility gate of a document, combined with its explicit ACL.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Any authenticated user inside the tenant.
    Public,
    /// Any authenticated workspace member inside the tenant.
    Workspace,
    /// Only the listed users or groups inside the tenant.
    #[default]
    Private,
}

/// Document-level access control copied onto every derived chunk so retrieval
/// can filter before a single byte of content is returned.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkPermissions {
    pub tenant_id: String,
    pub visibility: Visibility,
    #[serde(default)]
    pub allowed_users: Vec<String>,
    #[serde(default)]
    pub allowed_groups: Vec<String>,
}

/// Input document: stable identity, version, ordered elements and ACL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredDocument {
    pub id: String,
    pub file: String,
    pub version: String,
    pub permissions: ChunkPermissions,
    pub elements: Vec<Element>,
}

/// Sizing options for the chunking pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkOptions {
    /// Maximum number of characters in a chunk's embedded `text` (heading
    /// breadcrumb included).
    pub max_chars: usize,
}

impl Default for ChunkOptions {
    fn default() -> Self {
        Self { max_chars: 900 }
    }
}

/// A KB-ready chunk: embedded text, its citation anchor and permission ACL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredChunk {
    /// Deterministic id of document id + version + structural address + body.
    pub id: String,
    pub document_id: String,
    pub file: String,
    pub version: String,
    /// Ancestor heading titles, outermost first.
    pub heading_path: Vec<String>,
    pub page: Option<u32>,
    pub clause: Option<String>,
    /// Human-readable citation: file, section path, page/clause and version.
    pub citation: String,
    /// Heading breadcrumb followed by the body, ready to embed.
    pub text: String,
    /// Body only, without the breadcrumb.
    pub body: String,
    pub permissions: ChunkPermissions,
}
