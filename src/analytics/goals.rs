use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::schema::{okr_checkins, okr_key_results, okr_objectives, okr_templates};
use crate::shared::state::AppState;

fn get_bot_context() -> (Uuid, Uuid) {
    let org_id = std::env::var("DEFAULT_ORG_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);
    let bot_id = std::env::var("DEFAULT_BOT_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);
    (org_id, bot_id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = okr_objectives)]
pub struct ObjectiveRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub period: String,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub status: String,
    pub progress: BigDecimal,
    pub visibility: String,
    pub weight: BigDecimal,
    pub tags: Vec<Option<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = okr_key_results)]
pub struct KeyResultRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub objective_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub metric_type: String,
    pub start_value: BigDecimal,
    pub target_value: BigDecimal,
    pub current_value: BigDecimal,
    pub unit: Option<String>,
    pub weight: BigDecimal,
    pub status: String,
    pub due_date: Option<NaiveDate>,
    pub scoring_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = okr_checkins)]
pub struct CheckInRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub key_result_id: Uuid,
    pub user_id: Uuid,
    pub previous_value: Option<BigDecimal>,
    pub new_value: BigDecimal,
    pub note: Option<String>,
    pub confidence: Option<String>,
    pub blockers: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = okr_templates)]
pub struct TemplateRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub objective_template: serde_json::Value,
    pub key_result_templates: serde_json::Value,
    pub is_system: bool,
    pub usage_count: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub period: String,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub status: ObjectiveStatus,
    pub progress: f32,
    pub visibility: Visibility,
    pub weight: f32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveStatus {
    Draft,
    Active,
    OnTrack,
    AtRisk,
    Behind,
    Completed,
    Cancelled,
}

impl ObjectiveStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "on_track" => Self::OnTrack,
            "at_risk" => Self::AtRisk,
            "behind" => Self::Behind,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            _ => Self::Draft,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::OnTrack => "on_track",
            Self::AtRisk => "at_risk",
            Self::Behind => "behind",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Team,
    Organization,
}

impl Visibility {
    fn from_str(s: &str) -> Self {
        match s {
            "private" => Self::Private,
            "organization" => Self::Organization,
            _ => Self::Team,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Team => "team",
            Self::Organization => "organization",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResult {
    pub id: Uuid,
    pub objective_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub metric_type: MetricType,
    pub start_value: f64,
    pub target_value: f64,
    pub current_value: f64,
    pub unit: Option<String>,
    pub weight: f32,
    pub due_date: Option<NaiveDate>,
    pub status: KRStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Percentage,
    Number,
    Currency,
    Boolean,
}

impl MetricType {
    fn from_str(s: &str) -> Self {
        match s {
            "percentage" => Self::Percentage,
            "currency" => Self::Currency,
            "boolean" => Self::Boolean,
            _ => Self::Number,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Percentage => "percentage",
            Self::Number => "number",
            Self::Currency => "currency",
            Self::Boolean => "boolean",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KRStatus {
    NotStarted,
    InProgress,
    AtRisk,
    Completed,
}

impl KRStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "in_progress" => Self::InProgress,
            "at_risk" => Self::AtRisk,
            "completed" => Self::Completed,
            _ => Self::NotStarted,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::AtRisk => "at_risk",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckIn {
    pub id: Uuid,
    pub key_result_id: Uuid,
    pub user_id: Uuid,
    pub previous_value: f64,
    pub new_value: f64,
    pub note: String,
    pub confidence: Confidence,
    pub blockers: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    fn from_str(s: &str) -> Self {
        match s {
            "low" => Self::Low,
            "high" => Self::High,
            _ => Self::Medium,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub objective_template: ObjectiveTemplate,
    pub key_result_templates: Vec<KeyResultTemplate>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTemplate {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResultTemplate {
    pub title: String,
    pub metric_type: MetricType,
    pub suggested_target: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentNode {
    pub objective: Objective,
    pub key_results: Vec<KeyResult>,
    pub children: Vec<AlignmentNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsDashboard {
    pub total_objectives: i64,
    pub completed_objectives: i64,
    pub at_risk_objectives: i64,
    pub average_progress: f32,
    pub upcoming_check_ins: Vec<UpcomingCheckIn>,
    pub recent_activity: Vec<GoalActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingCheckIn {
    pub key_result_id: Uuid,
    pub key_result_title: String,
    pub objective_title: String,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalActivity {
    pub id: Uuid,
    pub activity_type: GoalActivityType,
    pub user_id: Uuid,
    pub user_name: String,
    pub objective_id: Option<Uuid>,
    pub key_result_id: Option<Uuid>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalActivityType {
    ObjectiveCreated,
    ObjectiveUpdated,
    ObjectiveCompleted,
    KeyResultCreated,
    KeyResultUpdated,
    CheckInRecorded,
    ProgressChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListObjectivesQuery {
    pub owner_id: Option<Uuid>,
    pub status: Option<String>,
    pub period: Option<String>,
    pub parent_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObjectiveRequest {
    pub title: String,
    pub description: Option<String>,
    pub period: String,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub parent_id: Option<Uuid>,
    pub visibility: Option<Visibility>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateObjectiveRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ObjectiveStatus>,
    pub visibility: Option<Visibility>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResultRequest {
    pub title: String,
    pub description: Option<String>,
    pub metric_type: MetricType,
    pub start_value: Option<f64>,
    pub target_value: f64,
    pub unit: Option<String>,
    pub weight: Option<f32>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKeyResultRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
    pub weight: Option<f32>,
    pub due_date: Option<NaiveDate>,
    pub status: Option<KRStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckInRequest {
    pub new_value: f64,
    pub note: Option<String>,
    pub confidence: Option<Confidence>,
    pub blockers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestRequest {
    pub context: String,
    pub role: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestion {
    pub objective: ObjectiveTemplate,
    pub key_results: Vec<KeyResultTemplate>,
    pub rationale: String,
}

fn record_to_objective(record: ObjectiveRecord) -> Objective {
    Objective {
        id: record.id,
        organization_id: record.org_id,
        owner_id: record.owner_id,
        parent_id: record.parent_id,
        title: record.title,
        description: record.description.unwrap_or_default(),
        period: record.period,
        period_start: record.period_start,
        period_end: record.period_end,
        status: ObjectiveStatus::from_str(&record.status),
        progress: record.progress.to_f32().unwrap_or(0.0),
        visibility: Visibility::from_str(&record.visibility),
        weight: record.weight.to_f32().unwrap_or(1.0),
        tags: record.tags.into_iter().flatten().collect(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn record_to_key_result(record: KeyResultRecord) -> KeyResult {
    KeyResult {
        id: record.id,
        objective_id: record.objective_id,
        owner_id: record.owner_id,
        title: record.title,
        description: record.description,
        metric_type: MetricType::from_str(&record.metric_type),
        start_value: record.start_value.to_f64().unwrap_or(0.0),
        target_value: record.target_value.to_f64().unwrap_or(0.0),
        current_value: record.current_value.to_f64().unwrap_or(0.0),
        unit: record.unit,
        weight: record.weight.to_f32().unwrap_or(1.0),
        due_date: record.due_date,
        status: KRStatus::from_str(&record.status),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn record_to_checkin(record: CheckInRecord) -> CheckIn {
    CheckIn {
        id: record.id,
        key_result_id: record.key_result_id,
        user_id: record.user_id,
        previous_value: record.previous_value.and_then(|v| v.to_f64()).unwrap_or(0.0),
        new_value: record.new_value.to_f64().unwrap_or(0.0),
        note: record.note.unwrap_or_default(),
        confidence: Confidence::from_str(record.confidence.as_deref().unwrap_or("medium")),
        blockers: record.blockers,
        created_at: record.created_at,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GoalsError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl IntoResponse for GoalsError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn list_objectives(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListObjectivesQuery>,
) -> Result<Json<Vec<Objective>>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut db_query = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(owner_id) = query.owner_id {
            db_query = db_query.filter(okr_objectives::owner_id.eq(owner_id));
        }
        if let Some(status) = query.status {
            db_query = db_query.filter(okr_objectives::status.eq(status));
        }
        if let Some(period) = query.period {
            db_query = db_query.filter(okr_objectives::period.eq(period));
        }
        if let Some(parent_id) = query.parent_id {
            db_query = db_query.filter(okr_objectives::parent_id.eq(parent_id));
        }

        db_query = db_query.order(okr_objectives::created_at.desc());

        if let Some(limit) = query.limit {
            db_query = db_query.limit(limit);
        }
        if let Some(offset) = query.offset {
            db_query = db_query.offset(offset);
        }

        db_query
            .load::<ObjectiveRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let objectives: Vec<Objective> = result.into_iter().map(record_to_objective).collect();
    Ok(Json(objectives))
}

pub async fn create_objective(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let tags: Vec<Option<String>> = req.tags.unwrap_or_default().into_iter().map(Some).collect();

    let new_objective = ObjectiveRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        owner_id,
        parent_id: req.parent_id,
        title: req.title.clone(),
        description: req.description.clone(),
        period: req.period.clone(),
        period_start: req.period_start,
        period_end: req.period_end,
        status: "draft".to_string(),
        progress: BigDecimal::from(0),
        visibility: req.visibility.as_ref().map(|v| v.to_str()).unwrap_or("team").to_string(),
        weight: BigDecimal::from(1),
        tags,
        created_at: now,
        updated_at: now,
    };

    let record = new_objective.clone();

    let _result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        diesel::insert_into(okr_objectives::table)
            .values(&new_objective)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;
        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created objective: {} ({})", record.title, record.id);
    Ok(Json(record_to_objective(record)))
}

pub async fn get_objective(
    State(state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Objective>, GoalsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_objectives::table
            .find(objective_id)
            .first::<ObjectiveRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    result
        .map(record_to_objective)
        .ok_or_else(|| GoalsError::NotFound("Objective not found".to_string()))
        .map(Json)
}

pub async fn update_objective(
    State(state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<UpdateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut objective = okr_objectives::table
            .find(objective_id)
            .first::<ObjectiveRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))?
            .ok_or_else(|| GoalsError::NotFound("Objective not found".to_string()))?;

        if let Some(title) = req.title {
            objective.title = title;
        }
        if let Some(description) = req.description {
            objective.description = Some(description);
        }
        if let Some(status) = req.status {
            objective.status = status.to_str().to_string();
        }
        if let Some(visibility) = req.visibility {
            objective.visibility = visibility.to_str().to_string();
        }
        if let Some(period_start) = req.period_start {
            objective.period_start = Some(period_start);
        }
        if let Some(period_end) = req.period_end {
            objective.period_end = Some(period_end);
        }
        if let Some(tags) = req.tags {
            objective.tags = tags.into_iter().map(Some).collect();
        }
        objective.updated_at = Utc::now();

        diesel::update(okr_objectives::table.find(objective_id))
            .set(&objective)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(objective)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Updated objective: {} ({})", result.title, result.id);
    Ok(Json(record_to_objective(result)))
}

pub async fn delete_objective(
    State(state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let pool = state.conn.clone();

    let _result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        let deleted = diesel::delete(okr_objectives::table.find(objective_id))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("Deleted objective: {objective_id}");
            Ok::<_, GoalsError>(())
        } else {
            Err(GoalsError::NotFound("Objective not found".to_string()))
        }
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn list_key_results(
    State(state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Vec<KeyResult>>, GoalsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_key_results::table
            .filter(okr_key_results::objective_id.eq(objective_id))
            .order(okr_key_results::created_at.asc())
            .load::<KeyResultRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let key_results: Vec<KeyResult> = result.into_iter().map(record_to_key_result).collect();
    Ok(Json(key_results))
}

pub async fn create_key_result(
    State(state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<CreateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let start_value = req.start_value.unwrap_or(0.0);

    let new_kr = KeyResultRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        objective_id,
        owner_id,
        title: req.title.clone(),
        description: req.description.clone(),
        metric_type: req.metric_type.to_str().to_string(),
        start_value: BigDecimal::try_from(start_value).unwrap_or_else(|_| BigDecimal::from(0)),
        target_value: BigDecimal::try_from(req.target_value).unwrap_or_else(|_| BigDecimal::from(0)),
        current_value: BigDecimal::try_from(start_value).unwrap_or_else(|_| BigDecimal::from(0)),
        unit: req.unit.clone(),
        weight: BigDecimal::try_from(req.weight.unwrap_or(1.0) as f64).unwrap_or_else(|_| BigDecimal::from(1)),
        status: "not_started".to_string(),
        due_date: req.due_date,
        scoring_type: "linear".to_string(),
        created_at: now,
        updated_at: now,
    };

    let record = new_kr.clone();

    let _result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        diesel::insert_into(okr_key_results::table)
            .values(&new_kr)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;
        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created key result: {} ({})", record.title, record.id);
    Ok(Json(record_to_key_result(record)))
}

pub async fn update_key_result(
    State(state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<UpdateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let mut kr = okr_key_results::table
            .find(key_result_id)
            .first::<KeyResultRecord>(&mut conn)
            .optional()
            .map_err(|e| GoalsError::Database(e.to_string()))?
            .ok_or_else(|| GoalsError::NotFound("Key result not found".to_string()))?;

        if let Some(title) = req.title {
            kr.title = title;
        }
        if let Some(description) = req.description {
            kr.description = Some(description);
        }
        if let Some(target_value) = req.target_value {
            kr.target_value = BigDecimal::try_from(target_value).unwrap_or_else(|_| BigDecimal::from(0));
        }
        if let Some(current_value) = req.current_value {
            kr.current_value = BigDecimal::try_from(current_value).unwrap_or_else(|_| BigDecimal::from(0));
        }
        if let Some(weight) = req.weight {
            kr.weight = BigDecimal::try_from(weight as f64).unwrap_or_else(|_| BigDecimal::from(1));
        }
        if let Some(due_date) = req.due_date {
            kr.due_date = Some(due_date);
        }
        if let Some(status) = req.status {
            kr.status = status.to_str().to_string();
        }
        kr.updated_at = Utc::now();

        diesel::update(okr_key_results::table.find(key_result_id))
            .set(&kr)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(kr)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Updated key result: {} ({})", result.title, result.id);
    Ok(Json(record_to_key_result(result)))
}

pub async fn delete_key_result(
    State(state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        let deleted = diesel::delete(okr_key_results::table.find(key_result_id))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("Deleted key result: {key_result_id}");
            Ok::<_, GoalsError>(())
        } else {
            Err(GoalsError::NotFound("Key result not found".to_string()))
        }
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn create_check_in(
    State(state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<CreateCheckInRequest>,
) -> Result<Json<CheckIn>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let user_id = Uuid::nil();
    let now = Utc::now();

    let pool_clone = pool.clone();
    let previous_value = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        okr_key_results::table
            .find(key_result_id)
            .select(okr_key_results::current_value)
            .first::<BigDecimal>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let new_checkin = CheckInRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        key_result_id,
        user_id,
        previous_value,
        new_value: BigDecimal::try_from(req.new_value).unwrap_or_else(|_| BigDecimal::from(0)),
        note: req.note.clone(),
        confidence: req.confidence.as_ref().map(|c| c.to_str().to_string()),
        blockers: req.blockers.clone(),
        created_at: now,
    };

    let record = new_checkin.clone();
    let new_val = req.new_value;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        diesel::insert_into(okr_checkins::table)
            .values(&new_checkin)
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        diesel::update(okr_key_results::table.find(key_result_id))
            .set((
                okr_key_results::current_value.eq(BigDecimal::try_from(new_val).unwrap_or_else(|_| BigDecimal::from(0))),
                okr_key_results::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        Ok::<_, GoalsError>(())
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    info!("Created check-in for key result: {key_result_id}");
    Ok(Json(record_to_checkin(record)))
}

pub async fn get_check_in_history(
    State(state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<Vec<CheckIn>>, GoalsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;
        okr_checkins::table
            .filter(okr_checkins::key_result_id.eq(key_result_id))
            .order(okr_checkins::created_at.desc())
            .load::<CheckInRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let history: Vec<CheckIn> = result.into_iter().map(record_to_checkin).collect();
    Ok(Json(history))
}

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GoalsDashboard>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let total: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let completed: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("completed"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let at_risk: i64 = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("at_risk"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let objectives = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .select(okr_objectives::progress)
            .load::<BigDecimal>(&mut conn)
            .unwrap_or_default();

        let avg_progress = if objectives.is_empty() {
            0.0
        } else {
            let sum: f32 = objectives.iter().map(|p| p.to_f32().unwrap_or(0.0)).sum();
            sum / objectives.len() as f32
        };

        let upcoming_krs = okr_key_results::table
            .filter(okr_key_results::org_id.eq(org_id))
            .filter(okr_key_results::bot_id.eq(bot_id))
            .filter(okr_key_results::due_date.is_not_null())
            .order(okr_key_results::due_date.asc())
            .limit(5)
            .load::<KeyResultRecord>(&mut conn)
            .unwrap_or_default();

        let upcoming_check_ins: Vec<UpcomingCheckIn> = upcoming_krs.into_iter().map(|kr| {
            UpcomingCheckIn {
                key_result_id: kr.id,
                key_result_title: kr.title,
                objective_title: String::new(),
                due_date: kr.due_date,
            }
        }).collect();

        Ok::<_, GoalsError>(GoalsDashboard {
            total_objectives: total,
            completed_objectives: completed,
            at_risk_objectives: at_risk,
            average_progress: avg_progress,
            upcoming_check_ins,
            recent_activity: vec![],
        })
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(result))
}

pub async fn get_alignment(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlignmentNode>>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        let objectives = okr_objectives::table
            .filter(okr_objectives::org_id.eq(org_id))
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::parent_id.is_null())
            .load::<ObjectiveRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))?;

        let nodes: Vec<AlignmentNode> = objectives.into_iter().map(|obj| {
            let key_results = okr_key_results::table
                .filter(okr_key_results::objective_id.eq(obj.id))
                .load::<KeyResultRecord>(&mut conn)
                .unwrap_or_default()
                .into_iter()
                .map(record_to_key_result)
                .collect();

            AlignmentNode {
                objective: record_to_objective(obj),
                key_results,
                children: vec![],
            }
        }).collect();

        Ok::<_, GoalsError>(nodes)
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    Ok(Json(result))
}

pub async fn ai_suggest(
    Json(_req): Json<AISuggestRequest>,
) -> Result<Json<Vec<AISuggestion>>, GoalsError> {
    let suggestions = vec![
        AISuggestion {
            objective: ObjectiveTemplate {
                title: "Improve customer satisfaction".to_string(),
                description: "Enhance customer experience across all touchpoints".to_string(),
            },
            key_results: vec![
                KeyResultTemplate {
                    title: "Increase NPS score".to_string(),
                    metric_type: MetricType::Number,
                    suggested_target: Some(50.0),
                },
                KeyResultTemplate {
                    title: "Reduce support ticket resolution time".to_string(),
                    metric_type: MetricType::Number,
                    suggested_target: Some(24.0),
                },
            ],
            rationale: "Customer satisfaction directly impacts retention and growth".to_string(),
        },
    ];
    Ok(Json(suggestions))
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<GoalTemplate>>, GoalsError> {
    let pool = state.conn.clone();
    let (org_id, _bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| GoalsError::Database(e.to_string()))?;

        okr_templates::table
            .filter(okr_templates::org_id.eq(org_id).or(okr_templates::is_system.eq(true)))
            .order(okr_templates::name.asc())
            .load::<TemplateRecord>(&mut conn)
            .map_err(|e| GoalsError::Database(e.to_string()))
    })
    .await
    .map_err(|e| GoalsError::Database(e.to_string()))??;

    let templates: Vec<GoalTemplate> = result.into_iter().map(|t| {
        let objective_template: ObjectiveTemplate = serde_json::from_value(t.objective_template)
            .unwrap_or(ObjectiveTemplate { title: String::new(), description: String::new() });
        let key_result_templates: Vec<KeyResultTemplate> = serde_json::from_value(t.key_result_templates)
            .unwrap_or_default();

        GoalTemplate {
            id: t.id,
            organization_id: t.org_id,
            name: t.name,
            description: t.description,
            category: t.category,
            objective_template,
            key_result_templates,
            is_system: t.is_system,
            created_at: t.created_at,
        }
    }).collect();

    Ok(Json(templates))
}

pub fn configure_goals_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/goals/objectives", get(list_objectives).post(create_objective))
        .route("/api/goals/objectives/:id", get(get_objective).put(update_objective).delete(delete_objective))
        .route("/api/goals/objectives/:id/key-results", get(list_key_results).post(create_key_result))
        .route("/api/goals/key-results/:id", put(update_key_result).delete(delete_key_result))
        .route("/api/goals/key-results/:id/check-in", post(create_check_in))
        .route("/api/goals/key-results/:id/history", get(get_check_in_history))
        .route("/api/goals/dashboard", get(get_dashboard))
        .route("/api/goals/alignment", get(get_alignment))
        .route("/api/goals/templates", get(list_templates))
        .route("/api/goals/ai/suggest", post(ai_suggest))
}
