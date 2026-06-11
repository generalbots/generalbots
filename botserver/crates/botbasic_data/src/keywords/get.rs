use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use botbasic_core::security_utils::sanitize_identifier;
use diesel::prelude::*;
use diesel::sql_types::Text;
use log::{error, trace};
use reqwest::{self, Client};
use rhai::{Dynamic, Engine};
use serde_json::Value;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    row_data: String,
}

pub fn register_get_keyword(state: Arc<dyn BasicRuntime>, user_session: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(["GET", "$expr$"], false, move |context, inputs| {
            let url = context.eval_expression_tree(&inputs[0])?;
            let url_str = url.to_string();
            if !is_safe_path(&url_str) {
                return Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "URL contains invalid or unsafe path sequences".into(),
                    rhai::Position::NONE,
                )));
            }
            let state_for_blocking = Arc::clone(&state_clone);
            let url_for_blocking = url_str;
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();
                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        if url_for_blocking.starts_with("https://")
                            || url_for_blocking.starts_with("http://")
                        {
                            execute_get(&url_for_blocking).await
                        } else {
                            get_from_bucket(
                                state_for_blocking,
                                &url_for_blocking,
                                user_session.bot_id,
                            )
                            .await
                        }
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("failed to build tokio runtime".into())).err()
                };
                if send_err.is_some() {
                    log::error!("Failed to send result from thread");
                }
            });
            match rx.recv_timeout(std::time::Duration::from_secs(40)) {
                Ok(Ok(content)) => Ok(Dynamic::from(content)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(Box::new(
                    rhai::EvalAltResult::ErrorRuntime("GET timed out".into(), rhai::Position::NONE),
                )),
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("GET failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

pub fn register_get_from_fn(
    state: Arc<dyn BasicRuntime>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let state_fn = Arc::clone(&state);
    let user_fn = user_session.clone();
    engine.register_fn("GET_FROM", move |table: String| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
        let filter = "1=1".to_string();
        let bot_id = user_fn.bot_id;
        let pool = state_fn.db_pool().clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let tbl = table;
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = match rt {
                Ok(rt) => rt.block_on(async move {
                    execute_get_from(&pool, &tbl, &filter, bot_id)
                }),
                Err(e) => Err(format!("Failed to create runtime: {e}")),
            };
            let _ = tx.send(result);
        });
        let result = rx.recv().unwrap_or(Err("Failed to receive result".into()))?;
        let results = result.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default();
        let results_value = serde_json::Value::Array(results);
        Ok(botbasic_core::utils::json_value_to_dynamic(&results_value))
    });

    let state_fn2 = Arc::clone(&state);
    let user_fn2 = user_session;
    engine.register_fn("GET_FROM", move |table: String, filter: String| -> Result<Dynamic, Box<rhai::EvalAltResult>> {
        let bot_id = user_fn2.bot_id;
        let pool = state_fn2.db_pool().clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let tbl = table;
        let flt = filter;
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = match rt {
                Ok(rt) => rt.block_on(async move {
                    execute_get_from(&pool, &tbl, &flt, bot_id)
                }),
                Err(e) => Err(format!("Failed to create runtime: {e}")),
            };
            let _ = tx.send(result);
        });
        let result = rx.recv().unwrap_or(Err("Failed to receive result".into()))?;
        let results = result.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default();
        let results_value = serde_json::Value::Array(results);
        Ok(botbasic_core::utils::json_value_to_dynamic(&results_value))
    });
}

pub fn register_get_from_keyword(
    state: Arc<dyn BasicRuntime>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let state_for_closure = state.clone();
    let user_for_closure = user_session;
    engine
        .register_custom_syntax(
            ["GET", "FROM", "$expr$", "WHERE", "$expr$"],
            false,
            move |context, inputs| {
                let table = context.eval_expression_tree(&inputs[0])?.to_string();
                let filter = context.eval_expression_tree(&inputs[1])?.to_string();
                let bot_id = user_for_closure.bot_id;

                let pool = state_for_closure.db_pool().clone();
                let (tx, rx) = std::sync::mpsc::channel();
                let table_clone = table.clone();
                let filter_clone = filter.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();
                    let result = match rt {
                        Ok(rt) => rt.block_on(async move {
                            execute_get_from(&pool, &table_clone, &filter_clone, bot_id)
                        }),
                        Err(e) => Err(format!("Failed to create runtime: {e}")),
                    };
                    let _ = tx.send(result);
                });
                let result = rx.recv().unwrap_or(Err("Failed to receive result".into()))?;

                let results = result.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default();
                let first = results.into_iter().next().unwrap_or(Value::Null);
                let dynamic = botbasic_core::utils::json_value_to_dynamic(&first);
                Ok(dynamic)
            },
        )
        .expect("valid syntax registration");
}

fn execute_get_from(
    pool: &botlib::db_pool::DbPool,
    table: &str,
    filter: &str,
    _bot_id: uuid::Uuid,
) -> Result<Value, String> {
    use botbasic_core::utils::parse_filter;

    let safe_table = sanitize_identifier(table);
    let (where_clause, params) = parse_filter(filter).map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT row_to_json(t)::text as row_data FROM (SELECT * FROM {safe_table} WHERE {where_clause} LIMIT 1) t"
    );

    let mut conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<JsonRow> = if params.is_empty() {
        diesel::sql_query(&query)
            .load(&mut conn)
            .map_err(|e| format!("SQL error: {e}"))?
    } else {
        diesel::sql_query(&query)
            .bind::<Text, _>(&params[0])
            .load(&mut conn)
            .map_err(|e| format!("SQL error: {e}"))?
    };

    let results: Vec<Value> = rows
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.row_data).ok())
        .collect();

    Ok(serde_json::json!({
        "command": "get_from",
        "table": table,
        "filter": filter,
        "results": results,
        "count": results.len()
    }))
}

fn is_safe_path(path: &str) -> bool {
    if path.starts_with("https://") || path.starts_with("http://") {
        if let Ok(parsed_url) = url::Url::parse(path) {
            if let Some(host) = parsed_url.host_str() {
                let host_lower = host.to_lowercase();
                if host_lower == "localhost" 
                    || host_lower.contains("169.254") 
                    || host_lower.starts_with("127.") 
                    || host_lower.starts_with("10.") 
                    || host_lower.starts_with("192.168.") 
                    || host_lower.starts_with("172.")
                    || host_lower == "::1"
                    || host_lower.contains("0x7f")
                    || host_lower.contains("metadata.google.internal") {
                    return false; // Prevent obvious SSRF
                }
            }
        }
        return true;
    }
    if path.contains("..") || path.starts_with('/') {
        return false;
    }
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        return false;
    }
    if path.contains("//") || path.contains('~') || path.contains('*') || path.contains('?') {
        return false;
    }
    if !path.starts_with("http") {
        let path_obj = Path::new(path);
        if path_obj.components().count()
            != path_obj
                .components()
                .filter(|c| matches!(c, std::path::Component::Normal(_)))
                .count()
        {
            return false;
        }
    }
    true
}
pub async fn execute_get(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to build HTTP client: {}", e);
            e
        })?;
    let response = client.get(url).send().await.map_err(|e| {
        log::error!("HTTP request failed for URL {}: {}", url, e);
        e
    })?;
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        error!(
            "HTTP request returned non-success status for URL {}: {} - {}",
            url, status, error_body
        );
        return Err(format!(
            "HTTP request failed with status: {} - {}",
            status, error_body
        )
        .into());
    }
    let content = response.text().await.map_err(|e| {
        log::error!("Failed to read response text for URL {}: {}", url, e);
        e
    })?;
    trace!(
        "Successfully executed GET request for URL: {}, content length: {}",
        url,
        content.len()
    );
    Ok(content)
}

#[cfg(feature = "drive")]
pub async fn get_from_bucket(
    state: Arc<dyn BasicRuntime>,
    file_path: &str,
    bot_id: uuid::Uuid,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if !is_safe_path(file_path) {
        log::error!("Unsafe file path detected: {}", file_path);
        return Err("Invalid file path".into());
    }
    let drive_repo = state.drive_repository().ok_or("S3 client not configured")?;
    let client = drive_repo.as_ref();
    let bot_name: String = {
        use botbasic_types::schema::bots::dsl::*;
        let mut db_conn = state.db_pool().get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                log::error!("Failed to query bot name for {}: {}", bot_id, e);
                e
            })?
    };
    let bucket_name = {
        let bucket = format!("{}.gbai", bot_name);
        bucket
    };
    let bytes: Vec<u8> = match tokio::time::timeout(Duration::from_secs(30), async {
        client
            .get_object(&bucket_name, file_path)
            .await
            .map_err(|e| format!("S3 operation failed: {}", e))
    })
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            log::error!("drive read failed: {}", e);
            return Err(format!("S3 operation failed: {}", e).into());
        }
        Err(_) => {
            log::error!("drive read timed out");
            return Err("drive operation timed out".into());
        }
    };
    let content = if file_path.to_ascii_lowercase().ends_with(".pdf") {
        #[cfg(feature = "drive")]
        match pdf_extract::extract_text_from_mem(&bytes) {
            Ok(text) => text,
            Err(e) => {
                log::error!("PDF extraction failed: {}", e);
                return Err(format!("PDF extraction failed: {}", e).into());
            }
        }
        #[cfg(not(feature = "drive"))]
        {
            return Err("PDF extraction requires drive feature".into());
        }
    } else {
        match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                log::error!("File content is not valid UTF-8 text");
                return Err("File content is not valid UTF-8 text".into());
            }
        }
    };
    trace!(
        "Successfully retrieved file from bucket: {}, content length: {}",
        file_path,
        content.len()
    );
    Ok(content)
}

#[cfg(not(feature = "drive"))]
pub async fn get_from_bucket(
    _state: Arc<dyn BasicRuntime>,
    _file_path: &str,
    _bot_id: uuid::Uuid,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    Err("S3 drive is not enabled. Configure MinIO to use this feature.".into())
}
