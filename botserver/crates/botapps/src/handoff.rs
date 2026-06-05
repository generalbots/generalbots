use axum::{extract::{State, Json, Path}, routing::{get, post}, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffRequest {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: String,
    pub bot_id: String,
    pub reason: String,
    pub priority: String,
    pub status: String,
    pub assigned_agent: Option<String>,
    pub channel: String,
    pub created_at: String,
    pub accepted_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub endpoint: String,
    pub active: bool,
    pub capacity: u32,
    pub current_load: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffAnalytics {
    pub id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub period: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Csatisfaction {
    pub id: Uuid,
    pub handoff_id: Uuid,
    pub rating: u32,
    pub feedback: Option<String>,
    pub created_at: String,
}

#[derive(Default)]
pub struct HandoffState {
    pub queue: HashMap<Uuid, HandoffRequest>,
    pub channels: HashMap<Uuid, HandoffChannel>,
    pub analytics: HashMap<Uuid, HandoffAnalytics>,
    pub csat: HashMap<Uuid, Csatisfaction>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(HandoffState::default()));
    Router::new()
        .route("/api/handoff/queue", get(list_queue).post(create_request))
        .route("/api/handoff/queue/{id}", get(get_request).put(update_request))
        .route("/api/handoff/transfer/{id}", post(transfer_request))
        .route("/api/handoff/analytics", get(list_analytics).post(create_analytics))
        .route("/api/handoff/channels", get(list_channels).post(create_channel))
        .route("/api/handoff/channels/{id}", get(get_channel).put(update_channel).delete(delete_channel))
        .route("/api/handoff/csat", get(list_csat).post(create_csat))
        .route("/api/handoff/csat/{id}", get(get_csat))
        .with_state(state)
}

async fn list_queue(State(state): State<Arc<RwLock<HandoffState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&HandoffRequest> = s.queue.values().collect();
    Json(serde_json::json!({"queue": items}))
}

async fn create_request(State(state): State<Arc<RwLock<HandoffState>>>, Json(mut req): Json<HandoffRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    req.id = id;
    req.status = "Pending".to_string();
    req.created_at = Utc::now().to_rfc3339();
    s.queue.insert(id, req.clone());
    Json(serde_json::json!({"request": req}))
}

async fn get_request(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.queue.get(&id) {
        Some(r) => Json(serde_json::json!({"request": r})),
        None => Json(serde_json::json!({"error": "Request not found"})),
    }
}

async fn update_request(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>, Json(req): Json<HandoffRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.queue.get_mut(&id) {
        *existing = req.clone();
        existing.id = id;
        Json(serde_json::json!({"request": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Request not found"}))
    }
}

async fn transfer_request(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    match s.queue.get_mut(&id) {
        Some(req) => {
            req.status = "Transferred".to_string();
            req.accepted_at = Some(Utc::now().to_rfc3339());
            Json(serde_json::json!({"request": req.clone(), "transferred": true}))
        }
        None => Json(serde_json::json!({"error": "Request not found"})),
    }
}

async fn list_channels(State(state): State<Arc<RwLock<HandoffState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&HandoffChannel> = s.channels.values().collect();
    Json(serde_json::json!({"channels": items}))
}

async fn create_channel(State(state): State<Arc<RwLock<HandoffState>>>, Json(mut ch): Json<HandoffChannel>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    ch.id = id;
    ch.active = true;
    ch.current_load = 0;
    ch.created_at = Utc::now().to_rfc3339();
    s.channels.insert(id, ch.clone());
    Json(serde_json::json!({"channel": ch}))
}

async fn get_channel(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.channels.get(&id) {
        Some(c) => Json(serde_json::json!({"channel": c})),
        None => Json(serde_json::json!({"error": "Channel not found"})),
    }
}

async fn update_channel(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>, Json(ch): Json<HandoffChannel>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.channels.get_mut(&id) {
        *existing = ch.clone();
        existing.id = id;
        Json(serde_json::json!({"channel": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Channel not found"}))
    }
}

async fn delete_channel(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.channels.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_analytics(State(state): State<Arc<RwLock<HandoffState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&HandoffAnalytics> = s.analytics.values().collect();
    Json(serde_json::json!({"analytics": items}))
}

async fn create_analytics(State(state): State<Arc<RwLock<HandoffState>>>, Json(mut metric): Json<HandoffAnalytics>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    metric.id = id;
    metric.created_at = Utc::now().to_rfc3339();
    s.analytics.insert(id, metric.clone());
    Json(serde_json::json!({"analytics": metric}))
}

async fn list_csat(State(state): State<Arc<RwLock<HandoffState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Csatisfaction> = s.csat.values().collect();
    Json(serde_json::json!({"csat": items}))
}

async fn create_csat(State(state): State<Arc<RwLock<HandoffState>>>, Json(mut csat): Json<Csatisfaction>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    csat.id = id;
    csat.created_at = Utc::now().to_rfc3339();
    s.csat.insert(id, csat.clone());
    Json(serde_json::json!({"csat": csat}))
}

async fn get_csat(State(state): State<Arc<RwLock<HandoffState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.csat.get(&id) {
        Some(c) => Json(serde_json::json!({"csat": c})),
        None => Json(serde_json::json!({"error": "CSAT not found"})),
    }
}
