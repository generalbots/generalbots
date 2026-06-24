use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::email_integration::{
    EmailIntegrationService, EmailTracking, EmailTrackingEvent, TrackedEmailRequest,
};
use crate::forecast::{DealInput, ForecastService, HistoricalTrend, MonthlyInput, SalesForecast};
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
    State(_state): State<Arc<CrateState>>,
) -> Result<Json<PipelineSummaryResponse>, (StatusCode, String)> {
    let stages = vec![
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Prospecting".to_string(),
            order: 1,
            probability: 10,
            color: Some("#6b7280".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Qualification".to_string(),
            order: 2,
            probability: 25,
            color: Some("#3b82f6".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Proposal".to_string(),
            order: 3,
            probability: 50,
            color: Some("#f59e0b".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Negotiation".to_string(),
            order: 4,
            probability: 75,
            color: Some("#f97316".to_string()),
        },
        PipelineStage {
            id: Uuid::new_v4(),
            name: "Closed Won".to_string(),
            order: 5,
            probability: 100,
            color: Some("#22c55e".to_string()),
        },
    ];

    let deals_by_stage: HashMap<String, Vec<(f64, Uuid)>> = HashMap::new();
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
    State(_state): State<Arc<CrateState>>,
    Query(query): Query<ForecastQuery>,
) -> Result<Json<SalesForecast>, (StatusCode, String)> {
    let bot_id = query.bot_id.unwrap_or(Uuid::nil());
    let monthly_inputs = vec![
        MonthlyInput {
            year: Utc::now().format("%Y").to_string().parse().unwrap_or(2025),
            month: Utc::now().format("%m").to_string().parse().unwrap_or(1),
            predicted_value: 150000.0,
            deal_count: 12,
        },
    ];

    let history = vec![HistoricalTrend {
        period: "last_quarter".to_string(),
        total_value: 500000.0,
        won_value: 125000.0,
        lost_value: 75000.0,
        win_rate: 0.25,
        deal_count: 40,
    }];

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
        .route("/api/sales/funnel", get(handle_funnel_summary))
        .route("/api/sales/funnel/move", post(handle_move_stage))
        .route("/api/sales/funnel/conversion-rates", get(handle_conversion_rates))
        .route("/api/sales/forecast", get(handle_forecast))
        .route("/api/sales/forecast/ai-predict", post(handle_ai_prediction))
        .route("/api/sales/email/send", post(handle_send_tracked_email))
        .route("/api/sales/email/webhook", post(handle_email_tracking_webhook))
}
