//! #2042 — the DOCX source snapshot must survive a restart (persisted beside
//! the document, not only in the process cache) and the save path must fall
//! back to a rebuild only when no usable package exists.

use super::*;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct StoredObject {
    bytes: Vec<u8>,
    content_type: String,
}

/// In-memory stand-in for the Drive bucket. Reading after
/// `remove_from_cache` is what a restart looks like to the save path.
#[derive(Debug, Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, StoredObject>>,
}

impl MemoryStore {
    fn stored(&self, key: &str) -> Option<StoredObject> {
        self.objects.lock().ok()?.get(key).cloned()
    }

    fn bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.stored(key).map(|object| object.bytes)
    }
}

impl DocxSourceStore for MemoryStore {
    fn read(&self, key: &str) -> BoxFutureVecU8 {
        let found = self.bytes(key);
        let missing = format!("missing object: {key}");
        Box::pin(async move { found.ok_or(missing) })
    }

    fn write(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> BoxFutureUnit {
        let object = StoredObject {
            bytes,
            content_type: content_type.to_string(),
        };
        match self.objects.lock() {
            Ok(mut objects) => {
                objects.insert(key.to_string(), object);
                Box::pin(async { Ok(()) })
            }
            Err(error) => {
                let message = format!("memory store unavailable: {error}");
                Box::pin(async move { Err(message) })
            }
        }
    }
}

fn new_doc_id(label: &str) -> String {
    format!("{label}-{}", uuid::Uuid::new_v4())
}

fn sample_docx(text: &str) -> Vec<u8> {
    use docx_rs::{Docx, Paragraph, Run};
    let docx = Docx::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)));
    let mut buffer = std::io::Cursor::new(Vec::new());
    docx.build()
        .pack(&mut buffer)
        .expect("pack the sample DOCX");
    buffer.into_inner()
}

fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).expect("zip");
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).expect("zip entry");
        let name = file.name().map(|name| name.to_string()).unwrap_or_default();
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("read zip entry");
        out.push((name, content));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn document_xml(bytes: &[u8]) -> String {
    parts(bytes)
        .into_iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, content)| String::from_utf8_lossy(&content).to_string())
        .unwrap_or_default()
}

fn is_docx(bytes: &[u8]) -> bool {
    zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).is_ok()
}

fn resume_fixture() -> Option<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../bottest/tests/fixtures/office-fidelity/Alex_Rivera_Resume_Futuristic.docx");
    std::fs::read(path).ok()
}

#[tokio::test]
async fn persisted_snapshot_survives_a_restart() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("restart");
    let original = sample_docx("Quarterly report");

    persist_original_to(
        &store,
        "alice",
        &doc_id,
        &original,
        "<p>Quarterly report</p>",
    )
    .await
    .expect("persist the snapshot");

    // A restart empties the process cache, so the package must come back from
    // the bucket copy.
    crate::storage_core::remove_from_cache(&doc_id).await;

    let reloaded = original_docx(&store, "alice", &doc_id)
        .await
        .expect("reload the snapshot");
    assert_eq!(reloaded, original, "the stored package must be unchanged");

    let stored = store
        .stored(&docx_path("alice", &doc_id))
        .expect("the package is stored beside the document");
    assert_eq!(stored.content_type, DOCX_CONTENT_TYPE);
    assert!(
        store
            .stored(&source_fingerprint_path("alice", &doc_id))
            .is_some(),
        "the fingerprint is persisted with the package"
    );
}

#[tokio::test]
async fn untouched_save_returns_the_stored_package() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("untouched");
    let original = sample_docx("Quarterly report");
    let html = "<p>Quarterly report</p>";

    persist_original_to(&store, "alice", &doc_id, &original, html)
        .await
        .expect("persist the snapshot");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let (bytes, fidelity) = bytes_with_fidelity_from(&store, "alice", &doc_id, "Report", html)
        .await
        .expect("save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    assert_eq!(fidelity.as_str(), "preserved");
    assert_eq!(
        bytes, original,
        "an untouched save must not rewrite the package"
    );
}

#[tokio::test]
async fn edited_save_patches_only_the_document_part() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("edited");
    let original = sample_docx("Quarterly report");

    persist_original_to(
        &store,
        "alice",
        &doc_id,
        &original,
        "<p>Quarterly report</p>",
    )
    .await
    .expect("persist the snapshot");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let (bytes, fidelity) =
        bytes_with_fidelity_from(&store, "alice", &doc_id, "Report", "<p>Annual report</p>")
            .await
            .expect("save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    assert!(document_xml(&bytes).contains("Annual report"));

    let before = parts(&original);
    let after = parts(&bytes);
    assert_eq!(before.len(), after.len(), "the part list must not change");
    for ((name, old), (patched_name, new)) in before.iter().zip(after.iter()) {
        assert_eq!(name, patched_name, "part order must be stable");
        if name != "word/document.xml" {
            assert_eq!(old, new, "part {name} must stay byte-identical");
        }
    }
}

#[tokio::test]
async fn reverting_after_a_save_repairs_the_text_instead_of_reusing_stale_bytes() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("revert");
    let original_html = "<p>Quarterly report</p>";
    let edited_html = "<p>Annual report</p>";

    persist_original_to(
        &store,
        "alice",
        &doc_id,
        &sample_docx("Quarterly report"),
        original_html,
    )
    .await
    .expect("persist the snapshot");

    // Saving an edit writes the patched package and its fingerprint together,
    // exactly like `save_document_as_docx` does.
    let (edited, _) = bytes_with_fidelity_from(&store, "alice", &doc_id, "Report", edited_html)
        .await
        .expect("edited save");
    persist_original_to(&store, "alice", &doc_id, &edited, edited_html)
        .await
        .expect("persist the edit");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let (reverted, fidelity) =
        bytes_with_fidelity_from(&store, "alice", &doc_id, "Report", original_html)
            .await
            .expect("revert save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    let xml = document_xml(&reverted);
    assert!(
        xml.contains("Quarterly report"),
        "the reverted text must be written back"
    );
    assert!(
        !xml.contains("Annual report"),
        "stale edited text must not be returned"
    );
}

#[tokio::test]
async fn save_without_a_stored_package_rebuilds_and_reports_it() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("no-source");

    let (bytes, fidelity) =
        bytes_with_fidelity_from(&store, "bob", &doc_id, "Report", "<p>Fresh document</p>")
            .await
            .expect("save");

    assert_eq!(fidelity, DocxFidelity::Regenerated);
    assert_eq!(fidelity.as_str(), "regenerated");
    assert!(is_docx(&bytes), "the fallback must still produce a DOCX");
    assert!(document_xml(&bytes).contains("Fresh document"));
    assert!(
        store.bytes(&docx_path("bob", &doc_id)).is_none(),
        "the fallback must not invent a stored package"
    );
}

#[tokio::test]
async fn unusable_stored_package_falls_back_to_a_rebuild() {
    let store = MemoryStore::default();
    let doc_id = new_doc_id("corrupt");

    persist_original_to(&store, "bob", &doc_id, b"not a zip archive", "<p>Old</p>")
        .await
        .expect("persist the unusable package");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let (bytes, fidelity) =
        bytes_with_fidelity_from(&store, "bob", &doc_id, "Report", "<p>Recovered</p>")
            .await
            .expect("save");

    assert_eq!(fidelity, DocxFidelity::Regenerated);
    assert!(is_docx(&bytes), "the fallback must still produce a DOCX");
    assert!(document_xml(&bytes).contains("Recovered"));
}

#[tokio::test]
async fn real_resume_survives_persist_and_an_untouched_save() {
    let Some(original) = resume_fixture() else {
        return;
    };
    let Ok(html) = crate::storage_docx::convert_docx_to_html(&original) else {
        return;
    };

    let store = MemoryStore::default();
    let doc_id = new_doc_id("resume");
    persist_original_to(&store, "carol", &doc_id, &original, &html)
        .await
        .expect("persist the resume");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let (bytes, fidelity) = bytes_with_fidelity_from(&store, "carol", &doc_id, "Resume", &html)
        .await
        .expect("save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    assert_eq!(
        bytes, original,
        "styles, headers/footers, tables and images must stay byte-identical"
    );
}

#[tokio::test]
async fn edited_resume_keeps_every_part_except_the_document() {
    let Some(original) = resume_fixture() else {
        return;
    };
    let Ok(html) = crate::storage_docx::convert_docx_to_html(&original) else {
        return;
    };

    let store = MemoryStore::default();
    let doc_id = new_doc_id("resume-edit");
    persist_original_to(&store, "carol", &doc_id, &original, &html)
        .await
        .expect("persist the resume");
    crate::storage_core::remove_from_cache(&doc_id).await;

    let edited_html = html.replace("</p>", " edited</p>");
    let (bytes, fidelity) =
        bytes_with_fidelity_from(&store, "carol", &doc_id, "Resume", &edited_html)
            .await
            .expect("save");

    assert_eq!(fidelity, DocxFidelity::Preserved);
    let before = parts(&original);
    let after = parts(&bytes);
    assert_eq!(before.len(), after.len(), "the part list must not change");
    for ((name, old), (patched_name, new)) in before.iter().zip(after.iter()) {
        assert_eq!(name, patched_name, "part order must be stable");
        if name == "word/document.xml" {
            assert_ne!(old, new, "the edited part must change");
        } else {
            assert_eq!(old, new, "part {name} must stay byte-identical");
        }
    }
}
