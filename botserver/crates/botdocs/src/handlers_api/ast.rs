//! #2451 (bounded slice) — compact structured read for the Docs app.
//!
//! `GET /api/docs/:id/ast` returns a page-addressable, token-budgeted view of a
//! document as flat nodes with *stable* ids. A stable id is derived from the
//! node's document order, kind, page and text — never from a random UUID or
//! from caller identity — so the same node keeps the same id across reads and
//! can be addressed later by a patch/preview endpoint (separate scope).
//!
//! Tenant isolation is inherited from [`crate::storage::load_document_from_drive`]:
//! it reads `{BUCKET}/users/{verified-owner}/docs/{id}.html`, so the bucket
//! (the branch/tenant boundary) and the verified `DocUser` namespace both
//! constrain the read. Nothing here accepts a caller-supplied bucket/owner and
//! no owner id, author name or other PII is copied into the response.

use crate::auth::DocUser;
use crate::state::DocState;
use crate::storage::load_document_from_drive;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Schema version of the AST wire format. Bump when the node shape changes.
pub const AST_SCHEMA_VERSION: u32 = 1;
/// Default read budget, chosen to match the epic's "under 8k tokens" target.
pub const DEFAULT_AST_TOKEN_BUDGET: usize = 8_000;
/// Hard ceiling so an abusive `?tokens=` cannot request an unbounded response.
pub const MAX_AST_TOKEN_BUDGET: usize = 32_000;

/// One addressable block in the document. Deliberately contains no author or
/// owner fields: ids are content-derived and the payload is structural only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstNode {
    /// Stable, deterministic id (`n-<hex>`), safe to use as a patch anchor.
    pub id: String,
    /// Block kind: `heading1`..`heading6`, `paragraph`, `list-item`, etc.
    pub kind: String,
    /// 1-based page/section address (a new `h1` starts a new page).
    pub page: u32,
    /// 0-based document order, used as the stable cursor for paging.
    pub index: usize,
    /// Plain text of the block (inline markup stripped, entities decoded).
    pub text: String,
    /// Estimated token cost of `text` (~4 characters per token).
    pub tokens: usize,
}

/// The compact AST response. It never includes `Document` fields such as
/// `owner_id`, `storage_path`, comments or track-change authors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDocument {
    pub schema_version: u32,
    pub doc_id: String,
    pub version: u64,
    pub total_nodes: usize,
    pub total_pages: u32,
    pub returned_nodes: usize,
    pub budget_tokens: usize,
    pub used_tokens: usize,
    pub truncated: bool,
    /// Index to pass back as `?from=` to read the next window, when truncated.
    pub next_index: Option<usize>,
    pub nodes: Vec<AstNode>,
}

/// Query parameters for the AST read. All are optional.
#[derive(Debug, Default, Deserialize)]
pub struct AstQuery {
    /// Token budget; clamped to `[1, MAX_AST_TOKEN_BUDGET]`.
    pub tokens: Option<usize>,
    /// Return only nodes on this 1-based page/section.
    pub page: Option<u32>,
    /// Start at this node index (cursor from a previous `next_index`).
    pub from: Option<usize>,
}

/// Pure outcome of applying a token budget to a node slice.
#[derive(Debug, PartialEq, Eq)]
struct BudgetedNodes {
    nodes: Vec<AstNode>,
    used_tokens: usize,
    truncated: bool,
    next_index: Option<usize>,
}

/// Estimate the token cost of already-extracted plain text. A cheap, pure
/// proxy (~4 chars/token) keeps the endpoint side-effect free and testable.
fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Build a deterministic node id. The same `(index, kind, page, text)` always
/// yields the same id, and no caller identity participates in the hash.
fn stable_node_id(index: usize, kind: &str, page: u32, text: &str) -> String {
    fn mix(hash: &mut u64, bytes: &[u8]) {
        for &byte in bytes {
            *hash ^= u64::from(byte);
            *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    mix(&mut hash, index.to_string().as_bytes());
    mix(&mut hash, b"|");
    mix(&mut hash, kind.as_bytes());
    mix(&mut hash, b"|");
    mix(&mut hash, page.to_string().as_bytes());
    mix(&mut hash, b"|");
    mix(&mut hash, text.as_bytes());
    format!("n-{hash:016x}")
}

/// Map a block tag name to its stable `kind`, or `None` for inline/ignored tags.
fn block_kind(name: &str) -> Option<&'static str> {
    match name {
        "h1" => Some("heading1"),
        "h2" => Some("heading2"),
        "h3" => Some("heading3"),
        "h4" => Some("heading4"),
        "h5" => Some("heading5"),
        "h6" => Some("heading6"),
        "p" => Some("paragraph"),
        "li" => Some("list-item"),
        "blockquote" => Some("quote"),
        "pre" => Some("code"),
        "tr" => Some("table-row"),
        "div" | "section" | "article" => Some("block"),
        _ => None,
    }
}

/// Push a collected text buffer as a node of the given kind, if non-empty.
fn push_node(
    nodes: &mut Vec<AstNode>,
    text: &mut String,
    index: &mut usize,
    kind: &str,
    page: u32,
) {
    let plain = crate::storage::decode_html_entities(text.trim());
    let plain = plain.trim();
    if plain.is_empty() {
        text.clear();
        return;
    }
    nodes.push(AstNode {
        id: stable_node_id(*index, kind, page, plain),
        kind: kind.to_string(),
        page,
        index: *index,
        text: plain.to_string(),
        tokens: estimate_tokens(plain),
    });
    *index += 1;
    text.clear();
}

/// Parse block-level HTML into flat, page-addressed nodes.
///
/// This is a small, dependency-free scanner (the crate has no HTML DOM
/// parser). Inline tags are dropped, their text is kept; tag *attributes* are
/// never copied into node text, which is what keeps attribute-carried PII out.
fn html_to_ast_nodes(html: &str) -> Vec<AstNode> {
    let mut nodes = Vec::new();
    let mut text = String::new();
    let mut open_kind: Option<&'static str> = None;
    let mut in_tag = false;
    let mut tag = String::new();
    let mut page = 1u32;
    let mut index = 0usize;

    for ch in html.chars() {
        match ch {
            '<' if !in_tag => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let raw = tag.trim();
                let closing = raw.starts_with('/');
                let name = raw
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if let Some(kind) = block_kind(&name) {
                    if closing {
                        if open_kind.take().is_some() {
                            push_node(&mut nodes, &mut text, &mut index, kind, page);
                        }
                    } else {
                        // A new top-level `h1` begins a new page/section. The
                        // first heading stays on page 1; every later one
                        // advances the counter *before* its content is pushed
                        // so heading and body share a page address.
                        if name == "h1" && !nodes.is_empty() {
                            page += 1;
                        }
                        // Flush stray text before opening a new block.
                        push_node(&mut nodes, &mut text, &mut index, "paragraph", page);
                        open_kind = Some(kind);
                    }
                }
            }
            '<' => tag.push(ch),
            _ if in_tag => tag.push(ch),
            _ => text.push(ch),
        }
    }
    push_node(&mut nodes, &mut text, &mut index, "paragraph", page);
    nodes
}

/// Pure page filter (`None`/0 means "all pages").
fn select_page(nodes: Vec<AstNode>, page: Option<u32>) -> Vec<AstNode> {
    match page {
        Some(p) if p >= 1 => nodes.into_iter().filter(|n| n.page == p).collect(),
        _ => nodes,
    }
}

/// Pure token-budget predicate. Nodes are taken in document order from `from`
/// while the cumulative estimate fits the budget. An oversized first node is
/// still returned once so a caller always makes forward progress; `truncated`
/// and `next_index` let it page through the rest.
fn truncate_to_budget(nodes: Vec<AstNode>, budget: usize, from: usize) -> BudgetedNodes {
    let mut included = Vec::new();
    let mut used = 0usize;
    let mut next_index = None;
    for node in nodes.into_iter().filter(|n| n.index >= from) {
        if !included.is_empty() && used.saturating_add(node.tokens) > budget {
            next_index = Some(node.index);
            break;
        }
        used = used.saturating_add(node.tokens);
        included.push(node);
    }
    BudgetedNodes {
        truncated: next_index.is_some(),
        nodes: included,
        used_tokens: used,
        next_index,
    }
}

/// Clamp a caller-supplied budget into the allowed range.
fn effective_budget(requested: Option<usize>) -> usize {
    requested
        .unwrap_or(DEFAULT_AST_TOKEN_BUDGET)
        .clamp(1, MAX_AST_TOKEN_BUDGET)
}

/// `GET /api/docs/:id/ast` — compact, tenant-scoped structured read.
pub async fn handle_docs_ast(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Path(doc_id): Path<String>,
    Query(query): Query<AstQuery>,
) -> Result<Json<AstDocument>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(
                        &e, "botdocs::ast", None)
                })),
            ))
        }
    };

    // Defense in depth: the drive path already scopes by verified owner and
    // bucket, but never serve a node tree for a document owned by someone else.
    if doc.owner_id != user_id {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        ));
    }

    let all_nodes = html_to_ast_nodes(&doc.content);
    let total_nodes = all_nodes.len();
    let total_pages = all_nodes.iter().map(|n| n.page).max().unwrap_or(1);
    let budget = effective_budget(query.tokens);
    let from = query.from.unwrap_or(0);
    let page_nodes = select_page(all_nodes, query.page);
    let result = truncate_to_budget(page_nodes, budget, from);

    Ok(Json(AstDocument {
        schema_version: AST_SCHEMA_VERSION,
        doc_id: doc.id,
        version: doc.version,
        total_nodes,
        total_pages,
        returned_nodes: result.nodes.len(),
        budget_tokens: budget,
        used_tokens: result.used_tokens,
        truncated: result.truncated,
        next_index: result.next_index,
        nodes: result.nodes,
    }))
}

#[cfg(test)]
#[path = "ast_tests.rs"]
mod tests;
