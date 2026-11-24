//! Unit tests for the structure-aware chunker (pure, offline).

use super::*;

fn public_acl() -> ChunkPermissions {
    ChunkPermissions {
        tenant_id: "acme".to_string(),
        visibility: Visibility::Public,
        ..ChunkPermissions::default()
    }
}

fn document(permissions: ChunkPermissions, source: &str) -> StructuredDocument {
    StructuredDocument {
        id: "doc-1".to_string(),
        file: "contract.docx".to_string(),
        version: "3".to_string(),
        permissions,
        elements: parse_structured_text(source),
    }
}

#[test]
fn heading_context_is_inherited_and_popped() {
    let doc = document(
        public_acl(),
        "# 7 Termination\n## 7.3 Notice\n[[CLAUSE:7.3]]\nNinety days notice.\n# 8 Governing\nCourts of Sao Paulo.",
    );
    let chunks = StructuredChunker::default().chunk(&doc);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].heading_path, ["7 Termination", "7.3 Notice"]);
    assert_eq!(chunks[0].clause.as_deref(), Some("7.3"));
    assert!(chunks[0]
        .text
        .starts_with("7 Termination > 7.3 Notice\nNinety days notice."));

    assert_eq!(chunks[1].heading_path, ["8 Governing"]);
    assert_eq!(chunks[1].body, "Courts of Sao Paulo.");
    assert!(!chunks[1].text.contains("7.3 Notice"));
}

#[test]
fn chunks_respect_the_size_bound() {
    let doc = document(public_acl(), &format!("# Large\n{}", "word ".repeat(200)));
    let max_chars = 120;
    let chunks = StructuredChunker::new(ChunkOptions { max_chars }).chunk(&doc);

    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(
            chunk.text.chars().count() <= max_chars,
            "text exceeded bound: {}",
            chunk.text.chars().count()
        );
        assert!(
            chunk.body.chars().count() <= max_chars,
            "body exceeded bound: {}",
            chunk.body.chars().count()
        );
        assert!(chunk.text.contains("Large"));
    }
}

#[test]
fn table_rows_are_never_split() {
    let doc = document(
        public_acl(),
        "# Data\n| A1 | B1 |\n| A2 | B2 |\n| A3 | B3 |",
    );
    let chunks = StructuredChunker::new(ChunkOptions { max_chars: 24 }).chunk(&doc);

    assert_eq!(chunks.len(), 2);
    let rows: Vec<&str> = chunks.iter().flat_map(|chunk| chunk.body.lines()).collect();
    assert_eq!(rows, ["A1 | B1", "A2 | B2", "A3 | B3"]);
    for chunk in &chunks {
        assert_eq!(chunk.heading_path, ["Data"]);
        assert!(chunk.text.starts_with("Data\n"));
        for line in chunk.body.lines() {
            assert!(line.contains('|'), "incomplete row: {line}");
        }
    }
}

#[test]
fn citation_ids_are_stable_and_versioned() {
    let doc = document(
        public_acl(),
        "# Employment\n## Termination\n[[PAGE:12]]\n[[CLAUSE:7.3]]\nThe notice period is ninety days.",
    );
    let chunker = StructuredChunker::default();
    let first = chunker.chunk(&doc);
    let second = chunker.chunk(&doc);
    assert_eq!(first, second);

    let chunk = &first[0];
    assert!(chunk.id.starts_with("kb-"));
    assert_eq!(chunk.page, Some(12));
    assert!(chunk.citation.contains("contract.docx"));
    assert!(chunk.citation.contains("Employment > Termination"));
    assert!(chunk.citation.contains("clause 7.3"));
    assert!(chunk.citation.contains("p.12"));
    assert!(chunk.citation.ends_with("v3"));

    let bumped = StructuredDocument {
        version: "4".to_string(),
        ..doc.clone()
    };
    assert_ne!(chunker.chunk(&bumped)[0].id, chunk.id);
}

#[test]
fn retrieval_filters_by_tenant_and_acl() {
    let acl = ChunkPermissions {
        tenant_id: "acme".to_string(),
        visibility: Visibility::Private,
        allowed_users: vec!["u-alice".to_string()],
        allowed_groups: vec!["legal".to_string()],
        ..ChunkPermissions::default()
    };
    let doc = document(acl, "# Secret\nFirst line.\n\nSecond line.");
    let chunks = StructuredChunker::default().chunk(&doc);
    assert_eq!(chunks.len(), 2);
    assert!(chunks
        .iter()
        .all(|chunk| chunk.permissions.visibility == Visibility::Private
            && chunk.permissions.tenant_id == "acme"));

    let alice = AccessContext {
        tenant_id: "acme",
        user_id: "u-alice",
        groups: &[],
    };
    assert_eq!(filter_for_read(&chunks, &alice).len(), 2);

    let bob = AccessContext {
        tenant_id: "acme",
        user_id: "u-bob",
        groups: &["legal".to_string()],
    };
    assert_eq!(filter_for_read(&chunks, &bob).len(), 2);

    let carol = AccessContext {
        tenant_id: "acme",
        user_id: "u-carol",
        groups: &[],
    };
    assert!(filter_for_read(&chunks, &carol).is_empty());
    assert!(!can_read(&chunks[0], &carol));

    let foreign = AccessContext {
        tenant_id: "other",
        user_id: "u-alice",
        groups: &[],
    };
    assert!(filter_for_read(&chunks, &foreign).is_empty());
}

#[test]
fn public_chunks_are_readable_within_the_tenant_only() {
    let doc = document(public_acl(), "# Public\nA line.");
    let chunks = StructuredChunker::default().chunk(&doc);

    let anonymous = AccessContext {
        tenant_id: "acme",
        user_id: "",
        groups: &[],
    };
    assert_eq!(filter_for_read(&chunks, &anonymous).len(), 1);

    let foreign = AccessContext {
        tenant_id: "other",
        user_id: "u-1",
        groups: &[],
    };
    assert!(filter_for_read(&chunks, &foreign).is_empty());
}

#[test]
fn parser_keeps_markers_off_the_body() {
    let elements = parse_structured_text("# T\n[[PAGE:9]]\n[[CLAUSE:2.1]]\nText here.\n| x | y |");
    assert_eq!(elements.len(), 3);
    assert_eq!(
        elements[0],
        Element::Heading {
            level: 1,
            text: "T".to_string()
        }
    );
    assert_eq!(
        elements[1],
        Element::Paragraph {
            text: "Text here.".to_string(),
            locator: Locator::page(9).with_clause("2.1")
        }
    );
}
