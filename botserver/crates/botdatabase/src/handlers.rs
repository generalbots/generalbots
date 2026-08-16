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
    pub limit: Option<i64>,
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
    #[serde(default)]
    pub table_name: Option<String>,
}

#[derive(Deserialize)]
pub struct ColumnDef {
    pub name: String,
    #[serde(default)]
    pub data_type: Option<String>,
    #[serde(default, alias = "type")]
    pub col_type: Option<String>,
    #[serde(default)]
    pub nullable: Option<bool>,
    #[serde(default)]
    pub not_null: Option<bool>,
    #[serde(default)]
    pub primary_key: Option<bool>,
    #[serde(default)]
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
    #[serde(default)]
    pub data_type: Option<String>,
    #[serde(default, alias = "type")]
    pub col_type: Option<String>,
    #[serde(default)]
    pub nullable: Option<bool>,
    #[serde(default)]
    pub not_null: Option<bool>,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Deserialize)]
pub struct InsertRowRequest {
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateCellRequest {
    pub column: String,
    pub value: Option<serde_json::Value>,
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
    pub column_count: usize,
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
    pub rows: Vec<serde_json::Value>,
    pub total: i64,
    pub total_rows: i64,
    pub page: i64,
    pub page_size: i64,
    pub pk_column: Option<String>,
}

#[derive(Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
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
    if let Some(v) = headers.get("X-Bot-Id").and_then(|v| v.to_str().ok()) {
        let bot_id = uuid::Uuid::parse_str(v)
            .map_err(|_| error_response("Invalid X-Bot-Id header"))?;
        authorize_bot_access(headers, bot_id)?;
        return Ok(bot_id);
    }

    // Fall back to the default bot when no header is provided so the
    // database app works without an explicit bot context.
    let pool = db::pool().map_err(|(code, msg)| (code, Json(serde_json::json!({"error": msg}))))?;
    let mut conn = pool
        .get()
        .map_err(|e| internal_error(&format!("Main DB connection error: {e}")))?;
    let (bot_id, _) = botcore::bot::get_default_bot(&mut conn);
    Ok(bot_id)
}

/// Tenant authorization for a client-supplied bot id (issue #850, following
/// the #734 pattern in botbanking::cashflow::resolve_bot_scope): a caller may
/// only address a bot that belongs to their workspace branch. The caller's
/// branch is resolved exclusively from the server-minted JWT `branch_id`
/// claim — never from client input. Callers without a branch claim
/// (global/super-admin tokens) and the legacy nil branch are unrestricted.
fn authorize_bot_access(
    headers: &HeaderMap,
    bot_id: uuid::Uuid,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let caller_branch = match botcore::shared::tenant::branch_from_claims(headers) {
        Some(branch) if branch != uuid::Uuid::nil() => branch,
        _ => return Ok(()),
    };

    let pool = db::pool().map_err(|(code, msg)| (code, Json(serde_json::json!({"error": msg}))))?;
    let mut conn = pool
        .get()
        .map_err(|e| internal_error(&format!("Main DB connection error: {e}")))?;

    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: uuid::Uuid,
    }
    let row: Option<BotRow> = diesel::sql_query(
        "SELECT branch_id FROM bots WHERE id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result(&mut conn)
    .ok();

    match row {
        Some(r) if r.branch_id == caller_branch => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Bot not accessible in this workspace"})),
        )),
        None => Err(error_response("Bot not found or inactive")),
    }
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
    let conn = pool
        .get()
        .map_err(|e| internal_error(&format!("Database connection error: {e}")))?;

    // Always resolve to the bot's OWN database. Previously an empty per-bot
    // database fell back to the platform's main (botserver) database, which
    // leaked the internal schema (bots, messages, __diesel_schema_migrations,
    // …) into the bot-facing DB dialog. An empty bot database now shows an
    // empty schema instead.
    Ok(conn)
}

/// Process-lifetime cache of the resolved per-bot database URL, so the
/// `bots.database_name` lookup does not run on every schema/table/query
/// request. The resolved name is stable within a process lifetime, so a
/// static cache is safe here.
fn bot_db_url_cache() -> &'static std::sync::Mutex<std::collections::HashMap<uuid::Uuid, String>> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<uuid::Uuid, String>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

async fn get_bot_database_url(
    state: &AppState,
    bot_id: uuid::Uuid,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    if let Some(cached) = bot_db_url_cache().lock().map(|c| c.get(&bot_id).cloned()).unwrap_or(None) {
        return Ok(cached);
    }

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

    // The bot's OWN database is always used. Falling back to the platform's
    // main database here (as was previously done for empty per-bot databases)
    // leaked the internal botserver schema into the DB dialog.
    let base_url = state.database_url.clone();
    let base = base_url
        .rfind('/')
        .map(|pos| &base_url[..pos])
        .unwrap_or(&base_url);
    let bot_db_url = format!("{base}/{db_name}");

    if let Ok(mut cache) = bot_db_url_cache().lock() {
        cache.insert(bot_id, bot_db_url.clone());
    }
    Ok(bot_db_url)
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

        let column_count = table_columns.len();
        result_tables.push(TableSchema {
            name: table.table_name.clone(),
            columns: table_columns,
            column_count,
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
    let page_size = params
        .page_size
        .or(params.limit)
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .min(1000);
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

    let pk_column: Option<String> = sql_query(
        "SELECT a.attname AS column_name
         FROM pg_index i
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
         WHERE i.indisprimary AND a.attrelid::regclass::text = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&safe_name)
    .get_results::<PkInfo>(&mut conn)
    .ok()
    .and_then(|rows| rows.first().map(|r| r.column_name.clone()));

    if col_names.is_empty() {
        return Ok(Json(TableDataResponse {
            columns: Vec::new(),
            rows: Vec::new(),
            total: count_result.count,
            total_rows: count_result.count,
            page,
            page_size,
            pk_column,
        }));
    }

    let cast_cols: Vec<String> = col_names
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}::text AS c{}", sanitize_identifier(c), i + 1))
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
            return execute_via_postgres(state, &bot_id, &query, page, page_size, count_result.count, pk_column).await;
        }
    };

    let rows: Vec<serde_json::Value> = raw_text
        .into_iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col) in col_names.iter().enumerate() {
                obj.insert(col.clone(), parse_cell(&row[i]));
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    Ok(Json(TableDataResponse {
        columns: col_names,
        rows,
        total: count_result.count,
        total_rows: count_result.count,
        page,
        page_size,
        pk_column,
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

/// Process-lifetime cache of idle per-bot postgres connections, so the SQL
/// runner reuses a connection instead of opening a fresh TCP connection
/// (plus auth handshake) for every query. Keyed by the resolved per-bot
/// database URL, which is stable within a process lifetime. `postgres::Client`
/// is `Send` but not `Sync`, so connections are moved in/out of the stash
/// rather than shared.
fn postgres_pool_stash() -> &'static std::sync::Mutex<
    std::collections::HashMap<String, Vec<postgres::Client>>,
> {
    static STASH: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Vec<postgres::Client>>>,
    > = std::sync::OnceLock::new();
    STASH.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn checkout_postgres_client(db_url: &str) -> Option<postgres::Client> {
    postgres_pool_stash()
        .lock()
        .ok()
        .and_then(|mut stash| stash.get_mut(db_url).and_then(|conns| conns.pop()))
}

fn checkin_postgres_client(db_url: &str, client: postgres::Client) {
    if let Ok(mut stash) = postgres_pool_stash().lock() {
        let conns = stash.entry(db_url.to_string()).or_default();
        // Cap idle connections per database to avoid unbounded growth.
        if conns.len() < 8 {
            conns.push(client);
        }
    }
}

fn execute_via_postgres_sync(
    db_url: &str,
    query: &str,
) -> Result<(Vec<String>, Vec<serde_json::Value>), String> {
    let mut client = match checkout_postgres_client(db_url) {
        Some(c) => c,
        None => postgres::Client::connect(db_url, postgres::NoTls)
            .map_err(|e| format!("Connection failed: {e}"))?,
    };

    // Use the simple (text) query protocol: every value arrives as a String,
    // so NUMERIC/arrays/enums and any other unmapped type degrade to text
    // instead of a binary deserialization panic. This is the only panic-free
    // path for a generic SQL runner over an arbitrary schema.
    let result = client
        .simple_query(query)
        .map_err(|e| format!("Query failed: {e}"))
        .map(|messages| {
            let mut columns: Vec<String> = Vec::new();
            for msg in &messages {
                if let postgres::SimpleQueryMessage::RowDescription(cols) = msg {
                    columns = cols.iter().map(|c| c.name().to_string()).collect();
                    break;
                }
            }

            let mut rows: Vec<serde_json::Value> = Vec::new();
            for msg in &messages {
                if let postgres::SimpleQueryMessage::Row(row) = msg {
                    let mut obj = serde_json::Map::new();
                    for (i, col) in columns.iter().enumerate() {
                        let value = row.try_get(i).unwrap_or(None).map(str::to_string);
                        obj.insert(col.clone(), parse_cell(&value));
                    }
                    rows.push(serde_json::Value::Object(obj));
                }
            }

            (columns, rows)
        });

    // A successful query means the connection is still healthy — return it to
    // the idle stash for reuse. On error the connection state is unknown, so
    // it is dropped (a fresh one is opened on the next request).
    match result {
        Ok(pair) => {
            checkin_postgres_client(db_url, client);
            Ok(pair)
        }
        Err(e) => Err(e),
    }
}

async fn execute_via_postgres(
    state: Arc<AppState>,
    bot_id: &uuid::Uuid,
    query: &str,
    page: i64,
    page_size: i64,
    total_rows: i64,
    pk_column: Option<String>,
) -> Result<Json<TableDataResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db_url = get_bot_database_url(&state, *bot_id).await?;
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
                total: total_rows,
                total_rows,
                page,
                page_size,
                pk_column,
            }))
        }
        Err(e) => Err(internal_error(&e)),
    }
}

/// Removes leading SQL comments (`-- ...` and `/* ... */`) so the statement
/// classifier inspects the real first keyword instead of being spoofed by a
/// comment prefix (e.g. `/* x */ DROP TABLE users`).
fn strip_leading_comments(sql: &str) -> &str {
    let mut s = sql.trim_start();
    loop {
        if s.starts_with("--") {
            match s.find('\n') {
                Some(i) => s = s[i + 1..].trim_start(),
                None => return "",
            }
        } else if s.starts_with("/*") {
            match s.find("*/") {
                Some(i) => s = s[i + 2..].trim_start(),
                None => return "",
            }
        } else {
            return s;
        }
    }
}

/// Returns true when the (lowercased, comment-stripped) query wraps a DML/DDL
/// statement inside a `WITH` common table expression (e.g. a data-modifying CTE).
fn contains_dml_keyword(sql: &str) -> bool {
    ["insert", "update", "delete", "drop", "alter", "create", "truncate"]
        .iter()
        .any(|kw| sql.split_whitespace().any(|w| w == *kw))
}

pub async fn execute_query(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<QueryResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;

    let stripped = strip_leading_comments(&payload.query);
    let trimmed = stripped.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(error_response("Query cannot be empty"));
    }

    // Reject multiple statements: a single trailing semicolon is allowed, but
    // any content after a semicolon (a second statement) is rejected.
    let body = trimmed.trim_end();
    if let Some(idx) = body.find(';') {
        if !body[idx + 1..].trim().is_empty() {
            return Err(error_response("Multiple statements not allowed"));
        }
    }

    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    let is_mutation = matches!(
        first_word,
        "insert" | "update" | "delete" | "drop" | "alter" | "create" | "truncate"
    ) || (first_word == "with" && contains_dml_keyword(&trimmed));

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

    let db_url = get_bot_database_url(&state, bot_id).await?;
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
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let mut conn = get_bot_conn(&state, bot_id)?;

    let obj = match payload {
        serde_json::Value::Object(map) => {
            if map.contains_key("data") {
                map.get("data")
                    .and_then(|d| d.as_object())
                    .cloned()
                    .ok_or_else(|| error_response("Request body must be a JSON object"))?
            } else {
                map
            }
        }
        _ => return Err(error_response("Request body must be a JSON object")),
    };

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
    let table_name = payload.table_name.clone().unwrap_or_else(|| payload.name.clone());
    let safe_name = sanitize_identifier(&table_name);
    if safe_name != table_name || safe_name.is_empty() {
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
            let data_type = col
                .data_type
                .clone()
                .or_else(|| col.col_type.clone())
                .ok_or_else(|| error_response("Missing column data type"))?;
            let nullable = col
                .nullable
                .or_else(|| col.not_null.map(|n| !n))
                .unwrap_or(true);
            let null_clause = if nullable { "" } else { " NOT NULL" };
            let pk_clause = if col.primary_key.unwrap_or(false) { " PRIMARY KEY" } else { "" };
            let default = col
                .default
                .as_ref()
                .map(|d| format!(" DEFAULT {d}"))
                .unwrap_or_default();
            Ok::<String, (StatusCode, Json<serde_json::Value>)>(format!(
                "    {col_name} {data_type}{null_clause}{pk_clause}{default}"
            ))
        })
        .collect::<Result<Vec<String>, _>>()?;

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
            let data_type = col
                .data_type
                .clone()
                .or_else(|| col.col_type.clone())
                .ok_or_else(|| error_response("Missing column data type"))?;
            let nullable = col
                .nullable
                .or_else(|| col.not_null.map(|n| !n))
                .unwrap_or(true);
            let null_clause = if nullable { "" } else { " NOT NULL" };
            let default = col
                .default
                .as_ref()
                .map(|d| format!(" DEFAULT {d}"))
                .unwrap_or_default();

            let sql = format!(
                "ALTER TABLE {safe_name} ADD COLUMN IF NOT EXISTS {col_name} {data_type}{null_clause}{default}",
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

    let data_type = payload
        .data_type
        .clone()
        .or_else(|| payload.col_type.clone())
        .ok_or_else(|| error_response("Missing column data type"))?;

    let mut conn = get_bot_conn(&state, bot_id)?;

    let nullable = payload
        .nullable
        .or_else(|| payload.not_null.map(|n| !n))
        .unwrap_or(true);
    let null_clause = if nullable { "" } else { " NOT NULL" };
    let default = payload
        .default
        .as_ref()
        .map(|d| format!(" DEFAULT {d}"))
        .unwrap_or_default();

    let sql = format!(
        "ALTER TABLE {safe_name} ADD COLUMN IF NOT EXISTS {col_name} {data_type}{null_clause}{default}",
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

fn json_literal(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => {
            if *b { "TRUE".to_string() } else { "FALSE".to_string() }
        }
        other => format!("'{}'", other.to_string().replace('\'', "''")),
    }
}

pub async fn update_row_by_pk(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((name, id)): Path<(String, String)>,
    Json(payload): Json<UpdateCellRequest>,
) -> Result<Json<ApiResult>, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let column = sanitize_identifier(&payload.column);
    if column != payload.column || column.is_empty() {
        return Err(error_response("Invalid column name"));
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
    let value = payload.value.as_ref().map(json_literal).unwrap_or_else(|| "NULL".to_string());

    let sql = format!(
        "UPDATE {safe_name} SET {column} = {value} WHERE {pk_col} = '{safe_id}'"
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

pub async fn export_table_csv(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let bot_id = extract_bot_id(&headers)?;
    let safe_name = sanitize_identifier(&name);
    if safe_name != name {
        return Err(error_response("Invalid table name"));
    }

    let db_url = get_bot_database_url(&state, bot_id).await?;
    let query = format!("SELECT * FROM {safe_name}");

    let result = tokio::task::spawn_blocking(move || execute_via_postgres_sync(&db_url, &query))
        .await
        .map_err(|e| internal_error(&format!("Task join error: {e}")))
        .map_err(|e: (StatusCode, Json<serde_json::Value>)| e)?
        .map_err(|e| internal_error(&format!("Export failed: {e}")))?;

    let (columns, rows) = result;

    let mut csv = String::new();
    csv.push_str(&columns.join(","));
    csv.push('\n');

    for row in rows {
        let fields: Vec<String> = columns
            .iter()
            .map(|col| {
                let value = row
                    .get(col)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                csv_escape(&value)
            })
            .collect();
        csv.push_str(&fields.join(","));
        csv.push('\n');
    }

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/csv")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{safe_name}.csv\""),
        )
        .body(axum::body::Body::from(csv))
        .map_err(|e| internal_error(&format!("Export failed: {e}")))?)
}

fn csv_escape(value: &serde_json::Value) -> String {
    let text = match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    };
    if text.contains(',') || text.contains('"') || text.contains('\n') || text.contains('\r') {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text
    }
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
