use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{Datelike, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::email_integration::{
    EmailIntegrationService, EmailTracking, EmailTrackingEvent, TrackedEmailRequest,
};
use crate::forecast::{DealInput, ForecastService, HistoricalTrend, MonthlyInput, SalesForecast};
use crate::handlers::{contacts as contacts_handlers, crm as crm_handlers};
use crate::sales_funnel::{DealStageHistory, PipelineStage, SalesFunnelService};
use crate::CrateState;

#[derive(Debug, Deserialize)]
pub struct MoveStageRequest {
    pub deal_id: Uuid,
    pub to_stage: String,
    pub from_stage: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipelineSummaryResponse {
    pub stages: Vec<PipelineStageResponse>,
    pub total_weighted: f64,
    pub total_predicted: f64,
}

#[derive(Debug, Serialize)]
pub struct PipelineStageResponse {
    pub name: String,
    pub deal_count: i32,
    pub total_value: f64,
    pub weighted_value: f64,
    pub order: i32,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastQuery {
    pub bot_id: Option<Uuid>,
    pub months: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct EmailWebhookBody {
    pub email_id: Uuid,
    pub event_type: String,
    pub recipient: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

async fn handle_funnel_summary(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
) -> Result<Json<PipelineSummaryResponse>, (StatusCode, String)> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
    let scoped_branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| state.get_bot_context());

    let stages = vec![
        PipelineStage {
            id: Uuid::new_v4(),
            name: "new".to_string(),
            order: 1,
            probability: 10,
            color: Some("#6b7280".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "qualified".to_string(),
            order: 2,
            probability: 25,
            color: Some("#3b82f6".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "proposal".to_string(),
            order: 3,
            probability: 50,
            color: Some("#f59e0b".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "negotiation".to_string(),
            order: 4,
            probability: 75,
            color: Some("#f97316".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "won".to_string(),
            order: 5,
            probability: 100,
            color: Some("#22c55e".to_string()),
        },
    ];

    let rows: Vec<(Option<String>, Option<f64>, Uuid)> = crate::schema::crm_deals::table
        .filter(crate::schema::crm_deals::branch_id.eq(scoped_branch_id))
        .select((
            crate::schema::crm_deals::stage,
            crate::schema::crm_deals::value,
            crate::schema::crm_deals::id,
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load deals: {e}")))?;

    let mut deals_by_stage: HashMap<String, Vec<(f64, Uuid)>> = HashMap::new();
    for (deal_stage, deal_value, deal_id) in rows {
        let key = deal_stage.unwrap_or_else(|| "new".to_string());
        deals_by_stage.entry(key).or_default().push((deal_value.unwrap_or(0.0), deal_id));
    }
    let summaries = SalesFunnelService::get_pipeline_summary(&stages, &deals_by_stage);

    let stage_responses: Vec<PipelineStageResponse> = summaries
        .iter()
        .map(|s| PipelineStageResponse {
            name: s.stage_name.clone(),
            deal_count: s.deal_count,
            total_value: s.total_value,
            weighted_value: s.weighted_value,
            order: s.stage_order,
            color: s.color.clone(),
        })
        .collect();

    let total_weighted: f64 = stage_responses.iter().map(|s| s.weighted_value).sum();
    let total_predicted: f64 = stage_responses.iter().map(|s| s.total_value).sum();

    Ok(Json(PipelineSummaryResponse {
        stages: stage_responses,
        total_weighted,
        total_predicted,
    }))
}

async fn handle_move_stage(
    State(_state): State<Arc<CrateState>>,
    Json(req): Json<MoveStageRequest>,
) -> Result<Json<DealStageHistory>, (StatusCode, String)> {
    let history = SalesFunnelService::move_stage(
        req.deal_id,
        req.from_stage,
        req.to_stage,
        None,
        req.reason,
    );
    Ok(Json(history))
}

async fn handle_forecast(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Query(query): Query<ForecastQuery>,
) -> Result<Json<SalesForecast>, (StatusCode, String)> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
    let scoped_branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or_else(|| state.get_bot_context());
    let bot_id = query.bot_id.unwrap_or(scoped_branch_id);

    #[derive(diesel::QueryableByName)]
    struct DealRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
        value: Option<f64>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
        won: Option<bool>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        stage: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }

    let deals: Vec<DealRow> = diesel::sql_query(
        "SELECT value, won, stage, created_at FROM crm_deals WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(scoped_branch_id)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load deals: {e}")))?;

    let now = Utc::now();
    let now_year = now.year();
    let now_month = now.month();

    let mut open_value: f64 = 0.0;
    let mut open_count: i32 = 0;
    let mut won_value: f64 = 0.0;
    let mut lost_value: f64 = 0.0;
    let mut won_count: i32 = 0;
    let mut total_count: i32 = 0;

    for deal in &deals {
        total_count += 1;
        let value = deal.value.unwrap_or(0.0);
        let stage = deal.stage.as_deref().unwrap_or("");
        let is_won = deal.won == Some(true) || stage == "won" || stage == "converted";
        let is_lost = deal.won == Some(false) || stage == "lost";
        if is_won {
            won_value += value;
            won_count += 1;
        } else if is_lost {
            lost_value += value;
        } else if deal.created_at.year() == now_year && deal.created_at.month() == now_month {
            open_value += value;
            open_count += 1;
        }
    }

    let monthly_inputs = if open_count > 0 {
        vec![MonthlyInput {
            year: now_year,
            month: now_month,
            predicted_value: open_value,
            deal_count: open_count,
        }]
    } else {
        Vec::new()
    };

    let history = if total_count > 0 {
        vec![HistoricalTrend {
            period: "current".to_string(),
            total_value: won_value + lost_value,
            won_value,
            lost_value,
            win_rate: won_count as f64 / total_count as f64,
            deal_count: total_count,
        }]
    } else {
        Vec::new()
    };

    let historical_win_rate = ForecastService::analyze_historical_trends(&history);
    let forecast = ForecastService::build_monthly_forecast(bot_id, &monthly_inputs, historical_win_rate);

    Ok(Json(forecast))
}

async fn handle_ai_prediction(
    State(_state): State<Arc<CrateState>>,
    Json(deals): Json<Vec<DealInput>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = ForecastService::ai_prediction_stub(&deals)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::json!(result)))
}

async fn handle_email_tracking_webhook(
    State(_state): State<Arc<CrateState>>,
    Json(body): Json<EmailWebhookBody>,
) -> Result<Json<EmailTrackingEvent>, (StatusCode, String)> {
    let event = match body.event_type.as_str() {
        "open" => EmailIntegrationService::open_tracking_pixel(body.email_id),
        "click" => {
            let url = body
                .metadata
                .as_ref()
                .and_then(|m| m.get("url").and_then(|u| u.as_str()))
                .unwrap_or("unknown");
            EmailIntegrationService::click_tracking(body.email_id, url)
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown event type: {}", body.event_type),
            ));
        }
    };
    Ok(Json(event))
}

async fn handle_send_tracked_email(
    State(_state): State<Arc<CrateState>>,
    Json(req): Json<TrackedEmailRequest>,
) -> Result<Json<EmailTracking>, (StatusCode, String)> {
    let result = EmailIntegrationService::send_tracked_email(req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

async fn handle_conversion_rates(
    State(_state): State<Arc<CrateState>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let stages = vec![
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Prospecting".to_string(),
            order: 1,
            probability: 10,
            color: None,
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Qualification".to_string(),
            order: 2,
            probability: 25,
            color: None,
        },
    ];
    let history = Vec::new();
    let rates = SalesFunnelService::conversion_rates(&stages, &history);
    let json_rates: Vec<serde_json::Value> = rates
        .iter()
        .map(|r| {
            serde_json::json!({
                "from_stage": r.from_stage,
                "to_stage": r.to_stage,
                "conversion_rate": r.conversion_rate,
                "deals_entered": r.deals_entered,
                "deals_converted": r.deals_converted,
            })
        })
        .collect();
    Ok(Json(json_rates))
}

pub fn configure_sales_routes() -> Router<Arc<CrateState>> {
    Router::new()
        .route("/api/sales/deals", get(crm_handlers::list_deals).post(crm_handlers::create_deal))
        .route(
            "/api/sales/deals/:id",
            get(crm_handlers::get_deal)
                .patch(crm_handlers::update_deal)
                .delete(crm_handlers::delete_deal),
        )
        .route("/api/sales/contacts", get(contacts_handlers::list_contacts))
        .route("/api/sales/activities", get(crm_handlers::list_activities))
        .route("/api/sales/funnel", get(handle_funnel_summary))
        .route("/api/sales/funnel/move", post(handle_move_stage))
        .route("/api/sales/funnel/conversion-rates", get(handle_conversion_rates))
        .route("/api/sales/forecast", get(handle_forecast))
        .route("/api/sales/forecast/ai-predict", post(handle_ai_prediction))
        .route("/api/sales/email/send", post(handle_send_tracked_email))
        .route("/api/sales/email/webhook", post(handle_email_tracking_webhook))
}
