use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub label: String,
    pub confidence: f64,
    pub bounding_box: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: Uuid,
    pub analysis_type: String,
    pub filename: String,
    pub results: Vec<DetectionResult>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub analysis_type: String,
    pub filename: String,
}

#[derive(Debug, Default)]
pub struct VisionState {
    pub analyses: HashMap<Uuid, Analysis>,
    pub history: Vec<Analysis>,
}



pub fn create_vision_state() -> SharedVisionState {
    Arc::new(RwLock::new(VisionState::default()))
}

async fn analyze_image(
    State(state): State<SharedVisionState>,
    Json(input): Json<AnalyzeRequest>,
) -> Result<(StatusCode, Json<Analysis>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = match input.analysis_type.as_str() {
        "ocr" => vec![DetectionResult {
            label: "text_detected".to_string(),
            confidence: 0.95,
            bounding_box: Some(BoundingBox {
                x: 10.0,
                y: 20.0,
                width: 200.0,
                height: 50.0,
            }),
        }],
        "object_detection" => vec![
            DetectionResult {
                label: "person".to_string(),
                confidence: 0.92,
                bounding_box: Some(BoundingBox {
                    x: 100.0,
                    y: 50.0,
                    width: 150.0,
                    height: 300.0,
                }),
            },
            DetectionResult {
                label: "car".to_string(),
                confidence: 0.87,
                bounding_box: Some(BoundingBox {
                    x: 300.0,
                    y: 150.0,
                    width: 250.0,
                    height: 120.0,
                }),
            },
        ],
        "damage" => vec![DetectionResult {
            label: "scratch".to_string(),
            confidence: 0.78,
            bounding_box: Some(BoundingBox {
                x: 50.0,
                y: 100.0,
                width: 80.0,
                height: 30.0,
            }),
        }],
        "plate" => vec![DetectionResult {
            label: "ABC1D23".to_string(),
            confidence: 0.91,
            bounding_box: Some(BoundingBox {
                x: 200.0,
                y: 250.0,
                width: 120.0,
                height: 40.0,
            }),
        }],
        _ => vec![],
    };
    let analysis = Analysis {
        id: Uuid::new_v4(),
        analysis_type: input.analysis_type,
        filename: input.filename,
        results,
        created_at: Utc::now().to_rfc3339(),
    };
    data.analyses.insert(analysis.id, analysis.clone());
    data.history.push(analysis.clone());
    Ok((StatusCode::CREATED, Json(analysis)))
}

async fn get_history(
    State(state): State<SharedVisionState>,
) -> Result<Json<Vec<Analysis>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.history.clone()))
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route("/api/vision/analyze", post(analyze_image))
        .route("/api/vision/history", get(get_history))
        .with_state(state)
}
