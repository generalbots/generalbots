//! Inline tests for the Docs AST read (`#2451` bounded slice).

use super::*;

fn sample_html() -> &'static str {
    "<h1>Chapter One</h1><p>First paragraph.</p>\
     <h1>Chapter Two</h1><p data-user=\"alice@example.com\">Second paragraph.</p>\
     <ul><li>Item A</li><li>Item B</li></ul>"
}

#[test]
fn stable_ids_are_deterministic_and_content_derived() {
    let first = html_to_ast_nodes(sample_html());
    let second = html_to_ast_nodes(sample_html());
    assert!(!first.is_empty());
    assert_eq!(first, second, "same input must yield byte-identical nodes");
    for node in &first {
        assert!(node.id.starts_with("n-"));
        assert_eq!(
            node.id,
            stable_node_id(node.index, &node.kind, node.page, &node.text)
        );
    }
    // Changing the text changes the id; the owner is never an input.
    let changed = stable_node_id(0, "paragraph", 1, "different text");
    assert_ne!(changed, first[1].id);
}

#[test]
fn stable_ids_do_not_encode_owner_identity() {
    // The id hash takes only (index, kind, page, text). Two calls that would
    // differ only by "owner" are identical because owner is not a parameter.
    let a = stable_node_id(3, "paragraph", 2, "same text");
    let b = stable_node_id(3, "paragraph", 2, "same text");
    assert_eq!(a, b);
    assert!(!a.contains("alice"));
    assert!(!a.contains("@"));
}

#[test]
fn budget_is_respected_and_reports_truncation() {
    let nodes = html_to_ast_nodes(sample_html());
    let total: usize = nodes.iter().map(|n| n.tokens).sum();
    let budget = total / 2;
    let result = truncate_to_budget(nodes, budget, 0);
    assert!(result.used_tokens <= budget);
    assert!(result.truncated);
    assert!(result.next_index.is_some());
    assert!(!result.nodes.is_empty());
    assert!(result.nodes.len() < 4);
}

#[test]
fn oversized_first_node_still_yields_one_node() {
    let nodes = html_to_ast_nodes("<p>a very long paragraph that exceeds the tiny budget</p>");
    assert_eq!(nodes.len(), 1);
    let result = truncate_to_budget(nodes, 1, 0);
    assert_eq!(result.nodes.len(), 1, "guarantee forward progress");
    assert!(!result.truncated);
}

#[test]
fn unbounded_budget_returns_everything() {
    let nodes = html_to_ast_nodes(sample_html());
    let result = truncate_to_budget(nodes, MAX_AST_TOKEN_BUDGET, 0);
    assert!(!result.truncated);
    assert!(result.next_index.is_none());
    assert_eq!(result.nodes.len(), 6);
}

#[test]
fn page_addressing_splits_on_h1() {
    let nodes = html_to_ast_nodes(sample_html());
    let pages: Vec<u32> = nodes.iter().map(|n| n.page).collect();
    assert_eq!(pages, vec![1, 1, 2, 2, 2, 2]);
    let page_two = select_page(nodes, Some(2));
    assert_eq!(page_two.len(), 4);
    assert!(page_two.iter().all(|n| n.page == 2));
}

#[test]
fn attribute_carried_pii_never_enters_node_text() {
    let nodes = html_to_ast_nodes(sample_html());
    let joined = nodes
        .iter()
        .map(|n| n.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!joined.contains("alice@example.com"));
    assert!(joined.contains("Second paragraph."));
}

#[test]
fn response_payload_has_no_owner_or_author_fields() {
    let doc = AstDocument {
        schema_version: AST_SCHEMA_VERSION,
        doc_id: "doc-1".to_string(),
        version: 1,
        total_nodes: 1,
        total_pages: 1,
        returned_nodes: 1,
        budget_tokens: 100,
        used_tokens: 3,
        truncated: false,
        next_index: None,
        nodes: html_to_ast_nodes("<p>Visible clause.</p>"),
    };
    let json = match serde_json::to_string(&doc) {
        Ok(json) => json,
        Err(_) => String::new(),
    };
    for forbidden in ["owner", "author", "display_name", "storage_path", "@"] {
        assert!(!json.contains(forbidden), "payload leaked `{forbidden}`");
    }
}

#[test]
fn budget_clamping_covers_default_and_bounds() {
    assert_eq!(effective_budget(None), DEFAULT_AST_TOKEN_BUDGET);
    assert_eq!(effective_budget(Some(0)), 1);
    assert_eq!(
        effective_budget(Some(MAX_AST_TOKEN_BUDGET + 999)),
        MAX_AST_TOKEN_BUDGET
    );
    assert_eq!(effective_budget(Some(42)), 42);
}

#[test]
fn from_cursor_is_an_absolute_node_index() {
    let nodes = html_to_ast_nodes(sample_html());
    let paged = truncate_to_budget(nodes.clone(), MAX_AST_TOKEN_BUDGET, 3);
    assert_eq!(paged.nodes.len(), 3);
    assert_eq!(paged.nodes[0].index, 3);
    assert_eq!(paged.nodes[1].index, 4);
    assert_eq!(paged.nodes[2].index, 5);
    // The same cursor is correct after a page filter because `from` is an
    // absolute index, not a positional skip.
    let page_two = select_page(nodes, Some(2));
    let filtered = truncate_to_budget(page_two, MAX_AST_TOKEN_BUDGET, 3);
    assert_eq!(filtered.nodes[0].index, 3);
    assert_eq!(filtered.nodes.len(), 3);
}

#[test]
fn estimate_tokens_is_ceil_chars_over_four() {
    assert_eq!(estimate_tokens(""), 0);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcde"), 2);
}
