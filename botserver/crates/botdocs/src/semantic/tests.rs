//! Offline unit tests for the semantic document layer (#2451).

use super::model::*;
use super::{ReadOptions, cite, diff, parse, patch, read_blocks, render};

fn apply(document: &Document, version: u64, operations: &[Operation]) -> PatchResult {
    patch(document, version, version, operations).unwrap()
}

fn text_of(document: &Document) -> Vec<&str> {
    document.blocks.iter().map(|b| b.text.as_str()).collect()
}

#[test]
fn stable_ids_survive_unrelated_edits() {
    let original =
        parse("<h1>Contract</h1><p>Alpha clause</p><p>Beta clause</p><p>Gamma clause</p>");
    let alpha_id = original.blocks[1].id.clone();
    let beta_id = original.blocks[2].id.clone();
    let gamma_id = original.blocks[3].id.clone();

    // Editing Alpha must not change Beta's or Gamma's ids.
    let edited = apply(
        &original,
        1,
        &[Operation::Replace {
            id: alpha_id.clone(),
            text: "Alpha revised".to_string(),
        }],
    );
    assert_eq!(edited.document.blocks[2].id, beta_id);
    assert_eq!(edited.document.blocks[3].id, gamma_id);

    // Inserting a block before Gamma must not change Gamma's id.
    let inserted = apply(
        &edited.document,
        2,
        &[Operation::Insert {
            after: Some(beta_id.clone()),
            kind: BlockKind::Paragraph,
            text: "New clause".to_string(),
        }],
    );
    let gamma_after = inserted
        .document
        .blocks
        .iter()
        .find(|block| block.text == "Gamma clause")
        .map(|block| block.id.clone());
    assert_eq!(gamma_after, Some(gamma_id));
}

#[test]
fn ids_are_deterministic_and_disambiguate_identical_blocks() {
    let first = parse("<p>Repeated</p><p>Unique</p><p>Repeated</p>");
    let second = parse("<p>Repeated</p><p>Unique</p><p>Repeated</p>");
    assert_eq!(first, second);
    assert_ne!(first.blocks[0].id, first.blocks[2].id);
    assert!(first.blocks[0].id.starts_with("s-"));
}

#[test]
fn parse_addresses_pages_and_table_cells() {
    let document = parse(
        "<h1>Part One</h1><p>Intro</p><h1>Part Two</h1><table><tr><td>A1</td><td>B1</td></tr></table>",
    );
    assert_eq!(document.blocks[0].kind, BlockKind::Heading1);
    assert_eq!(document.blocks[0].page, 1);
    assert_eq!(document.blocks[2].page, 2);
    let cells: Vec<&Block> = document
        .blocks
        .iter()
        .filter(|block| block.kind == BlockKind::TableCell)
        .collect();
    assert_eq!(cells.len(), 2);
    assert_ne!(cells[0].id, cells[1].id);
}

#[test]
fn patch_replace_insert_delete_and_inverse_restores() {
    let original = parse("<h1>Contract</h1><p>Alpha clause</p><p>Beta clause</p>");
    let alpha_id = original.blocks[1].id.clone();
    let beta_id = original.blocks[2].id.clone();

    let result = apply(
        &original,
        7,
        &[
            Operation::Replace {
                id: alpha_id.clone(),
                text: "Alpha revised".to_string(),
            },
            Operation::Insert {
                after: Some(alpha_id.clone()),
                kind: BlockKind::Paragraph,
                text: "Gamma inserted".to_string(),
            },
            Operation::Delete {
                id: beta_id.clone(),
            },
        ],
    );
    assert_eq!(result.new_version, 8);
    assert_eq!(result.applied.len(), 3);
    assert_eq!(
        text_of(&result.document),
        vec!["Contract", "Alpha revised", "Gamma inserted"]
    );

    // The returned inverse reverts the whole patch in one action.
    let reverted = apply(&result.document, 8, &result.inverse);
    assert_eq!(reverted.document, original);
}

#[test]
fn patch_rejects_stale_version_and_bad_ids() {
    let document = parse("<p>Only</p>");
    let id = document.blocks[0].id.clone();

    assert_eq!(
        patch(&document, 1, 2, &[Operation::Delete { id: id.clone() }]),
        Err(PatchError::Conflict {
            expected: 1,
            actual: 2
        })
    );
    assert_eq!(
        patch(&document, 1, 1, &[]),
        Err(PatchError::EmptyOperations)
    );
    assert_eq!(
        patch(
            &document,
            1,
            1,
            &[Operation::Delete {
                id: "missing".to_string()
            }]
        ),
        Err(PatchError::UnknownTarget {
            id: "missing".to_string()
        })
    );
    assert_eq!(
        patch(
            &document,
            1,
            1,
            &[Operation::Insert {
                after: Some("missing".to_string()),
                kind: BlockKind::Paragraph,
                text: "x".to_string(),
            }]
        ),
        Err(PatchError::UnknownAnchor {
            id: "missing".to_string()
        })
    );
}

#[test]
fn diff_reports_added_modified_and_removed_in_order() {
    let from = parse("<h1>Title</h1><p>Keep me</p><p>Old text</p><p>Remove me</p>");
    let keep_id = from.blocks[1].id.clone();
    let old_id = from.blocks[2].id.clone();

    let mut to_blocks = from.blocks.clone();
    to_blocks[2].text = "New text".to_string();
    to_blocks.insert(
        2,
        Block {
            id: "s-inserted".to_string(),
            kind: BlockKind::Paragraph,
            page: 1,
            text: "Inserted".to_string(),
        },
    );
    // After the insert, "Remove me" moved from index 3 to index 4.
    to_blocks.remove(4);
    let to = Document { blocks: to_blocks };

    let changes = diff(&from, &to);
    assert_eq!(changes.len(), 3);
    assert_eq!(keep_id, from.blocks[1].id);
    match &changes[0] {
        Change::Added { text, .. } => assert_eq!(text, "Inserted"),
        other => panic!("expected Added, got {other:?}"),
    }
    match &changes[1] {
        Change::Modified { id, before, after } => {
            assert_eq!(id, &old_id);
            assert_eq!(before, "Old text");
            assert_eq!(after, "New text");
        }
        other => panic!("expected Modified, got {other:?}"),
    }
    match &changes[2] {
        Change::Removed { text, .. } => assert_eq!(text, "Remove me"),
        other => panic!("expected Removed, got {other:?}"),
    }
}

#[test]
fn cite_centres_on_focus_and_reports_offsets() {
    let document = parse("<p>The quick brown fox jumps over the lazy dog near the river.</p>");
    let id = document.blocks[0].id.clone();
    let full = document.blocks[0].text.as_str();

    let citation = cite(&document, &id, Some("lazy dog"), 16).unwrap();
    assert_eq!(citation.id, id);
    assert_eq!(citation.page, 1);
    assert!(citation.snippet.contains("lazy dog"));
    assert_eq!(&full[citation.start..citation.end], citation.snippet);
    assert_eq!(
        citation.end - citation.start,
        citation.snippet.chars().count()
    );

    let leading = cite(&document, &id, None, 9).unwrap();
    assert_eq!(leading.snippet.chars().count(), 9);
    assert_eq!(leading.start, 0);

    assert_eq!(
        cite(&document, "missing", None, 10),
        Err(CiteError::UnknownId {
            id: "missing".to_string()
        })
    );
}

#[test]
fn read_blocks_honours_page_and_token_budget() {
    let document =
        parse("<h1>One</h1><p>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</p><h1>Two</h1><p>bbbb</p>");
    let all = read_blocks(&document, &ReadOptions::default());
    assert_eq!(all.total_pages, 2);
    assert_eq!(all.total_blocks, 4);

    let page_two = read_blocks(
        &document,
        &ReadOptions {
            page: Some(2),
            ..ReadOptions::default()
        },
    );
    assert_eq!(page_two.blocks.len(), 2);
    assert!(page_two.blocks.iter().all(|block| block.page == 2));

    let limited = read_blocks(
        &document,
        &ReadOptions {
            budget_tokens: 2,
            ..ReadOptions::default()
        },
    );
    assert!(limited.truncated);
    assert_eq!(limited.next_index, Some(1));
    assert_eq!(limited.blocks.len(), 1);
}

#[test]
fn render_escapes_text_and_never_emits_raw_html() {
    let document = parse("<p>original</p>");
    let id = document.blocks[0].id.clone();
    let patched = apply(
        &document,
        1,
        &[Operation::Replace {
            id,
            text: "<script>alert(1)</script> & done".to_string(),
        }],
    );
    let html = render(&patched.document);
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&amp;"));
    assert!(!html.contains("<script>"));
}
