use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateCategory {
    Business,
    Service,
    Lifestyle,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub icon: String,
    pub features: Vec<String>,
    pub sample_conversations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePreview {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub icon: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResult {
    pub template_id: Uuid,
    pub bot_name: String,
    pub deployed_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DeployRequest {
    pub bot_name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct TemplatesState {
    pub templates: Arc<RwLock<Vec<BotTemplate>>>,
}

impl TemplatesState {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub fn routes() -> Router {
    let state = TemplatesState::new();
    Router::new()
        .route("/api/templates/list", get(list_templates))
        .route("/api/templates/preview/:id", get(get_preview))
        .route("/api/templates/deploy/:id", post(deploy_template))
        .with_state(state)
}

async fn list_templates(
    AxumState(state): AxumState<TemplatesState>,
) -> Result<Json<ApiResponse<Vec<BotTemplate>>>, StatusCode> {
    let templates = state
        .templates
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(templates.clone()),
        error: None,
    }))
}

async fn get_preview(
    AxumState(state): AxumState<TemplatesState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TemplatePreview>>, StatusCode> {
    let templates = state
        .templates
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let preview = TemplatePreview {
        id: template.id,
        name: template.name.clone(),
        description: template.description.clone(),
        category: template.category.clone(),
        icon: template.icon.clone(),
        features: template.features.clone(),
    };
    Ok(Json(ApiResponse {
        success: true,
        data: Some(preview),
        error: None,
    }))
}

async fn deploy_template(
    AxumState(state): AxumState<TemplatesState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<DeployRequest>,
) -> Result<Json<ApiResponse<DeployResult>>, StatusCode> {
    let templates = state
        .templates
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let result = DeployResult {
        template_id: template.id,
        bot_name: payload.bot_name,
        deployed_at: Utc::now(),
        status: "deployed".to_string(),
    };
    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        error: None,
    }))
}
