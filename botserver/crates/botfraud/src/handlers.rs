use crate::engine::FraudEngine;
use crate::types::*;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
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

pub fn configure_fraud_routes() -> Router<Arc<FraudState>> {
    Router::new()
        .route("/api/fraud/assess", axum::routing::post(assess))
        .route("/api/fraud/rules", get(list_rules).post(create_rule))
        .route("/api/fraud/rules/{id}", axum::routing::put(toggle_rule))
        .route("/api/fraud/events", get(list_events))
        .route("/api/fraud/blocklist", get(list_blocklist).post(add_blocklist))
        .route("/api/fraud/blocklist/{id}", axum::routing::delete(remove_blocklist))
        .route("/api/fraud/stats", get(stats))
}

async fn assess(
    State(state): State<Arc<FraudState>>,
    Json(payload): Json<FraudAssessmentRequest>,
) -> Result<Json<FraudAssessmentResult>, StatusCode> {
    let result = state.engine.assess(&payload).await;
    Ok(Json(result))
}

async fn list_rules(
    State(state): State<Arc<FraudState>>,
) -> Result<Json<Vec<FraudRule>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let rules = diesel::sql_query(
        "SELECT id, bot_id, name, description, rule_type, condition_json, \
         action, severity, is_active, created_at \
         FROM fraud_rules ORDER BY severity DESC, name",
    )
    .load::<RuleRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudRule {
        id: r.id,
        bot_id: r.bot_id,
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
    Json(payload): Json<CreateRuleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let id = Uuid::new_v4();

    diesel::sql_query(
        "INSERT INTO fraud_rules (id, bot_id, name, description, rule_type, condition_json, action, severity, is_active) \
         VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, $4, $5, $6, $7, true)",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
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
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query(
        "UPDATE fraud_rules SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

async fn list_events(
    State(state): State<Arc<FraudState>>,
) -> Result<Json<Vec<FraudEvent>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let events = diesel::sql_query(
        "SELECT id, bot_id, event_type, entity_type, entity_id, risk_score, risk_level, \
         triggered_rules, ml_score, action_taken, details, reviewed_by, reviewed_at, created_at \
         FROM fraud_events ORDER BY created_at DESC LIMIT 100",
    )
    .load::<EventRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudEvent {
        id: r.id,
        bot_id: r.bot_id,
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
) -> Result<Json<Vec<FraudBlocklistEntry>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let entries = diesel::sql_query(
        "SELECT id, bot_id, block_type, block_value, reason, expires_at, created_at \
         FROM fraud_blocklist ORDER BY created_at DESC",
    )
    .load::<BlocklistRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| FraudBlocklistEntry {
        id: r.id,
        bot_id: r.bot_id,
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
    Json(payload): Json<BlocklistRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let id = Uuid::new_v4();
    let expires = payload.expires_in_hours.map(|h| chrono::Utc::now() + chrono::Duration::hours(h));

    diesel::sql_query(
        "INSERT INTO fraud_blocklist (id, bot_id, block_type, block_value, reason, expires_at) \
         VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, $4, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
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
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    diesel::sql_query("DELETE FROM fraud_blocklist WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(&id)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"success": true})))
}

async fn stats(
    State(state): State<Arc<FraudState>>,
) -> Result<Json<FraudStats>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let total = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events").unwrap_or(0);
    let blocked = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE action_taken = 'block'").unwrap_or(0);
    let flagged = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE action_taken = 'flag'").unwrap_or(0);
    let reviewed = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE reviewed_at IS NOT NULL").unwrap_or(0);
    let high_risk = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_events WHERE risk_level IN ('high','critical')").unwrap_or(0);
    let rules_active = count_sql(&mut conn, "SELECT COUNT(*) FROM fraud_rules WHERE is_active = true").unwrap_or(0);

    Ok(Json(FraudStats {
        total_events: total,
        blocked_count: blocked,
        flagged_count: flagged,
        reviewed_count: reviewed,
        high_risk_count: high_risk,
        rules_active,
    }))
}

fn count_sql(conn: &mut PgConnection, sql: &str) -> Result<i64, diesel::result::Error> {
    diesel::sql_query(sql)
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
    bot_id: Uuid,
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
    bot_id: Uuid,
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
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    reviewed_by: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BlocklistRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
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
