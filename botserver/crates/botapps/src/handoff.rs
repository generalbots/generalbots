use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueEntry {
    pub id: String,
    pub session_id: String,
    pub user_name: String,
    pub channel: String,
    pub priority: String,
    pub waiting_since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferRequest {
    pub agent_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandoffAnalytics {
    pub period: String,
    pub total_transfers: u64,
    pub avg_wait_seconds: f64,
    pub avg_handle_seconds: f64,
    pub satisfaction_avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub active_agents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CsatEntry {
    pub id: String,
    pub session_id: String,
    pub rating: u64,
    pub comment: Option<String>,
    pub submitted_at: String,
}

#[derive(Default)]
struct AppState {
    queue: HashMap<String, QueueEntry>,
    analytics: Vec<HandoffAnalytics>,
    channels: HashMap<String, Channel>,
    csat: HashMap<String, CsatEntry>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_queue() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&QueueEntry> = s.queue.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn transfer_item(Path(id): Path<String>, Json(req): Json<TransferRequest>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    match s.queue.remove(&id) {
        Some(entry) => {
            Json(serde_json::json!({
                "transferred": true,
                "session_id": entry.session_id,
                "agent_id": req.agent_id,
                "notes": req.notes
            }))
        }
        None => Json(serde_json::json!({"error": "Session not found in queue"})),
    }
}

pub async fn get_analytics() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&HandoffAnalytics> = s.analytics.iter().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_channels() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Channel> = s.channels.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_csat() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&CsatEntry> = s.csat.values().collect();
    Json(serde_json::json!({"items": items}))
}
