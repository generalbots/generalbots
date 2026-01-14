//! SEARCH keyword implementation for full-text search with autocomplete support
//!
//! Provides fast product search across tables using PostgreSQL full-text search
//! and trigram similarity for fuzzy matching and autocomplete.
//!
//! Usage in .bas files:
//!   results = SEARCH "Products.csv", "chocolate", 10
//!   results = SEARCH "products", query, limit

use super::table_access::{check_table_access, filter_fields_by_role, AccessType, UserRoles};
use crate::security::sql_guard::sanitize_identifier;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use crate::shared::utils;
use crate::shared::utils::to_array;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use log::{error, trace, warn};
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    row_data: String,
}

#[derive(QueryableByName)]
struct SearchResultRow {
    #[diesel(sql_type = Text)]
    row_data: String,
    #[diesel(sql_type = diesel::sql_types::Float)]
    relevance: f32,
}

/// Registers the SEARCH keyword with the Rhai engine
///
/// Syntax: SEARCH "table_name", "query", limit
pub fn search_keyword(state: &AppState, user: UserSession, engine: &mut Engine) {
    let connection = state.conn.clone();
    let user_roles = UserRoles::from_user_session(&user);

    // SEARCH table, query, limit
    engine
        .register_custom_syntax(
            ["SEARCH", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            {
                let conn = connection.clone();
                let roles = user_roles.clone();
                move |context, inputs| {
                    let table_name = context.eval_expression_tree(&inputs[0])?;
                    let query = context.eval_expression_tree(&inputs[1])?;
                    let limit = context.eval_expression_tree(&inputs[2])?;

                    let mut binding = conn.get().map_err(|e| format!("DB error: {e}"))?;
                    let table_str = table_name.to_string();
                    let query_str = query.to_string();
                    let limit_val = limit.as_int().unwrap_or(10) as i32;

                    let access_info = match check_table_access(
                        &mut binding,
                        &table_str,
                        &roles,
                        AccessType::Read,
                    ) {
                        Ok(info) => info,
                        Err(e) => {
                            warn!("SEARCH access denied: {e}");
                            return Err(e.into());
                        }
                    };

                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            execute_search(&mut binding, &table_str, &query_str, limit_val)
                        })
                    })
                    .map_err(|e| format!("Search error: {e}"))?;

                    if let Some(results) = result.get("results") {
                        let filtered =
                            filter_fields_by_role(results.clone(), &roles, &access_info);
                        let array = to_array(utils::json_value_to_dynamic(&filtered));
                        Ok(Dynamic::from(array))
                    } else {
                        Ok(Dynamic::from(rhai::Array::new()))
                    }
                }
            },
        )
        .expect("valid syntax registration");

    // SEARCH table, query (default limit = 10)
    engine
        .register_custom_syntax(["SEARCH", "$expr$", ",", "$expr$"], false, {
            let conn = connection.clone();
            let roles = user_roles.clone();
            move |context, inputs| {
                let table_name = context.eval_expression_tree(&inputs[0])?;
                let query = context.eval_expression_tree(&inputs[1])?;

                let mut binding = conn.get().map_err(|e| format!("DB error: {e}"))?;
                let table_str = table_name.to_string();
                let query_str = query.to_string();

                let access_info = match check_table_access(
                    &mut binding,
                    &table_str,
                    &roles,
                    AccessType::Read,
                ) {
                    Ok(info) => info,
                    Err(e) => {
                        warn!("SEARCH access denied: {e}");
                        return Err(e.into());
                    }
                };

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { execute_search(&mut binding, &table_str, &query_str, 10) })
                })
                .map_err(|e| format!("Search error: {e}"))?;

                if let Some(results) = result.get("results") {
                    let filtered = filter_fields_by_role(results.clone(), &roles, &access_info);
                    let array = to_array(utils::json_value_to_dynamic(&filtered));
                    Ok(Dynamic::from(array))
                } else {
                    Ok(Dynamic::from(rhai::Array::new()))
                }
            }
        })
        .expect("valid syntax registration");

    // Register AUTOCOMPLETE function for quick suggestions
    let conn_autocomplete = connection.clone();
    engine.register_fn(
        "AUTOCOMPLETE",
        move |table: String, prefix: String, limit: i64| -> rhai::Array {
            let mut binding = match conn_autocomplete.get() {
                Ok(c) => c,
                Err(_) => return rhai::Array::new(),
            };

            match execute_autocomplete(&mut binding, &table, &prefix, limit as i32) {
                Ok(suggestions) => suggestions
                    .into_iter()
                    .map(Dynamic::from)
                    .collect(),
                Err(_) => rhai::Array::new(),
            }
        },
    );

    // Register lowercase version
    let conn_autocomplete2 = connection.clone();
    engine.register_fn(
        "autocomplete",
        move |table: String, prefix: String, limit: i64| -> rhai::Array {
            let mut binding = match conn_autocomplete2.get() {
                Ok(c) => c,
                Err(_) => return rhai::Array::new(),
            };

            match execute_autocomplete(&mut binding, &table, &prefix, limit as i32) {
                Ok(suggestions) => suggestions
                    .into_iter()
                    .map(Dynamic::from)
                    .collect(),
                Err(_) => rhai::Array::new(),
            }
        },
    );
}

/// Execute full-text search with relevance ranking
pub fn execute_search(
    conn: &mut PgConnection,
    table_str: &str,
    query_str: &str,
    limit: i32,
) -> Result<Value, String> {
    trace!("Starting execute_search: table={table_str}, query={query_str}, limit={limit}");

    let safe_table = sanitize_identifier(table_str);
    let safe_query = query_str.replace('\'', "''").replace('\\', "\\\\");

    // Get searchable columns from the table
    let searchable_columns = get_searchable_columns(conn, &safe_table)?;

    if searchable_columns.is_empty() {
        warn!("No searchable columns found for table: {safe_table}");
        return Ok(json!({
            "command": "search",
            "table": table_str,
            "query": query_str,
            "results": [],
            "count": 0
        }));
    }

    // Build search expression combining multiple columns
    let search_columns: Vec<String> = searchable_columns
        .iter()
        .map(|col| format!("COALESCE({}::text, '')", col))
        .collect();

    let combined_columns = search_columns.join(" || ' ' || ");

    // Use trigram similarity for fuzzy matching + ILIKE for direct matches
    let query = format!(
        r#"
        SELECT
            row_to_json(t)::text as row_data,
            GREATEST(
                similarity({combined_columns}, $1),
                CASE WHEN {combined_columns} ILIKE '%' || $1 || '%' THEN 0.5 ELSE 0 END
            ) as relevance
        FROM {safe_table} t
        WHERE
            {combined_columns} ILIKE '%' || $1 || '%'
            OR similarity({combined_columns}, $1) > 0.1
        ORDER BY relevance DESC, id
        LIMIT $2
        "#,
        combined_columns = combined_columns,
        safe_table = safe_table
    );

    // Try with trigram extension, fall back to simple ILIKE if not available
    let raw_results: Vec<SearchResultRow> = match diesel::sql_query(&query)
        .bind::<Text, _>(&safe_query)
        .bind::<Integer, _>(limit)
        .load(conn)
    {
        Ok(results) => results,
        Err(e) => {
            trace!("Trigram search failed, falling back to ILIKE: {e}");
            // Fallback to simple ILIKE search
            return execute_simple_search(conn, &safe_table, &safe_query, limit, &searchable_columns);
        }
    };

    let results: Vec<Value> = raw_results
        .into_iter()
        .filter_map(|row| {
            let mut obj: Value = serde_json::from_str(&row.row_data).ok()?;
            if let Value::Object(ref mut map) = obj {
                map.insert("_relevance".to_string(), json!(row.relevance));
            }
            Some(obj)
        })
        .collect();

    trace!("Search returned {} results", results.len());

    Ok(json!({
        "command": "search",
        "table": table_str,
        "query": query_str,
        "results": results,
        "count": results.len()
    }))
}

/// Fallback simple search using ILIKE
fn execute_simple_search(
    conn: &mut PgConnection,
    safe_table: &str,
    safe_query: &str,
    limit: i32,
    searchable_columns: &[String],
) -> Result<Value, String> {
    let search_columns: Vec<String> = searchable_columns
        .iter()
        .map(|col| format!("COALESCE({}::text, '')", col))
        .collect();

    let combined_columns = search_columns.join(" || ' ' || ");

    let query = format!(
        r#"
        SELECT row_to_json(t)::text as row_data
        FROM {safe_table} t
        WHERE {combined_columns} ILIKE '%' || $1 || '%'
        LIMIT $2
        "#,
        safe_table = safe_table,
        combined_columns = combined_columns
    );

    let raw_results: Vec<JsonRow> = diesel::sql_query(&query)
        .bind::<Text, _>(safe_query)
        .bind::<Integer, _>(limit)
        .load(conn)
        .map_err(|e| {
            error!("Simple search error: {e}");
            e.to_string()
        })?;

    let results: Vec<Value> = raw_results
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.row_data).ok())
        .collect();

    Ok(json!({
        "command": "search",
        "table": safe_table,
        "query": safe_query,
        "results": results,
        "count": results.len()
    }))
}

/// Execute autocomplete query for quick suggestions
pub fn execute_autocomplete(
    conn: &mut PgConnection,
    table_str: &str,
    prefix: &str,
    limit: i32,
) -> Result<Vec<String>, String> {
    trace!("Autocomplete: table={table_str}, prefix={prefix}");

    let safe_table = sanitize_identifier(table_str);
    let safe_prefix = prefix.replace('\'', "''").replace('\\', "\\\\");

    // Find the primary text column (name, title, or first text column)
    let text_column = get_primary_text_column(conn, &safe_table)?;

    let query = format!(
        r#"
        SELECT DISTINCT {text_column}::text as suggestion
        FROM {safe_table}
        WHERE {text_column}::text ILIKE $1 || '%'
        ORDER BY {text_column}
        LIMIT $2
        "#,
        text_column = text_column,
        safe_table = safe_table
    );

    #[derive(QueryableByName)]
    struct SuggestionRow {
        #[diesel(sql_type = Text)]
        suggestion: String,
    }

    let results: Vec<SuggestionRow> = diesel::sql_query(&query)
        .bind::<Text, _>(&safe_prefix)
        .bind::<Integer, _>(limit)
        .load(conn)
        .map_err(|e| {
            error!("Autocomplete error: {e}");
            e.to_string()
        })?;

    Ok(results.into_iter().map(|r| r.suggestion).collect())
}

/// Get list of text/searchable columns from a table
fn get_searchable_columns(conn: &mut PgConnection, table_name: &str) -> Result<Vec<String>, String> {
    #[derive(QueryableByName)]
    struct ColumnInfo {
        #[diesel(sql_type = Text)]
        column_name: String,
    }

    let query = r#"
        SELECT column_name::text
        FROM information_schema.columns
        WHERE table_name = $1
        AND data_type IN ('character varying', 'varchar', 'text', 'character', 'char', 'name')
        AND column_name NOT LIKE '%password%'
        AND column_name NOT LIKE '%secret%'
        AND column_name NOT LIKE '%token%'
        ORDER BY ordinal_position
    "#;

    let columns: Vec<ColumnInfo> = diesel::sql_query(query)
        .bind::<Text, _>(table_name)
        .load(conn)
        .map_err(|e| e.to_string())?;

    // Prioritize common search columns
    let priority_columns = ["name", "title", "description", "sku", "product_name", "productname"];
    let mut result: Vec<String> = Vec::new();

    // Add priority columns first
    for col in &priority_columns {
        if columns.iter().any(|c| c.column_name.to_lowercase() == *col) {
            result.push(col.to_string());
        }
    }

    // Add remaining columns
    for col in columns {
        let col_lower = col.column_name.to_lowercase();
        if !result.contains(&col_lower) && !result.contains(&col.column_name) {
            result.push(col.column_name);
        }
    }

    // Limit to first 5 searchable columns for performance
    result.truncate(5);

    Ok(result)
}

/// Get the primary text column for autocomplete (usually name or title)
fn get_primary_text_column(conn: &mut PgConnection, table_name: &str) -> Result<String, String> {
    let columns = get_searchable_columns(conn, table_name)?;

    // Prefer specific column names
    let preferred = ["name", "title", "productname", "product_name", "label", "sku"];

    for pref in &preferred {
        if columns.iter().any(|c| c.to_lowercase() == *pref) {
            return Ok(pref.to_string());
        }
    }

    // Return first available text column
    columns
        .into_iter()
        .next()
        .ok_or_else(|| "No text columns found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_search_query() {
        let query = "test' OR '1'='1";
        let safe = query.replace('\'', "''").replace('\\', "\\\\");
        assert!(!safe.contains("' OR '"));
    }
}
