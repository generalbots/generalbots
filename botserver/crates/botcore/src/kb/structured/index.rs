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
mod tests {
    use super::*;
    use crate::kb::structured::AclMetadata;
    use uuid::Uuid;

    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";

    fn parse_uuid(value: &str) -> Uuid {
        match Uuid::parse_str(value) {
            Ok(id) => id,
            Err(_) => Uuid::nil(),
        }
    }

    fn tenant(id: &str) -> Uuid {
        parse_uuid(id)
    }

    fn user(tenant_id: &str) -> UserContext {
        UserContext::authenticated(
            parse_uuid("33333333-3333-3333-3333-333333333333"),
            None,
            Some(tenant(tenant_id)),
        )
    }

    fn contract(tenant_id: &str) -> IndexDocument {
        let text = [
            "# Employment",
            "## Termination",
            "[[PAGE:12]]",
            "7.3 Termination Notice",
            "The termination notice period is ninety days from the effective date.",
        ]
        .join("\n");
        IndexDocument::new(
            "contract",
            tenant_id,
            "contract.docx",
            "3",
            &text,
            AclMetadata::private(tenant_id),
        )
    }

    fn authorized(index: &StructuredKbIndex, tenant_id: &str) -> Vec<RetrievedChunk> {
        index.retrieve(tenant_id, &user(tenant_id), "termination notice period", 5)
    }

    #[test]
    fn clause_question_returns_file_section_and_page_citation() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        let results = authorized(&index, TENANT_A);
        assert!(!results.is_empty());
        if let Some(first) = results.first() {
            assert_eq!(first.citation.file, "contract.docx");
            assert_eq!(first.citation.version, "3");
            assert_eq!(first.citation.source.locator, "7.3");
            assert_eq!(first.citation.source.page, Some(12));
            assert!(first
                .citation
                .heading_path
                .iter()
                .any(|heading| heading.contains("Termination")));
        }
    }

    #[test]
    fn user_without_access_gets_no_content_and_no_existence_hint() {
        let mut index = StructuredKbIndex::new();
        let mut document = contract(TENANT_A);
        let stranger = parse_uuid("44444444-4444-4444-4444-444444444444");
        document.acl = AclMetadata::private(TENANT_A).with_users(vec![stranger.to_string()]);
        index.index_document(&document);

        let denied = index.retrieve(TENANT_A, &user(TENANT_A), "termination notice", 5);
        assert!(denied.is_empty());
        assert_eq!(index.status(TENANT_A, "contract"), Some(&IndexStatus::Indexed));
    }

    #[test]
    fn editing_a_document_replaces_its_index_entries() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        let before: Vec<String> = index
            .chunks
            .values()
            .map(|chunk| chunk.id.clone())
            .collect();

        let mut edited = contract(TENANT_A);
        edited.version = "4".to_string();
        edited.text = "7.3 Notice\nThe period is thirty days.".to_string();
        index.index_document(&edited);

        let after: Vec<String> = index
            .chunks
            .values()
            .map(|chunk| chunk.id.clone())
            .collect();
        assert!(!after.is_empty());
        assert!(before.iter().all(|id| !after.contains(id)));
        assert!(after.iter().all(|id| !before.contains(id)));
        assert_eq!(
            index.status(TENANT_A, "contract"),
            Some(&IndexStatus::Indexed)
        );
    }

    #[test]
    fn delete_and_archive_remove_chunks_immediately() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        assert!(!authorized(&index, TENANT_A).is_empty());

        assert!(index.archive_document(TENANT_A, "contract"));
        assert!(authorized(&index, TENANT_A).is_empty());
        assert_eq!(
            index.status(TENANT_A, "contract"),
            Some(&IndexStatus::Archived)
        );

        index.index_document(&contract(TENANT_A));
        assert!(index.delete_document(TENANT_A, "contract"));
        assert!(authorized(&index, TENANT_A).is_empty());
        assert_eq!(index.status(TENANT_A, "contract"), None);
    }

    #[test]
    fn backfill_report_accounts_for_every_document_including_skipped() {
        let mut index = StructuredKbIndex::new();
        let empty = IndexDocument::new(
            "empty",
            TENANT_A,
            "empty.txt",
            "1",
            "   ",
            AclMetadata::private(TENANT_A),
        );
        let documents = vec![contract(TENANT_A), empty];
        let ids: Vec<String> = documents
            .iter()
            .map(|document| document.document_id.clone())
            .collect();
        let report = index.backfill(&documents);

        assert_eq!(report.total(), 2);
        assert_eq!(report.indexed(), 1);
        assert_eq!(report.skipped(), 1);
        assert!(report.accounts_for(&ids));
    }

    #[test]
    fn document_content_never_crosses_tenant_boundaries() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        index.index_document(&contract(TENANT_B));

        let results_a = authorized(&index, TENANT_A);
        assert!(!results_a.is_empty());
        assert!(results_a
            .iter()
            .all(|item| item.chunk.tenant_id == TENANT_A));

        let results_b = authorized(&index, TENANT_B);
        assert!(!results_b.is_empty());
        assert!(results_b
            .iter()
            .all(|item| item.chunk.tenant_id == TENANT_B));

        let wrong_tenant = index.retrieve(TENANT_A, &user(TENANT_B), "termination notice", 5);
        assert!(wrong_tenant.is_empty());
    }
}
