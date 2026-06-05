use axum::extract::Json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisRequest {
    pub image_url: String,
    pub kind: String,
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisResult {
    pub id: String,
    pub image_url: String,
    pub kind: String,
    pub status: String,
    pub labels: Vec<String>,
    pub confidence: f64,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    results: HashMap<String, AnalysisResult>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn analyze_image(Json(req): Json<AnalysisRequest>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let result = AnalysisResult {
        id: id.clone(),
        image_url: req.image_url,
        kind: req.kind,
        status: "completed".to_string(),
        labels: vec!["detected".to_string()],
        confidence: 0.95,
        metadata: req.parameters,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    s.results.insert(id, result.clone());
    Ok(Json(serde_json::json!({"result": result})))
}

pub async fn list_history() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&AnalysisResult> = s.results.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}
