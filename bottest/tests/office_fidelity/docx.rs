//! DOCX half of the fidelity contract (#2042/#2044): the imported package is
//! persisted beside the document, an untouched save returns it byte for byte
//! after a restart, one text edit changes only `word/document.xml`, and a
//! missing or corrupt snapshot is reported as `regenerated`.

use std::sync::Arc;

use botdocs::state::DocState;
use botdocs::storage::{docx_bytes_with_fidelity, persist_original_docx, DocxFidelity};
use botdocs::storage_core::remove_from_cache;

use crate::memory_drive::MemoryDrive;
use crate::{changed_parts, fixture, part, part_hashes, DOCX_FIXTURE};

/// A `DocState` whose Drive is the in-memory bucket. The pool is never used by
/// the DOCX save path, so it is built without touching a database.
fn doc_state(drive: Arc<MemoryDrive>) -> DocState {
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
        "postgres://unused:unused@127.0.0.1:1/unused",
    );
    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    DocState {
        pool: Arc::new(pool),
        drive,
        bucket_name: "office-fidelity.gbai".to_string(),
        history: Default::default(),
    }
}

fn doc_id(label: &str) -> String {
    format!("{label}-{}", uuid::Uuid::new_v4())
}

/// Rule 1+2: the imported package is persisted beside the document, survives a
/// cache flush (a restart) and comes back untouched, byte for byte.
#[tokio::test]
async fn docx_untouched_save_returns_the_stored_package() {
    let Some(original) = fixture(DOCX_FIXTURE) else {
        return; // fixture is optional in a trimmed checkout
    };
    let state = doc_state(Arc::new(MemoryDrive::default()));
    let id = doc_id("untouched");
    let html = "<p>Alex Rivera</p>";

    persist_original_docx(&state, "alice", &id, &original, html)
        .await
        .expect("persist the source package");
    remove_from_cache(&id).await;

    let (bytes, fidelity) = docx_bytes_with_fidelity(&state, "alice", &id, "Resume", html)
        .await
        .expect("untouched save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    assert_eq!(fidelity.as_str(), "preserved", "export header value");
    assert_eq!(
        bytes, original,
        "an untouched save must not rebuild the DOCX"
    );
    assert_eq!(
        part_hashes(&bytes),
        part_hashes(&original),
        "every part must hash identically"
    );
}

/// Rule 2: a single text edit changes `word/document.xml` and nothing else.
#[tokio::test]
async fn docx_single_text_edit_changes_only_the_document_part() {
    let Some(original) = fixture(DOCX_FIXTURE) else {
        return;
    };
    let state = doc_state(Arc::new(MemoryDrive::default()));
    let id = doc_id("edited");

    persist_original_docx(&state, "alice", &id, &original, "<p>Alex Rivera</p>")
        .await
        .expect("persist the source package");
    remove_from_cache(&id).await;

    let (bytes, fidelity) =
        docx_bytes_with_fidelity(&state, "alice", &id, "Resume", "<p>Alex Rivera Silva</p>")
            .await
            .expect("edited save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    assert_eq!(
        changed_parts(&original, &bytes),
        vec!["word/document.xml".to_string()],
        "only the document part may change"
    );
    assert!(
        part(&bytes, "word/document.xml").contains("Silva"),
        "the edit must reach the document XML"
    );
}

/// Rule 4: no snapshot, or a snapshot that cannot be patched, is reported as
/// `regenerated` — the caller can warn instead of pretending nothing was lost.
#[tokio::test]
async fn docx_missing_or_corrupt_snapshot_is_reported_as_regenerated() {
    let state = doc_state(Arc::new(MemoryDrive::default()));

    let missing = doc_id("missing");
    let (_, fidelity) = docx_bytes_with_fidelity(&state, "alice", &missing, "New", "<p>New</p>")
        .await
        .expect("save without a snapshot");
    assert_eq!(fidelity, DocxFidelity::Regenerated);
    assert_eq!(fidelity.as_str(), "regenerated", "export header value");

    let corrupt = doc_id("corrupt");
    persist_original_docx(
        &state,
        "alice",
        &corrupt,
        b"not a docx package",
        "<p>Original</p>",
    )
    .await
    .expect("persist the corrupt snapshot");
    remove_from_cache(&corrupt).await;
    let (_, fidelity) = docx_bytes_with_fidelity(&state, "alice", &corrupt, "Doc", "<p>Edited</p>")
        .await
        .expect("save over a corrupt snapshot");
    assert_eq!(
        fidelity,
        DocxFidelity::Regenerated,
        "an unpatchable snapshot must be an explicit rebuild"
    );
}
