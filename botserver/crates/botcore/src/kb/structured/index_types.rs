use super::{AclMetadata, Citation, StructuredChunk};
use serde::{Deserialize, Serialize};

/// Per-document state exposed to the index-status UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexStatus {
    Indexed,
    Skipped(String),
    Archived,
}

#[derive(Debug, Clone)]
pub struct DocumentRecord {
    pub document_id: String,
    pub tenant_id: String,
    pub file: String,
    pub version: String,
    pub status: IndexStatus,
    pub chunk_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexDocument {
    pub document_id: String,
    pub tenant_id: String,
    pub file: String,
    pub version: String,
    pub text: String,
    pub acl: AclMetadata,
}

impl IndexDocument {
    pub fn new(
        document_id: &str,
        tenant_id: &str,
        file: &str,
        version: &str,
        text: &str,
        acl: AclMetadata,
    ) -> Self {
        Self {
            document_id: document_id.to_string(),
            tenant_id: tenant_id.to_string(),
            file: file.to_string(),
            version: version.to_string(),
            text: text.to_string(),
            acl,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackfillEntry {
    pub document_id: String,
    pub file: String,
    pub status: IndexStatus,
    pub chunk_count: usize,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BackfillReport {
    pub entries: Vec<BackfillEntry>,
}

impl BackfillReport {
    pub fn total(&self) -> usize {
        self.entries.len()
    }

    pub fn indexed(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.status == IndexStatus::Indexed)
            .count()
    }

    pub fn skipped(&self) -> usize {
        self.total() - self.indexed()
    }

    /// True when every supplied document appears exactly once in the report.
    pub fn accounts_for(&self, document_ids: &[String]) -> bool {
        document_ids.iter().all(|id| {
            self.entries
                .iter()
                .filter(|entry| &entry.document_id == id)
                .count()
                == 1
        })
    }
}

#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub chunk: StructuredChunk,
    pub citation: Citation,
    pub score: f32,
}
