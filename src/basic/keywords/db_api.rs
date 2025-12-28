use crate::core::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use diesel::prelude::*;
use diesel::sql_query;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub order_by: Option<String>,
    pub order_dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub data: Vec<Value>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub deleted: i64,
    pub message: Option<String>,
}

pub fn configure_db_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/db/{table}", get(list_records_handler))
        .route("/api/db/{table}", post(create_record_handler))
        .route("/api/db/{table}/{id}", get(get_record_handler))
        .route("/api/db/{table}/{id}", put(update_record_handler))
        .route("/api/db/{table}/{id}", delete(delete_record_handler))
        .route("/api/db/{table}/count", get(count_records_handler))
        .route("/api/db/{table}/search", post(search_records_handler))
}

fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub async fn list_records_handler(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let order_by = params
        .order_by
        .map(|o| sanitize_identifier(&o))
        .unwrap_or_else(|| "id".to_string());
    let order_dir = params
        .order_dir
        .map(|d| {
            if d.to_uppercase() == "DESC" {
                "DESC"
            } else {
                "ASC"
            }
        })
        .unwrap_or("ASC");

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database connection error: {e}") })),
            )
                .into_response()
        }
    };

    let query = format!(
        "SELECT row_to_json(t.*) as data FROM {} t ORDER BY {} {} LIMIT {} OFFSET {}",
        table_name, order_by, order_dir, limit, offset
    );

    let count_query = format!("SELECT COUNT(*) as count FROM {}", table_name);

    let rows: Result<Vec<JsonRow>, _> = sql_query(&query).get_results(&mut conn);
    let total: Result<CountResult, _> = sql_query(&count_query).get_result(&mut conn);

    match (rows, total) {
        (Ok(data), Ok(count_result)) => {
            let response = ListResponse {
                data: data.into_iter().map(|r| r.data).collect(),
                total: count_result.count,
                limit,
                offset,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Failed to list records from {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn get_record_handler(
    State(state): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);

    let record_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some("Invalid UUID format".to_string()),
                }),
            )
                .into_response()
        }
    };

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(format!("Database connection error: {e}")),
                }),
            )
                .into_response()
        }
    };

    let query = format!(
        "SELECT row_to_json(t.*) as data FROM {} t WHERE id = $1",
        table_name
    );

    let row: Result<Option<JsonRow>, _> = sql_query(&query)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .get_result(&mut conn)
        .optional();

    match row {
        Ok(Some(r)) => (
            StatusCode::OK,
            Json(RecordResponse {
                success: true,
                data: Some(r.data),
                message: None,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Record not found".to_string()),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get record from {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

pub async fn create_record_handler(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);

    let obj = match payload.as_object() {
        Some(o) => o,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some("Payload must be a JSON object".to_string()),
                }),
            )
                .into_response()
        }
    };

    let mut columns: Vec<String> = vec!["id".to_string()];
    let mut values: Vec<String> = vec![format!("'{}'", Uuid::new_v4())];

    for (key, value) in obj {
        let col = sanitize_identifier(key);
        if col.is_empty() || col == "id" {
            continue;
        }
        columns.push(col);
        values.push(value_to_sql(value));
    }

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(format!("Database connection error: {e}")),
                }),
            )
                .into_response()
        }
    };

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING row_to_json({}.*)::jsonb as data",
        table_name,
        columns.join(", "),
        values.join(", "),
        table_name
    );

    let row: Result<JsonRow, _> = sql_query(&query).get_result(&mut conn);

    match row {
        Ok(r) => {
            info!("Created record in {table_name}");
            (
                StatusCode::CREATED,
                Json(RecordResponse {
                    success: true,
                    data: Some(r.data),
                    message: None,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to create record in {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

pub async fn update_record_handler(
    State(state): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);

    let record_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some("Invalid UUID format".to_string()),
                }),
            )
                .into_response()
        }
    };

    let obj = match payload.as_object() {
        Some(o) => o,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some("Payload must be a JSON object".to_string()),
                }),
            )
                .into_response()
        }
    };

    let mut set_clauses: Vec<String> = Vec::new();

    for (key, value) in obj {
        let col = sanitize_identifier(key);
        if col.is_empty() || col == "id" {
            continue;
        }
        set_clauses.push(format!("{} = {}", col, value_to_sql(value)));
    }

    if set_clauses.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("No valid fields to update".to_string()),
            }),
        )
            .into_response();
    }

    set_clauses.push("updated_at = NOW()".to_string());

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(format!("Database connection error: {e}")),
                }),
            )
                .into_response()
        }
    };

    let query = format!(
        "UPDATE {} SET {} WHERE id = '{}' RETURNING row_to_json({}.*)::jsonb as data",
        table_name,
        set_clauses.join(", "),
        record_id,
        table_name
    );

    let row: Result<JsonRow, _> = sql_query(&query).get_result(&mut conn);

    match row {
        Ok(r) => {
            info!("Updated record in {table_name}: {record_id}");
            (
                StatusCode::OK,
                Json(RecordResponse {
                    success: true,
                    data: Some(r.data),
                    message: None,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to update record in {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

pub async fn delete_record_handler(
    State(state): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);

    let record_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DeleteResponse {
                    success: false,
                    deleted: 0,
                    message: Some("Invalid UUID format".to_string()),
                }),
            )
                .into_response()
        }
    };

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    deleted: 0,
                    message: Some(format!("Database connection error: {e}")),
                }),
            )
                .into_response()
        }
    };

    let query = format!("DELETE FROM {} WHERE id = $1", table_name);

    let deleted: Result<usize, _> = sql_query(&query)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .execute(&mut conn);

    match deleted {
        Ok(count) => {
            info!("Deleted {count} record(s) from {table_name}");
            (
                StatusCode::OK,
                Json(DeleteResponse {
                    success: count > 0,
                    deleted: count as i64,
                    message: if count == 0 {
                        Some("Record not found".to_string())
                    } else {
                        None
                    },
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to delete record from {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeleteResponse {
                    success: false,
                    deleted: 0,
                    message: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

pub async fn count_records_handler(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database connection error: {e}") })),
            )
                .into_response()
        }
    };

    let query = format!("SELECT COUNT(*) as count FROM {}", table_name);
    let result: Result<CountResult, _> = sql_query(&query).get_result(&mut conn);

    match result {
        Ok(r) => (StatusCode::OK, Json(json!({ "count": r.count }))).into_response(),
        Err(e) => {
            error!("Failed to count records in {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub fields: Option<Vec<String>>,
    pub limit: Option<i32>,
}

pub async fn search_records_handler(
    State(state): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let limit = payload.limit.unwrap_or(20).min(100);
    let search_term = payload.query.replace('\'', "''");

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database connection error: {e}") })),
            )
                .into_response()
        }
    };

    let query = format!(
        "SELECT row_to_json(t.*) as data FROM {} t WHERE
         COALESCE(t.title::text, '') || ' ' || COALESCE(t.name::text, '') || ' ' || COALESCE(t.description::text, '')
         ILIKE '%{}%' LIMIT {}",
        table_name, search_term, limit
    );

    let rows: Result<Vec<JsonRow>, _> = sql_query(&query).get_results(&mut conn);

    match rows {
        Ok(data) => (
            StatusCode::OK,
            Json(json!({ "data": data.into_iter().map(|r| r.data).collect::<Vec<_>>() })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to search in {table_name}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

fn value_to_sql(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        Value::Array(_) | Value::Object(_) => {
            format!("'{}'", value.to_string().replace('\'', "''"))
        }
    }
}

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    data: Value,
}

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}
