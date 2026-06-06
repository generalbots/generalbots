use axum::extract::Json;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Deserialize)]
pub struct SqlQuery {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub elapsed_ms: u64,
}

pub async fn list_schemas() -> Result<Json<Vec<TableSchema>>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct TableRow {
        #[diesel(sql_type = diesel::sql_types::Text)] table_name: String,
    }
    let tables: Vec<TableRow> = diesel::sql_query(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public' ORDER BY table_name ASC",
    )
    .get_results(&mut conn)
    .map_err(db::map_diesel_err)?;
    let mut schemas = Vec::new();
    for table in tables {
        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)] count: i64,
        }
        let count: CountRow = diesel::sql_query(&format!(
            "SELECT COUNT(*) AS count FROM {}",
            sanitize_table_name(&table.table_name)?
        ))
        .get_result(&mut conn)
        .map_err(db::map_diesel_err)?;
        #[derive(diesel::QueryableByName)]
        struct ColRow {
            #[diesel(sql_type = diesel::sql_types::Text)] column_name: String,
            #[diesel(sql_type = diesel::sql_types::Text)] data_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)] is_nullable: String,
        }
        let cols: Vec<ColRow> = diesel::sql_query(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = $1
             ORDER BY ordinal_position ASC",
        )
        .bind::<diesel::sql_types::Text, _>(&table.table_name)
        .get_results(&mut conn)
        .map_err(db::map_diesel_err)?;
        schemas.push(TableSchema {
            table_name: table.table_name,
            row_count: count.count,
            columns: cols.into_iter().map(|c| ColumnInfo {
                name: c.column_name,
                data_type: c.data_type,
                nullable: c.is_nullable == "YES",
                is_primary_key: false,
            }).collect(),
        });
    }
    Ok(Json(schemas))
}

fn sanitize_table_name(name: &str) -> Result<String, (StatusCode, String)> {
    if name.is_empty() || name.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Invalid table name".to_string()));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err((StatusCode::BAD_REQUEST, "Invalid table name characters".to_string()));
    }
    Ok(name.to_string())
}

pub async fn execute_query(Json(req): Json<SqlQuery>) -> Result<Json<QueryResult>, (StatusCode, String)> {
    let trimmed = req.sql.trim();
    let upper = trimmed.to_uppercase();
    if !upper.starts_with("SELECT") {
        return Err((StatusCode::FORBIDDEN, "Only SELECT statements are allowed".to_string()));
    }
    let pool = db::pool()?;
    let start = std::time::Instant::now();
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct JsonRow {
        #[diesel(sql_type = diesel::sql_types::Jsonb)] data: serde_json::Value,
    }
    let wrapped = format!("SELECT to_jsonb(t.*) AS data FROM ({trimmed}) t");
    let result: Result<Vec<JsonRow>, _> = diesel::sql_query(&wrapped).get_results(&mut conn);
    let rows = match result {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Query failed: {e}. Wrap multi-column queries in a subquery: SELECT * FROM (your_query) t"),
            ));
        }
    };
    let elapsed = start.elapsed().as_millis() as u64;
    let mut out_rows = Vec::with_capacity(rows.len());
    for row in rows {
        match row.data {
            serde_json::Value::Array(arr) => {
                out_rows.push(arr.into_iter().map(|v| v).collect());
            }
            other => {
                out_rows.push(vec![other]);
            }
        }
    }
    Ok(Json(QueryResult {
        columns: vec!["data".to_string()],
        rows: out_rows,
        row_count: rows.len(),
        elapsed_ms: elapsed,
    }))
}

pub async fn list_tables() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] table_name: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public' ORDER BY table_name ASC",
    )
    .get_results(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| r.table_name).collect()))
}
