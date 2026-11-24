//! The Drive decision of `output/`: asked inside the turn, never at its end.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use super::{fresh, hold_back, keep_published, not_held, settle_turn_end, take_turn, Held};
use crate::core::bot::pipeline::output_publish::{
    candidates, ArtifactPublisher, OutputFile, PublishReport, PublishedArtifact, PublishedFile,
};

#[derive(Default)]
struct CountingPublisher {
    uploads: Mutex<Vec<String>>,
}

impl CountingPublisher {
    fn uploads(&self) -> Vec<String> {
        self.uploads.lock().map(|list| list.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl ArtifactPublisher for CountingPublisher {
    async fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        Ok(name.as_bytes().to_vec())
    }

    async fn publish(&self, name: &str, bytes: Vec<u8>) -> Option<String> {
        assert_eq!(bytes, name.as_bytes());
        if let Ok(mut list) = self.uploads.lock() {
            list.push(name.to_string());
        }
        Some(format!("bucket/bot.gbdrive/conversations/demo/{name}"))
    }
}

fn file(name: &str, size: u64, modified: u64) -> OutputFile {
    OutputFile { name: name.to_string(), size, modified }
}

#[tokio::test]
async fn the_turn_end_without_a_grant_copies_nothing_and_lists_the_file_once() {
    let session = Uuid::new_v4();
    let publisher = CountingPublisher::default();
    let mut seen: HashMap<String, PublishedFile> = HashMap::new();
    let listing = vec![file("memo.docx", 9, 100)];

    let files = fresh(session, candidates(&listing, &seen));
    let (published, unsaved) = settle_turn_end(session, files, false, &mut seen, &publisher).await;
    assert!(published.is_empty());
    assert_eq!(unsaved, vec!["memo.docx".to_string()]);
    assert!(publisher.uploads().is_empty(), "no grant: nothing reaches Drive");

    // The next turn's end neither copies nor lists the same file again.
    let again = fresh(session, candidates(&listing, &seen));
    assert!(again.is_empty());
    let (published, unsaved) = settle_turn_end(session, again, false, &mut seen, &publisher).await;
    assert!(published.is_empty() && unsaved.is_empty());
}

#[tokio::test]
async fn a_conversation_grant_is_reused_by_later_turns() {
    let session = Uuid::new_v4();
    let publisher = CountingPublisher::default();
    let mut seen: HashMap<String, PublishedFile> = HashMap::new();

    let first = vec![file("a.csv", 5, 1)];
    let files = fresh(session, candidates(&first, &seen));
    let (published, _) = settle_turn_end(session, files, true, &mut seen, &publisher).await;
    assert_eq!(published.len(), 1);

    let second = vec![file("a.csv", 5, 1), file("b.csv", 5, 2)];
    let files = fresh(session, candidates(&second, &seen));
    assert_eq!(files, vec![file("b.csv", 5, 2)], "only the new file is pending");
    let (published, unsaved) = settle_turn_end(session, files, true, &mut seen, &publisher).await;
    assert_eq!(published.len(), 1);
    assert!(unsaved.is_empty());
    assert_eq!(publisher.uploads(), vec!["a.csv".to_string(), "b.csv".to_string()]);
}

#[tokio::test]
async fn a_declined_file_is_never_published() {
    let session = Uuid::new_v4();
    let publisher = CountingPublisher::default();
    let mut seen: HashMap<String, PublishedFile> = HashMap::new();
    let listing = vec![file("plan.xlsx", 7, 50)];

    // "Negar" on the card inside the turn.
    hold_back(session, &listing, true);
    let turn = take_turn(session);
    assert_eq!(turn.unsaved, vec!["plan.xlsx".to_string()], "the turn's end lists it once");
    assert!(take_turn(session).unsaved.is_empty());

    // Even a grant given later does not copy the declined version.
    let files = fresh(session, candidates(&listing, &seen));
    let (published, _) = settle_turn_end(session, files, true, &mut seen, &publisher).await;
    assert!(published.is_empty());
    assert!(publisher.uploads().is_empty());

    // A new version of the file is a new decision.
    let changed = vec![file("plan.xlsx", 8, 60)];
    assert_eq!(fresh(session, candidates(&changed, &seen)), changed);
}

#[test]
fn held_stamps_filter_only_the_exact_version() {
    let mut held = Held::new();
    super::hold(&mut held, &[file("x.txt", 1, 10)]);
    let kept = not_held(vec![file("x.txt", 1, 10), file("x.txt", 2, 11), file("y.txt", 1, 10)], &held);
    assert_eq!(kept, vec![file("x.txt", 2, 11), file("y.txt", 1, 10)]);
}

#[test]
fn files_copied_inside_the_turn_are_linked_once_at_its_end() {
    let session = Uuid::new_v4();
    let artifact = PublishedArtifact::new("r.pdf", "bucket/conversations/demo/r.pdf", session);
    keep_published(session, "demo".to_string(), vec![artifact.clone()]);
    keep_published(session, "demo".to_string(), vec![artifact.clone()]);
    let turn = take_turn(session);
    assert_eq!(turn.published, vec![artifact.clone()]);
    assert_eq!(turn.folder.as_deref(), Some("demo"));
    assert!(take_turn(session).published.is_empty());

    let report = PublishReport {
        folder: "demo".to_string(),
        published: turn.published,
        unsaved: vec![super::super::output_link("n.docx", session)],
    };
    let message = report.message();
    assert!(message.contains(&artifact.link));
    assert!(message.contains("Salvar no Drive"));
    assert!(message.contains(&format!("vmfile:{session}/output/n.docx")));
}
