//! Chat KB ingestion (the legacy, content-bearing Qdrant path).
//!
//! #2499 — DOC-QUESTION CITATION INJECTION IS NOT WIRED (static guard).
//!
//! The chat turn already reaches the right seam: `enriched_llm::build_and_stream`
//! holds the server-verified `user_id`, the user text and the mutable system
//! message array, and calls `inject_kb` here. A citation-only injection belongs
//! immediately after that call: fail closed when `user_id.is_nil()`, run
//! `DocState::search_text(query, &UserContext, limit)` for an authenticated
//! principal whose `organization_id` is `DocState::tenant_uuid(&state.bucket_name)`,
//! and append ONLY `{document_id, file, version, section, page}` (never document
//! content, never raw error text, counts only in logs).
//!
//! THE BLOCKER: the shared in-memory index is `DocState.kb_index`, built as
//! ROUTER-LOCAL state in `main_module/routes/feature_merge_office/sub_routes.rs`
//! (`DocState::with_in_memory_kb_index`, ~line 152) and handed only to
//! `configure_docs_routes` (~line 211). It is never published to
//! `AppState.extensions` (state_builder.rs ~line 368) or to any global registry —
//! botdocs' only globals are `METRICS` and `KB_CITATIONS` (citation metadata
//! keyed by the index's Arc address), neither of which is searchable. Therefore
//! this seam has no way to reach the index or the private citation map, and the
//! injection cannot be implemented without editing files outside this crate's
//! owned area (`sub_routes.rs` / `state_builder.rs`, which the owning agent must
//! change). The smallest producer change is to publish the same `Arc<DocState>`
//! into `app_state.extensions` with `insert_blocking`; `DocState::search_text`
//! and `DocState::citation_meta` are already public and citation-only.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

pub async fn inject_kb(
    state: &Arc<AppState>,
    _bot_uuid: Uuid,
    session_id: Uuid,
    _user_id: Uuid,
    _bot_name: &str,
    user_text: &str,
    messages: &mut serde_json::Value,
) -> Result<(), String> {
    let bot_uuid = _bot_uuid;
    crate::core::bot::kb_context::inject_kb_context(
        &state.conn,
        session_id,
        bot_uuid,
        user_text,
        messages,
        4000,
    )
    .await;
    Ok(())
}
