use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: Uuid,
    pub name: String,
    pub order: i32,
    pub probability: i32,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealStageHistory {
    pub id: Uuid,
    pub deal_id: Uuid,
    pub from_stage: Option<String>,
    pub to_stage: String,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSummary {
    pub stage_name: String,
    pub deal_count: i32,
    pub total_value: f64,
    pub weighted_value: f64,
    pub stage_order: i32,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRate {
    pub from_stage: String,
    pub to_stage: String,
    pub conversion_rate: f64,
    pub deals_entered: i32,
    pub deals_converted: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub predicted_revenue: f64,
    pub weighted_revenue: f64,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

pub struct SalesFunnelService;

impl SalesFunnelService {
    pub fn move_stage(
        deal_id: Uuid,
        from_stage: Option<String>,
        to_stage: String,
        changed_by: Option<Uuid>,
        reason: Option<String>,
    ) -> DealStageHistory {
        DealStageHistory {
            id: Uuid::new_v4(),
            deal_id,
            from_stage,
            to_stage,
            changed_by,
            changed_at: Utc::now(),
            reason,
        }
    }

    pub fn get_pipeline_summary(
        stages: &[PipelineStage],
        deals_by_stage: &HashMap<String, Vec<(f64, Uuid)>>,
    ) -> Vec<PipelineSummary> {
        let mut summaries: Vec<PipelineSummary> = stages
            .iter()
            .map(|stage| {
                let deals = deals_by_stage.get(&stage.name).map(|v| v.as_slice()).unwrap_or(&[]);
                let deal_count = deals.len() as i32;
                let total_value: f64 = deals.iter().map(|(v, _)| v).sum();
                let prob = stage.probability as f64 / 100.0;
                let weighted_value = total_value * prob;
                PipelineSummary {
                    stage_name: stage.name.clone(),
                    deal_count,
                    total_value,
                    weighted_value,
                    stage_order: stage.order,
                    color: stage.color.clone(),
                }
            })
            .collect();
        summaries.sort_by_key(|s| s.stage_order);
        summaries
    }

    pub fn conversion_rates(
        stages: &[PipelineStage],
        history: &[DealStageHistory],
    ) -> Vec<ConversionRate> {
        let mut rates = Vec::new();
        for window in stages.windows(2) {
            let from = &window[0];
            let to = &window[1];
            let deals_entered = history
                .iter()
                .filter(|h| h.to_stage == from.name || h.from_stage.as_deref() == Some(&from.name))
                .count() as i32;
            let deals_converted = history
                .iter()
                .filter(|h| {
                    h.from_stage.as_deref() == Some(&from.name) && h.to_stage == to.name
                })
                .count() as i32;
            let conversion_rate = if deals_entered > 0 {
                deals_converted as f64 / deals_entered as f64
            } else {
                0.0
            };
            rates.push(ConversionRate {
                from_stage: from.name.clone(),
                to_stage: to.name.clone(),
                conversion_rate,
                deals_entered,
                deals_converted,
            });
        }
        rates
    }

    pub fn calculate_forecast(
        stages: &[PipelineStage],
        deals_by_stage: &HashMap<String, Vec<(f64, Uuid)>>,
        historical_win_rate: f64,
    ) -> Forecast {
        let mut predicted_revenue = 0.0_f64;
        let mut weighted_revenue = 0.0_f64;
        for stage in stages {
            let deals = deals_by_stage.get(&stage.name).map(|v| v.as_slice()).unwrap_or(&[]);
            let stage_total: f64 = deals.iter().map(|(v, _)| v).sum();
            let prob = stage.probability as f64 / 100.0;
            predicted_revenue += stage_total;
            weighted_revenue += stage_total * prob;
        }
        let confidence = historical_win_rate * 100.0;
        Forecast {
            id: Uuid::new_v4(),
            bot_id: Uuid::nil(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            predicted_revenue,
            weighted_revenue,
            confidence,
            created_at: Utc::now(),
        }
    }
}

impl Default for SalesFunnelService {
    fn default() -> Self {
        Self
    }
}
