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
    const USER_ID: &str = "33333333-3333-3333-3333-333333333333";

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
        UserContext::authenticated(parse_uuid(USER_ID), None, Some(tenant(tenant_id)))
    }

    fn private_doc(id: &str, tenant_id: &str, text: &str) -> IndexDocument {
        IndexDocument::new(
            id,
            tenant_id,
            "contract.docx",
            "3",
            text,
            AclMetadata::private(tenant_id),
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
        private_doc("contract", tenant_id, &text)
    }

    fn authorized(index: &StructuredKbIndex, tenant_id: &str) -> Vec<RetrievedChunk> {
        index.retrieve(tenant_id, &user(tenant_id), "termination notice period", 5)
    }

    fn sorted_chunk_ids(index: &StructuredKbIndex) -> Vec<String> {
        let mut ids: Vec<String> = index.chunks.keys().cloned().collect();
        ids.sort();
        ids
    }

    #[test]
    fn delete_and_archive_remove_chunks_immediately() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        assert!(!authorized(&index, TENANT_A).is_empty());
        assert!(!sorted_chunk_ids(&index).is_empty());

        assert!(index.archive_document(TENANT_A, "contract"));
        assert!(
            index.chunks.is_empty(),
            "archive must drop every chunk immediately"
        );
        assert!(authorized(&index, TENANT_A).is_empty());
        assert!(matches!(
            index.status(TENANT_A, "contract"),
            Some(IndexStatus::Archived)
        ));
        assert!(index
            .documents
            .get(&StructuredKbIndex::key(TENANT_A, "contract"))
            .is_some_and(|record| record.chunk_ids.is_empty()));

        index.index_document(&contract(TENANT_A));
        assert!(!sorted_chunk_ids(&index).is_empty());
        assert!(index.delete_document(TENANT_A, "contract"));
        assert!(
            index.chunks.is_empty(),
            "delete must drop every chunk immediately"
        );
        assert!(authorized(&index, TENANT_A).is_empty());
        assert_eq!(index.status(TENANT_A, "contract"), None);
        assert!(!index.delete_document(TENANT_A, "contract"));
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
        let blank = IndexDocument::new(
            "blank",
            TENANT_A,
            "blank.txt",
            "1",
            "\n\t\n",
            AclMetadata::private(TENANT_A),
        );
        let documents = vec![contract(TENANT_A), empty, blank];
        let ids: Vec<String> = documents
            .iter()
            .map(|document| document.document_id.clone())
            .collect();
        let report = index.backfill(&documents);

        assert_eq!(report.total(), 3);
        assert_eq!(report.indexed(), 1);
        assert_eq!(report.skipped(), 2);
        assert!(report.accounts_for(&ids));

        let skipped: Vec<&BackfillEntry> = report
            .entries
            .iter()
            .filter(|entry| entry.status != IndexStatus::Indexed)
            .collect();
        assert_eq!(skipped.len(), 2);
        assert!(skipped.iter().all(|entry| {
            entry.chunk_count == 0
                && entry.reason.as_deref() == Some("empty document")
                && matches!(entry.status, IndexStatus::Skipped(_))
        }));
        assert!(skipped
            .iter()
            .all(|entry| index.status(TENANT_A, &entry.document_id)
                == Some(&IndexStatus::Skipped("empty document".to_string()))));
        assert!(report.entries.iter().any(|entry| {
            entry.status == IndexStatus::Indexed && entry.chunk_count > 0 && entry.reason.is_none()
        }));

        let mut single = StructuredKbIndex::new();
        single.index_document(&contract(TENANT_A));
        assert_eq!(sorted_chunk_ids(&index), sorted_chunk_ids(&single));
    }

    #[test]
    fn document_content_never_crosses_tenant_boundaries() {
        let mut index = StructuredKbIndex::new();
        let text_a = "# Employment\n## Termination\n7.3 Termination\nThe alpha-only termination notice is ninety days.";
        let text_b = "# Employment\n## Termination\n7.3 Termination\nThe beta-only termination notice is thirty days.";
        index.index_document(&private_doc("contract", TENANT_A, text_a));
        index.index_document(&private_doc("contract", TENANT_B, text_b));

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

        // Phrases that exist only in the other tenant must not leak across.
        assert!(index
            .retrieve(TENANT_A, &user(TENANT_A), "beta-only thirty", 5)
            .is_empty());
        assert!(index
            .retrieve(TENANT_B, &user(TENANT_B), "alpha-only ninety", 5)
            .is_empty());
        // A user cannot read a same-named document in another tenant.
        assert!(index
            .retrieve(TENANT_A, &user(TENANT_B), "termination notice", 5)
            .is_empty());
        assert!(
            retrieve_with_citations(&index, TENANT_A, &user(TENANT_B), "termination", 5).is_empty()
        );
    }

    #[test]
    fn user_without_access_gets_no_content_and_no_existence_hint() {
        let mut index = StructuredKbIndex::new();
        let stranger = parse_uuid("44444444-4444-4444-4444-444444444444");
        let mut document = contract(TENANT_A);
        document.acl = AclMetadata::private(TENANT_A).with_users(vec![stranger.to_string()]);
        index.index_document(&document);

        for limit in [1, 5, 100] {
            assert!(index
                .retrieve(TENANT_A, &user(TENANT_A), "termination notice", limit)
                .is_empty());
            assert!(retrieve_with_citations(
                &index,
                TENANT_A,
                &user(TENANT_A),
                "termination notice",
                limit
            )
            .is_empty());
        }
        // The explicit status row is the only observable existence signal.
        assert_eq!(
            index.status(TENANT_A, "contract"),
            Some(&IndexStatus::Indexed)
        );

        let allowed = UserContext::authenticated(stranger, None, Some(tenant(TENANT_A)));
        assert!(!index
            .retrieve(TENANT_A, &allowed, "termination notice", 5)
            .is_empty());
        let anonymous = UserContext {
            organization_id: Some(tenant(TENANT_A)),
            ..UserContext::anonymous()
        };
        assert!(index
            .retrieve(TENANT_A, &anonymous, "termination notice", 5)
            .is_empty());
    }

    #[test]
    fn reindexing_unchanged_document_keeps_stable_chunk_ids() {
        let mut index = StructuredKbIndex::new();
        index.index_document(&contract(TENANT_A));
        let first = sorted_chunk_ids(&index);
        assert!(!first.is_empty());

        // Identical logical document -> identical IDs, no duplicate growth.
        index.index_document(&contract(TENANT_A));
        assert_eq!(sorted_chunk_ids(&index), first);

        // A new version changes every ID while preserving the chunk count.
        let mut edited = contract(TENANT_A);
        edited.version = "4".to_string();
        index.index_document(&edited);
        let changed = sorted_chunk_ids(&index);
        assert_eq!(changed.len(), first.len());
        assert!(changed.iter().all(|id| !first.contains(id)));
    }
}
