//! #2499 — citation-only in-chat answers about a saved document.
//!
//! Saves project structured chunks into the Docs in-memory KB index. The HTTP
//! read side (`botdocs::handlers_api::kb_search`) already answers with citation
//! metadata only. This module is the chat counterpart: after `inject_kb` has
//! run, it searches the same index for the turn's authenticated principal and
//! appends at most three *identity-only* lines to the system message. Document
//! bytes never enter the index and therefore can never enter the prompt here.
//!
//! Fail closed: an anonymous (nil) principal or a Docs state that was never
//! published to `AppState.extensions` yields no injection at all.

use std::sync::Arc;

use botcore::kb::permissions::UserContext;
use botcore::kb::structured::SearchHit;
use botcore::shared::state::Extensions;
use botdocs::state::{DocState, KbCitationMeta};
use uuid::Uuid;

/// Hard ceiling on citation lines added to one turn (also the search limit).
const MAX_CITATIONS: usize = 3;

/// Same principal derivation as `kb_search.rs::principal_for`: the verified
/// user id plus the server's own bucket as tenant. A chat turn only holds a
/// `Uuid`, so the `local-` spelling is passed as the alternate identifier the
/// ACL checks (`AclMetadata::allows` matches `user_id` first, then `email`).
/// Nil means anonymous: no principal, no injection.
fn principal_for(user_id: Uuid, bucket_name: &str) -> Option<UserContext> {
    if user_id.is_nil() {
        return None;
    }
    let owner = user_id.to_string();
    Some(UserContext::authenticated(
        user_id,
        Some(format!("local-{owner}")),
        Some(DocState::tenant_uuid(bucket_name)),
    ))
}

/// One JSON line with the five allowed fields. No content, no score, no query
/// text and no raw error text — only the identity `citation_meta` captured at
/// projection time plus the hit's own section/page address.
fn citation_line(meta: &KbCitationMeta, hit: &SearchHit) -> String {
    serde_json::json!({
        "document_id": meta.document_id,
        "file": meta.file,
        "version": meta.version,
        "section": hit.section,
        "page": hit.source.page,
    })
    .to_string()
}

/// Append the lines to the existing system message, or create one at index 0
/// (mirroring `inject_kb_context` so the block always precedes the user turn).
fn append_lines(messages: &mut serde_json::Value, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    let Some(array) = messages.as_array_mut() else {
        // Constant message: never log the query, the principal or a document id.
        log::warn!("chat citation injection skipped: prompt is not a message array");
        return;
    };
    let block = lines.join("\n");
    if let Some(index) = array.iter().position(|m| m["role"] == "system") {
        if let Some(content) = array[index]["content"].as_str() {
            array[index]["content"] =
                serde_json::Value::String(format!("{content}\n{block}"));
            return;
        }
    }
    array.insert(0, serde_json::json!({ "role": "system", "content": block }));
}

/// Resolve the shared Docs state from `AppState.extensions`. The producer
/// inserts `Arc<DocState>` via `insert_blocking`; accept a directly inserted
/// `DocState` as well so the seam survives a small producer refactor.
async fn docs_from(extensions: &Extensions) -> Option<Arc<DocState>> {
    if let Some(shared) = extensions.get::<Arc<DocState>>().await {
        return Some(shared.as_ref().clone());
    }
    extensions.get::<DocState>().await
}

/// Injection seam, called immediately after `inject_kb` in `build_and_stream`.
pub(super) async fn inject_saved_doc_citations(
    extensions: &Extensions,
    bucket_name: &str,
    user_id: Uuid,
    user_query: &str,
    messages: &mut serde_json::Value,
) {
    let Some(principal) = principal_for(user_id, bucket_name) else {
        return;
    };
    let Some(docs) = docs_from(extensions).await else {
        return;
    };
    let hits = docs.search_text(user_query, &principal, MAX_CITATIONS);
    let lines: Vec<String> = hits
        .iter()
        .take(MAX_CITATIONS)
        .map(|hit| citation_line(&docs.citation_meta(&hit.id), hit))
        .collect();
    append_lines(messages, &lines);
}

#[cfg(test)]
mod tests {
    use super::*;
    use botcore::shared::analytics::MetricsCollector;

    const TENANT: &str = "11111111-1111-1111-1111-111111111111";
    const OWNER: &str = "33333333-3333-3333-3333-333333333333";
    const HTML: &str = "<h1>Employment</h1><h2>Termination</h2>\
                        <p>7.3 Notice</p>\
                        <p>The notice period is ninety days. ZANZIBARCLAUSE.</p>";

    fn uuid(value: &str) -> Uuid {
        match Uuid::parse_str(value) {
            Ok(id) => id,
            Err(_) => Uuid::nil(),
        }
    }

    fn test_docs_state(bucket: &str) -> Arc<DocState> {
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://unused:unused@127.0.0.1:1/unused",
        );
        let pool = diesel::r2d2::Pool::builder()
            .max_size(1)
            .build_unchecked(manager);
        Arc::new(DocState::with_in_memory_kb_index(
            Arc::new(pool),
            Arc::new(crate::drive::NoopDrive),
            bucket.to_string(),
            Arc::new(MetricsCollector::default()),
        ))
    }

    fn system_content(messages: &serde_json::Value) -> &str {
        match messages
            .get(0)
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
        {
            Some(content) => content,
            None => "",
        }
    }

    #[tokio::test]
    async fn authenticated_turn_gets_citation_only_lines() {
        let docs = test_docs_state(TENANT);
        docs.project_to_kb(OWNER, "doc-1", "Employment", HTML);

        let extensions = Extensions::new();
        extensions.insert(docs).await;
        let mut messages = serde_json::json!([{ "role": "system", "content": "You are helpful." }]);

        inject_saved_doc_citations(
            &extensions,
            TENANT,
            uuid(OWNER),
            "ZANZIBARCLAUSE notice",
            &mut messages,
        )
        .await;

        let system = system_content(&messages);
        let citations = system
            .lines()
            .filter(|line| line.contains("\"document_id\""))
            .count();
        assert!(citations >= 1 && citations <= MAX_CITATIONS, "{system}");
        assert!(system.contains("\"document_id\":\"doc-1\""), "{system}");
        for field in ["file", "version", "section", "page"] {
            assert!(
                system.contains(format!("\"{field}\"").as_str()),
                "{system}"
            );
        }
        assert!(system.contains("You are helpful."), "{system}");
        for leaked in ["ninety days", "ZANZIBARCLAUSE", "content", "payload"] {
            assert!(!system.contains(leaked), "prompt leaked {leaked}: {system}");
        }
    }

    #[tokio::test]
    async fn nil_principal_injects_no_citation_lines() {
        let docs = test_docs_state(TENANT);
        docs.project_to_kb(OWNER, "doc-2", "Employment", HTML);

        let extensions = Extensions::new();
        extensions.insert(docs).await;
        let mut messages = serde_json::json!([{ "role": "system", "content": "base prompt" }]);

        inject_saved_doc_citations(
            &extensions,
            TENANT,
            Uuid::nil(),
            "ZANZIBARCLAUSE notice",
            &mut messages,
        )
        .await;

        assert_eq!(system_content(&messages), "base prompt");
    }
}
