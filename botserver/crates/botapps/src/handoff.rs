use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelStatus {
    Connected,
    Disconnected,
    Overloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffItem {
    pub id: Uuid,
    pub user_name: String,
    pub conversation_preview: String,
    pub assigned_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffAnalytics {
    pub conversations_today: i32,
    pub avg_response_time: f64,
    pub resolution_rate: f64,
    pub bot_containment_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatusItem {
    pub channel: String,
    pub status: ChannelStatus,
    pub active_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsatEntry {
    pub id: Uuid,
    pub user_name: String,
    pub score: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCsatEntry {
    pub user_name: String,
    pub score: i32,
    pub comment: Option<String>,
}

#[derive(Clone)]
pub struct HandoffState {
    pub queue: Arc<RwLock<Vec<HandoffItem>>>,
    pub analytics: Arc<RwLock<HandoffAnalytics>>,
    pub channels: Arc<RwLock<HashMap<String, ChannelStatusItem>>>,
    pub csat: Arc<RwLock<Vec<CsatEntry>>>,
}

impl HandoffState {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            analytics: Arc::new(RwLock::new(HandoffAnalytics {
                conversations_today: 0,
                avg_response_time: 0.0,
                resolution_rate: 0.0,
                bot_containment_rate: 0.0,
            })),
            channels: Arc::new(RwLock::new(HashMap::new())),
            csat: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_queue(AxumState(state): AxumState<HandoffState>) -> Json<ApiResponse<Vec<HandoffItem>>> {
    let queue = state.queue.read().unwrap().clone();
    Json(ApiResponse { success: true, data: queue })
}

async fn create_queue_item(
    AxumState(state): AxumState<HandoffState>,
    Json(payload): Json<HandoffItem>,
) -> Json<ApiResponse<HandoffItem>> {
    let item = HandoffItem {
        id: Uuid::new_v4(),
        user_name: payload.user_name,
        conversation_preview: payload.conversation_preview,
        assigned_agent: None,
        created_at: Utc::now(),
    };
    state.queue.write().unwrap().push(item.clone());
    let mut analytics = state.analytics.write().unwrap();
    analytics.conversations_today += 1;
    Json(ApiResponse { success: true, data: item })
}

async fn transfer_to_agent(
    AxumState(state): AxumState<HandoffState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransferRequest>,
) -> Json<ApiResponse<HandoffItem>> {
    let mut queue = state.queue.write().unwrap();
    let item = queue.iter_mut().find(|i| i.id == id).expect("Item not found in queue");
    item.assigned_agent = Some(payload.agent);
    let result = item.clone();
    let mut analytics = state.analytics.write().unwrap();
    analytics.avg_response_time = (analytics.avg_response_time + 1.5) / 2.0;
    Json(ApiResponse { success: true, data: result })
}

async fn get_analytics(AxumState(state): AxumState<HandoffState>) -> Json<ApiResponse<HandoffAnalytics>> {
    let analytics = state.analytics.read().unwrap().clone();
    Json(ApiResponse { success: true, data: analytics })
}

async fn list_channels(AxumState(state): AxumState<HandoffState>) -> Json<ApiResponse<Vec<ChannelStatusItem>>> {
    let channels = state.channels.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: channels })
}

async fn update_channel(
    AxumState(state): AxumState<HandoffState>,
    Json(payload): Json<ChannelStatusItem>,
) -> Json<ApiResponse<ChannelStatusItem>> {
    state.channels.write().unwrap().insert(payload.channel.clone(), payload.clone());
    Json(ApiResponse { success: true, data: payload })
}

async fn list_csat(AxumState(state): AxumState<HandoffState>) -> Json<ApiResponse<Vec<CsatEntry>>> {
    let entries = state.csat.read().unwrap().clone();
    Json(ApiResponse { success: true, data: entries })
}

async fn create_csat(
    AxumState(state): AxumState<HandoffState>,
    Json(payload): Json<CreateCsatEntry>,
) -> Json<ApiResponse<CsatEntry>> {
    let entry = CsatEntry {
        id: Uuid::new_v4(),
        user_name: payload.user_name,
        score: payload.score,
        comment: payload.comment,
        created_at: Utc::now(),
    };
    state.csat.write().unwrap().push(entry.clone());
    let mut analytics = state.analytics.write().unwrap();
    let csat_entries = state.csat.read().unwrap();
    let total_score: i32 = csat_entries.iter().map(|e| e.score).sum();
    analytics.resolution_rate = (total_score as f64 / csat_entries.len() as f64) * 20.0;
    Json(ApiResponse { success: true, data: entry })
}

pub fn routes() -> Router {
    let state = HandoffState::new();
    Router::new()
        .route("/api/handoff/queue", get(list_queue).post(create_queue_item))
        .route("/api/handoff/transfer/{id}", post(transfer_to_agent))
        .route("/api/handoff/analytics", get(get_analytics))
        .route("/api/handoff/channels", get(list_channels).post(update_channel))
        .route("/api/handoff/csat", get(list_csat).post(create_csat))
        .with_state(state)
}
