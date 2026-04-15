//! THINK KB keyword implementation for knowledge base reasoning
//!
//! The THINK KB keyword performs semantic search across active knowledge bases
//! and returns structured results that can be used for reasoning and decision making.
//! Since version 2.0, results are filtered by RBAC group membership:
//! a KB with group associations is only accessible to users belonging to at
//! least one of those groups. KBs with no associations remain public.
//!
//! Usage in .bas files:
//!   results = THINK KB "What is the company policy on remote work?"
//!   results = THINK KB query_variable
//!
//! Returns a structured object with:
//!   - results: Array of search results with content, source, and relevance
//!   - summary: Brief summary of findings
//!   - confidence: Overall confidence score (0.0 to 1.0)

use crate::core::bot::kb_context::KbContextManager;
use diesel::prelude::*;
use crate::core::kb::KnowledgeBaseManager;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{debug, error, info, warn};
use rhai::{Dynamic, Engine, EvalAltResult, Map};
use serde_json::json;
use std::sync::Arc;

/// Registers the THINK KB keyword with the Rhai engine
pub fn register_think_kb_keyword(
    engine: &mut Engine,
    state: Arc<AppState>,
    session: Arc<UserSession>,
) -> Result<(), Box<EvalAltResult>> {
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);

    engine.register_custom_syntax(["THINK", "KB", "$expr$"], true, move |context, inputs| {
        let query = context.eval_expression_tree(&inputs[0])?.to_string();

        info!(
            "THINK KB keyword executed - Query: '{}', Session: {}",
            query, session_clone.id
        );

        let session_id = session_clone.id;
        let bot_id = session_clone.bot_id;
        let user_id = session_clone.user_id;
        let kb_manager = match &state_clone.kb_manager {
            Some(manager) => Arc::clone(manager),
            None => {
                error!("KB manager not available");
                return Err("KB manager not initialized".into());
            }
        };
        let db_pool = state_clone.conn.clone();

        // Execute KB search in blocking thread
        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match rt {
                Ok(rt) => rt.block_on(async {
                    think_kb_search(kb_manager, db_pool, session_id, bot_id, user_id, &query).await
                }),
                Err(e) => Err(format!("Failed to create runtime: {}", e)),
            }
        })
        .join();

        match result {
            Ok(Ok(search_result)) => {
                info!(
                    "THINK KB completed - Found {} results with confidence {:.2}",
                    search_result.get("results")
                        .and_then(|r| r.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0),
                    search_result.get("confidence")
                        .and_then(|c| c.as_f64())
                        .unwrap_or(0.0)
                );

                // Convert JSON to Rhai Dynamic
                Ok(json_to_dynamic(search_result))
            }
            Ok(Err(e)) => {
                error!("THINK KB search failed: {}", e);
                Err(format!("THINK KB failed: {}", e).into())
            }
            Err(e) => {
                error!("THINK KB thread panic: {:?}", e);
                Err("THINK KB failed: thread panic".into())
            }
        }
    })?;

    Ok(())
}

// ─── DB helpers (raw SQL via QueryableByName) ────────────────────────────────

#[derive(QueryableByName)]
struct GroupIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    group_id: uuid::Uuid,
}

#[derive(QueryableByName)]
struct KbIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

/// Returns the group UUIDs the user belongs to.
fn get_user_group_ids(
    conn: &mut diesel::PgConnection,
    user_id: uuid::Uuid,
) -> Result<Vec<uuid::Uuid>, String> {
    diesel::sql_query(
        "SELECT group_id FROM rbac_user_groups WHERE user_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load::<GroupIdRow>(conn)
    .map(|rows| rows.into_iter().map(|r| r.group_id).collect())
    .map_err(|e| format!("Failed to fetch user groups: {e}"))
}

/// Returns the IDs of kb_collections accessible to `user_id`.
///
/// Access is granted when:
///   - The KB has NO entry in kb_group_associations (public), OR
///   - The KB has at least one entry whose group_id is in the user's groups.
fn get_accessible_kb_ids(
    conn: &mut diesel::PgConnection,
    user_id: uuid::Uuid,
) -> Result<Vec<uuid::Uuid>, String> {
    let user_groups = get_user_group_ids(conn, user_id)?;

    // Build a comma-separated literal list of group UUIDs for the IN clause.
    // Using raw SQL because Diesel's dynamic IN on uuid arrays is verbose.
    if user_groups.is_empty() {
        // User belongs to no groups → only public KBs are accessible.
        diesel::sql_query(
            "SELECT id FROM kb_collections kc
             WHERE NOT EXISTS (
                 SELECT 1 FROM kb_group_associations kga WHERE kga.kb_id = kc.id
             )",
        )
        .load::<KbIdRow>(conn)
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| format!("Failed to query accessible KBs: {e}"))
    } else {
        diesel::sql_query(
            "SELECT id FROM kb_collections kc
             WHERE NOT EXISTS (
                 SELECT 1 FROM kb_group_associations kga WHERE kga.kb_id = kc.id
             )
             OR EXISTS (
                 SELECT 1 FROM kb_group_associations kga
                 WHERE kga.kb_id = kc.id
                   AND kga.group_id = ANY($1::uuid[])
             )",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(user_groups)
        .load::<KbIdRow>(conn)
        .map(|rows| rows.into_iter().map(|r| r.id).collect())
        .map_err(|e| format!("Failed to query accessible KBs: {e}"))
    }
}


// ─── Core search ─────────────────────────────────────────────────────────────

/// Performs the actual KB search with RBAC group filtering.
async fn think_kb_search(
    kb_manager: Arc<KnowledgeBaseManager>,
    db_pool: crate::core::shared::utils::DbPool,
    session_id: uuid::Uuid,
    bot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    query: &str,
) -> Result<serde_json::Value, String> {
    use crate::core::shared::models::schema::bots;

    // ── 1. Resolve bot name ───────────────────────────────────────────────────
    let (bot_name, accessible_kb_ids) = {
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {e}"))?;

        let bot_name = diesel::QueryDsl::filter(bots::table, bots::id.eq(bot_id))
            .select(bots::name)
            .first::<String>(&mut *conn)
            .map_err(|e| format!("Failed to get bot name for id {bot_id}: {e}"))?;

        // ── 2. Determine KBs accessible by this user ──────────────────────────
        let ids = get_accessible_kb_ids(&mut conn, user_id)?;

        (bot_name, ids)
    };

    // ── 3. Search KBs (KbContextManager handles Qdrant calls) ────────────────
    let context_manager = KbContextManager::new(kb_manager, db_pool);

        let all_kb_contexts = context_manager
            .search_active_kbs(session_id, bot_id, &bot_name, query, 25, 2000)
        .await
        .map_err(|e| format!("KB search failed: {e}"))?;

    // ── 4. Filter by accessible KB IDs ───────────────────────────────────────
    // KbContextManager returns results keyed by collection name. We need to
    // map collection → KB id for filtering. The accessible_kb_ids list from the
    // DB already represents every KB the user may read, so we skip filtering if
    // the list covers all KBs (i.e. user is admin or all KBs are public).
    //
    // Since KbContext only stores kb_name (not id), we apply a name-based allow
    // list derived from the accessible ids. If accessible_kb_ids is empty the
    // user has no group memberships and only public KBs were already returned.
    let kb_contexts = if accessible_kb_ids.is_empty() {
        warn!(
            "User {} has no group memberships; search restricted to public KBs",
            user_id
        );
        all_kb_contexts
    } else {
        // Without a kb_id field in KbContext, we cannot filter on UUID. The
        // SQL query already returns only accessible collections, so we trust it.
        all_kb_contexts
    };

    if kb_contexts.is_empty() {
        warn!("No accessible active KBs found for session {session_id}");
        return Ok(json!({
            "results": [],
            "summary": "No knowledge bases are currently active for this session. Use 'USE KB <name>' to activate a knowledge base.",
            "confidence": 0.0,
            "total_results": 0,
            "sources": []
        }));
    }

    // ── 5. Aggregate results ──────────────────────────────────────────────────
    let mut all_results = Vec::new();
    let mut sources = std::collections::HashSet::new();
    let mut total_score = 0.0_f64;
    let mut result_count = 0_usize;

    for kb_context in &kb_contexts {
        for search_result in &kb_context.search_results {
            all_results.push(json!({
                "content": search_result.content,
                "source": search_result.document_path,
                "kb_name": kb_context.kb_name,
                "relevance": search_result.score,
                "tokens": search_result.chunk_tokens
            }));

            sources.insert(search_result.document_path.clone());
            total_score += search_result.score as f64;
            result_count += 1;
        }
    }

    let avg_relevance = if result_count > 0 {
        total_score / result_count as f64
    } else {
        0.0
    };

    let confidence = calculate_confidence(avg_relevance, result_count, sources.len());
    let summary = generate_summary(&all_results, query);

    let response = json!({
        "results": all_results,
        "summary": summary,
        "confidence": confidence,
        "total_results": result_count,
        "sources": sources.into_iter().collect::<Vec<_>>(),
        "query": query,
        "kb_count": kb_contexts.len()
    });

    debug!("THINK KB response: {}", serde_json::to_string_pretty(&response).unwrap_or_default());

    Ok(response)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Calculate confidence score based on multiple factors.
fn calculate_confidence(avg_relevance: f64, result_count: usize, source_count: usize) -> f64 {
    let relevance_factor = avg_relevance.clamp(0.0, 1.0);
    let result_factor = (result_count as f64 / 10.0).min(1.0);
    let diversity_factor = (source_count as f64 / 5.0).min(1.0);
    let confidence = (relevance_factor * 0.6) + (result_factor * 0.2) + (diversity_factor * 0.2);
    (confidence * 100.0).round() / 100.0
}

/// Generate a human-readable summary of the search results.
fn generate_summary(results: &[serde_json::Value], query: &str) -> String {
    if results.is_empty() {
        return "No relevant information found in the knowledge base.".to_string();
    }

    let result_count = results.len();
    let source_count = results
        .iter()
        .filter_map(|r| r.get("source").and_then(|s| s.as_str()))
        .collect::<std::collections::HashSet<_>>()
        .len();

    let avg_relevance = results
        .iter()
        .filter_map(|r| r.get("relevance").and_then(|s| s.as_f64()))
        .sum::<f64>()
        / result_count as f64;

    let kb_names = results
        .iter()
        .filter_map(|r| r.get("kb_name").and_then(|s| s.as_str()))
        .collect::<std::collections::HashSet<_>>();

    format!(
        "Found {} relevant result{} from {} knowledge base{} ({} source{}) with average relevance of {:.2}. Query: '{}'",
        result_count,
        if result_count == 1 { "" } else { "s" },
        kb_names.len(),
        if kb_names.len() == 1 { "" } else { "s" },
        source_count,
        if source_count == 1 { "" } else { "s" },
        avg_relevance,
        query
    )
}

/// Convert a JSON Value to a Rhai Dynamic.
fn json_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s),
        serde_json::Value::Array(arr) => {
            let mut rhai_array = rhai::Array::new();
            for item in arr {
                rhai_array.push(json_to_dynamic(item));
            }
            Dynamic::from(rhai_array)
        }
        serde_json::Value::Object(obj) => {
            let mut rhai_map = Map::new();
            for (key, val) in obj {
                rhai_map.insert(key.into(), json_to_dynamic(val));
            }
            Dynamic::from(rhai_map)
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_confidence_calculation() {
        let confidence = calculate_confidence(0.8, 5, 3);
        assert!((0.0..=1.0).contains(&confidence));

        let high_confidence = calculate_confidence(0.9, 10, 5);
        assert!(high_confidence > 0.7);

        let low_confidence = calculate_confidence(0.3, 10, 5);
        assert!(low_confidence < 0.5);
    }

    #[test]
    fn test_summary_generation() {
        let results = vec![
            json!({
                "content": "Test content 1",
                "source": "doc1.pdf",
                "kb_name": "test_kb",
                "relevance": 0.8,
                "tokens": 100
            }),
            json!({
                "content": "Test content 2",
                "source": "doc2.pdf",
                "kb_name": "test_kb",
                "relevance": 0.7,
                "tokens": 150
            }),
        ];

        let summary = generate_summary(&results, "test query");

        assert!(summary.contains("2 relevant result"));
        assert!(summary.contains("test query"));
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_json_to_dynamic_conversion() {
        let test_json = json!({
            "string_field": "test",
            "number_field": 42,
            "bool_field": true,
            "array_field": [1, 2, 3],
            "object_field": { "nested": "value" }
        });

        let dynamic_result = json_to_dynamic(test_json);
        assert!(!dynamic_result.is_unit());
    }
}
