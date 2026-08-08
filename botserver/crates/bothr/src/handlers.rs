use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use diesel::RunQueryDsl;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

/// Resolves the caller's tenant branch from the server-minted JWT claims
/// (issue #734). Falls back to the global nil branch so anonymous/system
/// callers keep working, but every query is still constrained by the resolved
/// branch — a tenant can never see another tenant's rows.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

pub async fn list_employees(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
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
         FROM hr_employees WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "email": r.email, "department": r.department,
        "role": r.role, "status": r.status, "hired_at": r.hired_at, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_employee(headers: HeaderMap, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let department = item.get("department").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
    diesel::sql_query(
        "INSERT INTO hr_employees (id, name, email, department, role, status, hired_at, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&department)
    .bind::<diesel::sql_types::Text, _>(&role)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"item": {"id": id, "name": name, "email": email, "department": department, "role": role, "status": "active", "hired_at": now}})))
}

pub async fn update_employee(headers: HeaderMap, Path(id): Path<String>, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let department = item.get("department").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("active").to_string();
    let n = diesel::sql_query(
        "UPDATE hr_employees SET name = $1, email = $2, department = $3, role = $4, status = $5 WHERE id = $6 AND branch_id = $7",
    )
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&department)
    .bind::<diesel::sql_types::Text, _>(&role)
    .bind::<diesel::sql_types::Text, _>(&status)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, "Employee not found".to_string())); }
    Ok(Json(serde_json::json!({"item": {"id": parsed, "name": name, "email": email, "department": department, "role": role, "status": status}})))
}

pub async fn list_recruitment(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
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
         FROM hr_recruitment WHERE branch_id = $1 ORDER BY opened_at DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "position": r.position, "department": r.department,
        "status": r.status, "candidates": r.candidates, "opened_at": r.opened_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_attendance(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
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
         FROM hr_attendance WHERE branch_id = $1 ORDER BY date DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "employee_id": r.employee_id, "date": r.date.to_string(),
        "clock_in": r.clock_in, "clock_out": r.clock_out, "hours_worked": r.hours_worked.to_string(),
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_performance(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Cycle {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Date)] start_date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Date)] end_date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] completed: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] total: i64,
    }
    #[derive(diesel::QueryableByName)]
    struct Goal {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Integer)] completion: i32,
        #[diesel(sql_type = diesel::sql_types::Date)] due_date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Text)] employee: String,
    }
    let cycles: Vec<Cycle> = diesel::sql_query(
        "SELECT id, name, start_date, end_date, status, completed, total
         FROM hr_review_cycles WHERE branch_id = $1 ORDER BY start_date DESC LIMIT 200",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let goals: Vec<Goal> = diesel::sql_query(
        "SELECT g.id, g.title, g.completion, g.due_date, COALESCE(e.name, 'Unassigned') AS employee
         FROM hr_goals g LEFT JOIN hr_employees e ON e.id = g.employee_id
         WHERE g.branch_id = $1 ORDER BY g.due_date ASC LIMIT 200",
    ).bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn).map_err(db::map_diesel_err)?;
    let cycles_json: Vec<serde_json::Value> = cycles.into_iter().map(|c| serde_json::json!({
        "id": c.id, "name": c.name,
        "start_date": c.start_date.to_string(), "end_date": c.end_date.to_string(),
        "status": c.status, "completed": c.completed, "total": c.total,
    })).collect();
    let goals_json: Vec<serde_json::Value> = goals.into_iter().map(|g| serde_json::json!({
        "id": g.id, "title": g.title, "completion": g.completion,
        "due_date": g.due_date.to_string(), "employee": g.employee,
    })).collect();
    Ok(Json(serde_json::json!({
        "review_cycles": cycles_json,
        "goals": goals_json,
    })))
}