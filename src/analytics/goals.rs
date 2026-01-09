use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub period: String,
    pub status: ObjectiveStatus,
    pub progress: f32,
    pub visibility: Visibility,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Team,
    Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResult {
    pub id: Uuid,
    pub objective_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub metric_type: MetricType,
    pub start_value: f64,
    pub target_value: f64,
    pub current_value: f64,
    pub weight: f32,
    pub due_date: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KRStatus {
    NotStarted,
    InProgress,
    AtRisk,
    Completed,
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTemplate {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub category: String,
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
    pub total_objectives: i32,
    pub completed_objectives: i32,
    pub at_risk_objectives: i32,
    pub average_progress: f32,
    pub upcoming_check_ins: Vec<UpcomingCheckIn>,
    pub recent_activity: Vec<GoalActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingCheckIn {
    pub key_result_id: Uuid,
    pub key_result_title: String,
    pub objective_title: String,
    pub due_date: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize)]
pub struct ListObjectivesQuery {
    pub owner_id: Option<Uuid>,
    pub status: Option<ObjectiveStatus>,
    pub period: Option<String>,
    pub parent_id: Option<Uuid>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateObjectiveRequest {
    pub title: String,
    pub description: Option<String>,
    pub period: String,
    pub parent_id: Option<Uuid>,
    pub visibility: Option<Visibility>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectiveRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<ObjectiveStatus>,
    pub visibility: Option<Visibility>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyResultRequest {
    pub title: String,
    pub metric_type: MetricType,
    pub start_value: Option<f64>,
    pub target_value: f64,
    pub weight: Option<f32>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeyResultRequest {
    pub title: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
    pub weight: Option<f32>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: Option<KRStatus>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckInRequest {
    pub new_value: f64,
    pub note: Option<String>,
    pub confidence: Option<Confidence>,
}

#[derive(Debug, Deserialize)]
pub struct AISuggestRequest {
    pub context: Option<String>,
    pub role: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AISuggestion {
    pub objective: ObjectiveTemplate,
    pub key_results: Vec<KeyResultTemplate>,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct GoalsService {}

impl GoalsService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn list_objectives(
        &self,
        _organization_id: Uuid,
        _query: &ListObjectivesQuery,
    ) -> Result<Vec<Objective>, GoalsError> {
        Ok(vec![])
    }

    pub async fn create_objective(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        req: CreateObjectiveRequest,
    ) -> Result<Objective, GoalsError> {
        let now = Utc::now();
        Ok(Objective {
            id: Uuid::new_v4(),
            organization_id,
            owner_id,
            parent_id: req.parent_id,
            title: req.title,
            description: req.description.unwrap_or_default(),
            period: req.period,
            status: ObjectiveStatus::Draft,
            progress: 0.0,
            visibility: req.visibility.unwrap_or(Visibility::Team),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_objective(
        &self,
        _organization_id: Uuid,
        _objective_id: Uuid,
    ) -> Result<Option<Objective>, GoalsError> {
        Ok(None)
    }

    pub async fn update_objective(
        &self,
        _organization_id: Uuid,
        _objective_id: Uuid,
        _req: UpdateObjectiveRequest,
    ) -> Result<Objective, GoalsError> {
        Err(GoalsError::NotFound("Objective not found".to_string()))
    }

    pub async fn delete_objective(
        &self,
        _organization_id: Uuid,
        _objective_id: Uuid,
    ) -> Result<(), GoalsError> {
        Ok(())
    }

    pub async fn list_key_results(
        &self,
        _organization_id: Uuid,
        _objective_id: Uuid,
    ) -> Result<Vec<KeyResult>, GoalsError> {
        Ok(vec![])
    }

    pub async fn create_key_result(
        &self,
        _organization_id: Uuid,
        objective_id: Uuid,
        owner_id: Uuid,
        req: CreateKeyResultRequest,
    ) -> Result<KeyResult, GoalsError> {
        let now = Utc::now();
        Ok(KeyResult {
            id: Uuid::new_v4(),
            objective_id,
            owner_id,
            title: req.title,
            metric_type: req.metric_type,
            start_value: req.start_value.unwrap_or(0.0),
            target_value: req.target_value,
            current_value: req.start_value.unwrap_or(0.0),
            weight: req.weight.unwrap_or(1.0),
            due_date: req.due_date,
            status: KRStatus::NotStarted,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn update_key_result(
        &self,
        _organization_id: Uuid,
        _key_result_id: Uuid,
        _req: UpdateKeyResultRequest,
    ) -> Result<KeyResult, GoalsError> {
        Err(GoalsError::NotFound("Key result not found".to_string()))
    }

    pub async fn delete_key_result(
        &self,
        _organization_id: Uuid,
        _key_result_id: Uuid,
    ) -> Result<(), GoalsError> {
        Ok(())
    }

    pub async fn create_check_in(
        &self,
        _organization_id: Uuid,
        key_result_id: Uuid,
        user_id: Uuid,
        req: CreateCheckInRequest,
    ) -> Result<CheckIn, GoalsError> {
        Ok(CheckIn {
            id: Uuid::new_v4(),
            key_result_id,
            user_id,
            previous_value: 0.0,
            new_value: req.new_value,
            note: req.note.unwrap_or_default(),
            confidence: req.confidence.unwrap_or(Confidence::Medium),
            created_at: Utc::now(),
        })
    }

    pub async fn get_check_in_history(
        &self,
        _organization_id: Uuid,
        _key_result_id: Uuid,
    ) -> Result<Vec<CheckIn>, GoalsError> {
        Ok(vec![])
    }

    pub async fn get_dashboard(
        &self,
        _organization_id: Uuid,
        _user_id: Uuid,
    ) -> Result<GoalsDashboard, GoalsError> {
        Ok(GoalsDashboard {
            total_objectives: 0,
            completed_objectives: 0,
            at_risk_objectives: 0,
            average_progress: 0.0,
            upcoming_check_ins: vec![],
            recent_activity: vec![],
        })
    }

    pub async fn get_alignment_tree(
        &self,
        _organization_id: Uuid,
    ) -> Result<Vec<AlignmentNode>, GoalsError> {
        Ok(vec![])
    }

    pub async fn suggest_goals(
        &self,
        _organization_id: Uuid,
        _req: AISuggestRequest,
    ) -> Result<Vec<AISuggestion>, GoalsError> {
        Ok(vec![
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
        ])
    }

    pub async fn get_templates(
        &self,
        _organization_id: Uuid,
    ) -> Result<Vec<GoalTemplate>, GoalsError> {
        Ok(vec![])
    }


}

impl Default for GoalsService {
    fn default() -> Self {
        Self::new()
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
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn handle_list_objectives(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ListObjectivesQuery>,
) -> Result<Json<Vec<Objective>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let objectives = service.list_objectives(org_id, &query).await?;
    Ok(Json(objectives))
}

pub async fn handle_create_objective(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let objective = service.create_objective(org_id, user_id, req).await?;
    Ok(Json(objective))
}

pub async fn handle_get_objective(
    State(_state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Option<Objective>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let objective = service.get_objective(org_id, objective_id).await?;
    Ok(Json(objective))
}

pub async fn handle_update_objective(
    State(_state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<UpdateObjectiveRequest>,
) -> Result<Json<Objective>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let objective = service.update_objective(org_id, objective_id, req).await?;
    Ok(Json(objective))
}

pub async fn handle_delete_objective(
    State(_state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    service.delete_objective(org_id, objective_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_key_results(
    State(_state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
) -> Result<Json<Vec<KeyResult>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let key_results = service.list_key_results(org_id, objective_id).await?;
    Ok(Json(key_results))
}

pub async fn handle_create_key_result(
    State(_state): State<Arc<AppState>>,
    Path(objective_id): Path<Uuid>,
    Json(req): Json<CreateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let key_result = service
        .create_key_result(org_id, objective_id, user_id, req)
        .await?;
    Ok(Json(key_result))
}

pub async fn handle_update_key_result(
    State(_state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<UpdateKeyResultRequest>,
) -> Result<Json<KeyResult>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let key_result = service.update_key_result(org_id, key_result_id, req).await?;
    Ok(Json(key_result))
}

pub async fn handle_delete_key_result(
    State(_state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    service.delete_key_result(org_id, key_result_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_create_check_in(
    State(_state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
    Json(req): Json<CreateCheckInRequest>,
) -> Result<Json<CheckIn>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let check_in = service
        .create_check_in(org_id, key_result_id, user_id, req)
        .await?;
    Ok(Json(check_in))
}

pub async fn handle_get_check_in_history(
    State(_state): State<Arc<AppState>>,
    Path(key_result_id): Path<Uuid>,
) -> Result<Json<Vec<CheckIn>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let history = service.get_check_in_history(org_id, key_result_id).await?;
    Ok(Json(history))
}

pub async fn handle_get_dashboard(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<GoalsDashboard>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let dashboard = service.get_dashboard(org_id, user_id).await?;
    Ok(Json(dashboard))
}

pub async fn handle_get_alignment(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlignmentNode>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let tree = service.get_alignment_tree(org_id).await?;
    Ok(Json(tree))
}

pub async fn handle_ai_suggest(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AISuggestRequest>,
) -> Result<Json<Vec<AISuggestion>>, GoalsError> {
    let service = GoalsService::new();
    let org_id = Uuid::nil();
    let suggestions = service.suggest_goals(org_id, req).await?;
    Ok(Json(suggestions))
}

pub fn configure_goals_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/goals/objectives", get(handle_list_objectives))
        .route("/api/goals/objectives", post(handle_create_objective))
        .route("/api/goals/objectives/:id", get(handle_get_objective))
        .route("/api/goals/objectives/:id", put(handle_update_objective))
        .route("/api/goals/objectives/:id", delete(handle_delete_objective))
        .route(
            "/api/goals/objectives/:id/key-results",
            get(handle_list_key_results),
        )
        .route(
            "/api/goals/objectives/:id/key-results",
            post(handle_create_key_result),
        )
        .route("/api/goals/key-results/:id", put(handle_update_key_result))
        .route(
            "/api/goals/key-results/:id",
            delete(handle_delete_key_result),
        )
        .route(
            "/api/goals/key-results/:id/check-in",
            post(handle_create_check_in),
        )
        .route(
            "/api/goals/key-results/:id/history",
            get(handle_get_check_in_history),
        )
        .route("/api/goals/dashboard", get(handle_get_dashboard))
        .route("/api/goals/alignment", get(handle_get_alignment))
        .route("/api/goals/ai/suggest", post(handle_ai_suggest))
}
