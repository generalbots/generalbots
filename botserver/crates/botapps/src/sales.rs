use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealStage {
    Lead,
    Qualified,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: Uuid,
    pub title: String,
    pub company: String,
    pub value: f64,
    pub stage: DealStage,
    pub owner: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub company: String,
    pub last_contact: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleActivity {
    pub id: Uuid,
    pub deal_id: Uuid,
    pub activity_type: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub total_pipeline: f64,
    pub weighted_pipeline: f64,
    pub deals_by_stage: Vec<(String, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeal {
    pub title: String,
    pub company: String,
    pub value: f64,
    pub stage: DealStage,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDeal {
    pub title: Option<String>,
    pub company: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<DealStage>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContact {
    pub name: String,
    pub email: String,
    pub company: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivity {
    pub deal_id: Uuid,
    pub activity_type: String,
    pub notes: String,
}

#[derive(Clone)]
pub struct SalesState {
    pub deals: Arc<RwLock<Vec<Deal>>>,
    pub contacts: Arc<RwLock<Vec<Contact>>>,
    pub activities: Arc<RwLock<Vec<SaleActivity>>>,
}

impl SalesState {
    pub fn new() -> Self {
        Self {
            deals: Arc::new(RwLock::new(Vec::new())),
            contacts: Arc::new(RwLock::new(Vec::new())),
            activities: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_deals(AxumState(state): AxumState<SalesState>) -> Json<ApiResponse<Vec<Deal>>> {
    let deals = state.deals.read().unwrap().clone();
    Json(ApiResponse { success: true, data: deals })
}

async fn create_deal(
    AxumState(state): AxumState<SalesState>,
    Json(payload): Json<CreateDeal>,
) -> Json<ApiResponse<Deal>> {
    let deal = Deal {
        id: Uuid::new_v4(),
        title: payload.title,
        company: payload.company,
        value: payload.value,
        stage: payload.stage,
        owner: payload.owner,
        created_at: Utc::now(),
    };
    state.deals.write().unwrap().push(deal.clone());
    Json(ApiResponse { success: true, data: deal })
}

async fn update_deal(
    AxumState(state): AxumState<SalesState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDeal>,
) -> Json<ApiResponse<Deal>> {
    let mut deals = state.deals.write().unwrap();
    let deal = deals.iter_mut().find(|d| d.id == id).expect("Deal not found");
    if let Some(title) = payload.title { deal.title = title; }
    if let Some(company) = payload.company { deal.company = company; }
    if let Some(value) = payload.value { deal.value = value; }
    if let Some(stage) = payload.stage { deal.stage = stage; }
    if let Some(owner) = payload.owner { deal.owner = owner; }
    Json(ApiResponse { success: true, data: deal.clone() })
}

async fn list_contacts(AxumState(state): AxumState<SalesState>) -> Json<ApiResponse<Vec<Contact>>> {
    let contacts = state.contacts.read().unwrap().clone();
    Json(ApiResponse { success: true, data: contacts })
}

async fn create_contact(
    AxumState(state): AxumState<SalesState>,
    Json(payload): Json<CreateContact>,
) -> Json<ApiResponse<Contact>> {
    let contact = Contact {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
        company: payload.company,
        last_contact: None,
    };
    state.contacts.write().unwrap().push(contact.clone());
    Json(ApiResponse { success: true, data: contact })
}

async fn list_activities(AxumState(state): AxumState<SalesState>) -> Json<ApiResponse<Vec<SaleActivity>>> {
    let activities = state.activities.read().unwrap().clone();
    Json(ApiResponse { success: true, data: activities })
}

async fn create_activity(
    AxumState(state): AxumState<SalesState>,
    Json(payload): Json<CreateActivity>,
) -> Json<ApiResponse<SaleActivity>> {
    let activity = SaleActivity {
        id: Uuid::new_v4(),
        deal_id: payload.deal_id,
        activity_type: payload.activity_type,
        notes: payload.notes,
        created_at: Utc::now(),
    };
    state.activities.write().unwrap().push(activity.clone());
    Json(ApiResponse { success: true, data: activity })
}

async fn get_forecast(AxumState(state): AxumState<SalesState>) -> Json<ApiResponse<Forecast>> {
    let deals = state.deals.read().unwrap();
    let total_pipeline: f64 = deals.iter().map(|d| d.value).sum();
    let weighted_pipeline: f64 = deals.iter().map(|d| match d.stage {
        DealStage::Lead => d.value * 0.1,
        DealStage::Qualified => d.value * 0.25,
        DealStage::Proposal => d.value * 0.5,
        DealStage::Negotiation => d.value * 0.75,
        DealStage::Won => d.value,
        DealStage::Lost => 0.0,
    }).sum();
    let mut stage_counts: Vec<(String, i32)> = Vec::new();
    for deal in deals.iter() {
        let stage_name = match &deal.stage {
            DealStage::Lead => "lead",
            DealStage::Qualified => "qualified",
            DealStage::Proposal => "proposal",
            DealStage::Negotiation => "negotiation",
            DealStage::Won => "won",
            DealStage::Lost => "lost",
        };
        if let Some(entry) = stage_counts.iter_mut().find(|(s, _)| s == stage_name) {
            entry.1 += 1;
        } else {
            stage_counts.push((stage_name.to_string(), 1));
        }
    }
    let forecast = Forecast {
        total_pipeline,
        weighted_pipeline,
        deals_by_stage: stage_counts,
    };
    Json(ApiResponse { success: true, data: forecast })
}

pub fn routes() -> Router {
    let state = SalesState::new();
    Router::new()
        .route("/api/sales/deals", get(list_deals).post(create_deal))
        .route("/api/sales/deals/{id}", put(update_deal))
        .route("/api/sales/contacts", get(list_contacts).post(create_contact))
        .route("/api/sales/activities", get(list_activities).post(create_activity))
        .route("/api/sales/forecast", get(get_forecast))
        .with_state(state)
}
