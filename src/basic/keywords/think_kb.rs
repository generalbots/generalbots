//! THINK KB keyword implementation for knowledge base reasoning
//!
//! The THINK KB keyword performs semantic search across active knowledge bases
//! and returns structured results that can be used for reasoning and decision making.
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
        let bot_name = session_clone.bot_name.clone();
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
            tokio::runtime::Handle::current().block_on(async {
                think_kb_search(kb_manager, db_pool, session_id, &bot_name, &query).await
            })
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

/// Performs the actual KB search and reasoning
async fn think_kb_search(
    kb_manager: Arc<KnowledgeBaseManager>,
    db_pool: crate::core::shared::utils::DbPool,
    session_id: uuid::Uuid,
    bot_name: &str,
    query: &str,
) -> Result<serde_json::Value, String> {
    let context_manager = KbContextManager::new(kb_manager, db_pool);

    // Search active KBs with reasonable limits
    let kb_contexts = context_manager
        .search_active_kbs(session_id, bot_name, query, 10, 2000)
        .await
        .map_err(|e| format!("KB search failed: {}", e))?;

    if kb_contexts.is_empty() {
        warn!("No active KBs found for session {}", session_id);
        return Ok(json!({
            "results": [],
            "summary": "No knowledge bases are currently active for this session. Use 'USE KB <name>' to activate a knowledge base.",
            "confidence": 0.0,
            "total_results": 0,
            "sources": []
        }));
    }

    let mut all_results = Vec::new();
    let mut sources = std::collections::HashSet::new();
    let mut total_score = 0.0;
    let mut result_count = 0;

    // Process results from all KBs
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

    // Calculate overall confidence based on average relevance and result count
    let avg_relevance = if result_count > 0 {
        total_score / result_count as f64
    } else {
        0.0
    };

    // Confidence factors: relevance score, number of results, source diversity
    let confidence = calculate_confidence(avg_relevance, result_count, sources.len());

    // Generate summary based on results
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

/// Calculate confidence score based on multiple factors
fn calculate_confidence(avg_relevance: f64, result_count: usize, source_count: usize) -> f64 {
    // Base confidence from average relevance (0.0 to 1.0)
    let relevance_factor = avg_relevance.min(1.0).max(0.0);
    
    // Boost confidence with more results (diminishing returns)
    let result_factor = (result_count as f64 / 10.0).min(1.0);
    
    // Boost confidence with source diversity
    let diversity_factor = (source_count as f64 / 5.0).min(1.0);
    
    // Weighted combination
    let confidence = (relevance_factor * 0.6) + (result_factor * 0.2) + (diversity_factor * 0.2);
    
    // Round to 2 decimal places
    (confidence * 100.0).round() / 100.0
}

/// Generate a summary of the search results
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
        .sum::<f64>() / result_count as f64;

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

/// Convert JSON Value to Rhai Dynamic
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_confidence_calculation() {
        // Test the confidence calculation function
        let confidence = calculate_confidence(0.8, 5, 3);
        assert!(confidence >= 0.0 && confidence <= 1.0);
        
        // High relevance, many results, diverse sources should give high confidence
        let high_confidence = calculate_confidence(0.9, 10, 5);
        assert!(high_confidence > 0.7);
        
        // Low relevance should give low confidence
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
            })
        ];
        
        let summary = generate_summary(&results, "test query");
        
        assert!(summary.contains("2 relevant result"));
        assert!(summary.contains("test query"));
        assert!(summary.len() > 0);
    }

    #[test]
    fn test_json_to_dynamic_conversion() {
        let test_json = json!({
            "string_field": "test",
            "number_field": 42,
            "bool_field": true,
            "array_field": [1, 2, 3],
            "object_field": {
                "nested": "value"
            }
        });
        
        let dynamic_result = json_to_dynamic(test_json);
        
        // The conversion should not panic and should return a Dynamic value
        assert!(!dynamic_result.is_unit());
    }
}
