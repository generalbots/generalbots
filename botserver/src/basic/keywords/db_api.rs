use super::table_access::{
    check_field_write_access, check_table_access, filter_fields_by_role, AccessType, UserRoles,
};
use crate::core::shared::state::AppState;
use crate::core::shared::sanitize_identifier;
use crate::core::urls::ApiUrls;
use crate::security::error_sanitizer::log_and_sanitize;
use crate::security::sql_guard::{
    build_safe_count_query, build_safe_select_by_id_query, build_safe_select_query,
    is_table_allowed_with_conn, validate_table_name,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use diesel::sql_query;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

fn user_roles_from_headers(headers: &HeaderMap) -> UserRoles {
    let roles = headers
        .get("X-User-Roles")
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            s.split(';')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    if let Some(uid) = user_id {
        UserRoles::with_user_id(roles, uid)
    } else {
        UserRoles::new(roles)
    }
}

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
        .route(ApiUrls::DB_TABLE, get(list_records_handler).post(create_record_handler))
        .route(ApiUrls::DB_TABLE_RECORD, get(get_record_handler).put(update_record_handler).delete(delete_record_handler))
        .route(ApiUrls::DB_TABLE_COUNT, get(count_records_handler))
        .route(ApiUrls::DB_TABLE_SEARCH, post(search_records_handler))
}

pub async fn list_records_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(table): Path<String>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    // Validate table name (basic check - no SQL injection)
    if let Err(e) = validate_table_name(&table_name) {
        warn!("Invalid table name attempted: {} - {}", table_name, e);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid table name" })),
        )
            .into_response();
    }

    let order_by = params.order_by.as_deref();
    let order_dir = params.order_dir.as_deref();

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

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Table '{}' not found", table_name) })),
        )
            .into_response();
    }

    // Check table-level read access
    let access_info =
        match check_table_access(&mut conn, &table_name, &user_roles, AccessType::Read) {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "Access denied to table {} for user {:?}",
                    table_name, user_roles.user_id
                );
                return (StatusCode::FORBIDDEN, Json(json!({ "error": e }))).into_response();
            }
        };

    // Build safe queries using sql_guard
    let query = match build_safe_select_query(&table_name, order_by, order_dir, limit, offset) {
        Ok(q) => q,
        Err(e) => {
            warn!("Failed to build safe query: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid query parameters" })),
            )
                .into_response();
        }
    };

    let count_query = match build_safe_count_query(&table_name) {
        Ok(q) => q,
        Err(e) => {
            warn!("Failed to build count query: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid table name" })),
            )
                .into_response();
        }
    };

    let rows: Result<Vec<JsonRow>, _> = sql_query(&query).get_results(&mut conn);
    let total: Result<CountResult, _> = sql_query(&count_query).get_result(&mut conn);

    match (rows, total) {
        (Ok(data), Ok(count_result)) => {
            // Filter fields based on user roles
            let filtered_data: Vec<Value> = data
                .into_iter()
                .map(|r| filter_fields_by_role(r.data, &user_roles, &access_info))
                .collect();

            let response = ListResponse {
                data: filtered_data,
                total: count_result.count,
                limit,
                offset,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        (Err(e), _) | (_, Err(e)) => {
            let sanitized = log_and_sanitize(&e, &format!("list_records_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub async fn get_record_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((table, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);

    let Ok(record_id) = Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Invalid UUID format".to_string()),
            }),
        )
            .into_response();
    };

    let Ok(mut conn) = state.conn.get() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Database connection error".to_string()),
            }),
        )
            .into_response();
    };

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some(format!("Table '{}' not found", table_name)),
            }),
        )
            .into_response();
    }

    // Check table-level read access
    let access_info =
        match check_table_access(&mut conn, &table_name, &user_roles, AccessType::Read) {
            Ok(info) => info,
            Err(e) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(RecordResponse {
                        success: false,
                        data: None,
                        message: Some(e),
                    }),
                )
                    .into_response();
            }
        };

    let query = match build_safe_select_by_id_query(&table_name) {
        Ok(q) => q,
        Err(e) => {
            warn!("Failed to build safe query for {}: {}", table_name, e);
            return (
                StatusCode::BAD_REQUEST,
                Json(RecordResponse {
                    success: false,
                    data: None,
                    message: Some("Invalid table name".to_string()),
                }),
            )
                .into_response();
        }
    };

    let row: Result<Option<JsonRow>, _> = sql_query(&query)
        .bind::<diesel::sql_types::Uuid, _>(record_id)
        .get_result(&mut conn)
        .optional();

    match row {
        Ok(Some(r)) => {
            // Filter fields based on user roles
            let filtered_data = filter_fields_by_role(r.data, &user_roles, &access_info);
            (
                StatusCode::OK,
                Json(RecordResponse {
                    success: true,
                    data: Some(filtered_data),
                    message: None,
                }),
            )
                .into_response()
        }
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
            let sanitized = log_and_sanitize(&e, &format!("get_record_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub async fn create_record_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(table): Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);

    let Some(obj) = payload.as_object() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Payload must be a JSON object".to_string()),
            }),
        )
            .into_response();
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

    let Ok(mut conn) = state.conn.get() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Database connection error".to_string()),
            }),
        )
            .into_response();
    };

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some(format!("Table '{}' not found", table_name)),
            }),
        )
            .into_response();
    }

    let access_info =
        match check_table_access(&mut conn, &table_name, &user_roles, AccessType::Write) {
            Ok(info) => info,
            Err(e) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(RecordResponse {
                        success: false,
                        data: None,
                        message: Some(e),
                    }),
                )
                    .into_response();
            }
        };

    // Check field-level write access for fields being inserted
    let field_names: Vec<String> = obj
        .keys()
        .map(|k| sanitize_identifier(k))
        .filter(|k| !k.is_empty() && k != "id")
        .collect();
    if let Err(e) = check_field_write_access(&field_names, &user_roles, &access_info) {
        return (
            StatusCode::FORBIDDEN,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some(e),
            }),
        )
            .into_response();
    }

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
            let sanitized = log_and_sanitize(&e, &format!("create_record_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub async fn update_record_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((table, id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);

    let Ok(record_id) = Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Invalid UUID format".to_string()),
            }),
        )
            .into_response();
    };

    let Some(obj) = payload.as_object() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some("Payload must be a JSON object".to_string()),
            }),
        )
            .into_response();
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

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some(format!("Table '{}' not found", table_name)),
            }),
        )
            .into_response();
    }

    // Check table-level write access
    let access_info =
        match check_table_access(&mut conn, &table_name, &user_roles, AccessType::Write) {
            Ok(info) => info,
            Err(e) => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(RecordResponse {
                        success: false,
                        data: None,
                        message: Some(e),
                    }),
                )
                    .into_response();
            }
        };

    // Check field-level write access for fields being updated
    let field_names: Vec<String> = obj
        .keys()
        .map(|k| sanitize_identifier(k))
        .filter(|k| !k.is_empty() && k != "id")
        .collect();
    if let Err(e) = check_field_write_access(&field_names, &user_roles, &access_info) {
        return (
            StatusCode::FORBIDDEN,
            Json(RecordResponse {
                success: false,
                data: None,
                message: Some(e),
            }),
        )
            .into_response();
    }

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
            let sanitized = log_and_sanitize(&e, &format!("update_record_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub async fn delete_record_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((table, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);

    let Ok(record_id) = Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(DeleteResponse {
                success: false,
                deleted: 0,
                message: Some("Invalid UUID format".to_string()),
            }),
        )
            .into_response();
    };

    let Ok(mut conn) = state.conn.get() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DeleteResponse {
                success: false,
                deleted: 0,
                message: Some("Database connection error".to_string()),
            }),
        )
            .into_response();
    };

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(DeleteResponse {
                success: false,
                deleted: 0,
                message: Some(format!("Table '{}' not found", table_name)),
            }),
        )
            .into_response();
    }

    if let Err(e) = check_table_access(&mut conn, &table_name, &user_roles, AccessType::Write) {
        return (
            StatusCode::FORBIDDEN,
            Json(DeleteResponse {
                success: false,
                deleted: 0,
                message: Some(e),
            }),
        )
            .into_response();
    }

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
            let sanitized = log_and_sanitize(&e, &format!("delete_record_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub async fn count_records_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(table): Path<String>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);

    // Validate table name (basic check - no SQL injection)
    if let Err(e) = validate_table_name(&table_name) {
        warn!("Invalid table name attempted: {} - {}", table_name, e);
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid table name" })),
        )
            .into_response();
    }

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            let sanitized = log_and_sanitize(&e, "count_records_db_connection", None);
            return (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    };

    // Check if table actually exists in database (supports dynamic tables from app_generator)
    if !is_table_allowed_with_conn(&mut conn, &table_name) {
        warn!("Table not found in database: {}", table_name);
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Table '{}' not found", table_name) })),
        )
            .into_response();
    }

    // Check table-level read access (count requires read permission)
    if let Err(e) = check_table_access(&mut conn, &table_name, &user_roles, AccessType::Read) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": e }))).into_response();
    }

    let query = match build_safe_count_query(&table_name) {
        Ok(q) => q,
        Err(e) => {
            warn!("Failed to build count query: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid table name" })),
            )
                .into_response();
        }
    };
    let result: Result<CountResult, _> = sql_query(&query).get_result(&mut conn);

    match result {
        Ok(r) => (StatusCode::OK, Json(json!({ "count": r.count }))).into_response(),
        Err(e) => {
            let sanitized = log_and_sanitize(&e, &format!("count_records_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
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
    headers: HeaderMap,
    Path(table): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    let table_name = sanitize_identifier(&table);
    let user_roles = user_roles_from_headers(&headers);
    let limit = payload.limit.unwrap_or(20).min(100);
    let search_term = payload.query.replace('\'', "''");

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            let sanitized = log_and_sanitize(&e, "search_records_db_connection", None);
            return (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    };

    // Check table-level read access
    let access_info =
        match check_table_access(&mut conn, &table_name, &user_roles, AccessType::Read) {
            Ok(info) => info,
            Err(e) => {
                return (StatusCode::FORBIDDEN, Json(json!({ "error": e }))).into_response();
            }
        };

    let safe_search = search_term.replace('%', "\\%").replace('_', "\\_");

    let query = format!(
        "SELECT row_to_json(t.*) as data FROM {} t WHERE
         COALESCE(t.title::text, '') || ' ' || COALESCE(t.name::text, '') || ' ' || COALESCE(t.description::text, '')
         ILIKE '%' || $1 || '%' LIMIT {}",
        table_name, limit
    );

    let rows: Result<Vec<JsonRow>, _> = sql_query(&query)
        .bind::<diesel::sql_types::Text, _>(&safe_search)
        .get_results(&mut conn);

    match rows {
        Ok(data) => {
            // Filter fields based on user roles
            let filtered_data: Vec<Value> = data
                .into_iter()
                .map(|r| filter_fields_by_role(r.data, &user_roles, &access_info))
                .collect();
            (StatusCode::OK, Json(json!({ "data": filtered_data }))).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize(&e, &format!("search_records_{}", table_name), None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized)
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
