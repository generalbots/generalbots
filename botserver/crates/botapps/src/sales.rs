use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Deal {
    pub id: String,
    pub title: String,
    pub contact_id: String,
    pub value: f64,
    pub stage: String,
    pub status: String,
    pub probability: f64,
    pub created_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    pub id: String,
    pub deal_id: String,
    pub kind: String,
    pub description: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Forecast {
    pub period: String,
    pub pipeline_value: f64,
    pub weighted_value: f64,
    pub deals_count: u64,
    pub expected_close: String,
}

#[derive(Default)]
struct AppState {
    deals: HashMap<String, Deal>,
    contacts: HashMap<String, Contact>,
    activities: HashMap<String, Activity>,
    forecast: Vec<Forecast>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_deals() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Deal> = s.deals.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_deal(Json(item): Json<Deal>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "open".to_string();
    s.deals.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn update_deal(Path(id): Path<String>, Json(item): Json<Deal>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    if let Some(existing) = s.deals.get_mut(&id) {
        existing.title = item.title;
        existing.contact_id = item.contact_id;
        existing.value = item.value;
        existing.stage = item.stage;
        existing.status = item.status;
        existing.probability = item.probability;
        Json(serde_json::json!({"item": existing}))
    } else {
        Json(serde_json::json!({"error": "Deal not found"}))
    }
}

pub async fn list_contacts() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Contact> = s.contacts.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_activities() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Activity> = s.activities.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn get_forecast() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Forecast> = s.forecast.iter().collect();
    Json(serde_json::json!({"items": items}))
}
