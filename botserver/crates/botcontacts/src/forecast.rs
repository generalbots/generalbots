use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyForecast {
    pub year: i32,
    pub month: u32,
    pub predicted_revenue: f64,
    pub weighted_revenue: f64,
    pub deal_count: i32,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesForecast {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub overall_predicted: f64,
    pub overall_weighted: f64,
    pub overall_confidence: f64,
    pub monthly_breakdown: Vec<MonthlyForecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTrend {
    pub period: String,
    pub total_value: f64,
    pub won_value: f64,
    pub lost_value: f64,
    pub win_rate: f64,
    pub deal_count: i32,
}

pub struct ForecastService;

impl ForecastService {
    pub fn analyze_historical_trends(history: &[HistoricalTrend]) -> f64 {
        if history.is_empty() {
            return 0.5;
        }
        let total_win_rate: f64 = history.iter().map(|t| t.win_rate).sum();
        total_win_rate / history.len() as f64
    }

    pub fn weighted_pipeline_calculation(
        stage_values: &[(f64, f64)],
    ) -> (f64, f64) {
        let predicted: f64 = stage_values.iter().map(|(value, _)| value).sum();
        let weighted: f64 = stage_values
            .iter()
            .map(|(value, probability)| value * probability)
            .sum();
        (predicted, weighted)
    }

    pub fn build_monthly_forecast(
        bot_id: Uuid,
        monthly_data: &[MonthlyInput],
        historical_win_rate: f64,
    ) -> SalesForecast {
        let mut monthly_breakdown: Vec<MonthlyForecast> = monthly_data
            .iter()
            .map(|input| {
                let confidence = if input.deal_count > 5 {
                    historical_win_rate.min(0.95)
                } else if input.deal_count > 0 {
                    historical_win_rate.min(0.8) * 0.7
                } else {
                    0.1
                };
                MonthlyForecast {
                    year: input.year,
                    month: input.month,
                    predicted_revenue: input.predicted_value,
                    weighted_revenue: input.predicted_value * historical_win_rate,
                    deal_count: input.deal_count,
                    confidence: confidence * 100.0,
                }
            })
            .collect();
        monthly_breakdown.sort_by(|a, b| a.year.cmp(&b.year).then(a.month.cmp(&b.month)));

        let overall_predicted: f64 = monthly_breakdown.iter().map(|m| m.predicted_revenue).sum();
        let overall_weighted: f64 = monthly_breakdown.iter().map(|m| m.weighted_revenue).sum();
        let avg_confidence = if monthly_breakdown.is_empty() {
            0.0
        } else {
            monthly_breakdown.iter().map(|m| m.confidence).sum::<f64>()
                / monthly_breakdown.len() as f64
        };

        SalesForecast {
            id: Uuid::new_v4(),
            bot_id,
            created_at: Utc::now(),
            overall_predicted,
            overall_weighted,
            overall_confidence: avg_confidence,
            monthly_breakdown,
        }
    }

    pub fn ai_prediction_stub(
        deal_descriptions: &[DealInput],
    ) -> Result<AiPredictionResult, String> {
        let total_deals = deal_descriptions.len();
        if total_deals == 0 {
            return Err("No deals provided for AI prediction".to_string());
        }
        let avg_value: f64 = deal_descriptions.iter().map(|d| d.value).sum::<f64>() / total_deals as f64;
        let close_soon = deal_descriptions.iter().filter(|d| d.stage_probability > 0.7).count();
        let confidence = (close_soon as f64 / total_deals as f64).min(1.0);
        Ok(AiPredictionResult {
            predicted_close_rate: confidence,
            estimated_total: avg_value * total_deals as f64 * confidence,
            confidence_score: confidence * 100.0,
            suggested_focus: if close_soon < total_deals / 2 {
                "Nurture high-probability deals to improve close rate".to_string()
            } else {
                "Pipeline health appears strong; focus on accelerating existing deals".to_string()
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyInput {
    pub year: i32,
    pub month: u32,
    pub predicted_value: f64,
    pub deal_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealInput {
    pub id: Uuid,
    pub title: String,
    pub value: f64,
    pub stage: String,
    pub stage_probability: f64,
    pub expected_close: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPredictionResult {
    pub predicted_close_rate: f64,
    pub estimated_total: f64,
    pub confidence_score: f64,
    pub suggested_focus: String,
}

impl Default for ForecastService {
    fn default() -> Self {
        Self
    }
}
