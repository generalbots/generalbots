use axum::{extract::{State, Json, Path}, routing::{get, post}, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType { ObjectDetection, FaceRecognition, Ocr, SceneClassification }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisionAnalysis {
    pub id: Uuid,
    pub input_type: String,
    pub input_url: String,
    pub analysis_type: String,
    pub result: String,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisionHistoryEntry {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub action: String,
    pub details: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct VisionState {
    pub analyses: HashMap<Uuid, VisionAnalysis>,
    pub history: HashMap<Uuid, VisionHistoryEntry>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(VisionState::default()));
    Router::new()
        .route("/api/vision/analyze", post(create_analysis))
        .route("/api/vision/analyze/{id}", get(get_analysis).delete(delete_analysis))
        .route("/api/vision/history", get(list_history))
        .route("/api/vision/history/{id}", get(get_history_entry))
        .with_state(state)
}

async fn create_analysis(State(state): State<Arc<RwLock<VisionState>>>, Json(mut analysis): Json<VisionAnalysis>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    analysis.id = id;
    analysis.created_at = Utc::now().to_rfc3339();
    s.analyses.insert(id, analysis.clone());
    let history_id = Uuid::new_v4();
    let entry = VisionHistoryEntry {
        id: history_id,
        analysis_id: id,
        action: "created".to_string(),
        details: format!("Analysis of type {} created", analysis.analysis_type),
        created_at: Utc::now().to_rfc3339(),
    };
    s.history.insert(history_id, entry);
    Json(serde_json::json!({"analysis": analysis}))
}

async fn get_analysis(State(state): State<Arc<RwLock<VisionState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.analyses.get(&id) {
        Some(a) => Json(serde_json::json!({"analysis": a})),
        None => Json(serde_json::json!({"error": "Analysis not found"})),
    }
}

async fn delete_analysis(State(state): State<Arc<RwLock<VisionState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.analyses.remove(&id);
    let history_id = Uuid::new_v4();
    let entry = VisionHistoryEntry {
        id: history_id,
        analysis_id: id,
        action: "deleted".to_string(),
        details: "Analysis deleted".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    s.history.insert(history_id, entry);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_history(State(state): State<Arc<RwLock<VisionState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&VisionHistoryEntry> = s.history.values().collect();
    Json(serde_json::json!({"history": items}))
}

async fn get_history_entry(State(state): State<Arc<RwLock<VisionState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.history.get(&id) {
        Some(e) => Json(serde_json::json!({"entry": e})),
        None => Json(serde_json::json!({"error": "History entry not found"})),
    }
}
