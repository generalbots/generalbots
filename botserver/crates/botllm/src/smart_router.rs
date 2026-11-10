use botcore::shared::state::AppState;
use botcore::config::ConfigManager;
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
    pub fn from_str_name(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "speed" => Self::Speed,
            "cost" => Self::Cost,
            "quality" => Self::Quality,
            _ => Self::Balanced,
        }
    }
}

/// Model returned when no candidate satisfies the requested constraints.
pub const DEFAULT_MODEL: &str = "gpt-5.6-sol";

/// Score used by [`OptimizationGoal::Balanced`]: 40 % success rate, 30 % speed
/// and 30 % cost.
///
/// A zero latency is treated as the 1 ms floor: `track_performance` records
/// `elapsed().as_millis()`, which is legitimately `0` for an instant response,
/// and `1000.0 / 0.0` is `+inf` — one such sample would pin that model as the
/// permanent winner of every balanced comparison.
fn balanced_score(p: &ModelPerformance) -> f64 {
    let latency_ms = p.avg_latency_ms.max(1) as f64;
    (p.success_rate * 0.4)
        + ((1000.0 / latency_ms) * 0.3)
        + ((1.0 / (p.avg_cost_per_token + 0.001)) * 0.3)
}

/// Pure model-selection decision: apply the cost and latency ceilings, then
/// pick the best candidate for `goal`.
///
/// Returns [`DEFAULT_MODEL`] when the candidate set is empty, before or after
/// filtering, so a caller never has to handle a missing model.
pub fn select_model(
    candidates: &[ModelPerformance],
    goal: OptimizationGoal,
    max_cost: Option<f64>,
    max_latency: Option<u64>,
) -> Option<String> {
    let filtered: Vec<&ModelPerformance> = candidates
        .iter()
        .filter(|p| max_cost.is_none_or(|ceiling| p.avg_cost_per_token <= ceiling))
        .filter(|p| max_latency.is_none_or(|ceiling| p.avg_latency_ms <= ceiling))
        .collect();

    if filtered.is_empty() {
        return Some(DEFAULT_MODEL.to_string());
    }

    let selected = match goal {
        OptimizationGoal::Speed => filtered.iter().min_by_key(|p| p.avg_latency_ms),
        OptimizationGoal::Cost => filtered.iter().min_by(|a, b| {
            a.avg_cost_per_token
                .partial_cmp(&b.avg_cost_per_token)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        OptimizationGoal::Quality => filtered.iter().max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        OptimizationGoal::Balanced => filtered.iter().max_by(|a, b| {
            balanced_score(a)
                .partial_cmp(&balanced_score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    };

    Some(
        selected
            .map(|p| p.model_name.clone())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
    )
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
        let candidates: Vec<ModelPerformance> = performance_data.values().cloned().collect();

        Ok(select_model(
            &candidates,
            optimization_goal,
            max_cost,
            max_latency,
        )
        .unwrap_or_else(|| DEFAULT_MODEL.to_string()))
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
    state: &Arc<AppState>,
    router: &SmartLLMRouter,
    prompt: &str,
    optimization_goal: OptimizationGoal,
    max_cost: Option<f64>,
    max_latency: Option<u64>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();

    // Select optimal model (for tracking)
    let model = router
        .select_optimal_model("general", optimization_goal, max_cost, max_latency)
        .await?;

    // Get actual LLM configuration from bot's config
    let config_manager = ConfigManager::new(state.conn.clone());
    let actual_model = config_manager
        .get_config(&uuid::Uuid::nil(), "llm-model", None)
        .unwrap_or_else(|_| model.clone());
    let key = config_manager
        .get_config(&uuid::Uuid::nil(), "llm-key", None)
        .unwrap_or_else(|_| String::new());

    let response = state
        .llm_provider
        .as_ref()
        .ok_or("No LLM provider configured")?
        .generate(prompt, &serde_json::json!({}), &actual_model, &key)
        .await
        .map_err(|e| format!("LLM error: {}", e))?;

    // Track performance
    let latency = start_time.elapsed().as_millis() as u64;
    let cost_per_token = match actual_model.as_str() {
        "gpt-5.6-sol" => 0.00125,
        "gpt-4o-mini" => 0.0015,
        "claude-sonnet-5-5" => 0.003,
        "claude-3-sonnet" => 0.015,
        _ => 0.01,
    };

    router
        .track_performance(&actual_model, latency, cost_per_token, true)
        .await?;

    Ok(response)
}

#[cfg(test)]
#[path = "smart_router_tests.rs"]
mod smart_router_tests;
