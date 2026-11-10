//! Read-only SQL execution against the bot's own database.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

pub fn execute_sql_query(
    conn: &mut PgConnection,
    sql: &str,
) -> Result<Vec<serde_json::Value>, String> {
    #[derive(QueryableByName, Debug)]
    struct JsonRow {
        #[diesel(sql_type = Text)]
        row_data: String,
    }

    let safe_sql = sql.trim().trim_end_matches(';');
    let upper = safe_sql.to_uppercase();
    if !upper.starts_with("SELECT") && !upper.starts_with("WITH") {
        return Err("Only SELECT queries are allowed".to_string());
    }
    if upper.contains("INSERT")
        || upper.contains("UPDATE")
        || upper.contains("DELETE")
        || upper.contains("DROP")
        || upper.contains("TRUNCATE")
        || upper.contains("ALTER")
        || upper.contains("CREATE")
    {
        return Err("Only SELECT queries are allowed".to_string());
    }

    let wrapped_sql =
        format!("SELECT row_to_json(t)::text as row_data FROM ({safe_sql}) t LIMIT 100");

    let rows: Vec<JsonRow> = sql_query(&wrapped_sql)
        .load(conn)
        .map_err(|e| format!("SQL execution error: {e}"))?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str(&r.row_data).ok())
        .collect();

    Ok(results)
}
