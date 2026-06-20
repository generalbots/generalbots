use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use diesel::RunQueryDsl;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

pub async fn list_employees() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] email: String,
        #[diesel(sql_type = diesel::sql_types::Text)] department: String,
        #[diesel(sql_type = diesel::sql_types::Text)] role: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] hired_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, email, department, role, status, hired_at, created_at
         FROM hr_employees ORDER BY name ASC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "email": r.email, "department": r.department,
        "role": r.role, "status": r.status, "hired_at": r.hired_at, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_employee(Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let department = item.get("department").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
    diesel::sql_query(
        "INSERT INTO hr_employees (id, name, email, department, role, status, hired_at, created_at)
         VALUES ($1, $2, $3, $4, $5, 'active', $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&department)
    .bind::<diesel::sql_types::Text, _>(&role)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"item": {"id": id, "name": name, "email": email, "department": department, "role": role, "status": "active", "hired_at": now}})))
}

pub async fn update_employee(Path(id): Path<String>, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let department = item.get("department").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("active").to_string();
    let n = diesel::sql_query(
        "UPDATE hr_employees SET name = $1, email = $2, department = $3, role = $4, status = $5 WHERE id = $6",
    )
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&department)
    .bind::<diesel::sql_types::Text, _>(&role)
    .bind::<diesel::sql_types::Text, _>(&status)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, "Employee not found".to_string())); }
    Ok(Json(serde_json::json!({"item": {"id": parsed, "name": name, "email": email, "department": department, "role": role, "status": status}})))
}

pub async fn list_recruitment() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] position: String,
        #[diesel(sql_type = diesel::sql_types::Text)] department: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] candidates: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] opened_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, position, department, status, candidates, opened_at
         FROM hr_recruitment ORDER BY opened_at DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "position": r.position, "department": r.department,
        "status": r.status, "candidates": r.candidates, "opened_at": r.opened_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_attendance() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] employee_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Date)] date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] clock_in: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] clock_out: Option<chrono::DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Numeric)] hours_worked: rust_decimal::Decimal,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, employee_id, date, clock_in, clock_out, hours_worked
         FROM hr_attendance ORDER BY date DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "employee_id": r.employee_id, "date": r.date.to_string(),
        "clock_in": r.clock_in, "clock_out": r.clock_out, "hours_worked": r.hours_worked.to_string(),
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}
