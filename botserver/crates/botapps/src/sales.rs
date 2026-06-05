use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Deal {
    pub id: Uuid,
    pub title: String,
    pub company: String,
    pub contact: String,
    pub value: f64,
    pub currency: String,
    pub stage: String,
    pub probability: u32,
    pub expected_close_date: String,
    pub assigned_to: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub position: String,
    pub tags: String,
    pub last_contact_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SalesActivity {
    pub id: Uuid,
    pub deal_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub activity_type: String,
    pub description: String,
    pub due_date: Option<String>,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForecastEntry {
    pub id: Uuid,
    pub period: String,
    pub pipeline_value: f64,
    pub weighted_value: f64,
    pub closed_value: f64,
    pub won_value: f64,
    pub lost_value: f64,
    pub created_at: String,
}

#[derive(Default)]
pub struct SalesState {
    pub deals: HashMap<Uuid, Deal>,
    pub contacts: HashMap<Uuid, Contact>,
    pub activities: HashMap<Uuid, SalesActivity>,
    pub forecasts: HashMap<Uuid, ForecastEntry>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(SalesState::default()));
    Router::new()
        .route("/api/sales/deals", get(list_deals).post(create_deal))
        .route("/api/sales/deals/{id}", get(get_deal).put(update_deal).delete(delete_deal))
        .route("/api/sales/contacts", get(list_contacts).post(create_contact))
        .route("/api/sales/contacts/{id}", get(get_contact).put(update_contact).delete(delete_contact))
        .route("/api/sales/activities", get(list_activities).post(create_activity))
        .route("/api/sales/activities/{id}", get(get_activity).put(update_activity).delete(delete_activity))
        .route("/api/sales/forecast", get(list_forecast).post(create_forecast))
        .route("/api/sales/forecast/{id}", get(get_forecast))
        .with_state(state)
}

async fn list_deals(State(state): State<Arc<RwLock<SalesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Deal> = s.deals.values().collect();
    Json(serde_json::json!({"deals": items}))
}

async fn create_deal(State(state): State<Arc<RwLock<SalesState>>>, Json(mut deal): Json<Deal>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    deal.id = id;
    deal.stage = "Prospecting".to_string();
    deal.created_at = Utc::now().to_rfc3339();
    s.deals.insert(id, deal.clone());
    Json(serde_json::json!({"deal": deal}))
}

async fn get_deal(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.deals.get(&id) {
        Some(d) => Json(serde_json::json!({"deal": d})),
        None => Json(serde_json::json!({"error": "Deal not found"})),
    }
}

async fn update_deal(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>, Json(deal): Json<Deal>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.deals.get_mut(&id) {
        *existing = deal.clone();
        existing.id = id;
        Json(serde_json::json!({"deal": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Deal not found"}))
    }
}

async fn delete_deal(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.deals.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_contacts(State(state): State<Arc<RwLock<SalesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Contact> = s.contacts.values().collect();
    Json(serde_json::json!({"contacts": items}))
}

async fn create_contact(State(state): State<Arc<RwLock<SalesState>>>, Json(mut contact): Json<Contact>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    contact.id = id;
    contact.created_at = Utc::now().to_rfc3339();
    s.contacts.insert(id, contact.clone());
    Json(serde_json::json!({"contact": contact}))
}

async fn get_contact(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.contacts.get(&id) {
        Some(c) => Json(serde_json::json!({"contact": c})),
        None => Json(serde_json::json!({"error": "Contact not found"})),
    }
}

async fn update_contact(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>, Json(contact): Json<Contact>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.contacts.get_mut(&id) {
        *existing = contact.clone();
        existing.id = id;
        Json(serde_json::json!({"contact": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Contact not found"}))
    }
}

async fn delete_contact(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.contacts.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_activities(State(state): State<Arc<RwLock<SalesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&SalesActivity> = s.activities.values().collect();
    Json(serde_json::json!({"activities": items}))
}

async fn create_activity(State(state): State<Arc<RwLock<SalesState>>>, Json(mut activity): Json<SalesActivity>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    activity.id = id;
    activity.completed = false;
    activity.created_at = Utc::now().to_rfc3339();
    s.activities.insert(id, activity.clone());
    Json(serde_json::json!({"activity": activity}))
}

async fn get_activity(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.activities.get(&id) {
        Some(a) => Json(serde_json::json!({"activity": a})),
        None => Json(serde_json::json!({"error": "Activity not found"})),
    }
}

async fn update_activity(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>, Json(activity): Json<SalesActivity>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.activities.get_mut(&id) {
        *existing = activity.clone();
        existing.id = id;
        Json(serde_json::json!({"activity": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Activity not found"}))
    }
}

async fn delete_activity(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.activities.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_forecast(State(state): State<Arc<RwLock<SalesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&ForecastEntry> = s.forecasts.values().collect();
    Json(serde_json::json!({"forecasts": items}))
}

async fn create_forecast(State(state): State<Arc<RwLock<SalesState>>>, Json(mut forecast): Json<ForecastEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    forecast.id = id;
    forecast.created_at = Utc::now().to_rfc3339();
    s.forecasts.insert(id, forecast.clone());
    Json(serde_json::json!({"forecast": forecast}))
}

async fn get_forecast(State(state): State<Arc<RwLock<SalesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.forecasts.get(&id) {
        Some(f) => Json(serde_json::json!({"forecast": f})),
        None => Json(serde_json::json!({"error": "Forecast not found"})),
    }
}
