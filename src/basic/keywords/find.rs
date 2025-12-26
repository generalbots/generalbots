use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use crate::shared::utils;
use crate::shared::utils::to_array;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use log::error;
use log::trace;
use rhai::Dynamic;
use rhai::Engine;
use serde_json::{json, Value};
pub fn find_keyword(state: &AppState, _user: UserSession, engine: &mut Engine) {
    let connection = state.conn.clone();
    engine
        .register_custom_syntax(["FIND", "$expr$", ",", "$expr$"], false, {
            move |context, inputs| {
                let table_name = context.eval_expression_tree(&inputs[0])?;
                let filter = context.eval_expression_tree(&inputs[1])?;
                let mut binding = connection.get().map_err(|e| format!("DB error: {}", e))?;
                let binding2 = table_name.to_string();
                let binding3 = filter.to_string();
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { execute_find(&mut binding, &binding2, &binding3) })
                })
                .map_err(|e| format!("DB error: {}", e))?;
                if let Some(results) = result.get("results") {
                    let array = to_array(utils::json_value_to_dynamic(results));
                    Ok(Dynamic::from(array))
                } else {
                    Err("No results".into())
                }
            }
        })
        .unwrap();
}
pub fn execute_find(
    conn: &mut PgConnection,
    table_str: &str,
    filter_str: &str,
) -> Result<Value, String> {
    trace!(
        "Starting execute_find with table: {}, filter: {}",
        table_str,
        filter_str
    );
    let (where_clause, params) = utils::parse_filter(filter_str).map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT * FROM {} WHERE {} LIMIT 10",
        table_str, where_clause
    );
    let _raw_result = diesel::sql_query(&query)
        .bind::<diesel::sql_types::Text, _>(&params[0])
        .execute(conn)
        .map_err(|e| {
            error!("SQL execution error: {}", e);
            e.to_string()
        })?;
    let mut results = Vec::new();
    let json_row = serde_json::json!({
    "note": "Dynamic row deserialization not implemented - need table schema"
    });
    results.push(json_row);
    Ok(json!({
    "command": "find",
    "table": table_str,
    "filter": filter_str,
    "results": results
    }))
}
