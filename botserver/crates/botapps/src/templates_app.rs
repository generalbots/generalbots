use axum::{extract::{State, Json, Path}, routing::{get, post}, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,
    pub variables: String,
    pub version: u32,
    pub author: String,
    pub public: bool,
    pub usage_count: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateDeployment {
    pub id: Uuid,
    pub template_id: Uuid,
    pub bot_id: String,
    pub parameters: String,
    pub status: String,
    pub deployed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplatePreview {
    pub id: Uuid,
    pub template_id: Uuid,
    pub rendered_content: String,
    pub preview_url: Option<String>,
    pub created_at: String,
}

#[derive(Default)]
pub struct TemplatesAppState {
    pub templates: HashMap<Uuid, Template>,
    pub deployments: HashMap<Uuid, TemplateDeployment>,
    pub previews: HashMap<Uuid, TemplatePreview>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(TemplatesAppState::default()));
    Router::new()
        .route("/api/templates/list", get(list_templates).post(create_template))
        .route("/api/templates/list/{id}", get(get_template).put(update_template).delete(delete_template))
        .route("/api/templates/preview/{id}", get(preview_template))
        .route("/api/templates/deploy/{id}", post(deploy_template))
        .with_state(state)
}

async fn list_templates(State(state): State<Arc<RwLock<TemplatesAppState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Template> = s.templates.values().collect();
    Json(serde_json::json!({"templates": items}))
}

async fn create_template(State(state): State<Arc<RwLock<TemplatesAppState>>>, Json(mut tmpl): Json<Template>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    tmpl.id = id;
    tmpl.version = 1;
    tmpl.usage_count = 0;
    tmpl.created_at = Utc::now().to_rfc3339();
    s.templates.insert(id, tmpl.clone());
    Json(serde_json::json!({"template": tmpl}))
}

async fn get_template(State(state): State<Arc<RwLock<TemplatesAppState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.templates.get(&id) {
        Some(t) => Json(serde_json::json!({"template": t})),
        None => Json(serde_json::json!({"error": "Template not found"})),
    }
}

async fn update_template(State(state): State<Arc<RwLock<TemplatesAppState>>>, Path(id): Path<Uuid>, Json(tmpl): Json<Template>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.templates.get_mut(&id) {
        *existing = tmpl.clone();
        existing.id = id;
        existing.version += 1;
        Json(serde_json::json!({"template": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Template not found"}))
    }
}

async fn delete_template(State(state): State<Arc<RwLock<TemplatesAppState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.templates.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn preview_template(State(state): State<Arc<RwLock<TemplatesAppState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.templates.get(&id) {
        Some(tmpl) => {
            let preview = TemplatePreview {
                id: Uuid::new_v4(),
                template_id: id,
                rendered_content: tmpl.content.clone(),
                preview_url: None,
                created_at: Utc::now().to_rfc3339(),
            };
            Json(serde_json::json!({"preview": preview}))
        }
        None => Json(serde_json::json!({"error": "Template not found"})),
    }
}

async fn deploy_template(
    State(state): State<Arc<RwLock<TemplatesAppState>>>,
    Path(id): Path<Uuid>,
    Json(deployment): Json<TemplateDeployment>,
) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(tmpl) = s.templates.get_mut(&id) {
        tmpl.usage_count += 1;
        let mut dep = deployment;
        dep.id = Uuid::new_v4();
        dep.template_id = id;
        dep.status = "Deployed".to_string();
        dep.deployed_at = Some(Utc::now().to_rfc3339());
        dep.created_at = Utc::now().to_rfc3339();
        s.deployments.insert(dep.id, dep.clone());
        Json(serde_json::json!({"deployment": dep}))
    } else {
        Json(serde_json::json!({"error": "Template not found"}))
    }
}
