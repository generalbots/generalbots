use super::table_access::{check_table_access, AccessType, UserRoles};
use crate::core::shared::{sanitize_identifier, sanitize_sql_value};
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use crate::core::shared::utils::{convert_date_to_iso_format, json_value_to_dynamic, to_array};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, trace, warn};
use rhai::{Array, Dynamic, Engine, Map};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub fn register_data_operations(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_save_keyword(state.clone(), user.clone(), engine);
    register_insert_keyword(state.clone(), user.clone(), engine);
    register_update_keyword(state.clone(), user.clone(), engine);
    register_delete_keyword(state.clone(), user.clone(), engine);
    register_merge_keyword(Arc::clone(&state), user.clone(), engine);
    register_fill_keyword(Arc::clone(&state), user.clone(), engine);
    register_map_keyword(Arc::clone(&state), user.clone(), engine);
    register_filter_keyword(Arc::clone(&state), user.clone(), engine);
    register_aggregate_keyword(Arc::clone(&state), user.clone(), engine);
    register_join_keyword(Arc::clone(&state), user.clone(), engine);
    register_pivot_keyword(Arc::clone(&state), user.clone(), engine);
    register_group_by_keyword(state, user, engine);
}

pub fn register_save_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_roles = UserRoles::from_user_session(&user);

    // SAVE with variable arguments: SAVE "table", id, field1, field2, ...
    // Each pattern: table + id + (1 to 64 fields)
    // Minimum: table + id + 1 field = 4 expressions total
    register_save_variants(&state, user_roles, engine);
}

fn register_save_variants(state: &Arc<AppState>, user_roles: UserRoles, engine: &mut Engine) {
    // Register positional saves FIRST (in descending order), so longer patterns
    // are tried before shorter ones. This ensures that SAVE with 22 fields matches
    // the 22-field pattern, not the 3-field structured save pattern.
    // Pattern: SAVE + table + (field1 + field2 + ... + fieldN)
    // Total elements = 2 (SAVE + table) + num_fields * 2 (comma + expr)
    // For 22 fields: 2 + 22*2 = 46 elements

    // Register in descending order (70 down to 2) so longer patterns override shorter ones
    for num_fields in (2..=70).rev() {
        let mut pattern = vec!["SAVE", "$expr$"];
        for _ in 0..num_fields {
            pattern.push(",");
            pattern.push("$expr$");
        }

        // Log pattern registration for key values
        if num_fields == 22 || num_fields == 21 || num_fields == 23 {
            log::info!("Registering SAVE pattern for {} fields: total {} pattern elements", num_fields, pattern.len());
        }

        let state_clone = Arc::clone(state);
        let user_roles_clone = user_roles.clone();
        let field_count = num_fields;

        engine
            .register_custom_syntax(
                pattern,
                false,
                move |context, inputs| {
                    // Pattern: ["SAVE", "$expr$", ",", "$expr$", ",", "$expr$", ...]
                    // inputs[0] = table, inputs[2], inputs[4], inputs[6], ... = field values
                    // Commas are at inputs[1], inputs[3], inputs[5], ...
                    let table = context.eval_expression_tree(&inputs[0])?.to_string();

                    trace!("SAVE positional: table={}, fields={}", table, field_count);

                    let mut conn = state_clone
                        .conn
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?;

                    if let Err(e) =
                        check_table_access(&mut conn, &table, &user_roles_clone, AccessType::Write)
                    {
                        warn!("SAVE access denied: {}", e);
                        return Err(e.into());
                    }

                    // Get column names from database schema
                    let column_names = crate::basic::keywords::table_access::get_table_columns(&mut conn, &table);

                    // Build data map from positional field values
                    let mut data_map: Map = Map::new();

                    // Field values are at inputs[2], inputs[4], inputs[6], ... (every other element starting from 2)
                    for i in 0..field_count {
                        if i < column_names.len() {
                            let value_expr = &inputs[i * 2 + 2]; // 2, 4, 6, 8, ...
                            let value = context.eval_expression_tree(value_expr)?;
                            data_map.insert(column_names[i].clone().into(), value);
                        }
                    }

                    let data = Dynamic::from(data_map);

                    // No ID parameter - use execute_insert instead
                    let result = execute_insert(&mut conn, &table, &data)
                        .map_err(|e| format!("SAVE error: {}", e))?;

                    Ok(json_value_to_dynamic(&result))
                },
            )
            .expect("valid syntax registration");
    }

    // Register structured save LAST (after all positional saves)
    // This ensures that SAVE statements with many fields use positional patterns,
    // and only SAVE statements with exactly 3 expressions use the structured pattern
    {
        let state_clone = Arc::clone(state);
        let user_roles_clone = user_roles.clone();
        engine
            .register_custom_syntax(
                ["SAVE", "$expr$", ",", "$expr$", ",", "$expr$"],
                false,
                move |context, inputs| {
                    let table = context.eval_expression_tree(&inputs[0])?.to_string();
                    let id = context.eval_expression_tree(&inputs[1])?;
                    let data = context.eval_expression_tree(&inputs[2])?;

                    trace!("SAVE structured: table={}, id={:?}", table, id);

                    let mut conn = state_clone
                        .conn
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?;

                    if let Err(e) =
                        check_table_access(&mut conn, &table, &user_roles_clone, AccessType::Write)
                    {
                        warn!("SAVE access denied: {}", e);
                        return Err(e.into());
                    }

                    let result = execute_save(&mut conn, &table, &id, &data)
                        .map_err(|e| format!("SAVE error: {}", e))?;

                    Ok(json_value_to_dynamic(&result))
                },
            )
            .expect("valid syntax registration");
    }
}

pub fn register_insert_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();
    let user_roles = UserRoles::from_user_session(&user);

    engine
        .register_custom_syntax(
            ["INSERT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("INSERT into table: {}", table);

                // Get bot's database connection instead of main connection
                let bot_pool = state_clone
                    .bot_database_manager
                    .get_bot_pool(user_clone.bot_id);

                let mut conn = match bot_pool {
                    Ok(pool) => pool.get().map_err(|e| format!("Bot DB error: {}", e))?,
                    Err(_) => state_clone
                        .conn
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?,
                };

                // Check write access
                if let Err(e) =
                    check_table_access(&mut conn, &table, &user_roles, AccessType::Write)
                {
                    warn!("INSERT access denied: {}", e);
                    return Err(e.into());
                }

                let result = execute_insert(&mut conn, &table, &data)
                    .map_err(|e| format!("INSERT error: {}", e))?;

                Ok(json_value_to_dynamic(&result))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_update_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_roles = UserRoles::from_user_session(&user);

    engine
        .register_custom_syntax(
            ["UPDATE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let filter = context.eval_expression_tree(&inputs[1])?.to_string();
                let data = context.eval_expression_tree(&inputs[2])?;

                trace!("UPDATE table: {}, filter: {}", table, filter);

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                // Check write access
                if let Err(e) =
                    check_table_access(&mut conn, &table, &user_roles, AccessType::Write)
                {
                    warn!("UPDATE access denied: {}", e);
                    return Err(e.into());
                }

                let result = execute_update(&mut conn, &table, &filter, &data)
                    .map_err(|e| format!("UPDATE error: {}", e))?;

                Ok(Dynamic::from(result))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_delete_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_roles = UserRoles::from_user_session(&user);

    engine
        .register_custom_syntax(
            ["DELETE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let first_arg = context.eval_expression_tree(&inputs[0])?.to_string();
                let second_arg = context.eval_expression_tree(&inputs[1])?.to_string();

                if first_arg.starts_with("http://") || first_arg.starts_with("https://") {
                    trace!("DELETE HTTP with data: {}", first_arg);

                    let (tx, rx) = std::sync::mpsc::channel();
                    let url_clone = first_arg;

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_multi_thread()
                            .worker_threads(2)
                            .enable_all()
                            .build();

                        let _ = if let Ok(rt) = rt {
                            let result = rt.block_on(async move {
                                let client = reqwest::Client::new();
                                client
                                    .delete(&url_clone)
                                    .timeout(std::time::Duration::from_secs(60))
                                    .send()
                                    .await
                                    .map_err(|e| format!("HTTP error: {}", e))?
                                    .text()
                                    .await
                                    .map_err(|e| format!("Response error: {}", e))
                            });
                            tx.send(result)
                        } else {
                            tx.send(Err("Failed to build runtime".to_string()))
                        };
                    });

                    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                        Ok(Ok(response)) => Ok(Dynamic::from(response)),
                        Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            format!("DELETE failed: {}", e).into(),
                            rhai::Position::NONE,
                        ))),
                        Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE timed out".into(),
                            rhai::Position::NONE,
                        ))),
                    }
                } else {
                    trace!("DELETE from table: {}, filter: {}", first_arg, second_arg);

                    let mut conn = state_clone
                        .conn
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?;

                    // Check write access (delete requires write permission)
                    if let Err(e) =
                        check_table_access(&mut conn, &first_arg, &user_roles, AccessType::Write)
                    {
                        warn!("DELETE access denied: {}", e);
                        return Err(e.into());
                    }

                    let result = execute_delete(&mut conn, &first_arg, &second_arg)
                        .map_err(|e| format!("DELETE error: {}", e))?;

                    Ok(Dynamic::from(result))
                }
            },
        )
        .expect("valid syntax registration");

    let state_clone2 = Arc::clone(&state);
    engine
        .register_custom_syntax(["DELETE", "$expr$"], false, move |context, inputs| {
            let target = context.eval_expression_tree(&inputs[0])?.to_string();

            if target.starts_with("http://") || target.starts_with("https://") {
                trace!("DELETE HTTP: {}", target);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = target;

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let _ = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            let client = reqwest::Client::new();
                            client
                                .delete(&url_clone)
                                .timeout(std::time::Duration::from_secs(60))
                                .send()
                                .await
                                .map_err(|e| format!("HTTP error: {}", e))?
                                .text()
                                .await
                                .map_err(|e| format!("Response error: {}", e))
                        });
                        tx.send(result)
                    } else {
                        tx.send(Err("Failed to build runtime".to_string()))
                    };
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(Dynamic::from(response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "DELETE timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            } else {
                trace!("DELETE file: {}", target);

                let _state = Arc::clone(&state_clone2);

                let file_path = std::path::Path::new(&target);
                if file_path.exists() {
                    std::fs::remove_file(file_path)
                        .map_err(|e| format!("File delete error: {}", e))?;
                    Ok(Dynamic::from(true))
                } else {
                    Ok(Dynamic::from(format!("File not found: {}", target)))
                }
            }
        })
        .expect("valid syntax registration");
}

pub fn register_merge_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["MERGE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;
                let key_field = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("MERGE into table: {}, key: {}", table, key_field);

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let result = execute_merge(&mut conn, &table, &data, &key_field)
                    .map_err(|e| format!("MERGE error: {}", e))?;

                Ok(json_value_to_dynamic(&result))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_fill_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["FILL", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let template = context.eval_expression_tree(&inputs[1])?;

                trace!("FILL with template");

                let result = execute_fill(&data, &template)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_map_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["MAP", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let mapping = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("MAP with mapping: {}", mapping);

                let result = execute_map(&data, &mapping)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_filter_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["FILTER", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let condition = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("FILTER with condition: {}", condition);

                let result = execute_filter(&data, &condition)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_aggregate_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["AGGREGATE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let operation = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;
                let field = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("AGGREGATE {} on field: {}", operation, field);

                let result = execute_aggregate(&operation, &data, &field)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_join_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["JOIN", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let left = context.eval_expression_tree(&inputs[0])?;
                let right = context.eval_expression_tree(&inputs[1])?;
                let key = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("JOIN on key: {}", key);

                let result = execute_join(&left, &right, &key)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_pivot_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["PIVOT", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let row_field = context.eval_expression_tree(&inputs[1])?.to_string();
                let value_field = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("PIVOT on row: {}, value: {}", row_field, value_field);

                let result = execute_pivot(&data, &row_field, &value_field)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

pub fn register_group_by_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["GROUP_BY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let field = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("GROUP_BY field: {}", field);

                let result = execute_group_by(&data, &field)?;

                Ok(result)
            },
        )
        .expect("valid syntax registration");
}

fn execute_save(
    conn: &mut diesel::PgConnection,
    table: &str,
    id: &Dynamic,
    data: &Dynamic,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let data_map = dynamic_to_map(data);
    let id_value = id.to_string();

    let mut columns: Vec<String> = vec!["id".to_string()];
    let mut values: Vec<String> = vec![format!("'{}'", sanitize_sql_value(&id_value))];
    let mut update_sets: Vec<String> = Vec::new();

    for (key, value) in &data_map {
        let sanitized_key = sanitize_identifier(key);
        let value_str = value.to_string();
        let converted_value = convert_date_to_iso_format(&value_str);
        let sanitized_value = format!("'{}'", sanitize_sql_value(&converted_value));
        columns.push(sanitized_key.clone());
        values.push(sanitized_value.clone());
        update_sets.push(format!("{} = {}", sanitized_key, sanitized_value));
    }

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT (id) DO UPDATE SET {}",
        sanitize_identifier(table),
        columns.join(", "),
        values.join(", "),
        update_sets.join(", ")
    );

    trace!("Executing SAVE query: {}", query);

    let result = sql_query(&query).execute(conn).map_err(|e| {
        error!("SAVE SQL error: {}", e);
        e.to_string()
    })?;

    Ok(json!({
        "command": "save",
        "table": table,
        "id": id_value,
        "rows_affected": result
    }))
}

fn execute_insert(
    conn: &mut diesel::PgConnection,
    table: &str,
    data: &Dynamic,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let data_map = dynamic_to_map(data);

    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    for (key, value) in &data_map {
        columns.push(sanitize_identifier(key));
        let value_str = value.to_string();
        let converted_value = convert_date_to_iso_format(&value_str);
        values.push(format!("'{}'", sanitize_sql_value(&converted_value)));
    }

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
        sanitize_identifier(table),
        columns.join(", "),
        values.join(", ")
    );

    trace!("Executing INSERT query: {}", query);

    #[derive(QueryableByName)]
    struct InsertResult {
        #[diesel(sql_type = Text)]
        id: String,
    }

    let result: Result<Vec<InsertResult>, _> = sql_query(&query).load(conn);

    match result {
        Ok(rows) => {
            let id = rows.first().map(|r| r.id.clone()).unwrap_or_default();
            Ok(json!({
                "command": "insert",
                "table": table,
                "id": id,
                "success": true
            }))
        }
        Err(e) => {
            error!("INSERT SQL error: {}", e);
            Ok(json!({
                "command": "insert",
                "table": table,
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

fn execute_update(
    conn: &mut diesel::PgConnection,
    table: &str,
    filter: &str,
    data: &Dynamic,
) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let data_map = dynamic_to_map(data);
    let where_clause = parse_filter_clause(filter)?;

    let mut update_sets: Vec<String> = Vec::new();
    for (key, value) in &data_map {
        let value_str = value.to_string();
        let converted_value = convert_date_to_iso_format(&value_str);
        update_sets.push(format!(
            "{} = '{}'",
            sanitize_identifier(key),
            sanitize_sql_value(&converted_value)
        ));
    }

    let query = format!(
        "UPDATE {} SET {} WHERE {}",
        sanitize_identifier(table),
        update_sets.join(", "),
        where_clause
    );

    trace!("Executing UPDATE query: {}", query);

    let result = sql_query(&query).execute(conn).map_err(|e| {
        error!("UPDATE SQL error: {}", e);
        e.to_string()
    })?;

    Ok(result as i64)
}

fn execute_delete(
    conn: &mut diesel::PgConnection,
    table: &str,
    filter: &str,
) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let where_clause = parse_filter_clause(filter)?;

    let query = format!(
        "DELETE FROM {} WHERE {}",
        sanitize_identifier(table),
        where_clause
    );

    trace!("Executing DELETE query: {}", query);

    let result = sql_query(&query).execute(conn).map_err(|e| {
        error!("DELETE SQL error: {}", e);
        e.to_string()
    })?;

    Ok(result as i64)
}

fn execute_merge(
    conn: &mut diesel::PgConnection,
    table: &str,
    data: &Dynamic,
    key_field: &str,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let array = to_array(data.clone());
    let mut inserted = 0;
    let mut updated = 0;

    for item in array {
        let item_map = dynamic_to_map(&item);

        let key_value = item_map
            .get(key_field)
            .map(|v| v.to_string())
            .unwrap_or_default();

        if key_value.is_empty() {
            continue;
        }

        let check_query = format!(
            "SELECT COUNT(*) as cnt FROM {} WHERE {} = '{}'",
            sanitize_identifier(table),
            sanitize_identifier(key_field),
            sanitize_sql_value(&key_value)
        );

        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            cnt: i64,
        }

        let count_result: Result<Vec<CountResult>, _> = sql_query(&check_query).load(conn);
        let exists = count_result
            .map(|r| r.first().map(|c| c.cnt > 0).unwrap_or(false))
            .unwrap_or(false);

        if exists {
            let mut update_sets: Vec<String> = Vec::new();
            for (key, value) in &item_map {
                if key != key_field {
                    update_sets.push(format!(
                        "{} = '{}'",
                        sanitize_identifier(key),
                        sanitize_sql_value(&value.to_string())
                    ));
                }
            }

            if !update_sets.is_empty() {
                let update_query = format!(
                    "UPDATE {} SET {} WHERE {} = '{}'",
                    sanitize_identifier(table),
                    update_sets.join(", "),
                    sanitize_identifier(key_field),
                    sanitize_sql_value(&key_value)
                );
                let _ = sql_query(&update_query).execute(conn);
                updated += 1;
            }
        } else {
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<String> = Vec::new();

            for (key, value) in &item_map {
                columns.push(sanitize_identifier(key));
                values.push(format!("'{}'", sanitize_sql_value(&value.to_string())));
            }

            let insert_query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                sanitize_identifier(table),
                columns.join(", "),
                values.join(", ")
            );
            let _ = sql_query(&insert_query).execute(conn);
            inserted += 1;
        }
    }

    Ok(json!({
        "command": "merge",
        "table": table,
        "key_field": key_field,
        "inserted": inserted,
        "updated": updated
    }))
}

fn execute_fill(data: &Dynamic, template: &Dynamic) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let template_map = dynamic_to_map(template);
    let array = to_array(data.clone());
    let mut results: Array = Array::new();

    for item in array {
        let item_map = dynamic_to_map(&item);
        let mut result_map: Map = Map::new();

        for (template_key, template_value) in &template_map {
            let template_str = template_value.to_string();

            let mut filled_value = template_str.clone();
            for (data_key, data_value) in &item_map {
                let placeholder = format!("{{{{{}}}}}", data_key);
                filled_value = filled_value.replace(&placeholder, &data_value.to_string());
            }

            result_map.insert(template_key.clone().into(), Dynamic::from(filled_value));
        }

        results.push(Dynamic::from(result_map));
    }

    Ok(Dynamic::from(results))
}

fn execute_map(data: &Dynamic, mapping: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let mappings: HashMap<String, String> = mapping
        .split(',')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.split("->").collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect();

    let array = to_array(data.clone());
    let mut results: Array = Array::new();

    for item in array {
        let item_map = dynamic_to_map(&item);
        let mut result_map: Map = Map::new();

        for (old_key, value) in &item_map {
            let new_key = mappings.get(old_key).unwrap_or(old_key);
            result_map.insert(new_key.clone().into(), value.clone());
        }

        results.push(Dynamic::from(result_map));
    }

    Ok(Dynamic::from(results))
}

fn execute_filter(data: &Dynamic, condition: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let (field, operator, value) = parse_condition(condition)?;
    let array = to_array(data.clone());
    let mut results: Array = Array::new();

    for item in array {
        let item_map = dynamic_to_map(&item);

        if let Some(field_value) = item_map.get(&field) {
            let matches = match operator.as_str() {
                "=" | "==" => field_value.to_string() == value,
                "!=" | "<>" => field_value.to_string() != value,
                ">" => {
                    field_value.to_string().parse::<f64>().unwrap_or(0.0)
                        > value.parse::<f64>().unwrap_or(0.0)
                }
                "<" => {
                    field_value.to_string().parse::<f64>().unwrap_or(0.0)
                        < value.parse::<f64>().unwrap_or(0.0)
                }
                ">=" => {
                    field_value.to_string().parse::<f64>().unwrap_or(0.0)
                        >= value.parse::<f64>().unwrap_or(0.0)
                }
                "<=" => {
                    field_value.to_string().parse::<f64>().unwrap_or(0.0)
                        <= value.parse::<f64>().unwrap_or(0.0)
                }
                "like" | "LIKE" => field_value
                    .to_string()
                    .to_lowercase()
                    .contains(&value.to_lowercase()),
                _ => false,
            };

            if matches {
                results.push(item.clone());
            }
        }
    }

    Ok(Dynamic::from(results))
}

fn execute_aggregate(
    operation: &str,
    data: &Dynamic,
    field: &str,
) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let array = to_array(data.clone());
    let mut values: Vec<f64> = Vec::new();

    for item in array {
        let item_map = dynamic_to_map(&item);
        if let Some(field_value) = item_map.get(field) {
            if let Ok(num) = field_value.to_string().parse::<f64>() {
                values.push(num);
            }
        }
    }

    let result = match operation.to_uppercase().as_str() {
        "SUM" => values.iter().sum::<f64>(),
        "AVG" | "AVERAGE" => {
            if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            }
        }
        "COUNT" => values.len() as f64,
        "MIN" => values.iter().copied().fold(f64::INFINITY, f64::min),
        "MAX" => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        _ => {
            return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                format!("Unknown aggregate operation: {}", operation).into(),
                rhai::Position::NONE,
            )));
        }
    };

    Ok(Dynamic::from(result))
}

fn execute_join(
    left: &Dynamic,
    right: &Dynamic,
    key: &str,
) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let left_array = to_array(left.clone());
    let right_array = to_array(right.clone());
    let mut results: Array = Array::new();

    let mut right_index: HashMap<String, Vec<Map>> = HashMap::new();
    for item in &right_array {
        let item_map = dynamic_to_map(item);
        if let Some(key_value) = item_map.get(key) {
            let key_str = key_value.to_string();
            right_index
                .entry(key_str)
                .or_default()
                .push(dynamic_to_rhai_map(item));
        }
    }

    for left_item in &left_array {
        let left_map = dynamic_to_map(left_item);
        if let Some(key_value) = left_map.get(key) {
            let key_str = key_value.to_string();
            if let Some(right_matches) = right_index.get(&key_str) {
                for right_map in right_matches {
                    let mut joined_map: Map = dynamic_to_rhai_map(left_item);
                    for (k, v) in right_map {
                        if !joined_map.contains_key(k.as_str()) {
                            joined_map.insert(k.clone(), v.clone());
                        }
                    }
                    results.push(Dynamic::from(joined_map));
                }
            }
        }
    }

    Ok(Dynamic::from(results))
}

fn execute_pivot(
    data: &Dynamic,
    row_field: &str,
    value_field: &str,
) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let array = to_array(data.clone());
    let mut pivot: HashMap<String, f64> = HashMap::new();

    for item in array {
        let item_map = dynamic_to_map(&item);

        let row_key = item_map
            .get(row_field)
            .map(|v| v.to_string())
            .unwrap_or_default();

        let value = item_map
            .get(value_field)
            .and_then(|v| v.to_string().parse::<f64>().ok())
            .unwrap_or(0.0);

        *pivot.entry(row_key).or_insert(0.0) += value;
    }

    let mut results: Array = Array::new();
    for (key, sum) in pivot {
        let mut row: Map = Map::new();
        row.insert(row_field.into(), Dynamic::from(key));
        row.insert(value_field.into(), Dynamic::from(sum));
        results.push(Dynamic::from(row));
    }

    Ok(Dynamic::from(results))
}

fn execute_group_by(data: &Dynamic, field: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let array = to_array(data.clone());
    let mut groups: HashMap<String, Array> = HashMap::new();

    for item in array {
        let item_map = dynamic_to_map(&item);

        let group_key = item_map
            .get(field)
            .map(|v| v.to_string())
            .unwrap_or_default();

        groups.entry(group_key).or_default().push(item);
    }

    let mut result_map: Map = Map::new();
    for (key, items) in groups {
        result_map.insert(key.into(), Dynamic::from(items));
    }

    Ok(Dynamic::from(result_map))
}

fn dynamic_to_map(value: &Dynamic) -> HashMap<String, Dynamic> {
    let mut result = HashMap::new();

    if let Some(map) = value.clone().try_cast::<Map>() {
        for (k, v) in map {
            result.insert(k.to_string(), v);
        }
    }

    result
}

fn dynamic_to_rhai_map(value: &Dynamic) -> Map {
    value.clone().try_cast::<Map>().unwrap_or_default()
}

fn parse_filter_clause(filter: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (field, operator, value) = parse_condition_internal(filter)?;

    let sql_operator = match operator.as_str() {
        "!=" | "<>" => "!=",
        ">" => ">",
        "<" => "<",
        ">=" => ">=",
        "<=" => "<=",
        "like" | "LIKE" => "LIKE",
        _ => "=",
    };

    Ok(format!(
        "{} {} '{}'",
        sanitize_identifier(&field),
        sql_operator,
        sanitize_sql_value(&value)
    ))
}

fn parse_condition(condition: &str) -> Result<(String, String, String), Box<rhai::EvalAltResult>> {
    parse_condition_internal(condition).map_err(|e| {
        Box::new(rhai::EvalAltResult::ErrorRuntime(
            e.to_string().into(),
            rhai::Position::NONE,
        ))
    })
}

fn parse_condition_internal(
    condition: &str,
) -> Result<(String, String, String), Box<dyn Error + Send + Sync>> {
    let operators = [">=", "<=", "!=", "<>", "==", "=", ">", "<", "like", "LIKE"];

    for op in operators {
        if let Some(pos) = condition.find(op) {
            let field = condition[..pos].trim().to_string();
            let value = condition[pos + op.len()..].trim().to_string();
            return Ok((field, op.to_string(), value));
        }
    }

    Err("Invalid condition format".into())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("users"), "users");
        assert_eq!(sanitize_identifier("user_name"), "user_name");
        assert_eq!(
            sanitize_identifier("users; DROP TABLE users;"),
            "usersDROPTABLEusers"
        );
    }

    #[test]
    fn test_sanitize_sql_value() {
        assert_eq!(sanitize_sql_value("hello"), "hello");
        assert_eq!(sanitize_sql_value("it's"), "it''s");
        assert_eq!(sanitize_sql_value("O'Brien"), "O''Brien");
    }

    #[test]
    fn test_parse_condition() {
        let (field, op, value) = parse_condition_internal("status=active").unwrap();
        assert_eq!(field, "status");
        assert_eq!(op, "=");
        assert_eq!(value, "active");

        let (field, op, value) = parse_condition_internal("age>=18").unwrap();
        assert_eq!(field, "age");
        assert_eq!(op, ">=");
        assert_eq!(value, "18");
    }

    #[test]
    fn test_parse_filter_clause() {
        let clause = parse_filter_clause("name=John").unwrap();
        assert!(clause.contains("name"));
        assert!(clause.contains("John"));
    }
}
