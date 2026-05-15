use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use diesel::prelude::*;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use log::{error, info, warn};
use rhai::{Dynamic, Engine, Map};
use serde_json::json;
use std::sync::Arc;

pub fn register_think_kb_keyword(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = state;
    let session_clone = user;

    engine.register_custom_syntax(["THINK", "KB", "$expr$"], true, move |context, inputs| {
        let query = context.eval_expression_tree(&inputs[0])?.to_string();

        info!(
            "THINK KB keyword executed - Query: '{}', Session: {}",
            query, session_clone.id
        );

        let session_id = session_clone.id;
        let bot_id = session_clone.bot_id;
        let user_id = session_clone.user_id;
        let db_pool = state_clone.db_pool().clone();
        let query_clone = query.clone();

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match rt {
                Ok(rt) => rt.block_on(async { think_kb_search(&db_pool, session_id, bot_id, user_id, &query_clone).await }),
                Err(e) => Err(format!("Failed to create runtime: {}", e)),
            }
        })
        .join();

        match result {
            Ok(Ok(search_result)) => {
                info!(
                    "THINK KB completed - Found {} results, {} consolidation insights",
                    search_result.get("results")
                        .and_then(|r| r.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0),
                    search_result.get("consolidation_insights")
                        .and_then(|r| r.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
                );
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
    })
    .expect("valid THINK KB syntax registration");
}

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

fn get_accessible_kb_ids(
    conn: &mut diesel::PgConnection,
    user_id: uuid::Uuid,
) -> Result<Vec<uuid::Uuid>, String> {
    let user_groups = get_user_group_ids(conn, user_id)?;
    if user_groups.is_empty() {
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

#[derive(QueryableByName, Debug, Clone)]
struct KbCollectionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    qdrant_collection: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    document_count: i32,
}

#[derive(QueryableByName, Debug, Clone)]
struct ConsolidationRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    insight: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    summary: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    created_at: String,
}

fn get_accessible_kb_collections(
    conn: &mut diesel::PgConnection,
    bot_id: uuid::Uuid,
    accessible_ids: &[uuid::Uuid],
) -> Result<Vec<KbCollectionRow>, String> {
    if accessible_ids.is_empty() {
        return Ok(Vec::new());
    }

    let id_str: String = accessible_ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<_>>().join(",");

    let query_str = format!(
        "SELECT name, qdrant_collection, document_count FROM kb_collections WHERE bot_id = $1 AND id IN ({})",
        id_str
    );

    diesel::sql_query(&query_str)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .load::<KbCollectionRow>(conn)
        .map_err(|e| format!("Failed to query KB collections: {e}"))
}

fn get_recent_consolidations(
    conn: &mut diesel::PgConnection,
    bot_id: uuid::Uuid,
    limit: i64,
) -> Vec<ConsolidationRow> {
    diesel::sql_query(
        "SELECT insight, summary, source_ids, created_at FROM kb_consolidations
        WHERE bot_id = $1
        ORDER BY created_at DESC
        LIMIT $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<ConsolidationRow>(conn)
    .unwrap_or_default()
}

fn get_kb_memory_stats(
    conn: &mut diesel::PgConnection,
    bot_id: uuid::Uuid,
) -> serde_json::Value {
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_memories: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM drive_files WHERE bot_id = $1 AND file_type = 'kb' AND indexed = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<CountRow>(conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let total_consolidations: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM kb_consolidations WHERE bot_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<CountRow>(conn)
    .map(|r| r.count)
    .unwrap_or(0);

    let unconsolidated: i64 = diesel::sql_query(
        "SELECT COUNT(*) as count FROM drive_files WHERE bot_id = $1 AND file_type = 'kb' AND indexed = true
        AND NOT EXISTS (
            SELECT 1 FROM kb_consolidation_sources kcs
            JOIN kb_consolidations kc ON kc.id = kcs.consolidation_id
            WHERE kc.bot_id = $1 AND kcs.file_path = drive_files.file_path
        )",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<CountRow>(conn)
    .map(|r| r.count)
    .unwrap_or(0);

    json!({
        "total_memories": total_memories,
        "total_consolidations": total_consolidations,
        "unconsolidated": unconsolidated
    })
}

async fn search_qdrant_with_real_embeddings(
    collection_name: &str,
    query: &str,
    limit: usize,
    bot_id: uuid::Uuid,
    db_pool: &botbasic_types::types::DbPool,
) -> Result<Vec<serde_json::Value>, String> {
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| std::env::var("VECTORDB_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string()));
    let api_key = std::env::var("QDRANT_API_KEY")
        .unwrap_or_else(|_| std::env::var("VECTORDB_API_KEY").unwrap_or_default());

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let check_url = format!("{}/collections/{}", qdrant_url.trim_end_matches('/'), collection_name);
    let dim = match client.get(&check_url).header("api-key", &api_key).send().await {
        Ok(r) if r.status().is_success() => {
            let info: serde_json::Value = r.json().await.unwrap_or_default();
            info["result"]["config"]["params"]["vectors"]["size"].as_u64().map(|d| d as usize)
        }
        _ => None,
    };

    let dimensions = dim.unwrap_or(384);

    let vector = {
        let config = botcore::kb::EmbeddingConfig::from_bot_config(db_pool, &bot_id);
        if config.embedding_url.is_empty() {
            generate_hash_embedding(query, dimensions)
        } else {
            let generator = botcore::kb::KbEmbeddingGenerator::new(config);
            match generator.generate_single_embedding(query).await {
                Ok(embedding) => embedding.vector,
                Err(e) => {
                    warn!("THINK KB: embedding generation failed ({}), using hash fallback", e);
                    generate_hash_embedding(query, dimensions)
                }
            }
        }
    };

    let search_url = format!("{}/collections/{}/points/search", qdrant_url.trim_end_matches('/'), collection_name);
    let mut request = client
        .post(&search_url)
        .json(&serde_json::json!({
            "limit": limit,
            "with_vector": false,
            "with_payload": true,
            "params": { "hnsw_ef": 128, "exact": false },
            "vector": vector
        }));

    if !api_key.is_empty() {
        request = request.header("api-key", &api_key);
    }

    let response = request.send().await.map_err(|e| format!("Qdrant request failed: {e}"))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let result: serde_json::Value = response.json().await.map_err(|e| format!("Qdrant parse error: {e}"))?;

    let points = result["result"].as_array().cloned().unwrap_or_default();

    let mut results = Vec::new();
    for point in &points {
        let score = point.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        if score < 0.15 {
            continue;
        }

        let payload = point.get("payload");
        let content = payload
            .and_then(|p| p.get("content").or_else(|| p.get("text")).or_else(|| p.get("data")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let source = payload
            .and_then(|p| p.get("document_path").or_else(|| p.get("source")).or_else(|| p.get("file")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if content.is_empty() || content.len() < 10 {
            continue;
        }

        results.push(json!({
            "content": content,
            "source": source,
            "score": score,
        }));
    }

    Ok(results)
}

fn generate_hash_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut embedding = Vec::with_capacity(dimensions);
    for i in 0..dimensions {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        i.hash(&mut hasher);
        let hash = hasher.finish();
        embedding.push((hash as f32) / (u64::MAX as f32) * 2.0 - 1.0);
    }
    embedding
}

fn calculate_confidence(avg_score: f32, total_results: usize, consolidation_count: usize) -> f32 {
    let score_factor = avg_score;
    let result_factor = (total_results as f32 / 10.0).min(1.0) * 0.3;
    let consolidation_factor = (consolidation_count as f32 / 5.0).min(1.0) * 0.2;
    (score_factor * 0.5 + result_factor + consolidation_factor).min(1.0)
}

async fn think_kb_search(
    db_pool: &botbasic_types::types::DbPool,
    session_id: uuid::Uuid,
    bot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    query: &str,
) -> Result<serde_json::Value, String> {
    use botbasic_types::schema::bots;

    let (bot_name, kb_collections, consolidations, memory_stats) = {
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {e}"))?;

        let bot_name: String = bots::table
            .filter(bots::id.eq(bot_id))
            .select(bots::name)
            .first(&mut conn)
            .map_err(|e| format!("Failed to get bot name for id {bot_id}: {e}"))?;

        let accessible_ids = get_accessible_kb_ids(&mut conn, user_id)?;
        let kb_collections = get_accessible_kb_collections(&mut conn, bot_id, &accessible_ids)?;
        let consolidations = get_recent_consolidations(&mut conn, bot_id, 10);
        let memory_stats = get_kb_memory_stats(&mut conn, bot_id);

        (bot_name, kb_collections, consolidations, memory_stats)
    };

    info!(
        "THINK KB: bot_name={}, session={}, query='{}', {} KBs, {} consolidations",
        bot_name, session_id, query, kb_collections.len(), consolidations.len()
    );

    let mut all_results = Vec::new();
    let mut total_score = 0.0f32;
    let mut scored_count = 0usize;

    for kb in &kb_collections {
        match search_qdrant_with_real_embeddings(&kb.qdrant_collection, query, 10, bot_id, db_pool).await {
            Ok(results) if !results.is_empty() => {
                info!("THINK KB: {} results from KB '{}' ({})", results.len(), kb.name, kb.qdrant_collection);
                for r in &results {
                    if let Some(s) = r.get("score").and_then(|v| v.as_f64()) {
                        total_score += s as f32;
                        scored_count += 1;
                    }
                }
                all_results.push(json!({
                    "kb_name": kb.name,
                    "collection": kb.qdrant_collection,
                    "document_count": kb.document_count,
                    "results": results,
                }));
            }
            Ok(_) => {
                info!("THINK KB: no results from KB '{}' ({})", kb.name, kb.qdrant_collection);
            }
            Err(e) => {
                warn!("THINK KB: search failed for KB '{}': {}", kb.name, e);
            }
        }
    }

    let consolidation_insights: Vec<serde_json::Value> = consolidations.iter().map(|c| {
        json!({
            "insight": c.insight,
            "summary": c.summary,
            "created_at": c.created_at,
        })
    }).collect();

    let sources: Vec<serde_json::Value> = all_results.iter().flat_map(|kb| {
        kb["results"].as_array().unwrap_or(&Vec::new()).iter().filter_map(|r| {
            r.get("source").and_then(|s| s.as_str()).filter(|s| !s.is_empty()).map(|s| json!(s))
        }).collect::<Vec<_>>()
    }).take(20).collect();

    let avg_score = if scored_count > 0 { total_score / scored_count as f32 } else { 0.0 };
    let confidence = calculate_confidence(avg_score, all_results.len(), consolidation_insights.len());

    Ok(json!({
        "results": all_results,
        "consolidation_insights": consolidation_insights,
        "confidence": confidence,
        "total_results": all_results.iter().map(|kb| kb["results"].as_array().map(|a| a.len()).unwrap_or(0)).sum::<usize>(),
        "sources": sources,
        "query": query,
        "bot_name": bot_name,
        "memory_stats": memory_stats,
    }))
}

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

    #[test]
    fn test_confidence_calculation() {
        let confidence = calculate_confidence(0.8, 5, 3);
        assert!((0.0..=1.0).contains(&confidence));
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
