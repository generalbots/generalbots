//! Tests for the turn-end artifact publisher (#2143 gap D).
//!
//! Split from `output_publish.rs` to keep every file under the repository's
//! 450-line limit. The publisher is exercised through a fake
//! [`ArtifactPublisher`], so no VM or object store is needed.

use super::{
    candidates, digest, is_unchanged, publish_candidates, ArtifactPublisher, OutputFile,
    PublishReport, PublishedArtifact, PublishedFile, MAX_PUBLISH_FILES, MAX_PUBLISH_FILE_BYTES,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct FakePublisher {
    reads: Mutex<Vec<String>>,
    uploads: Mutex<Vec<String>>,
    failing: bool,
}

impl FakePublisher {
    fn healthy() -> Self {
        Self::default()
    }

    fn unavailable() -> Self {
        Self {
            failing: true,
            ..Self::default()
        }
    }

    fn upload_count(&self) -> usize {
        self.uploads.lock().map(|u| u.len()).unwrap_or(0)
    }

    fn read_count(&self) -> usize {
        self.reads.lock().map(|r| r.len()).unwrap_or(0)
    }
}

#[async_trait]
impl ArtifactPublisher for FakePublisher {
    async fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        if let Ok(mut reads) = self.reads.lock() {
            reads.push(name.to_string());
        }
        Ok(format!("content of {name}").into_bytes())
    }

    async fn publish(&self, name: &str, _bytes: Vec<u8>) -> Option<String> {
        if self.failing {
            return None;
        }
        if let Ok(mut uploads) = self.uploads.lock() {
            uploads.push(name.to_string());
        }
        Some(format!("bot.gbdrive/conversations/demo/{name}"))
    }
}

fn file(name: &str, size: u64, modified: u64) -> OutputFile {
    OutputFile {
        name: name.to_string(),
        size,
        modified,
    }
}

#[tokio::test]
async fn a_new_file_is_published_once() {
    let conversation = Uuid::new_v4();
    let publisher = FakePublisher::healthy();
    let mut seen = HashMap::new();

    let first = publish_candidates(
        conversation,
        candidates(&[file("report.pdf", 21, 1_700_000_000)], &seen),
        &mut seen,
        &publisher,
    )
    .await;
    assert_eq!(first.len(), 1, "the new artifact must reach Drive");
    assert_eq!(publisher.upload_count(), 1);

    // Same listing on the next turn: nothing to read, nothing to upload.
    let second = publish_candidates(
        conversation,
        candidates(&[file("report.pdf", 21, 1_700_000_000)], &seen),
        &mut seen,
        &publisher,
    )
    .await;
    assert!(
        second.is_empty(),
        "an unchanged file must not be re-attached"
    );
    assert_eq!(publisher.upload_count(), 1, "upload must happen once");
    assert_eq!(publisher.read_count(), 1, "unchanged file must not be read");
}

#[test]
fn an_unchanged_file_is_skipped() {
    let seen = HashMap::from([(
        "report.pdf".to_string(),
        PublishedFile {
            size: 21,
            modified: 1_700_000_000,
            digest: digest(b"content of report.pdf"),
        },
    )]);
    assert!(is_unchanged(&seen, "report.pdf", 21, 1_700_000_000));
    assert!(candidates(&[file("report.pdf", 21, 1_700_000_000)], &seen).is_empty());
    // A rewrite moves the stamp, so the file becomes a candidate again.
    assert!(!is_unchanged(&seen, "report.pdf", 21, 1_700_000_050));
    assert_eq!(
        candidates(&[file("report.pdf", 21, 1_700_000_050)], &seen).len(),
        1
    );
}

#[tokio::test]
async fn a_touched_but_identical_file_is_not_uploaded_again() {
    let conversation = Uuid::new_v4();
    let publisher = FakePublisher::healthy();
    let mut seen = HashMap::from([(
        "report.pdf".to_string(),
        PublishedFile {
            size: 21,
            modified: 1_700_000_000,
            digest: digest(b"content of report.pdf"),
        },
    )]);
    let published = publish_candidates(
        conversation,
        candidates(&[file("report.pdf", 21, 1_700_000_099)], &seen),
        &mut seen,
        &publisher,
    )
    .await;
    assert!(
        published.is_empty(),
        "identical content must not be re-uploaded"
    );
    assert_eq!(publisher.upload_count(), 0);
    assert_eq!(
        seen.get("report.pdf").map(|entry| entry.modified),
        Some(1_700_000_099),
        "the stamp advances so the next turn skips it"
    );
}

#[tokio::test]
async fn drive_failure_does_not_fail_the_turn() {
    let conversation = Uuid::new_v4();
    let publisher = FakePublisher::unavailable();
    let mut seen = HashMap::new();
    let published = publish_candidates(
        conversation,
        candidates(&[file("report.pdf", 21, 1_700_000_000)], &seen),
        &mut seen,
        &publisher,
    )
    .await;
    assert!(published.is_empty(), "a Drive outage returns no artifacts");
    assert!(
        !seen.contains_key("report.pdf"),
        "a failed upload must stay a candidate for the next turn"
    );
}

#[test]
fn the_attached_link_matches_the_published_key() {
    let conversation = Uuid::new_v4();
    let artifact = PublishedArtifact::new(
        "report.pdf",
        "bot.gbdrive/conversations/demo/report.pdf",
        conversation,
    );
    assert_eq!(artifact.drive_name(), artifact.name.as_str());
    assert!(artifact.drive_key.ends_with(&format!("/{}", artifact.name)));
    assert_eq!(
        artifact.link,
        format!("[report.pdf](vmfile:{conversation}/output/report.pdf)")
    );
    let report = PublishReport {
        folder: "demo".to_string(),
        published: vec![artifact.clone()],
    };
    let message = report.message();
    assert!(message.contains(&artifact.link));
    assert!(message.contains("conversations/demo/"));
}

#[test]
fn publishing_is_bounded() {
    let files: Vec<OutputFile> = (0..20)
        .map(|index| file(&format!("f{index:02}.pdf"), 10, 1))
        .collect();
    assert_eq!(candidates(&files, &HashMap::new()).len(), MAX_PUBLISH_FILES);

    let oversized = vec![file("huge.zip", MAX_PUBLISH_FILE_BYTES + 1, 1)];
    assert!(candidates(&oversized, &HashMap::new()).is_empty());
}

#[test]
fn the_digest_notices_a_same_size_rewrite() {
    assert_ne!(digest(b"version one"), digest(b"version two"));
    assert_eq!(digest(b"same"), digest(b"same"));
}
