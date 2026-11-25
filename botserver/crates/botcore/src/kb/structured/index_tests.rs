//! Structured index tests. Kept in a sibling file so `index.rs` stays under the
//! repository size limit; `#[path]` keeps the module as `index::tests`.

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
    // A private ACL with no allowed_users denies everyone (fail-closed), so
    // the fixture must grant the test user explicitly for retrieve() tests.
    IndexDocument::new(
        id,
        tenant_id,
        "contract.docx",
        "3",
        text,
        AclMetadata::private(tenant_id).with_users(vec![USER_ID.to_string()]),
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
