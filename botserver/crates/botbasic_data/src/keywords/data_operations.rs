use crate::keywords::table_access::{check_table_access, AccessType, UserRoles};
use botbasic_core::{sanitize_identifier, sanitize_sql_value};
use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use botbasic_core::utils::{convert_date_to_iso_format, json_value_to_dynamic, to_array};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{trace, warn};
use rhai::{Array, Dynamic, Engine, Map};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// Query a single text field from a table using the same filter/WHERE clause as UPDATE.
/// Returns None if no matching row is found.
fn query_table_field(
    conn: &mut diesel::PgConnection,
    table: &str,
    field: &str,
    filter: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let where_clause = parse_filter_clause(filter)?;
    let query = format!(
        "SELECT {} FROM {} WHERE {} LIMIT 1",
        sanitize_identifier(field),
        sanitize_identifier(table),
        where_clause
    );

    #[derive(QueryableByName)]
    struct FieldResult {
        #[diesel(sql_type = Text)]
        value: String,
    }

    match sql_query(&query).get_result::<FieldResult>(conn) {
        Ok(row) => Ok(Some(row.value)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Query a single text field from a table by id.
fn query_table_field_by_id(
    conn: &mut diesel::PgConnection,
    table: &str,
    field: &str,
    id: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = '{}' LIMIT 1",
        sanitize_identifier(field),
        sanitize_identifier(table),
        sanitize_sql_value(id)
    );

    #[derive(QueryableByName)]
    struct FieldResult {
        #[diesel(sql_type = Text)]
        value: String,
    }

    match sql_query(&query).get_result::<FieldResult>(conn) {
        Ok(row) => Ok(Some(row.value)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn register_data_operations(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    register_save_keyword(state.clone(), user.clone(), engine);
    register_insert_keyword(state.clone(), user.clone(), engine);
    register_update_keyword(state.clone(), user.clone(), engine);
    register_delete_keyword(state.clone(), user.clone(), engine);
    register_merge_keyword(state.clone(), user.clone(), engine);
    register_fill_keyword(state.clone(), user.clone(), engine);
    register_map_keyword(state.clone(), user.clone(), engine);
    register_filter_keyword(state.clone(), user.clone(), engine);
    register_aggregate_keyword(state.clone(), user.clone(), engine);
    register_join_keyword(state.clone(), user.clone(), engine);
    register_pivot_keyword(state.clone(), user.clone(), engine);
    register_group_by_keyword(state, user, engine);
}

pub fn register_save_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let user_roles = UserRoles::from_user_session(&user);

    // SAVE with variable arguments: SAVE "table", id, field1, field2, ...
    // Each pattern: table + id + (1 to 64 fields)
    // Minimum: table + id + 1 field = 4 expressions total
    register_save_variants(state.clone(), user.clone(), user_roles, engine);
}

fn register_save_variants(state: Arc<dyn BasicRuntime>, user: UserSession, user_roles: UserRoles, engine: &mut Engine) {
    // Rhai 1.25.x stores custom syntax in a BTreeMap keyed by the first token,
    // so only ONE pattern per starting keyword survives.
    // The compiler converts positional SAVE → SAVE table, data (2-arg)
    // and SAVE () TO WHERE → SAVE "table", data (2-arg with id embedded in data).
    // The handler checks if data contains an "id" field to decide insert vs upsert.
    {
        let state_clone = Arc::clone(&state);
        let user_roles_clone = user_roles.clone();
        let user_clone = user.clone();
        engine
            .register_custom_syntax(
                ["SAVE", "$expr$", ",", "$expr$"],
                false,
                move |context, inputs| {
                    let table = context.eval_expression_tree(&inputs[0])?.to_string();
                    let data = context.eval_expression_tree(&inputs[1])?;

                    trace!("SAVE: table={}", table);

                    // Use bot database if available (tables are per-bot), fallback to main
                    let pool_opt = state_clone
                        .bot_database_manager()
                        .get_bot_pool(user_clone.bot_id);
                    let mut conn = if let Some(pool) = pool_opt {
                        pool.get().map_err(|e| format!("Bot DB error: {}", e))?
                    } else {
                        state_clone
                            .db_pool()
                            .get()
                            .map_err(|e| format!("DB error: {}", e))?
                    };

                    if let Err(e) =
                        check_table_access(&mut conn, &table, &user_roles_clone, AccessType::Write)
                    {
                        warn!("SAVE access denied: {}", e);
                        return Err(e.into());
                    }

                    // Check if data has an "id" field to decide insert vs upsert
                    let data_map = dynamic_to_map(&data);
                    let has_id = data_map.contains_key("id");
                    let id_value = data_map.get("id").map(|v| v.to_string()).unwrap_or_default();

                    let result = if has_id {
                        // SAVE () TO WHERE with id → upsert
                        execute_save(&mut conn, &table, &Dynamic::from(id_value.clone()), &data)
                            .map_err(|e| format!("SAVE error: {}", e))?
                    } else {
                        // Positional SAVE without id → insert
                        execute_insert(&mut conn, &table, &data)
                            .map_err(|e| format!("SAVE error: {}", e))?
                    };

                    let rid = if id_value.is_empty() {
                        result.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                    } else {
                        id_value.clone()
                    };

                    if has_id && !rid.is_empty() {
                        fire_table_triggers(
                            &mut conn,
                            &state_clone,
                            &user_clone,
                            &table,
                            1,
                            Some(rid),
                            None,
                            None,
                        );
                    }

                    if !has_id {
                        fire_table_triggers(
                            &mut conn,
                            &state_clone,
                            &user_clone,
                            &table,
                            2,
                            None,
                            None,
                            None,
                        );
                    }

                    Ok(json_value_to_dynamic(&result))
                },
            )
            .expect("valid syntax registration");
    }
}

pub fn register_insert_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();
    let user_roles = UserRoles::from_user_session(&user);

    engine
        .register_custom_syntax(
            ["INSERT", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("INSERT into table: {}", table);

                // Get bot's database connection instead of main connection
                let bot_pool = state_clone
.bot_database_manager()
.get_bot_pool(user_clone.bot_id);

let mut conn = match bot_pool {
            Some(pool) => pool.get().map_err(|e| format!("Bot DB error: {}", e))?,
            None => state_clone
                .db_pool()
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

                fire_table_triggers(&mut conn, &state_clone, &user_clone, &table, 2, None, None, None);

                Ok(json_value_to_dynamic(&result))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_update_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();
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

                // Use bot database if available, fallback to main
                let pool_opt = state_clone
                    .bot_database_manager()
                    .get_bot_pool(user_clone.bot_id);
                let mut conn = if let Some(pool) = pool_opt {
                    pool.get().map_err(|e| format!("Bot DB error: {}", e))?
                } else {
                    state_clone
                        .db_pool()
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?
                };

                // Check write access
                if let Err(e) =
                    check_table_access(&mut conn, &table, &user_roles, AccessType::Write)
                {
                    warn!("UPDATE access denied: {}", e);
                    return Err(e.into());
                }

                // Capture old status BEFORE the UPDATE (for trigger context)
                let old_status = query_table_field(&mut conn, &table, "status", &filter)
                    .map_err(|e| format!("Failed to query old status: {}", e))?;

                let (row_count, ids) = execute_update(&mut conn, &table, &filter, &data)
                    .map_err(|e| format!("UPDATE error: {}", e))?;

                // Capture new status AFTER the UPDATE (from first updated record)
                let new_status = ids
                    .first()
                    .and_then(|id| query_table_field_by_id(&mut conn, &table, "status", id).ok())
                    .flatten();

                fire_table_triggers(
                    &mut conn,
                    &state_clone,
                    &user_clone,
                    &table,
                    1,
                    ids.first().cloned(),
                    old_status,
                    new_status,
                );

                Ok(Dynamic::from(row_count))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_delete_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();
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
                        let rt = tokio::runtime::Builder::new_current_thread()
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

                    let pool_opt = state_clone
                        .bot_database_manager()
                        .get_bot_pool(user_clone.bot_id);
                    let mut conn = if let Some(pool) = pool_opt {
                        pool.get().map_err(|e| format!("Bot DB error: {}", e))?
                    } else {
                        state_clone
                            .db_pool()
                            .get()
                            .map_err(|e| format!("DB error: {}", e))?
                    };

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

    let state_clone2 = state.clone();
    engine
        .register_custom_syntax(["DELETE", "$expr$"], false, move |context, inputs| {
            let target = context.eval_expression_tree(&inputs[0])?.to_string();

            if target.starts_with("http://") || target.starts_with("https://") {
                trace!("DELETE HTTP: {}", target);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = target;

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
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

pub fn register_merge_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            ["MERGE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;
                let key_field = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("MERGE into table: {}, key: {}", table, key_field);

                let pool_opt = state_clone
                    .bot_database_manager()
                    .get_bot_pool(user_clone.bot_id);
                let mut conn = if let Some(pool) = pool_opt {
                    pool.get().map_err(|e| format!("Bot DB error: {}", e))?
                } else {
                    state_clone
                        .db_pool()
                        .get()
                        .map_err(|e| format!("DB error: {}", e))?
                };

                let result = execute_merge(&mut conn, &table, &data, &key_field)
                    .map_err(|e| format!("MERGE error: {}", e))?;

                Ok(json_value_to_dynamic(&result))
            },
        )
        .expect("valid syntax registration");
}

pub fn register_fill_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_map_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_filter_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_aggregate_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_join_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_pivot_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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

pub fn register_group_by_keyword(_state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
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
        if key == "id" {
            continue;
        }
        let sanitized_key = sanitize_identifier(key).to_lowercase();
        let value_str = value.to_string();
        let converted_value = convert_date_to_iso_format(&value_str);
        let sanitized_value = if converted_value.trim().is_empty() {
            "NULL".to_string()
        } else {
            format!("'{}'", sanitize_sql_value(&converted_value))
        };
        columns.push(sanitized_key.clone());
        values.push(sanitized_value.clone());
        update_sets.push(format!("{} = {}", sanitized_key, sanitized_value));
    }

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT (id) DO UPDATE SET {}",
        sanitize_identifier(table).to_lowercase(),
        columns.join(", "),
        values.join(", "),
        update_sets.join(", ")
    );

    trace!("Executing SAVE query: {}", query);

    let result = sql_query(&query).execute(conn).map_err(|e| {
        log::error!("SAVE SQL error: {}", e);
        e.to_string()
    })?;

    Ok(json!({
        "command": "save",
        "table": table,
        "id": id_value,
        "rows_affected": result
    }))
}

fn fire_table_triggers(
    conn: &mut diesel::PgConnection,
    state: &Arc<dyn BasicRuntime>,
    user: &botbasic_types::UserSession,
    table: &str,
    trigger_kind_val: i32,
    record_id: Option<String>,
    old_status: Option<String>,
    new_status: Option<String>,
) {
    // Use main database connection to query system_automations (metadata)
    let mut meta_conn = match state.db_pool().get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to get metadata connection for triggers: {}", e);
            return;
        }
    };
    let query = "SELECT param FROM system_automations WHERE bot_id = $1::uuid AND kind = $2 AND target = $3 AND is_active = true";
    let triggers: Vec<String> = match diesel::sql_query(query)
        .bind::<diesel::sql_types::Text, _>(&user.bot_id.to_string())
        .bind::<diesel::sql_types::Integer, _>(trigger_kind_val)
        .bind::<diesel::sql_types::Text, _>(table)
        .load::<TriggerRow>(&mut meta_conn)
    {
        Ok(rows) => rows.into_iter().map(|r| r.param).collect(),
        Err(e) => {
            log::warn!("Failed to query table triggers: {}", e);
            return;
        }
    };

    if triggers.is_empty() {
        return;
    }

    let work_root = botbasic_core::utils::get_work_path();

    // Resolve bot name from bot_id to locate the correct work directory
    let bot_name: String = diesel::sql_query(
        "SELECT name FROM bots WHERE id = $1::uuid"
    )
    .bind::<diesel::sql_types::Text, _>(&user.bot_id.to_string())
    .get_result::<BotNameRow>(conn)
    .ok()
    .map(|r| r.name)
    .unwrap_or_else(|| user.bot_id.to_string());

    for script_name in triggers {
        // Build trigger context from record data
        let mut ctx = serde_json::Map::new();
        if let Some(ref rid) = record_id {
            ctx.insert("trigger_record_id".to_string(), serde_json::Value::String(rid.clone()));
        }
        if let Some(ref os) = old_status {
            ctx.insert("trigger_old_status".to_string(), serde_json::Value::String(os.clone()));
        }
        if let Some(ref ns) = new_status {
            ctx.insert("trigger_new_status".to_string(), serde_json::Value::String(ns.clone()));
        }

        let mut trigger_user = user.clone();
        if !ctx.is_empty() {
            trigger_user.context_data = serde_json::Value::Object(ctx);
        }

        // Try .ast first (pre-compiled Rhai), fall back to .bas (raw BASIC source)
        let ast_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{script_name}.ast", org_id = botcore::shared::utils::current_org_id());
        let ast_path = std::path::Path::new(&work_root).join(&ast_rel);
        let bas_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/{script_name}.bas", org_id = botcore::shared::utils::current_org_id());
        let bas_path = std::path::Path::new(&work_root).join(&bas_rel);

        let content = std::fs::read_to_string(&ast_path)
            .or_else(|_| std::fs::read_to_string(&bas_path));

        match content {
            Ok(script_content) if !script_content.is_empty() => {
                if let Err(e) = state.execute_script(trigger_user, &script_content) {
                    log::warn!("Table trigger '{}' execution failed: {}", script_name, e);
                } else {
                    log::info!("Table trigger '{}' executed successfully", script_name);
                }
            }
            _ => log::warn!("Table trigger script not found: {:?} or {:?}", ast_path, bas_path),
        }
    }
}

#[derive(diesel::QueryableByName)]
struct BotNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(diesel::QueryableByName)]
struct TriggerRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    param: String,
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
        if converted_value.trim().is_empty() {
            values.push("NULL".to_string());
        } else {
            values.push(format!("'{}'", sanitize_sql_value(&converted_value)));
        }
    }

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
        sanitize_identifier(table).to_lowercase(),
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
            log::error!("INSERT SQL error: {}", e);
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
) -> Result<(i64, Vec<String>), Box<dyn Error + Send + Sync>> {
    let data_map = dynamic_to_map(data);
    let where_clause = parse_filter_clause(filter)?;

    let mut update_sets: Vec<String> = Vec::new();
    for (key, value) in &data_map {
        let value_str = value.to_string();
        let converted_value = convert_date_to_iso_format(&value_str);
        if converted_value.trim().is_empty() {
            update_sets.push(format!("{} = NULL", sanitize_identifier(key)));
        } else {
            update_sets.push(format!(
                "{} = '{}'",
                sanitize_identifier(key),
                sanitize_sql_value(&converted_value)
            ));
        }
    }

    let query = format!(
        "UPDATE {} SET {} WHERE {} RETURNING id",
        sanitize_identifier(table),
        update_sets.join(", "),
        where_clause
    );

    trace!("Executing UPDATE query: {}", query);

    #[derive(QueryableByName)]
    struct UpdateResult {
        #[diesel(sql_type = Text)]
        id: String,
    }

    let rows: Vec<UpdateResult> = sql_query(&query).load(conn).map_err(|e| {
        log::error!("UPDATE SQL error: {}", e);
        e.to_string()
    })?;

    let ids: Vec<String> = rows.into_iter().map(|r| r.id).collect();
    let count = ids.len() as i64;

    Ok((count, ids))
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
        log::error!("DELETE SQL error: {}", e);
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
    let array = to_array(data);
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
        columns.push(sanitize_identifier(key).to_lowercase());
                values.push(format!("'{}'", sanitize_sql_value(&value.to_string())));
            }

            let insert_query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                sanitize_identifier(table).to_lowercase(),
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
    let array = to_array(data);
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

    let array = to_array(data);
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
    let array = to_array(data);
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
    let array = to_array(data);
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
    let left_array = to_array(left);
    let right_array = to_array(right);
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
    let array = to_array(data);
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
    let array = to_array(data);
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

    if value.is_map() {
        let map = value.clone().cast::<Map>();
        for (k, v) in map.iter() {
            result.insert(k.to_string(), v.clone());
        }
    }

    result
}

fn dynamic_to_rhai_map(value: &Dynamic) -> Map {
    if value.is_map() {
        value.clone().cast::<Map>()
    } else {
        Map::new()
    }
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
