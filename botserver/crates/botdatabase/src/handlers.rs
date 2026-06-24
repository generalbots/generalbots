use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use diesel::prelude::*;
use diesel::sql_query;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::sql_guard::sanitize_identifier;
use botcore::shared::state::AppState;

use crate::db;

const MAX_QUERY_ROWS: i64 = 10_000;
const DEFAULT_PAGE_SIZE: i64 = 100;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub sort_order: Option<String>,
    pub filter_col: Option<String>,
    pub filter_op: Option<String>,
    pub filter_val: Option<String>,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Deserialize)]
pub struct CreateTableRequest {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: Option<bool>,
    pub default: Option<String>,
}

#[derive(Deserialize)]
pub struct AlterTableRequest {
    pub add_columns: Option<Vec<ColumnDef>>,
    pub drop_columns: Option<Vec<String>>,
    pub rename_to: Option<String>,
}

#[derive(Deserialize)]
pub struct AddColumnRequest {
    pub name: String,
    pub data_type: String,
    pub nullable: Option<bool>,
    pub default: Option<String>,
}

#[derive(Deserialize)]
pub struct InsertRowRequest {
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateRowRequest {
    pub pk_value: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<String>,
}

#[derive(Serialize)]
pub struct SchemaResponse {
    pub tables: Vec<TableSchema>,
}

#[derive(Serialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
    pub table_size: String,
}

#[derive(Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_pk: bool,
    pub is_fk: bool,
}

#[derive(Serialize)]
pub struct TableDataResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: i64,
    pub is_mutation: bool,
    pub duration_ms: u128,
}

#[derive(Serialize)]
pub struct ForeignKeyInfo {
    pub constraint_name: String,
    pub column_name: String,
    pub foreign_table: String,
    pub foreign_column: String,
}

#[derive(Serialize)]
pub struct ApiResult {
    pub success: bool,
    pub message: String,
}

fn error_response(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": msg})),
    )
}

fn internal_error(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": msg})),
    )
}

fn extract_bot_id(headers: &HeaderMap) -> Result<uuid::Uuid, (StatusCode, Json<serde_json::Value>)> {
    headers
        .get("X-Bot-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| error_response("Missing or invalid X-Bot-Id header"))
}

fn get_bot_pool(
    state: &AppState,
    bot_id: uuid::Uuid,
) -> Result<botcore::shared::utils::DbPool, (StatusCode, Json<serde_json::Value>)> {
    state
        .bot_database_manager
        .get_bot_pool(bot_id)
        .ok_or_else(|| error_response("Bot database not available"))
}

fn get_bot_conn(
    state: &AppState,
    bot_id: uuid::Uuid,
) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, (StatusCode, Json<serde_json::Value>)> {
    let pool = get_bot_pool(state, bot_id)?;
    pool.get()
        .map_err(|e| internal_error(&format!("Database connection error: {e}")))
}

fn get_bot_database_url(state: &AppState, bot_id: uuid::Uuid) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let db_name: String = {
        let pool = db::pool().map_err(|(code, msg)| (code, Json(serde_json::json!({"error": msg}))))?;
        let mut conn = pool.get().map_err(|e| internal_error(&format!("Main DB connection error: {e}")))?;

        #[derive(QueryableByName, Debug)]
        struct DbName {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            database_name: Option<String>,
        }

        let result: Option<DbName> = sql_query("SELECT database_name::text FROM bots WHERE id = $1 AND is_active = true")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .get_result(&mut conn)
            .ok();

        result.and_then(|r| r.database_name).ok_or_else(|| error_response("Bot database not configured"))?
    };

    let base_url = state.database_url.clone();
    let base = base_url
        .rfind('/')
        .map(|pos| &base_url[..pos])
        .unwrap_or(&base_url);
    Ok(format!("{base}/{db_name}"))
}

#[derive(QueryableByName, Debug)]
struct SchemaTable {
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    row_count: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_size: String,
}

#[derive(QueryableByName, Debug)]
struct SchemaColumn {
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    column_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    data_type: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_nullable: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    column_default: Option<String>,
}

#[derive(QueryableByName, Debug)]
struct PkInfo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    column_name: String,
}

#[derive(QueryableByName, Debug)]
struct FkInfo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    column_name: String,
}

#[derive(QueryableByName, Debug)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(QueryableByName, Debug)]
struct ColName {
    #[diesel(sql_type = diesel::sql_types::Text)]
    column_name: String,
}

pub async fn get_schema(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let mut conn = get_bot_conn(&state, bot_id)?;

    let tables: Vec<SchemaTable> = sql_query(
        "SELECT t.table_name::text,
                COALESCE(c.reltuples::bigint, 0)::bigint AS row_count,
                pg_size_pretty(pg_total_relation_size(quote_ident(t.table_name))) AS table_size
         FROM information_schema.tables t
         LEFT JOIN pg_class c ON c.relname = t.table_name
         WHERE t.table_schema = 'public'
           AND t.table_type = 'BASE TABLE'
         ORDER BY t.table_name",
    )
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query tables: {e}")))?;

    if tables.is_empty() {
        return Ok(Json(SchemaResponse { tables: Vec::new() }));
    }

    let table_names: Vec<String> = tables.iter().map(|t| t.table_name.clone()).collect();
    let table_list: Vec<String> = table_names
        .iter()
        .map(|n| format!("'{}'", n.replace('\'', "''")))
        .collect();

    let columns: Vec<SchemaColumn> = sql_query(format!(
        "SELECT c.table_name::text, c.column_name::text,
                c.data_type::text, c.is_nullable::boolean,
                c.column_default::text
         FROM information_schema.columns c
         WHERE c.table_schema = 'public'
           AND c.table_name IN ({})
         ORDER BY c.table_name, c.ordinal_position",
        table_list.join(",")
    ))
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query columns: {e}")))?;

    let pk_cols: Vec<PkInfo> = sql_query(format!(
        "SELECT a.attrelid::regclass::text AS table_name,
                a.attname AS column_name
         FROM pg_index i
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
         WHERE i.indisprimary
           AND a.attrelid::regclass::text IN ({})",
        table_list.join(",")
    ))
    .get_results(&mut conn)
    .unwrap_or_default();

    let fk_cols: Vec<FkInfo> = sql_query(format!(
        "SELECT kcu.table_name::text, kcu.column_name::text
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name
         WHERE tc.constraint_type = 'FOREIGN KEY'
           AND tc.table_schema = 'public'
           AND tc.table_name IN ({})",
        table_list.join(",")
    ))
    .get_results(&mut conn)
    .unwrap_or_default();

    let pk_set: std::collections::HashSet<(String, String)> = pk_cols
        .iter()
        .map(|p| (p.table_name.clone(), p.column_name.clone()))
        .collect();
    let fk_set: std::collections::HashSet<(String, String)> = fk_cols
        .iter()
        .map(|f| (f.table_name.clone(), f.column_name.clone()))
        .collect();

    let mut result_tables = Vec::new();
    for table in &tables {
        let table_columns: Vec<ColumnInfo> = columns
            .iter()
            .filter(|c| c.table_name == table.table_name)
            .map(|c| ColumnInfo {
                name: c.column_name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.is_nullable,
                default_value: c.column_default.clone(),
                is_pk: pk_set.contains(&(table.table_name.clone(), c.column_name.clone())),
                is_fk: fk_set.contains(&(table.table_name.clone(), c.column_name.clone())),
            })
            .collect();

        result_tables.push(TableSchema {
            name: table.table_name.clone(),
            columns: table_columns,
            row_count: table.row_count,
            table_size: table.table_size.clone(),
        });
    }

    Ok(Json(SchemaResponse {
        tables: result_tables,
    }))
}

pub async fn get_table_data(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<TableDataResponse>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(1000);
    let offset = (page - 1) * page_size;

    let count_result: CountResult = sql_query(format!("SELECT COUNT(*)::bigint AS count FROM {safe_name}"))
        .get_result(&mut conn)
        .map_err(|e| internal_error(&format!("Failed to count rows: {e}")))?;

    let columns: Vec<ColName> = sql_query(format!(
        "SELECT column_name::text FROM information_schema.columns
         WHERE table_schema = 'public' AND table_name = '{safe_name}'
         ORDER BY ordinal_position"
    ))
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to get columns: {e}")))?;

    let col_names: Vec<String> = columns.iter().map(|c| c.column_name.clone()).collect();

    if col_names.is_empty() {
        return Ok(Json(TableDataResponse {
            columns: Vec::new(),
            rows: Vec::new(),
            total_rows: count_result.count,
            page,
            page_size,
        }));
    }

    let cast_cols: Vec<String> = col_names
        .iter()
        .map(|c| format!("{}::text", sanitize_identifier(c)))
        .collect();

    let mut query = format!("SELECT {} FROM {safe_name}", cast_cols.join(", "));

    if let Some(filter_col) = &params.filter_col {
        let safe_filter_col = sanitize_identifier(filter_col);
        if safe_filter_col == *filter_col && !safe_filter_col.is_empty() {
            if let Some(filter_val) = &params.filter_val {
                let op = match params.filter_op.as_deref() {
                    Some("eq") | None => "=",
                    Some("neq") => "!=",
                    Some("gt") => ">",
                    Some("gte") => ">=",
                    Some("lt") => "<",
                    Some("lte") => "<=",
                    Some("like") => "LIKE",
                    Some("ilike") => "ILIKE",
                    _ => "=",
                };
                let escaped_val = filter_val.replace('\'', "''");
                let like_val = if matches!(op, "LIKE" | "ILIKE") {
                    format!("{escaped_val}%")
                } else {
                    escaped_val
                };
                query.push_str(&format!(" WHERE {safe_filter_col} {op} '{like_val}'"));
            }
        }
    }

    if let Some(sort_col) = &params.sort {
        let safe_sort = sanitize_identifier(sort_col);
        if safe_sort == *sort_col && !safe_sort.is_empty() {
            let order = match params.sort_order.as_deref() {
                Some("desc") | Some("DESC") => "DESC",
                _ => "ASC",
            };
            query.push_str(&format!(" ORDER BY {safe_sort} {order}"));
        }
    }

    query.push_str(&format!(" LIMIT {page_size} OFFSET {offset}"));
    let row_count = col_names.len();
    let raw_text: Vec<Vec<Option<String>>> = match row_count {
        1 => {
            #[derive(QueryableByName)]
            struct R { #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c1: Option<String> }
            let rows: Vec<R> = sql_query(&query).get_results(&mut conn)
                .map_err(|e| internal_error(&format!("Query failed: {e}")))?;
            rows.into_iter().map(|r| vec![r.c1]).collect()
        }
        2 => {
            #[derive(QueryableByName)]
            struct R { #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c1: Option<String>, #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c2: Option<String> }
            let rows: Vec<R> = sql_query(&query).get_results(&mut conn)
                .map_err(|e| internal_error(&format!("Query failed: {e}")))?;
            rows.into_iter().map(|r| vec![r.c1, r.c2]).collect()
        }
        3 => {
            #[derive(QueryableByName)]
            struct R { #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c1: Option<String>, #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c2: Option<String>, #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] c3: Option<String> }
            let rows: Vec<R> = sql_query(&query).get_results(&mut conn)
                .map_err(|e| internal_error(&format!("Query failed: {e}")))?;
            rows.into_iter().map(|r| vec![r.c1, r.c2, r.c3]).collect()
        }
        _ => {
            return execute_via_postgres(state, &bot_id, &query, page, page_size, count_result.count, false).await;
        }
    };

    let rows: Vec<Vec<serde_json::Value>> = raw_text
        .into_iter()
        .map(|row| row.into_iter().map(|cell| parse_cell(&cell)).collect())
        .collect();

    Ok(Json(TableDataResponse {
        columns: col_names,
        rows,
        total_rows: count_result.count,
        page,
        page_size,
    }))
}

fn parse_cell(opt: &Option<String>) -> serde_json::Value {
    match opt {
        None => serde_json::Value::Null,
        Some(s) => {
            if s == "true" {
                serde_json::Value::Bool(true)
            } else if s == "false" {
                serde_json::Value::Bool(false)
            } else if let Ok(i) = s.parse::<i64>() {
                serde_json::Value::Number(i.into())
            } else if let Ok(f) = s.parse::<f64>() {
                serde_json::json!(f)
            } else {
                serde_json::Value::String(s.clone())
            }
        }
    }
}

fn execute_via_postgres_sync(
    db_url: &str,
    query: &str,
) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), String> {
    let mut client = postgres::Client::connect(db_url, postgres::NoTls)
        .map_err(|e| format!("Connection failed: {e}"))?;

    let rows = client
        .query(query, &[])
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut columns = Vec::new();
    if let Some(first) = rows.first() {
        for i in 0..first.len() {
            let col = first.columns().get(i).ok_or("Column index out of bounds")?;
            columns.push(col.name().to_string());
        }
    }

    let result: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            (0..row.len())
                .map(|i| {
                    let col = &row.columns()[i];
                    pg_value_to_json(row, i, col.type_().name())
                })
                .collect()
        })
        .collect();

    Ok((columns, result))
}

fn pg_value_to_json(row: &postgres::Row, idx: usize, type_name: &str) -> serde_json::Value {
    match type_name {
        "bool" => row.get::<_, Option<bool>>(idx).map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null),
        "int2" | "smallint" => row.get::<_, Option<i16>>(idx).map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null),
        "int4" | "integer" => row.get::<_, Option<i32>>(idx).map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null),
        "int8" | "bigint" => row.get::<_, Option<i64>>(idx).map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null),
        "float4" | "real" => row.get::<_, Option<f32>>(idx).map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null),
        "float8" | "double precision" => row.get::<_, Option<f64>>(idx).map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null),
        "numeric" | "decimal" => {
            row.get::<_, Option<String>>(idx)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
        "uuid" => {
            row.get::<_, Option<uuid::Uuid>>(idx)
                .map(|v| serde_json::json!(v.to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        "timestamptz" | "timestamp with time zone" => {
            row.get::<_, Option<chrono::DateTime<chrono::Utc>>>(idx)
                .map(|v| serde_json::json!(v.to_rfc3339()))
                .unwrap_or(serde_json::Value::Null)
        }
        "timestamp" | "timestamp without time zone" => {
            row.get::<_, Option<chrono::NaiveDateTime>>(idx)
                .map(|v| serde_json::json!(v.format("%Y-%m-%dT%H:%M:%S").to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        "date" => {
            row.get::<_, Option<chrono::NaiveDate>>(idx)
                .map(|v| serde_json::json!(v.format("%Y-%m-%d").to_string()))
                .unwrap_or(serde_json::Value::Null)
        }
        "json" | "jsonb" => {
            row.get::<_, Option<serde_json::Value>>(idx)
                .unwrap_or(serde_json::Value::Null)
        }
        "bytea" => {
            row.get::<_, Option<Vec<u8>>>(idx)
                .map(|v| serde_json::json!(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &v)))
                .unwrap_or(serde_json::Value::Null)
        }
        _ => {
            row.try_get::<_, Option<String>>(idx)
                .unwrap_or(None)
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null)
        }
    }
}

async fn execute_via_postgres(
    state: Arc<AppState>,
    bot_id: &uuid::Uuid,
    query: &str,
    page: i64,
    page_size: i64,
    total_rows: i64,
    _is_mutation: bool,
) -> Result<Json<TableDataResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db_url = get_bot_database_url(&state, *bot_id)?;
    let query = query.to_string();

    let result = tokio::task::spawn_blocking(move || execute_via_postgres_sync(&db_url, &query))
        .await
        .map_err(|e| internal_error(&format!("Task join error: {e}")))
        .map_err(|e: (StatusCode, Json<serde_json::Value>)| e)?;

    match result {
        Ok((columns, rows)) => {
            Ok(Json(TableDataResponse {
                columns,
                rows,
                total_rows,
                page,
                page_size,
            }))
        }
        Err(e) => Err(internal_error(&e)),
    }
}

pub async fn execute_query(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<QueryResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;

    let trimmed = payload.query.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(error_response("Query cannot be empty"));
    }

    let is_mutation = trimmed.starts_with("insert")
        || trimmed.starts_with("update")
        || trimmed.starts_with("delete")
        || trimmed.starts_with("drop")
        || trimmed.starts_with("alter")
        || trimmed.starts_with("create")
        || trimmed.starts_with("truncate");

    let semicolon_count = trimmed.matches(';').count();
    if semicolon_count > 1 {
        return Err(error_response("Multiple statements not allowed"));
    }

    let start = std::time::Instant::now();

    if is_mutation {
        let mut conn = get_bot_conn(&state, bot_id)?;
        let affected = diesel::sql_query(&payload.query)
            .execute(&mut conn)
            .map_err(|e| internal_error(&format!("Query execution failed: {e}")))?;

        let duration = start.elapsed().as_millis();
        return Ok(Json(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: affected as i64,
            is_mutation: true,
            duration_ms: duration,
        }));
    }

    let limited_query = if trimmed.starts_with("select") && !trimmed.contains("limit") {
        format!(
            "{} LIMIT {MAX_QUERY_ROWS}",
            payload.query.trim_end_matches(';')
        )
    } else {
        payload.query.clone()
    };

    let db_url = get_bot_database_url(&state, bot_id)?;
    let query_clone = limited_query.clone();

    let pg_result = tokio::task::spawn_blocking(move || execute_via_postgres_sync(&db_url, &query_clone))
        .await
        .map_err(|e| internal_error(&format!("Task join error: {e}")))?;

    let duration = start.elapsed().as_millis();

    match pg_result {
        Ok((columns, rows)) => Ok(Json(QueryResult {
            row_count: rows.len() as i64,
            columns,
            rows,
            is_mutation: false,
            duration_ms: duration,
        })),
        Err(e) => Err(internal_error(&format!("Query failed: {e}"))),
    }
}

pub async fn insert_row(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<InsertRowRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let obj = payload
        .data
        .as_object()
        .ok_or_else(|| error_response("Request body must be a JSON object"))?;

    if obj.is_empty() {
        return Err(error_response("Cannot insert empty row"));
    }

    let columns: Vec<String> = obj.keys().map(|k| sanitize_identifier(k)).collect();
    let values: Vec<String> = obj
        .values()
        .map(|v| match v {
            serde_json::Value::String(s) => {
                format!("'{}'", s.replace('\'', "''"))
            }
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => {
                if *b { "TRUE".to_string() } else { "FALSE".to_string() }
            }
            other => format!("'{}'", other.to_string().replace('\'', "''")),
        })
        .collect();

    let col_list = columns.join(", ");
    let val_list = values.join(", ");

    let sql = format!("INSERT INTO {safe_name} ({col_list}) VALUES ({val_list})");

    diesel::sql_query(sql)
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Insert failed: {e}")))?;

    Ok(Json(ApiResult {
        success: true,
        message: format!("Row inserted into {safe_name}"),
    }))
}

pub async fn delete_row(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let pk_info: Vec<PkInfo> = sql_query(
        "SELECT a.attrelid::regclass::text AS table_name,
                a.attname AS column_name
         FROM pg_index i
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
         WHERE i.indisprimary AND a.attrelid::regclass::text = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&safe_name)
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query primary keys: {e}")))?;

    if pk_info.is_empty() {
        return Err(error_response("Table has no primary key"));
    }

    let pk_col = sanitize_identifier(&pk_info[0].column_name);
    let safe_id = id.replace('\'', "''");

    let affected = diesel::sql_query(format!(
        "DELETE FROM {safe_name} WHERE {pk_col} = '{safe_id}'"
    ))
    .execute(&mut conn)
    .map_err(|e| internal_error(&format!("Delete failed: {e}")))?;

    if affected == 0 {
        return Err(error_response("Row not found"));
    }

    Ok(Json(ApiResult {
        success: true,
        message: format!("Row deleted from {safe_name}"),
    }))
}

pub async fn create_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTableRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&payload.name);
    if safe_name != payload.name || safe_name.is_empty() {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    if payload.columns.is_empty() {
        return Err(error_response("At least one column is required"));
    }

    let col_defs: Vec<String> = payload
        .columns
        .iter()
        .map(|col| {
            let col_name = sanitize_identifier(&col.name);
            let nullable = if col.nullable.unwrap_or(true) { "" } else { " NOT NULL" };
            let default = col
                .default
                .as_ref()
                .map(|d| format!(" DEFAULT {d}"))
                .unwrap_or_default();
            format!("    {col_name} {}{nullable}{default}", col.data_type)
        })
        .collect();

    let sql = format!(
        "CREATE TABLE IF NOT EXISTS {safe_name} (\n{}\n)",
        col_defs.join(",\n")
    );

    diesel::sql_query(sql)
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Create table failed: {e}")))?;

    Ok(Json(ApiResult {
        success: true,
        message: format!("Table {safe_name} created successfully"),
    }))
}

pub async fn alter_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<AlterTableRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;
    let mut messages = Vec::new();

    if let Some(add_cols) = &payload.add_columns {
        for col in add_cols {
            let col_name = sanitize_identifier(&col.name);
            let nullable = if col.nullable.unwrap_or(true) { "" } else { " NOT NULL" };
            let default = col
                .default
                .as_ref()
                .map(|d| format!(" DEFAULT {d}"))
                .unwrap_or_default();

            let sql = format!(
                "ALTER TABLE {safe_name} ADD COLUMN IF NOT EXISTS {col_name} {}{nullable}{default}",
                col.data_type
            );
            diesel::sql_query(sql)
                .execute(&mut conn)
                .map_err(|e| internal_error(&format!("Add column failed: {e}")))?;
            messages.push(format!("Added column {col_name}"));
        }
    }

    if let Some(drop_cols) = &payload.drop_columns {
        for col in drop_cols {
            let col_name = sanitize_identifier(col);
            let sql = format!("ALTER TABLE {safe_name} DROP COLUMN IF EXISTS {col_name}");
            diesel::sql_query(sql)
                .execute(&mut conn)
                .map_err(|e| internal_error(&format!("Drop column failed: {e}")))?;
            messages.push(format!("Dropped column {col_name}"));
        }
    }

    if let Some(new_name) = &payload.rename_to {
        let safe_new = sanitize_identifier(new_name);
        if safe_new != *new_name {
            return Err(error_response("Invalid new table name"));
        }
        let sql = format!("ALTER TABLE {safe_name} RENAME TO {safe_new}");
        diesel::sql_query(sql)
            .execute(&mut conn)
            .map_err(|e| internal_error(&format!("Rename table failed: {e}")))?;
        messages.push(format!("Renamed table to {safe_new}"));
    }

    if messages.is_empty() {
        return Err(error_response("No alter operations specified"));
    }

    Ok(Json(ApiResult {
        success: true,
        message: messages.join("; "),
    }))
}

pub async fn drop_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    diesel::sql_query(format!("DROP TABLE IF EXISTS {safe_name}"))
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Drop table failed: {e}")))?;

    Ok(Json(ApiResult {
        success: true,
        message: format!("Table {safe_name} dropped"),
    }))
}

pub async fn add_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<AddColumnRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let col_name = sanitize_identifier(&payload.name);
    if col_name != payload.name || col_name.is_empty() {
        return Err(error_response("Invalid column name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let nullable = if payload.nullable.unwrap_or(true) { "" } else { " NOT NULL" };
    let default = payload
        .default
        .as_ref()
        .map(|d| format!(" DEFAULT {d}"))
        .unwrap_or_default();

    let sql = format!(
        "ALTER TABLE {safe_name} ADD COLUMN IF NOT EXISTS {col_name} {}{nullable}{default}",
        payload.data_type
    );

    diesel::sql_query(sql)
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Add column failed: {e}")))?;

    Ok(Json(ApiResult {
        success: true,
        message: format!("Column {col_name} added to {safe_name}"),
    }))
}

pub async fn update_row(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<UpdateRowRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let pk_info: Vec<PkInfo> = sql_query(
        "SELECT a.attrelid::regclass::text AS table_name,
                a.attname AS column_name
         FROM pg_index i
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
         WHERE i.indisprimary AND a.attrelid::regclass::text = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&safe_name)
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query primary keys: {e}")))?;

    if pk_info.is_empty() {
        return Err(error_response("Table has no primary key"));
    }

    let pk_col = sanitize_identifier(&pk_info[0].column_name);
    let safe_pk = payload.pk_value.replace('\'', "''");

    let obj = payload
        .data
        .as_object()
        .ok_or_else(|| error_response("Request body must be a JSON object"))?;

    if obj.is_empty() {
        return Err(error_response("No columns to update"));
    }

    let set_clauses: Vec<String> = obj
        .iter()
        .map(|(k, v)| {
            let col = sanitize_identifier(k);
            let val = match v {
                serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                serde_json::Value::Null => "NULL".to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => {
                    if *b { "TRUE".to_string() } else { "FALSE".to_string() }
                }
                other => format!("'{}'", other.to_string().replace('\'', "''")),
            };
            format!("{col} = {val}")
        })
        .collect();

    let sql = format!(
        "UPDATE {safe_name} SET {} WHERE {pk_col} = '{safe_pk}'",
        set_clauses.join(", ")
    );

    let affected = diesel::sql_query(sql)
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Update failed: {e}")))?;

    if affected == 0 {
        return Err(error_response("Row not found"));
    }

    Ok(Json(ApiResult {
        success: true,
        message: format!("Row updated in {safe_name}"),
    }))
}

pub async fn batch_delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<BatchDeleteRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    if payload.ids.is_empty() {
        return Err(error_response("No rows selected for deletion"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let pk_info: Vec<PkInfo> = sql_query(
        "SELECT a.attrelid::regclass::text AS table_name,
                a.attname AS column_name
         FROM pg_index i
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
         WHERE i.indisprimary AND a.attrelid::regclass::text = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&safe_name)
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query primary keys: {e}")))?;

    if pk_info.is_empty() {
        return Err(error_response("Table has no primary key"));
    }

    let pk_col = sanitize_identifier(&pk_info[0].column_name);
    let escaped_ids: Vec<String> = payload
        .ids
        .iter()
        .map(|id| format!("'{}'", id.replace('\'', "''")))
        .collect();

    let sql = format!(
        "DELETE FROM {safe_name} WHERE {pk_col} IN ({})",
        escaped_ids.join(", ")
    );

    let affected = diesel::sql_query(sql)
        .execute(&mut conn)
        .map_err(|e| internal_error(&format!("Batch delete failed: {e}")))?;

    Ok(Json(ApiResult {
        success: true,
        message: format!("{affected} row(s) deleted from {safe_name}"),
    }))
}

pub async fn get_foreign_keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<Vec<ForeignKeyInfo>>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    #[derive(QueryableByName, Debug)]
    struct FkRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        constraint_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        foreign_table: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        foreign_column: String,
    }

    let fks: Vec<FkRow> = sql_query(
        "SELECT tc.constraint_name::text,
                kcu.column_name::text,
                ccu.table_name::text AS foreign_table,
                ccu.column_name::text AS foreign_column
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
           ON tc.constraint_name = kcu.constraint_name
         JOIN information_schema.constraint_column_usage ccu
           ON ccu.constraint_name = tc.constraint_name
         WHERE tc.constraint_type = 'FOREIGN KEY'
           AND tc.table_schema = 'public'
           AND tc.table_name = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&safe_name)
    .get_results(&mut conn)
    .map_err(|e| internal_error(&format!("Failed to query foreign keys: {e}")))?;

    let result: Vec<ForeignKeyInfo> = fks
        .into_iter()
        .map(|fk| ForeignKeyInfo {
            constraint_name: fk.constraint_name,
            column_name: fk.column_name,
            foreign_table: fk.foreign_table,
            foreign_column: fk.foreign_column,
        })
        .collect();

    Ok(Json(result))
}
