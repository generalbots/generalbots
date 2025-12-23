use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

pub fn save_from_unstructured_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    // Register with spaces: SAVE FROM UNSTRUCTURED "table", text
    engine
        .register_custom_syntax(
            &["SAVE", "FROM", "UNSTRUCTURED", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let table_name = context.eval_expression_tree(&inputs[0])?.to_string();
                let text = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!(
                    "SAVE FROM UNSTRUCTURED: table={}, text_len={} for user={}",
                    table_name,
                    text.len(),
                    user_clone.user_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_save_from_unstructured(
                                &state_for_task,
                                &user_for_task,
                                &table_name,
                                &text,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SAVE FROM UNSTRUCTURED result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(record_id)) => Ok(Dynamic::from(record_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SAVE FROM UNSTRUCTURED failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SAVE FROM UNSTRUCTURED timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SAVE FROM UNSTRUCTURED thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

async fn execute_save_from_unstructured(
    state: &AppState,
    user: &UserSession,
    table_name: &str,
    text: &str,
) -> Result<String, String> {
    // Get table schema to understand what fields to extract
    let schema = get_table_schema(state, table_name).await?;

    // Use LLM to extract structure from text
    let extraction_prompt = build_extraction_prompt(table_name, &schema, text);
    let extracted_json = call_llm_for_extraction(state, &extraction_prompt).await?;

    // Validate and clean the extracted data
    let cleaned_data = validate_and_clean_data(&extracted_json, &schema)?;

    // Save to database
    let record_id = save_to_table(state, user, table_name, cleaned_data).await?;

    trace!(
        "Saved unstructured data to table '{}': {}",
        table_name,
        record_id
    );

    Ok(record_id)
}

async fn get_table_schema(state: &AppState, table_name: &str) -> Result<Value, String> {
    // Get table schema from database
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    // Query PostgreSQL information schema for table columns
    let query = diesel::sql_query(
        "SELECT column_name, data_type, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_name = $1
         ORDER BY ordinal_position",
    )
    .bind::<diesel::sql_types::Text, _>(table_name);

    #[derive(QueryableByName, Debug)]
    struct ColumnInfo {
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        column_default: Option<String>,
    }

    let columns: Vec<ColumnInfo> = query.load(&mut *conn).map_err(|e| {
        error!("Failed to get table schema: {}", e);
        format!("Table '{}' not found or error: {}", table_name, e)
    })?;

    if columns.is_empty() {
        // Table doesn't exist, use default schema based on table name
        return Ok(get_default_schema(table_name));
    }

    let schema: Vec<Value> = columns
        .into_iter()
        .map(|col| {
            json!({
                "name": col.column_name,
                "type": col.data_type,
                "nullable": col.is_nullable == "YES",
                "default": col.column_default
            })
        })
        .collect();

    Ok(json!(schema))
}

fn get_default_schema(table_name: &str) -> Value {
    // Provide default schemas for common tables
    match table_name {
        "leads" | "rob" => json!([
            {"name": "id", "type": "uuid", "nullable": false},
            {"name": "name", "type": "text", "nullable": true},
            {"name": "company", "type": "text", "nullable": true},
            {"name": "email", "type": "text", "nullable": true},
            {"name": "phone", "type": "text", "nullable": true},
            {"name": "website", "type": "text", "nullable": true},
            {"name": "notes", "type": "text", "nullable": true},
            {"name": "status", "type": "text", "nullable": true},
            {"name": "created_at", "type": "timestamp", "nullable": false}
        ]),
        "tasks" => json!([
            {"name": "id", "type": "uuid", "nullable": false},
            {"name": "title", "type": "text", "nullable": false},
            {"name": "description", "type": "text", "nullable": true},
            {"name": "assignee", "type": "text", "nullable": true},
            {"name": "due_date", "type": "timestamp", "nullable": true},
            {"name": "priority", "type": "text", "nullable": true},
            {"name": "status", "type": "text", "nullable": true},
            {"name": "created_at", "type": "timestamp", "nullable": false}
        ]),
        "meetings" => json!([
            {"name": "id", "type": "uuid", "nullable": false},
            {"name": "subject", "type": "text", "nullable": false},
            {"name": "attendees", "type": "jsonb", "nullable": true},
            {"name": "date", "type": "timestamp", "nullable": true},
            {"name": "duration", "type": "integer", "nullable": true},
            {"name": "location", "type": "text", "nullable": true},
            {"name": "notes", "type": "text", "nullable": true},
            {"name": "created_at", "type": "timestamp", "nullable": false}
        ]),
        "opportunities" => json!([
            {"name": "id", "type": "uuid", "nullable": false},
            {"name": "company", "type": "text", "nullable": false},
            {"name": "contact", "type": "text", "nullable": true},
            {"name": "value", "type": "numeric", "nullable": true},
            {"name": "stage", "type": "text", "nullable": true},
            {"name": "probability", "type": "integer", "nullable": true},
            {"name": "close_date", "type": "date", "nullable": true},
            {"name": "notes", "type": "text", "nullable": true},
            {"name": "created_at", "type": "timestamp", "nullable": false}
        ]),
        _ => json!([
            {"name": "id", "type": "uuid", "nullable": false},
            {"name": "data", "type": "jsonb", "nullable": true},
            {"name": "created_at", "type": "timestamp", "nullable": false}
        ]),
    }
}

fn build_extraction_prompt(table_name: &str, schema: &Value, text: &str) -> String {
    let schema_str = serde_json::to_string_pretty(schema).unwrap_or_default();

    let table_context = match table_name {
        "leads" | "rob" => "This is a CRM lead/contact record. Extract contact information, company details, and any relevant notes.",
        "tasks" => "This is a task record. Extract the task title, description, who it should be assigned to, when it's due, and priority.",
        "meetings" => "This is a meeting record. Extract the meeting subject, attendees, date/time, duration, and any notes.",
        "opportunities" => "This is a sales opportunity. Extract the company, contact person, deal value, sales stage, and expected close date.",
        _ => "Extract relevant structured data from the text."
    };

    format!(
        r#"Extract structured data from the following unstructured text and return it as JSON that matches this table schema:

Table: {}
Context: {}

Schema:
{}

Text to extract from:
"""
{}
"""

Instructions:
1. Extract ONLY information that is present in the text
2. Return a valid JSON object with field names matching the schema
3. Use null for fields that cannot be extracted from the text
4. For date/time fields, parse natural language dates (e.g., "next Friday" -> actual date)
5. For email fields, extract valid email addresses
6. For numeric fields, extract numbers and convert to appropriate type
7. Do NOT make up or invent data that isn't in the text
8. If the text mentions multiple entities, extract the primary/first one

Return ONLY the JSON object, no explanations or markdown formatting."#,
        table_name, table_context, schema_str, text
    )
}

async fn call_llm_for_extraction(state: &AppState, prompt: &str) -> Result<Value, String> {
    // Get LLM configuration
    let config_manager = crate::config::ConfigManager::new(state.conn.clone());
    let model = config_manager
        .get_config(&Uuid::nil(), "llm-model", None)
        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    let key = config_manager
        .get_config(&Uuid::nil(), "llm-key", None)
        .unwrap_or_default();

    // Call LLM
    let response = state
        .llm_provider
        .generate(prompt, &Value::Null, &model, &key)
        .await
        .map_err(|e| format!("LLM extraction failed: {}", e))?;

    // Parse LLM response as JSON
    let extracted = serde_json::from_str::<Value>(&response).unwrap_or_else(|_| {
        // If LLM didn't return valid JSON, try to extract JSON from the response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                serde_json::from_str(json_str).unwrap_or_else(|_| json!({}))
            } else {
                json!({})
            }
        } else {
            json!({})
        }
    });

    Ok(extracted)
}

fn validate_and_clean_data(data: &Value, schema: &Value) -> Result<Value, String> {
    let mut cleaned = json!({});

    if let Some(data_obj) = data.as_object() {
        if let Some(schema_arr) = schema.as_array() {
            for column_def in schema_arr {
                if let Some(column_name) = column_def.get("name").and_then(|n| n.as_str()) {
                    // Skip system fields that will be auto-generated
                    if column_name == "id" || column_name == "created_at" {
                        continue;
                    }

                    if let Some(value) = data_obj.get(column_name) {
                        // Clean and validate based on type
                        let column_type = column_def
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("text");

                        let cleaned_value = clean_value_for_type(value, column_type);

                        if !cleaned_value.is_null() {
                            cleaned[column_name] = cleaned_value;
                        }
                    }
                }
            }
        }
    }

    // Ensure we have at least some data
    if cleaned.as_object().map_or(true, |o| o.is_empty()) {
        return Err("No valid data could be extracted from the text".to_string());
    }

    Ok(cleaned)
}

fn clean_value_for_type(value: &Value, data_type: &str) -> Value {
    match data_type {
        "text" | "varchar" => {
            if value.is_string() {
                value.clone()
            } else {
                json!(value.to_string())
            }
        }
        "integer" | "bigint" | "smallint" => {
            if let Some(n) = value.as_i64() {
                json!(n)
            } else if let Some(s) = value.as_str() {
                s.parse::<i64>().map(|n| json!(n)).unwrap_or(json!(null))
            } else {
                json!(null)
            }
        }
        "numeric" | "decimal" | "real" | "double precision" => {
            if let Some(n) = value.as_f64() {
                json!(n)
            } else if let Some(s) = value.as_str() {
                s.parse::<f64>().map(|n| json!(n)).unwrap_or(json!(null))
            } else {
                json!(null)
            }
        }
        "boolean" => {
            if let Some(b) = value.as_bool() {
                json!(b)
            } else if let Some(s) = value.as_str() {
                json!(s.to_lowercase() == "true" || s == "1" || s.to_lowercase() == "yes")
            } else {
                json!(false)
            }
        }
        "timestamp" | "timestamptz" | "date" | "time" => {
            if value.is_string() {
                value.clone() // Let PostgreSQL handle the parsing
            } else {
                json!(null)
            }
        }
        "jsonb" | "json" => value.clone(),
        "uuid" => {
            if let Some(s) = value.as_str() {
                // Validate UUID format
                if Uuid::parse_str(s).is_ok() {
                    value.clone()
                } else {
                    json!(Uuid::new_v4().to_string())
                }
            } else {
                json!(Uuid::new_v4().to_string())
            }
        }
        _ => value.clone(),
    }
}

async fn save_to_table(
    state: &AppState,
    user: &UserSession,
    table_name: &str,
    data: Value,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let record_id = Uuid::new_v4().to_string();
    let user_id = user.user_id.to_string();
    let created_at = Utc::now();

    // Build dynamic INSERT query
    let mut fields = vec!["id", "created_at"];
    let mut placeholders = vec!["$1".to_string(), "$2".to_string()];
    let mut bind_index = 3;

    let data_obj = data.as_object().ok_or("Invalid data format")?;

    for (field, _) in data_obj {
        fields.push(field);
        placeholders.push(format!("${}", bind_index));
        bind_index += 1;
    }

    // Add user tracking if not already present
    if !data_obj.contains_key("user_id") {
        fields.push("user_id");
        placeholders.push(format!("${}", bind_index));
    }

    // Build values as JSON for simpler handling
    let mut values_map = serde_json::Map::new();
    values_map.insert("id".to_string(), json!(record_id));
    values_map.insert("created_at".to_string(), json!(created_at));

    // Add data fields
    for (field, value) in data_obj {
        values_map.insert(field.clone(), value.clone());
    }

    // Add user_id if needed
    if !data_obj.contains_key("user_id") {
        values_map.insert("user_id".to_string(), json!(user_id));
    }

    // Convert to JSON and use JSONB insert
    let values_json = json!(values_map);

    // Use a simpler approach with JSON
    let insert_query = format!(
        "INSERT INTO {} SELECT * FROM jsonb_populate_record(NULL::{},'{}');",
        table_name,
        table_name,
        values_json.to_string().replace("'", "''")
    );

    diesel::sql_query(&insert_query)
        .execute(&mut *conn)
        .map_err(|e| {
            error!("Failed to save to table '{}': {}", table_name, e);
            format!("Failed to save record: {}", e)
        })?;

    trace!(
        "Saved record {} to table '{}' for user {}",
        record_id,
        table_name,
        user_id
    );

    Ok(record_id)
}
