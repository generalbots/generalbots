use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use diesel::QueryableByName;
use diesel::RunQueryDsl;
use log::{error, info};
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
                Ok(rt) => rt.block_on(async {
                    think_kb_search(&db_pool, session_id, bot_id, user_id, &query_clone).await
                }),
                Err(e) => Err(format!("Failed to create runtime: {}", e)),
            }
        })
        .join();

        match result {
            Ok(Ok(search_result)) => {
                info!(
                    "THINK KB completed - Found {} results",
                    search_result.get("results")
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

async fn think_kb_search(
    db_pool: &botbasic_types::types::DbPool,
    session_id: uuid::Uuid,
    bot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    query: &str,
) -> Result<serde_json::Value, String> {
    use diesel::prelude::*;
use diesel::RunQueryDsl;
    use botbasic_types::schema::bots;

    let (bot_name, _accessible_kb_ids) = {
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {e}"))?;

        let bot_name: String = bots::table
            .filter(bots::id.eq(bot_id))
            .select(bots::name)
            .first(&mut conn)
            .map_err(|e| format!("Failed to get bot name for id {bot_id}: {e}"))?;

        let ids = get_accessible_kb_ids(&mut conn, user_id)?;

        (bot_name, ids)
    };

    info!(
        "THINK KB: bot_name={}, session={}, query='{}'",
        bot_name, session_id, query
    );

    Ok(json!({
        "results": [],
        "summary": "KB search requires vector database integration. Use 'USE KB <name>' to activate a knowledge base first.",
        "confidence": 0.0,
        "total_results": 0,
        "sources": [],
        "query": query,
        "bot_name": bot_name
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
