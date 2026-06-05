use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub version: String,
    pub author: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplatePreview {
    pub id: String,
    pub name: String,
    pub files: Vec<TemplateFile>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployResult {
    pub id: String,
    pub template_id: String,
    pub status: String,
    pub deployed_at: String,
    pub target: String,
}

#[derive(Default)]
struct AppState {
    templates: HashMap<String, Template>,
    previews: HashMap<String, TemplatePreview>,
    deploys: HashMap<String, DeployResult>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_templates() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Template> = s.templates.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn preview_template(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.previews.get(&id) {
        Some(preview) => Ok(Json(serde_json::json!({"preview": preview}))),
        None => Err((StatusCode::NOT_FOUND, "Template not found".to_string())),
    }
}

pub async fn deploy_template(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    if !s.templates.contains_key(&id) {
        return Err((StatusCode::NOT_FOUND, "Template not found".to_string()));
    }
    let deploy_id = uuid::Uuid::new_v4().to_string();
    let result = DeployResult {
        id: deploy_id.clone(),
        template_id: id,
        status: "deployed".to_string(),
        deployed_at: chrono::Utc::now().to_rfc3339(),
        target: "production".to_string(),
    };
    s.deploys.insert(deploy_id, result.clone());
    Ok(Json(serde_json::json!({"result": result})))
}
