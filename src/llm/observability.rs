//! LLM Observability - Metrics, Tracing, and Cost Tracking
//!
//! This module provides comprehensive observability for LLM operations including:
//!
//! - Request/response metrics (latency, tokens, cache hits)
//! - Cost tracking and budgeting
//! - Trace visualization for debugging
//! - Model performance comparison
//! - Real-time monitoring via SSE
//!
//! ## Features
//!
//! - Token usage tracking per conversation
//! - Response latency histograms
//! - Cache hit/miss rates
//! - Model selection distribution
//! - Error rate monitoring
//! - Cost estimation and budgeting
//!
//! ## Config.csv Properties
//!
//! ```csv
//! name,value
//! observability-enabled,true
//! observability-metrics-interval,60
//! observability-cost-tracking,true
//! observability-trace-enabled,true
//! observability-trace-sample-rate,1.0
//! observability-budget-daily,100.00
//! observability-budget-monthly,2000.00
//! observability-alert-threshold,0.8
//! ```

use chrono::{DateTime, Utc};
use rhai::{Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// LLM request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequestMetrics {
    /// Request ID
    pub request_id: Uuid,
    /// Session ID
    pub session_id: Uuid,
    /// Bot ID
    pub bot_id: Uuid,
    /// Model used
    pub model: String,
    /// Request type (chat, completion, embedding)
    pub request_type: RequestType,
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Time to first token (for streaming)
    pub ttft_ms: Option<u64>,
    /// Whether response was cached
    pub cached: bool,
    /// Whether request succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Estimated cost in USD
    pub estimated_cost: f64,
    /// When the request was made
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Type of LLM request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    Chat,
    Completion,
    Embedding,
    Rerank,
    Moderation,
    ImageGeneration,
    AudioTranscription,
    AudioGeneration,
}

impl Default for RequestType {
    fn default() -> Self {
        RequestType::Chat
    }
}

impl std::fmt::Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::Chat => write!(f, "chat"),
            RequestType::Completion => write!(f, "completion"),
            RequestType::Embedding => write!(f, "embedding"),
            RequestType::Rerank => write!(f, "rerank"),
            RequestType::Moderation => write!(f, "moderation"),
            RequestType::ImageGeneration => write!(f, "image_generation"),
            RequestType::AudioTranscription => write!(f, "audio_transcription"),
            RequestType::AudioGeneration => write!(f, "audio_generation"),
        }
    }
}

/// Aggregated metrics for a time period
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregatedMetrics {
    /// Time period start
    pub period_start: DateTime<Utc>,
    /// Time period end
    pub period_end: DateTime<Utc>,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total estimated cost
    pub total_cost: f64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// P50 latency
    pub p50_latency_ms: f64,
    /// P95 latency
    pub p95_latency_ms: f64,
    /// P99 latency
    pub p99_latency_ms: f64,
    /// Max latency
    pub max_latency_ms: u64,
    /// Min latency
    pub min_latency_ms: u64,
    /// Requests by model
    pub requests_by_model: HashMap<String, u64>,
    /// Tokens by model
    pub tokens_by_model: HashMap<String, u64>,
    /// Cost by model
    pub cost_by_model: HashMap<String, f64>,
    /// Requests by type
    pub requests_by_type: HashMap<String, u64>,
    /// Error count by type
    pub errors_by_type: HashMap<String, u64>,
}

/// Model pricing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model name/identifier
    pub model: String,
    /// Cost per 1K input tokens
    pub input_cost_per_1k: f64,
    /// Cost per 1K output tokens
    pub output_cost_per_1k: f64,
    /// Cost per request (flat fee)
    pub cost_per_request: f64,
    /// Whether this is a local model (no cost)
    pub is_local: bool,
}

impl Default for ModelPricing {
    fn default() -> Self {
        ModelPricing {
            model: "default".to_string(),
            input_cost_per_1k: 0.0,
            output_cost_per_1k: 0.0,
            cost_per_request: 0.0,
            is_local: true,
        }
    }
}

/// Budget configuration and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Daily budget in USD
    pub daily_limit: f64,
    /// Monthly budget in USD
    pub monthly_limit: f64,
    /// Alert threshold (0-1, e.g., 0.8 = 80%)
    pub alert_threshold: f64,
    /// Current daily spend
    pub daily_spend: f64,
    /// Current monthly spend
    pub monthly_spend: f64,
    /// Last reset date for daily
    pub daily_reset_date: DateTime<Utc>,
    /// Last reset date for monthly
    pub monthly_reset_date: DateTime<Utc>,
    /// Whether daily alert has been sent
    pub daily_alert_sent: bool,
    /// Whether monthly alert has been sent
    pub monthly_alert_sent: bool,
}

impl Default for Budget {
    fn default() -> Self {
        let now = Utc::now();
        Budget {
            daily_limit: 100.0,
            monthly_limit: 2000.0,
            alert_threshold: 0.8,
            daily_spend: 0.0,
            monthly_spend: 0.0,
            daily_reset_date: now,
            monthly_reset_date: now,
            daily_alert_sent: false,
            monthly_alert_sent: false,
        }
    }
}

/// Trace event for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Event ID
    pub id: Uuid,
    /// Parent event ID (for nested spans)
    pub parent_id: Option<Uuid>,
    /// Trace ID (groups related events)
    pub trace_id: Uuid,
    /// Event name
    pub name: String,
    /// Component that generated the event
    pub component: String,
    /// Event type
    pub event_type: TraceEventType,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp
    pub end_time: Option<DateTime<Utc>>,
    /// Event attributes
    pub attributes: HashMap<String, String>,
    /// Status
    pub status: TraceStatus,
    /// Error message if failed
    pub error: Option<String>,
}

/// Type of trace event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    Span,
    Event,
    Log,
    Metric,
}

/// Trace status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TraceStatus {
    Ok,
    Error,
    InProgress,
}

impl Default for TraceStatus {
    fn default() -> Self {
        TraceStatus::InProgress
    }
}

/// Configuration for observability
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Whether observability is enabled
    pub enabled: bool,
    /// Metrics aggregation interval in seconds
    pub metrics_interval: u64,
    /// Whether to track costs
    pub cost_tracking: bool,
    /// Whether tracing is enabled
    pub trace_enabled: bool,
    /// Trace sampling rate (0-1)
    pub trace_sample_rate: f64,
    /// Daily budget
    pub budget_daily: f64,
    /// Monthly budget
    pub budget_monthly: f64,
    /// Alert threshold (0-1)
    pub alert_threshold: f64,
    /// Model pricing configurations
    pub model_pricing: HashMap<String, ModelPricing>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        ObservabilityConfig {
            enabled: true,
            metrics_interval: 60,
            cost_tracking: true,
            trace_enabled: true,
            trace_sample_rate: 1.0,
            budget_daily: 100.0,
            budget_monthly: 2000.0,
            alert_threshold: 0.8,
            model_pricing: default_model_pricing(),
        }
    }
}

/// Default pricing for common models
fn default_model_pricing() -> HashMap<String, ModelPricing> {
    let mut pricing = HashMap::new();

    // OpenAI models
    pricing.insert(
        "gpt-4".to_string(),
        ModelPricing {
            model: "gpt-4".to_string(),
            input_cost_per_1k: 0.03,
            output_cost_per_1k: 0.06,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    pricing.insert(
        "gpt-4-turbo".to_string(),
        ModelPricing {
            model: "gpt-4-turbo".to_string(),
            input_cost_per_1k: 0.01,
            output_cost_per_1k: 0.03,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    pricing.insert(
        "gpt-3.5-turbo".to_string(),
        ModelPricing {
            model: "gpt-3.5-turbo".to_string(),
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    // Anthropic models
    pricing.insert(
        "claude-3-opus".to_string(),
        ModelPricing {
            model: "claude-3-opus".to_string(),
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    pricing.insert(
        "claude-3-sonnet".to_string(),
        ModelPricing {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    // Groq (fast inference)
    pricing.insert(
        "mixtral-8x7b-32768".to_string(),
        ModelPricing {
            model: "mixtral-8x7b-32768".to_string(),
            input_cost_per_1k: 0.00027,
            output_cost_per_1k: 0.00027,
            cost_per_request: 0.0,
            is_local: false,
        },
    );

    // Local models (free)
    pricing.insert(
        "local".to_string(),
        ModelPricing {
            model: "local".to_string(),
            input_cost_per_1k: 0.0,
            output_cost_per_1k: 0.0,
            cost_per_request: 0.0,
            is_local: true,
        },
    );

    pricing
}

/// Observability Manager
#[derive(Debug)]
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    /// In-memory metrics buffer
    metrics_buffer: Arc<RwLock<Vec<LLMRequestMetrics>>>,
    /// Current aggregated metrics
    current_metrics: Arc<RwLock<AggregatedMetrics>>,
    /// Budget tracking
    budget: Arc<RwLock<Budget>>,
    /// Trace buffer
    trace_buffer: Arc<RwLock<Vec<TraceEvent>>>,
    /// Counters for quick access
    request_count: AtomicU64,
    token_count: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    error_count: AtomicU64,
}

impl ObservabilityManager {
    /// Create a new observability manager
    pub fn new(config: ObservabilityConfig) -> Self {
        let budget = Budget {
            daily_limit: config.budget_daily,
            monthly_limit: config.budget_monthly,
            alert_threshold: config.alert_threshold,
            ..Default::default()
        };

        ObservabilityManager {
            config,
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            current_metrics: Arc::new(RwLock::new(AggregatedMetrics::default())),
            budget: Arc::new(RwLock::new(budget)),
            trace_buffer: Arc::new(RwLock::new(Vec::new())),
            request_count: AtomicU64::new(0),
            token_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    /// Create from config map
    pub fn from_config(config_map: &HashMap<String, String>) -> Self {
        let config = ObservabilityConfig {
            enabled: config_map
                .get("observability-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            metrics_interval: config_map
                .get("observability-metrics-interval")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            cost_tracking: config_map
                .get("observability-cost-tracking")
                .map(|v| v == "true")
                .unwrap_or(true),
            trace_enabled: config_map
                .get("observability-trace-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            trace_sample_rate: config_map
                .get("observability-trace-sample-rate")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            budget_daily: config_map
                .get("observability-budget-daily")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
            budget_monthly: config_map
                .get("observability-budget-monthly")
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000.0),
            alert_threshold: config_map
                .get("observability-alert-threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.8),
            model_pricing: default_model_pricing(),
        };
        ObservabilityManager::new(config)
    }

    /// Record a request
    pub async fn record_request(&self, metrics: LLMRequestMetrics) {
        if !self.config.enabled {
            return;
        }

        // Update atomic counters
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.token_count
            .fetch_add(metrics.total_tokens, Ordering::Relaxed);

        if metrics.cached {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        if !metrics.success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Update budget if cost tracking enabled
        if self.config.cost_tracking && metrics.estimated_cost > 0.0 {
            let mut budget = self.budget.write().await;
            budget.daily_spend += metrics.estimated_cost;
            budget.monthly_spend += metrics.estimated_cost;
        }

        // Add to buffer
        let mut buffer = self.metrics_buffer.write().await;
        buffer.push(metrics);

        // Limit buffer size
        if buffer.len() > 10000 {
            buffer.drain(0..1000);
        }
    }

    /// Calculate estimated cost for a request
    pub fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        if let Some(pricing) = self.config.model_pricing.get(model) {
            if pricing.is_local {
                return 0.0;
            }
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
            input_cost + output_cost + pricing.cost_per_request
        } else {
            // Default to local (free) if model not found
            0.0
        }
    }

    /// Get current budget status
    pub async fn get_budget_status(&self) -> BudgetStatus {
        let budget = self.budget.read().await;
        BudgetStatus {
            daily_limit: budget.daily_limit,
            daily_spend: budget.daily_spend,
            daily_remaining: budget.daily_limit - budget.daily_spend,
            daily_percentage: (budget.daily_spend / budget.daily_limit) * 100.0,
            monthly_limit: budget.monthly_limit,
            monthly_spend: budget.monthly_spend,
            monthly_remaining: budget.monthly_limit - budget.monthly_spend,
            monthly_percentage: (budget.monthly_spend / budget.monthly_limit) * 100.0,
            daily_exceeded: budget.daily_spend > budget.daily_limit,
            monthly_exceeded: budget.monthly_spend > budget.monthly_limit,
            near_daily_limit: budget.daily_spend >= budget.daily_limit * budget.alert_threshold,
            near_monthly_limit: budget.monthly_spend
                >= budget.monthly_limit * budget.alert_threshold,
        }
    }

    /// Check if budget allows a request
    pub async fn check_budget(&self, estimated_cost: f64) -> BudgetCheckResult {
        let budget = self.budget.read().await;

        let daily_after = budget.daily_spend + estimated_cost;
        let monthly_after = budget.monthly_spend + estimated_cost;

        if daily_after > budget.daily_limit {
            return BudgetCheckResult::DailyExceeded;
        }

        if monthly_after > budget.monthly_limit {
            return BudgetCheckResult::MonthlyExceeded;
        }

        if daily_after >= budget.daily_limit * budget.alert_threshold {
            return BudgetCheckResult::NearDailyLimit;
        }

        if monthly_after >= budget.monthly_limit * budget.alert_threshold {
            return BudgetCheckResult::NearMonthlyLimit;
        }

        BudgetCheckResult::Ok
    }

    /// Reset daily budget
    pub async fn reset_daily_budget(&self) {
        let mut budget = self.budget.write().await;
        budget.daily_spend = 0.0;
        budget.daily_reset_date = Utc::now();
        budget.daily_alert_sent = false;
    }

    /// Reset monthly budget
    pub async fn reset_monthly_budget(&self) {
        let mut budget = self.budget.write().await;
        budget.monthly_spend = 0.0;
        budget.monthly_reset_date = Utc::now();
        budget.monthly_alert_sent = false;
    }

    /// Record a trace event
    pub async fn record_trace(&self, event: TraceEvent) {
        if !self.config.enabled || !self.config.trace_enabled {
            return;
        }

        // Sample rate check
        if self.config.trace_sample_rate < 1.0 {
            let sample: f64 = rand::random();
            if sample > self.config.trace_sample_rate {
                return;
            }
        }

        let mut buffer = self.trace_buffer.write().await;
        buffer.push(event);

        // Limit buffer size
        if buffer.len() > 5000 {
            buffer.drain(0..500);
        }
    }

    /// Start a trace span
    pub fn start_span(
        &self,
        trace_id: Uuid,
        name: &str,
        component: &str,
        parent_id: Option<Uuid>,
    ) -> TraceEvent {
        TraceEvent {
            id: Uuid::new_v4(),
            parent_id,
            trace_id,
            name: name.to_string(),
            component: component.to_string(),
            event_type: TraceEventType::Span,
            duration_ms: None,
            start_time: Utc::now(),
            end_time: None,
            attributes: HashMap::new(),
            status: TraceStatus::InProgress,
            error: None,
        }
    }

    /// End a trace span
    pub fn end_span(&self, span: &mut TraceEvent, status: TraceStatus, error: Option<String>) {
        let end_time = Utc::now();
        span.end_time = Some(end_time);
        span.duration_ms = Some((end_time - span.start_time).num_milliseconds() as u64);
        span.status = status;
        span.error = error;
    }

    /// Get aggregated metrics for a time range
    pub async fn get_aggregated_metrics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> AggregatedMetrics {
        let buffer = self.metrics_buffer.read().await;

        let filtered: Vec<&LLMRequestMetrics> = buffer
            .iter()
            .filter(|m| m.timestamp >= start && m.timestamp <= end)
            .collect();

        if filtered.is_empty() {
            return AggregatedMetrics {
                period_start: start,
                period_end: end,
                ..Default::default()
            };
        }

        let mut metrics = AggregatedMetrics {
            period_start: start,
            period_end: end,
            total_requests: filtered.len() as u64,
            ..Default::default()
        };

        let mut latencies: Vec<u64> = Vec::new();

        for m in &filtered {
            if m.success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
                *metrics
                    .errors_by_type
                    .entry(m.error.clone().unwrap_or_else(|| "unknown".to_string()))
                    .or_insert(0) += 1;
            }

            if m.cached {
                metrics.cache_hits += 1;
            } else {
                metrics.cache_misses += 1;
            }

            metrics.total_input_tokens += m.input_tokens;
            metrics.total_output_tokens += m.output_tokens;
            metrics.total_tokens += m.total_tokens;
            metrics.total_cost += m.estimated_cost;

            latencies.push(m.latency_ms);

            *metrics
                .requests_by_model
                .entry(m.model.clone())
                .or_insert(0) += 1;
            *metrics.tokens_by_model.entry(m.model.clone()).or_insert(0) += m.total_tokens;
            *metrics.cost_by_model.entry(m.model.clone()).or_insert(0.0) += m.estimated_cost;
            *metrics
                .requests_by_type
                .entry(m.request_type.to_string())
                .or_insert(0) += 1;
        }

        // Calculate latency statistics
        if !latencies.is_empty() {
            latencies.sort();
            let len = latencies.len();

            metrics.avg_latency_ms = latencies.iter().sum::<u64>() as f64 / len as f64;
            metrics.min_latency_ms = latencies[0];
            metrics.max_latency_ms = latencies[len - 1];
            metrics.p50_latency_ms = latencies[len / 2] as f64;
            metrics.p95_latency_ms = latencies[(len as f64 * 0.95) as usize] as f64;
            metrics.p99_latency_ms =
                latencies[(len as f64 * 0.99).min((len - 1) as f64) as usize] as f64;
        }

        metrics
    }

    /// Get quick stats
    pub async fn get_current_metrics(&self) -> AggregatedMetrics {
        self.current_metrics.read().await.clone()
    }

    pub async fn update_current_metrics(&self) {
        let now = Utc::now();
        let start = now - chrono::Duration::hours(1);
        let metrics = self.get_aggregated_metrics(start, now).await;
        let mut current = self.current_metrics.write().await;
        *current = metrics;
    }

    pub fn get_quick_stats(&self) -> QuickStats {
        QuickStats {
            total_requests: self.request_count.load(Ordering::Relaxed),
            total_tokens: self.token_count.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            cache_hit_rate: {
                let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
                let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
                if hits + misses > 0.0 {
                    hits / (hits + misses)
                } else {
                    0.0
                }
            },
            error_rate: {
                let errors = self.error_count.load(Ordering::Relaxed) as f64;
                let total = self.request_count.load(Ordering::Relaxed) as f64;
                if total > 0.0 {
                    errors / total
                } else {
                    0.0
                }
            },
        }
    }

    /// Get recent traces
    pub async fn get_recent_traces(&self, limit: usize) -> Vec<TraceEvent> {
        let buffer = self.trace_buffer.read().await;
        buffer.iter().rev().take(limit).cloned().collect()
    }

    /// Get traces for a specific trace ID
    pub async fn get_trace(&self, trace_id: Uuid) -> Vec<TraceEvent> {
        let buffer = self.trace_buffer.read().await;
        buffer
            .iter()
            .filter(|t| t.trace_id == trace_id)
            .cloned()
            .collect()
    }
}

/// Budget status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub daily_limit: f64,
    pub daily_spend: f64,
    pub daily_remaining: f64,
    pub daily_percentage: f64,
    pub monthly_limit: f64,
    pub monthly_spend: f64,
    pub monthly_remaining: f64,
    pub monthly_percentage: f64,
    pub daily_exceeded: bool,
    pub monthly_exceeded: bool,
    pub near_daily_limit: bool,
    pub near_monthly_limit: bool,
}

/// Budget check result
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetCheckResult {
    Ok,
    NearDailyLimit,
    NearMonthlyLimit,
    DailyExceeded,
    MonthlyExceeded,
}

/// Quick stats (from atomic counters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub error_count: u64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
}

/// Convert metrics to Rhai Dynamic
impl LLMRequestMetrics {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("request_id".into(), self.request_id.to_string().into());
        map.insert("session_id".into(), self.session_id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert("model".into(), self.model.clone().into());
        map.insert("request_type".into(), self.request_type.to_string().into());
        map.insert("input_tokens".into(), (self.input_tokens as i64).into());
        map.insert("output_tokens".into(), (self.output_tokens as i64).into());
        map.insert("total_tokens".into(), (self.total_tokens as i64).into());
        map.insert("latency_ms".into(), (self.latency_ms as i64).into());
        map.insert("cached".into(), self.cached.into());
        map.insert("success".into(), self.success.into());
        map.insert("estimated_cost".into(), self.estimated_cost.into());
        map.insert("timestamp".into(), self.timestamp.to_rfc3339().into());

        if let Some(ttft) = self.ttft_ms {
            map.insert("ttft_ms".into(), (ttft as i64).into());
        }

        if let Some(ref error) = self.error {
            map.insert("error".into(), error.clone().into());
        }

        Dynamic::from(map)
    }
}

impl AggregatedMetrics {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("period_start".into(), self.period_start.to_rfc3339().into());
        map.insert("period_end".into(), self.period_end.to_rfc3339().into());
        map.insert("total_requests".into(), (self.total_requests as i64).into());
        map.insert(
            "successful_requests".into(),
            (self.successful_requests as i64).into(),
        );
        map.insert(
            "failed_requests".into(),
            (self.failed_requests as i64).into(),
        );
        map.insert("cache_hits".into(), (self.cache_hits as i64).into());
        map.insert("cache_misses".into(), (self.cache_misses as i64).into());
        map.insert(
            "total_input_tokens".into(),
            (self.total_input_tokens as i64).into(),
        );
        map.insert(
            "total_output_tokens".into(),
            (self.total_output_tokens as i64).into(),
        );
        map.insert("total_tokens".into(), (self.total_tokens as i64).into());
        map.insert("total_cost".into(), self.total_cost.into());
        map.insert("avg_latency_ms".into(), self.avg_latency_ms.into());
        map.insert("p50_latency_ms".into(), self.p50_latency_ms.into());
        map.insert("p95_latency_ms".into(), self.p95_latency_ms.into());
        map.insert("p99_latency_ms".into(), self.p99_latency_ms.into());
        map.insert("max_latency_ms".into(), (self.max_latency_ms as i64).into());
        map.insert("min_latency_ms".into(), (self.min_latency_ms as i64).into());

        Dynamic::from(map)
    }
}

impl BudgetStatus {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("daily_limit".into(), self.daily_limit.into());
        map.insert("daily_spend".into(), self.daily_spend.into());
        map.insert("daily_remaining".into(), self.daily_remaining.into());
        map.insert("daily_percentage".into(), self.daily_percentage.into());
        map.insert("monthly_limit".into(), self.monthly_limit.into());
        map.insert("monthly_spend".into(), self.monthly_spend.into());
        map.insert("monthly_remaining".into(), self.monthly_remaining.into());
        map.insert("monthly_percentage".into(), self.monthly_percentage.into());
        map.insert("daily_exceeded".into(), self.daily_exceeded.into());
        map.insert("monthly_exceeded".into(), self.monthly_exceeded.into());
        map.insert("near_daily_limit".into(), self.near_daily_limit.into());
        map.insert("near_monthly_limit".into(), self.near_monthly_limit.into());

        Dynamic::from(map)
    }
}

/// Register observability functions with Rhai engine
pub fn register_observability_keywords(engine: &mut Engine) {
    // Helper functions for metrics in scripts

    engine.register_fn("metrics_total_requests", |metrics: Map| -> i64 {
        metrics
            .get("total_requests")
            .and_then(|v| v.clone().try_cast::<i64>())
            .unwrap_or(0)
    });

    engine.register_fn("metrics_cache_hit_rate", |metrics: Map| -> f64 {
        let hits = metrics
            .get("cache_hits")
            .and_then(|v| v.clone().try_cast::<i64>())
            .unwrap_or(0) as f64;
        let misses = metrics
            .get("cache_misses")
            .and_then(|v| v.clone().try_cast::<i64>())
            .unwrap_or(0) as f64;
        if hits + misses > 0.0 {
            hits / (hits + misses)
        } else {
            0.0
        }
    });

    engine.register_fn("metrics_error_rate", |metrics: Map| -> f64 {
        let failed = metrics
            .get("failed_requests")
            .and_then(|v| v.clone().try_cast::<i64>())
            .unwrap_or(0) as f64;
        let total = metrics
            .get("total_requests")
            .and_then(|v| v.clone().try_cast::<i64>())
            .unwrap_or(0) as f64;
        if total > 0.0 {
            failed / total
        } else {
            0.0
        }
    });

    engine.register_fn("budget_is_exceeded", |status: Map| -> bool {
        status
            .get("daily_exceeded")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false)
            || status
                .get("monthly_exceeded")
                .and_then(|v| v.clone().try_cast::<bool>())
                .unwrap_or(false)
    });

    engine.register_fn("budget_remaining_daily", |status: Map| -> f64 {
        status
            .get("daily_remaining")
            .and_then(|v| v.clone().try_cast::<f64>())
            .unwrap_or(0.0)
    });

    engine.register_fn("budget_remaining_monthly", |status: Map| -> f64 {
        status
            .get("monthly_remaining")
            .and_then(|v| v.clone().try_cast::<f64>())
            .unwrap_or(0.0)
    });

    info!("Observability keywords registered");
}

/// SQL for creating observability tables
pub const OBSERVABILITY_SCHEMA: &str = r#"
-- LLM request metrics
CREATE TABLE IF NOT EXISTS llm_metrics (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model VARCHAR(200) NOT NULL,
    request_type VARCHAR(50) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    ttft_ms BIGINT,
    cached BOOLEAN NOT NULL DEFAULT false,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    estimated_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Aggregated metrics (hourly rollup)
CREATE TABLE IF NOT EXISTS llm_metrics_hourly (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    hour TIMESTAMP WITH TIME ZONE NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    successful_requests BIGINT NOT NULL DEFAULT 0,
    failed_requests BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p50_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p99_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_latency_ms BIGINT NOT NULL DEFAULT 0,
    min_latency_ms BIGINT NOT NULL DEFAULT 0,
    requests_by_model JSONB NOT NULL DEFAULT '{}',
    tokens_by_model JSONB NOT NULL DEFAULT '{}',
    cost_by_model JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, hour)
);

-- Budget tracking
CREATE TABLE IF NOT EXISTS llm_budget (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL UNIQUE,
    daily_limit DOUBLE PRECISION NOT NULL DEFAULT 100,
    monthly_limit DOUBLE PRECISION NOT NULL DEFAULT 2000,
    alert_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.8,
    daily_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    monthly_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    monthly_reset_date DATE NOT NULL DEFAULT DATE_TRUNC('month', CURRENT_DATE),
    daily_alert_sent BOOLEAN NOT NULL DEFAULT false,
    monthly_alert_sent BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Trace events
CREATE TABLE IF NOT EXISTS llm_traces (
    id UUID PRIMARY KEY,
    parent_id UUID,
    trace_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    component VARCHAR(100) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    duration_ms BIGINT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    attributes JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress',
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_llm_metrics_bot_id ON llm_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_session_id ON llm_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_timestamp ON llm_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_model ON llm_metrics(model);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_bot_id ON llm_metrics_hourly(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_hour ON llm_metrics_hourly(hour DESC);

CREATE INDEX IF NOT EXISTS idx_llm_traces_trace_id ON llm_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_llm_traces_start_time ON llm_traces(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_llm_traces_component ON llm_traces(component);
"#;
