use super::chunker::StructureAwareChunker;
use super::index_types::{
    BackfillEntry, BackfillReport, DocumentRecord, IndexDocument, IndexStatus, RetrievedChunk,
};
use super::StructuredChunk;
use crate::kb::permissions::UserContext;
use std::cmp::Ordering;
use std::collections::HashMap;

/// In-memory, tenant-scoped projection of the structured KB index.
///
/// It is intentionally storage-agnostic: the production indexer can upsert the
/// same chunks and ACL payload into Qdrant while reusing `retrieve` as the
/// post-filter that guarantees no unauthorized or cross-tenant text is
/// returned.
#[derive(Debug, Default)]
pub struct StructuredKbIndex {
    chunker: StructureAwareChunker,
    documents: HashMap<String, DocumentRecord>,
    chunks: HashMap<String, StructuredChunk>,
}

impl StructuredKbIndex {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(tenant_id: &str, document_id: &str) -> String {
        format!("{tenant_id}|{document_id}")
    }

    pub fn index_document(&mut self, document: &IndexDocument) -> BackfillEntry {
        let key = Self::key(&document.tenant_id, &document.document_id);
        if let Some(previous) = self.documents.remove(&key) {
            for id in previous.chunk_ids {
                self.chunks.remove(&id);
            }
        }

        let (status, reason) = if document.text.trim().is_empty() {
            (
                IndexStatus::Skipped("empty document".to_string()),
                Some("empty document".to_string()),
            )
        } else {
            (IndexStatus::Indexed, None)
        };

        let chunks = if status == IndexStatus::Indexed {
            self.chunker.chunk_document(
                &document.document_id,
                &document.tenant_id,
                &document.file,
                &document.version,
                &document.text,
                &document.acl,
            )
        } else {
            Vec::new()
        };
        let chunk_ids: Vec<String> = chunks.iter().map(|chunk| chunk.id.clone()).collect();
        let chunk_count = chunk_ids.len();
        for chunk in chunks {
            self.chunks.insert(chunk.id.clone(), chunk);
        }
        self.documents.insert(
            key,
            DocumentRecord {
                document_id: document.document_id.clone(),
                tenant_id: document.tenant_id.clone(),
                file: document.file.clone(),
                version: document.version.clone(),
                status: status.clone(),
                chunk_ids,
            },
        );

        BackfillEntry {
            document_id: document.document_id.clone(),
            file: document.file.clone(),
            status,
            chunk_count,
            reason,
        }
    }

    pub fn backfill(&mut self, documents: &[IndexDocument]) -> BackfillReport {
        BackfillReport {
            entries: documents
                .iter()
                .map(|document| self.index_document(document))
                .collect(),
        }
    }

    pub fn status(&self, tenant_id: &str, document_id: &str) -> Option<&IndexStatus> {
        self.documents
            .get(&Self::key(tenant_id, document_id))
            .map(|record| &record.status)
    }

    /// Removes all chunks and the status row. Retrieval cannot reveal that the
    /// document ever existed.
    pub fn delete_document(&mut self, tenant_id: &str, document_id: &str) -> bool {
        match self.documents.remove(&Self::key(tenant_id, document_id)) {
            Some(record) => {
                for id in record.chunk_ids {
                    self.chunks.remove(&id);
                }
                true
            }
            None => false,
        }
    }

    /// Removes all chunks but keeps a visible `Archived` status row.
    pub fn archive_document(&mut self, tenant_id: &str, document_id: &str) -> bool {
        let key = Self::key(tenant_id, document_id);
        match self.documents.get_mut(&key) {
            Some(record) => {
                for id in record.chunk_ids.drain(..) {
                    self.chunks.remove(&id);
                }
                record.status = IndexStatus::Archived;
                true
            }
            None => false,
        }
    }

    /// Permission-aware, tenant-scoped retrieval. Unauthorized chunks are
    /// dropped before scoring, so the caller sees neither content nor a hint
    /// that a matching document exists.
    pub fn retrieve(
        &self,
        tenant_id: &str,
        user: &UserContext,
        query: &str,
        limit: usize,
    ) -> Vec<RetrievedChunk> {
        if limit == 0 {
            return Vec::new();
        }
        let mut matches: Vec<(f32, &StructuredChunk)> = self
            .chunks
            .values()
            .filter(|chunk| chunk.tenant_id == tenant_id)
            .filter(|chunk| chunk.acl.allows(user))
            .filter_map(|chunk| {
                let score = score(query, &chunk.content);
                if score > 0.0 {
                    Some((score, chunk))
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });

        matches
            .into_iter()
            .take(limit)
            .map(|(score, chunk)| RetrievedChunk {
                citation: chunk.citation(),
                chunk: chunk.clone(),
                score,
            })
            .collect()
    }
}

pub fn retrieve_with_citations(
    index: &StructuredKbIndex,
    tenant_id: &str,
    user: &UserContext,
    query: &str,
    limit: usize,
) -> Vec<RetrievedChunk> {
    index.retrieve(tenant_id, user, query, limit)
}

fn score(query: &str, content: &str) -> f32 {
    let terms: Vec<&str> = query
        .split_whitespace()
        .filter(|term| term.chars().count() > 1)
        .collect();
    if terms.is_empty() {
        return 0.0;
    }
    let haystack = content.to_lowercase();
    let matched = terms
        .iter()
        .filter(|term| {
            let lowered = term.to_lowercase();
            haystack.contains(lowered.as_str())
        })
        .count();
    matched as f32 / terms.len() as f32
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
