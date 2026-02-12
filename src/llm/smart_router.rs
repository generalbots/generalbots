use crate::core::shared::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model_name: String,
    pub avg_latency_ms: u64,
    pub avg_cost_per_token: f64,
    pub success_rate: f64,
    pub total_requests: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum OptimizationGoal {
    Speed,
    Cost,
    Quality,
    Balanced,
}

impl OptimizationGoal {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "speed" => Self::Speed,
            "cost" => Self::Cost,
            "quality" => Self::Quality,
            _ => Self::Balanced,
        }
    }
}

pub struct SmartLLMRouter {
    performance_cache: Arc<tokio::sync::RwLock<HashMap<String, ModelPerformance>>>,
    _app_state: Arc<AppState>,
}

impl SmartLLMRouter {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            performance_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            _app_state: app_state,
        }
    }

    pub async fn select_optimal_model(
        &self,
        _task_type: &str,
        optimization_goal: OptimizationGoal,
        max_cost: Option<f64>,
        max_latency: Option<u64>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let performance_data = self.performance_cache.read().await;

        let mut candidates: Vec<&ModelPerformance> = performance_data.values().collect();

        // Filter by constraints
        if let Some(max_cost) = max_cost {
            candidates.retain(|p| p.avg_cost_per_token <= max_cost);
        }

        if let Some(max_latency) = max_latency {
            candidates.retain(|p| p.avg_latency_ms <= max_latency);
        }

        if candidates.is_empty() {
            return Ok("gpt-4o-mini".to_string()); // Fallback model
        }

        // Select based on optimization goal
        let selected = match optimization_goal {
            OptimizationGoal::Speed => candidates.iter().min_by_key(|p| p.avg_latency_ms),
            OptimizationGoal::Cost => candidates.iter().min_by(|a, b| {
                a.avg_cost_per_token
                    .partial_cmp(&b.avg_cost_per_token)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            OptimizationGoal::Quality => candidates
                .iter()
                .max_by(|a, b| a.success_rate.partial_cmp(&b.success_rate).unwrap_or(std::cmp::Ordering::Equal)),
            OptimizationGoal::Balanced => {
                // Weighted score: 40% success rate, 30% speed, 30% cost
                candidates.iter().max_by(|a, b| {
                    let score_a = (a.success_rate * 0.4)
                        + ((1000.0 / a.avg_latency_ms as f64) * 0.3)
                        + ((1.0 / (a.avg_cost_per_token + 0.001)) * 0.3);
                    let score_b = (b.success_rate * 0.4)
                        + ((1000.0 / b.avg_latency_ms as f64) * 0.3)
                        + ((1.0 / (b.avg_cost_per_token + 0.001)) * 0.3);
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        };

        Ok(selected
            .map(|p| p.model_name.clone())
            .unwrap_or_else(|| "gpt-4o-mini".to_string()))
    }

    pub async fn track_performance(
        &self,
        model_name: &str,
        latency_ms: u64,
        cost_per_token: f64,
        success: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut performance_data = self.performance_cache.write().await;

        let performance = performance_data
            .entry(model_name.to_string())
            .or_insert_with(|| ModelPerformance {
                model_name: model_name.to_string(),
                avg_latency_ms: latency_ms,
                avg_cost_per_token: cost_per_token,
                success_rate: if success { 1.0 } else { 0.0 },
                total_requests: 0,
                last_updated: chrono::Utc::now(),
            });

        // Update running averages
        let total = performance.total_requests as f64;
        performance.avg_latency_ms = ((performance.avg_latency_ms as f64 * total)
            + latency_ms as f64) as u64
            / (total + 1.0) as u64;
        performance.avg_cost_per_token =
            (performance.avg_cost_per_token * total + cost_per_token) / (total + 1.0);

        let success_count = (performance.success_rate * total) + if success { 1.0 } else { 0.0 };
        performance.success_rate = success_count / (total + 1.0);

        performance.total_requests += 1;
        performance.last_updated = chrono::Utc::now();

        Ok(())
    }

    pub async fn get_performance_stats(&self) -> HashMap<String, ModelPerformance> {
        self.performance_cache.read().await.clone()
    }
}

// Enhanced LLM keyword with optimization
pub async fn enhanced_llm_call(
    router: &SmartLLMRouter,
    prompt: &str,
    optimization_goal: OptimizationGoal,
    max_cost: Option<f64>,
    max_latency: Option<u64>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();

    // Select optimal model
    let model = router
        .select_optimal_model("general", optimization_goal, max_cost, max_latency)
        .await?;

    // Make LLM call (simplified - would use actual LLM provider)
    let response = format!("Response from {} for: {}", model, prompt);

    // Track performance
    let latency = start_time.elapsed().as_millis() as u64;
    let cost_per_token = match model.as_str() {
        "gpt-4" => 0.03,
        "gpt-4o-mini" => 0.0015,
        "claude-3-sonnet" => 0.015,
        _ => 0.01,
    };

    router
        .track_performance(&model, latency, cost_per_token, true)
        .await?;

    Ok(response)
}
