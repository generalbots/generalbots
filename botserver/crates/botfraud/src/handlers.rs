use crate::engine::FraudEngine;
use crate::types::*;
use axum::{
    extract::{State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use base64::Engine as _;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct FraudState {
    pub pool: DbPool,
    pub engine: FraudEngine,
}

impl FraudState {
    pub fn new(pool: DbPool) -> Self {
        let engine = FraudEngine::new(pool.clone());
        Self { pool, engine }
    }
}

/// Resolves the caller's tenant branch from the JWT/session claims, never from
/// client-supplied query params (issue #734). Falls back to the global nil
/// branch for anonymous/internal callers so system-wide operations still work.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    let header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")));

    let Some(header) = header else {
        return Uuid::nil();
    };
    let Some(payload) = header.split('.').nth(1) else {
        return Uuid::nil();
    };
    // JWT payloads are base64url without padding, but some issuers emit trailing
    // padding; strip it so URL_SAFE_NO_PAD can always decode.
    let unpadded: String = payload.trim_end_matches('=').to_string();
    let Ok(b64) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(unpadded) else {
        return Uuid::nil();
    };
    let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&b64) else {
        return Uuid::nil();
    };

    claims
        .get("branch_id")
        .or_else(|| claims.get("org_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil())
}

pub fn configure_fraud_routes() -> Router<Arc<FraudState>> {
    Router::new()
        .route("/api/fraud/assess", axum::routing::post(assess))
        .route("/api/fraud/transactions", get(list_transactions).post(create_transaction))
        .route("/api/fraud/rules", get(list_rules).post(create_rule))
        .route("/api/fraud/rules/:id", axum::routing::put(toggle_rule))
        .route("/api/fraud/events", get(list_events))
        .route("/api/fraud/blocklist", get(list_blocklist).post(add_blocklist))
        .route("/api/fraud/blocklist/:id", axum::routing::delete(remove_blocklist))
        .route("/api/fraud/stats", get(stats))
}

async fn assess(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    payload: Json<FraudAssessmentRequest>,
) -> Result<Json<FraudAssessmentResult>, StatusCode> {
// Tenant is resolved exclusively from the JWT claims (issue #734); any
    // branch_id sent by the client is ignored because it is untrusted input.
    let branch = resolve_branch(&headers);
    let result = state.engine.assess(branch, &payload.0).await;
    Ok(Json(result))
}

async fn list_rules(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<FraudRule>>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let rules = diesel::sql_query(
        "SELECT id, branch_id, name, description, rule_type, condition_json, \
         action, severity, is_active, created_at \
         FROM fraud_rules WHERE branch_id = $1 ORDER BY severity DESC, name",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load::<RuleRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudRule {
        id: r.id,
        branch_id: r.branch_id,
        name: r.name,
        description: r.description,
        rule_type: r.rule_type,
        condition_json: r.condition_json,
        action: r.action,
        severity: r.severity,
        is_active: r.is_active,
        created_at: r.created_at,
    })
    .collect();
    Ok(Json(rules))
}

async fn create_rule(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateRuleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let id = Uuid::new_v4();

    diesel::sql_query(
        "INSERT INTO fraud_rules (id, branch_id, name, description, rule_type, condition_json, action, severity, is_active) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&payload.name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.description)
    .bind::<diesel::sql_types::Text, _>(&payload.rule_type)
    .bind::<diesel::sql_types::Jsonb, _>(&payload.condition)
    .bind::<diesel::sql_types::Text, _>(&payload.action)
    .bind::<diesel::sql_types::Text, _>(&payload.severity)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"id": id})))
}

async fn toggle_rule(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query(
        "UPDATE fraud_rules SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1 AND branch_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

async fn list_events(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<FraudEvent>>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let events = diesel::sql_query(
        "SELECT id, branch_id, event_type, entity_type, entity_id, risk_score, risk_level, \
         triggered_rules, ml_score, action_taken, details, reviewed_by, reviewed_at, created_at \
         FROM fraud_events WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load::<EventRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudEvent {
        id: r.id,
        branch_id: r.branch_id,
        event_type: r.event_type,
        entity_type: r.entity_type,
        entity_id: r.entity_id,
        risk_score: r.risk_score,
        risk_level: r.risk_level,
        triggered_rules: r.triggered_rules.as_array().cloned().unwrap_or_default(),
        ml_score: r.ml_score,
        action_taken: r.action_taken,
        details: r.details,
        created_at: r.created_at,
    })
    .collect();
    Ok(Json(events))
}

async fn list_blocklist(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<FraudBlocklistEntry>>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let entries = diesel::sql_query(
        "SELECT id, branch_id, block_type, block_value, reason, expires_at, created_at \
         FROM fraud_blocklist WHERE branch_id = $1 ORDER BY created_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load::<BlocklistRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudBlocklistEntry {
        id: r.id,
        branch_id: r.branch_id,
        block_type: r.block_type,
        block_value: r.block_value,
        reason: r.reason,
        expires_at: r.expires_at,
        created_at: r.created_at,
    })
    .collect();
    Ok(Json(entries))
}

async fn add_blocklist(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    Json(payload): Json<BlocklistRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let id = Uuid::new_v4();
    let expires = payload.expires_in_hours.map(|h| chrono::Utc::now() + chrono::Duration::hours(h));

    diesel::sql_query(
        "INSERT INTO fraud_blocklist (id, branch_id, block_type, block_value, reason, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&payload.block_type)
    .bind::<diesel::sql_types::Text, _>(&payload.block_value)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.reason)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&expires)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"id": id})))
}

async fn remove_blocklist(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query("DELETE FROM fraud_blocklist WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

async fn stats(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
) -> Result<Json<FraudStats>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let total = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE branch_id = $1", branch).unwrap_or(0);
    let blocked = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE action_taken = 'block' AND branch_id = $1", branch).unwrap_or(0);
    let flagged = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE action_taken = 'flag' AND branch_id = $1", branch).unwrap_or(0);
    let reviewed = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE reviewed_at IS NOT NULL AND branch_id = $1", branch).unwrap_or(0);
    let high_risk = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE risk_level IN ('high','critical') AND branch_id = $1", branch).unwrap_or(0);
    let rules_active = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_rules WHERE is_active = true AND branch_id = $1", branch).unwrap_or(0);

    Ok(Json(FraudStats {
        total_events: total,
        blocked_count: blocked,
        flagged_count: flagged,
        reviewed_count: reviewed,
        high_risk_count: high_risk,
        rules_active,
    }))
}

fn count_sql(
    conn: &mut PgConnection,
    sql: &str,
    branch: Uuid,
) -> Result<i64, diesel::result::Error> {
    diesel::sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result::<CountRow>(conn)
        .map(|r| r.count)
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RuleRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    rule_type: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    condition_json: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Text)]
    action: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    severity: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct EventRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    event_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    entity_type: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    entity_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    risk_score: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    risk_level: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    triggered_rules: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
    ml_score: Option<rust_decimal::Decimal>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    action_taken: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    details: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BlocklistRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    block_type: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    block_value: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    reason: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}


// MARK: - Transaction endpoints (merged from botapps/fraud.rs)

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FraudTransaction {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub user_id: Uuid,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub risk_score: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_transactions(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<FraudTransaction>>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] branch_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] user_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Numeric)] amount: rust_decimal::Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] currency: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] risk_score: Option<i32>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<chrono::Utc>,
    }
    // Ensure fraud_transactions table exists
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS fraud_transactions (
            id UUID PRIMARY KEY, branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            user_id UUID NOT NULL DEFAULT gen_random_uuid(),
            amount NUMERIC(18,4) NOT NULL DEFAULT 0, currency VARCHAR(8) NOT NULL DEFAULT 'BRL',
            status VARCHAR(30) NOT NULL DEFAULT 'pending', risk_score INTEGER,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())"
    ).execute(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, branch_id, user_id, amount, currency, status, risk_score, created_at
         FROM fraud_transactions WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    ).bind::<diesel::sql_types::Uuid, _>(branch).load(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| FraudTransaction {
        id: r.id, branch_id: r.branch_id, user_id: r.user_id, amount: r.amount.to_string(),
        currency: r.currency, status: r.status, risk_score: r.risk_score, created_at: r.created_at,
    }).collect()))
}

async fn create_transaction(
    State(state): State<Arc<FraudState>>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let branch = resolve_branch(&headers);
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS fraud_transactions (
            id UUID PRIMARY KEY, branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            user_id UUID NOT NULL DEFAULT gen_random_uuid(),
            amount NUMERIC(18,4) NOT NULL DEFAULT 0, currency VARCHAR(8) NOT NULL DEFAULT 'BRL',
            status VARCHAR(30) NOT NULL DEFAULT 'pending', risk_score INTEGER,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())"
    ).execute(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO fraud_transactions (id, branch_id, user_id, amount, currency, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Uuid, _>(Uuid::parse_str(payload.get("user_id").and_then(|v| v.as_str()).unwrap_or("00000000-0000-0000-0000-000000000000")).unwrap_or(Uuid::nil()))
    .bind::<diesel::sql_types::Numeric, _>(rust_decimal::Decimal::new((payload.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0) as i64, 2))
    .bind::<diesel::sql_types::Text, _>(payload.get("currency").and_then(|v| v.as_str()).unwrap_or("BRL"))
    .execute(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"id": id, "status": "pending"})))
}
