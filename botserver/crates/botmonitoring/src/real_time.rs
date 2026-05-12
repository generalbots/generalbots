use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Acknowledged,
    Silenced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub unit: Option<String>,
    pub labels: HashMap<String, String>,
    pub current_value: f64,
    pub data_points: Vec<MetricDataPoint>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub duration_seconds: u64,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan(f64),
    LessThan(f64),
    Equals(f64),
    NotEquals(f64),
    GreaterThanOrEqual(f64),
    LessThanOrEqual(f64),
    AbsentFor(u64),
    RateOfChange { threshold: f64, window_seconds: u64 },
}

impl AlertCondition {
    pub fn evaluate(&self, current: f64, previous: Option<f64>, absent_seconds: u64) -> bool {
        match self {
            Self::GreaterThan(threshold) => current > *threshold,
            Self::LessThan(threshold) => current < *threshold,
            Self::Equals(threshold) => (current - threshold).abs() < f64::EPSILON,
            Self::NotEquals(threshold) => (current - threshold).abs() >= f64::EPSILON,
            Self::GreaterThanOrEqual(threshold) => current >= *threshold,
            Self::LessThanOrEqual(threshold) => current <= *threshold,
            Self::AbsentFor(seconds) => absent_seconds >= *seconds,
            Self::RateOfChange {
                threshold,
                window_seconds: _,
            } => {
                if let Some(prev) = previous {
                    let rate = (current - prev).abs();
                    rate > *threshold
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub message: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub active_connections: u64,
    pub requests_per_second: f64,
    pub error_rate_percent: f64,
    pub average_latency_ms: f64,
    pub uptime_seconds: u64,
    pub last_check: DateTime<Utc>,
    pub components: Vec<ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<f64>,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitoringMessage {
    MetricUpdate {
        metric: Metric,
    },
    AlertFired {
        alert: Alert,
    },
    AlertResolved {
        alert_id: Uuid,
    },
    HealthUpdate {
        health: SystemHealth,
    },
    Subscribe {
        metrics: Vec<String>,
        alerts: bool,
        health: bool,
    },
    Unsubscribe {
        metrics: Vec<String>,
    },
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardQuery {
    pub metrics: Option<Vec<String>>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub metrics: Vec<Metric>,
    pub health: SystemHealth,
    pub active_alerts: Vec<Alert>,
    pub recent_alerts: Vec<Alert>,
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    health: Arc<RwLock<SystemHealth>>,
    broadcast_tx: broadcast::Sender<MonitoringMessage>,
    request_counter: AtomicU64,
    error_counter: AtomicU64,
    total_latency_ms: AtomicU64,
    start_time: DateTime<Utc>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);

        let health = SystemHealth {
            status: HealthStatus::Unknown,
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            disk_usage_percent: 0.0,
            active_connections: 0,
            requests_per_second: 0.0,
            error_rate_percent: 0.0,
            average_latency_ms: 0.0,
            uptime_seconds: 0,
            last_check: Utc::now(),
            components: Vec::new(),
        };

        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            health: Arc::new(RwLock::new(health)),
            broadcast_tx: tx,
            request_counter: AtomicU64::new(0),
            error_counter: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            start_time: Utc::now(),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MonitoringMessage> {
        self.broadcast_tx.subscribe()
    }

    pub async fn record_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let now = Utc::now();
        let data_point = MetricDataPoint {
            timestamp: now,
            value,
            labels: labels.clone(),
        };

        let mut metrics = self.metrics.write().await;

        if let Some(metric) = metrics.get_mut(name) {
            metric.current_value = value;
            metric.updated_at = now;

            metric.data_points.push(data_point);
            if metric.data_points.len() > 1000 {
                metric.data_points.remove(0);
            }

            let _ = self.broadcast_tx.send(MonitoringMessage::MetricUpdate {
                metric: metric.clone(),
            });
        } else {
            let metric = Metric {
                name: name.to_string(),
                metric_type: MetricType::Gauge,
                description: String::new(),
                unit: None,
                labels,
                current_value: value,
                data_points: vec![data_point],
                updated_at: now,
            };

            let _ = self.broadcast_tx.send(MonitoringMessage::MetricUpdate {
                metric: metric.clone(),
            });

            metrics.insert(name.to_string(), metric);
        }

        drop(metrics);

        self.check_alert_rules(name, value).await;
    }

    pub async fn increment_counter(&self, name: &str, labels: HashMap<String, String>) {
        let metrics = self.metrics.read().await;
        let current = metrics.get(name).map(|m| m.current_value).unwrap_or(0.0);
        drop(metrics);

        self.record_metric(name, current + 1.0, labels).await;
    }

    pub async fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        self.record_metric(name, value, labels).await;
    }

    pub async fn register_metric(
        &self,
        name: &str,
        metric_type: MetricType,
        description: &str,
        unit: Option<String>,
    ) {
        let metric = Metric {
            name: name.to_string(),
            metric_type,
            description: description.to_string(),
            unit,
            labels: HashMap::new(),
            current_value: 0.0,
            data_points: Vec::new(),
            updated_at: Utc::now(),
        };

        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), metric);
    }

    pub async fn add_alert_rule(&self, rule: AlertRule) {
        let mut rules = self.alert_rules.write().await;
        rules.push(rule);
    }

    pub async fn remove_alert_rule(&self, rule_id: Uuid) -> bool {
        let mut rules = self.alert_rules.write().await;
        let initial_len = rules.len();
        rules.retain(|r| r.id != rule_id);
        rules.len() < initial_len
    }

    async fn check_alert_rules(&self, metric_name: &str, value: f64) {
        let rules = self.alert_rules.read().await;
        let relevant_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.enabled && r.metric_name == metric_name)
            .cloned()
            .collect();
        drop(rules);

        for rule in relevant_rules {
            let should_fire = rule.condition.evaluate(value, None, 0);

            if should_fire {
                self.fire_alert(&rule, value).await;
            } else {
                self.resolve_alert_for_rule(rule.id).await;
            }
        }
    }

    async fn fire_alert(&self, rule: &AlertRule, value: f64) {
        let mut alerts = self.alerts.write().await;

        let existing = alerts.values().find(|a| {
            a.rule_id == rule.id && matches!(a.status, AlertStatus::Firing | AlertStatus::Acknowledged)
        });

        if existing.is_some() {
            return;
        }

        let threshold = match &rule.condition {
            AlertCondition::GreaterThan(t)
            | AlertCondition::LessThan(t)
            | AlertCondition::Equals(t)
            | AlertCondition::NotEquals(t)
            | AlertCondition::GreaterThanOrEqual(t)
            | AlertCondition::LessThanOrEqual(t) => *t,
            AlertCondition::AbsentFor(s) => *s as f64,
            AlertCondition::RateOfChange { threshold, .. } => *threshold,
        };

        let alert = Alert {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Firing,
            metric_name: rule.metric_name.clone(),
            metric_value: value,
            threshold,
            message: format!(
                "{}: {} is {} (threshold: {})",
                rule.name, rule.metric_name, value, threshold
            ),
            labels: rule.labels.clone(),
            annotations: rule.annotations.clone(),
            started_at: Utc::now(),
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
        };

        let _ = self.broadcast_tx.send(MonitoringMessage::AlertFired {
            alert: alert.clone(),
        });

        alerts.insert(alert.id, alert);
    }

    async fn resolve_alert_for_rule(&self, rule_id: Uuid) {
        let mut alerts = self.alerts.write().await;

        let alert_ids: Vec<_> = alerts
            .iter()
            .filter(|(_, a)| a.rule_id == rule_id && a.status == AlertStatus::Firing)
            .map(|(id, _)| *id)
            .collect();

        for alert_id in alert_ids {
            if let Some(alert) = alerts.get_mut(&alert_id) {
                alert.status = AlertStatus::Resolved;
                alert.resolved_at = Some(Utc::now());

                let _ = self
                    .broadcast_tx
                    .send(MonitoringMessage::AlertResolved { alert_id });
            }
        }
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: &str) -> bool {
        let mut alerts = self.alerts.write().await;

        if let Some(alert) = alerts.get_mut(&alert_id) {
            if alert.status == AlertStatus::Firing {
                alert.status = AlertStatus::Acknowledged;
                alert.acknowledged_at = Some(Utc::now());
                alert.acknowledged_by = Some(acknowledged_by.to_string());
                return true;
            }
        }

        false
    }

    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts
            .values()
            .filter(|a| matches!(a.status, AlertStatus::Firing | AlertStatus::Acknowledged))
            .cloned()
            .collect()
    }

    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        let mut all_alerts: Vec<_> = alerts.values().cloned().collect();
        all_alerts.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        all_alerts.truncate(limit);
        all_alerts
    }

    pub fn record_request(&self, latency_ms: u64, is_error: bool) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);

        if is_error {
            self.error_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn update_system_health(&self) {
        let now = Utc::now();

        let cpu_usage = self.collect_cpu_usage().await;
        let memory_usage = self.collect_memory_usage().await;
        let disk_usage = self.collect_disk_usage().await;

        let request_count = self.request_counter.load(Ordering::Relaxed);
        let error_count = self.error_counter.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);

        let error_rate = if request_count > 0 {
            (error_count as f64 / request_count as f64) * 100.0
        } else {
            0.0
        };

        let avg_latency = if request_count > 0 {
            total_latency as f64 / request_count as f64
        } else {
            0.0
        };

        let uptime = (now - self.start_time).num_seconds() as u64;

        let rps = if uptime > 0 {
            request_count as f64 / uptime as f64
        } else {
            0.0
        };

        let components = self.check_component_health().await;

        let overall_status = self.calculate_overall_status(
            cpu_usage,
            memory_usage,
            error_rate,
            &components,
        );

        let health = SystemHealth {
            status: overall_status,
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            disk_usage_percent: disk_usage,
            active_connections: 0,
            requests_per_second: rps,
            error_rate_percent: error_rate,
            average_latency_ms: avg_latency,
            uptime_seconds: uptime,
            last_check: now,
            components,
        };

        {
            let mut h = self.health.write().await;
            *h = health.clone();
        }

        let _ = self
            .broadcast_tx
            .send(MonitoringMessage::HealthUpdate { health });
    }

    async fn collect_cpu_usage(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/stat").await {
                if let Some(line) = contents.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let user: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let nice: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let system: u64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let idle: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);

                        let total = user + nice + system + idle;
                        let active = user + nice + system;

                        if total > 0 {
                            return (active as f64 / total as f64) * 100.0;
                        }
                    }
                }
            }
        }

        0.0
    }

    async fn collect_memory_usage(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = tokio::fs::read_to_string("/proc/meminfo").await {
                let mut total: u64 = 0;
                let mut available: u64 = 0;

                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        total = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    } else if line.starts_with("MemAvailable:") {
                        available = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    }
                }

                if total > 0 {
                    return ((total - available) as f64 / total as f64) * 100.0;
                }
            }
        }

        0.0
    }

    async fn collect_disk_usage(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("df")
                .args(["-h", "/"])
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Some(line) = stdout.lines().nth(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(usage_str) = parts.get(4) {
                            if let Ok(usage) = usage_str.trim_end_matches('%').parse::<f64>() {
                                return usage;
                            }
                        }
                    }
                }
            }
        }

        0.0
    }

    async fn check_component_health(&self) -> Vec<ComponentHealth> {
        let mut components = Vec::new();

        let db_health = self.check_database_health().await;
        components.push(db_health);

        let cache_health = self.check_cache_health().await;
        components.push(cache_health);

        let vector_db_health = self.check_vector_db_health().await;
        components.push(vector_db_health);

        let llm_health = self.check_llm_health().await;
        components.push(llm_health);

        components
    }

    async fn check_database_health(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        let (status, message) = (HealthStatus::Healthy, None);

        ComponentHealth {
            name: "database".to_string(),
            status,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
            last_check: Utc::now(),
        }
    }

    async fn check_cache_health(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        let (status, message) = (HealthStatus::Healthy, None);

        ComponentHealth {
            name: "cache".to_string(),
            status,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
            last_check: Utc::now(),
        }
    }

    async fn check_vector_db_health(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        let (status, message) = (HealthStatus::Healthy, None);

        ComponentHealth {
            name: "vector_db".to_string(),
            status,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
            last_check: Utc::now(),
        }
    }

    async fn check_llm_health(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        let (status, message) = (HealthStatus::Healthy, None);

        ComponentHealth {
            name: "llm".to_string(),
            status,
            latency_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            message,
            last_check: Utc::now(),
        }
    }

    fn calculate_overall_status(
        &self,
        cpu_usage: f64,
        memory_usage: f64,
        error_rate: f64,
        components: &[ComponentHealth],
    ) -> HealthStatus {
        let unhealthy_components = components
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();

        let degraded_components = components
            .iter()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();

        if unhealthy_components > 0 || cpu_usage > 95.0 || memory_usage > 95.0 || error_rate > 10.0 {
            HealthStatus::Unhealthy
        } else if degraded_components > 0
            || cpu_usage > 80.0
            || memory_usage > 80.0
            || error_rate > 5.0
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    pub async fn get_health(&self) -> SystemHealth {
        let health = self.health.read().await;
        health.clone()
    }

    pub async fn get_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().await;
        metrics.values().cloned().collect()
    }

    pub async fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned()
    }

    pub async fn get_metric_history(
        &self,
        name: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<MetricDataPoint> {
        let metrics = self.metrics.read().await;

        if let Some(metric) = metrics.get(name) {
            metric
                .data_points
                .iter()
                .filter(|p| p.timestamp >= from && p.timestamp <= to)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_dashboard(&self) -> DashboardResponse {
        DashboardResponse {
            metrics: self.get_metrics().await,
            health: self.get_health().await,
            active_alerts: self.get_active_alerts().await,
            recent_alerts: self.get_recent_alerts(10).await,
        }
    }

    pub async fn start_background_collection(self: Arc<Self>) {
        let collector = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;
                collector.update_system_health().await;
            }
        });

        let collector = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                collector.record_builtin_metrics().await;
            }
        });
    }

    async fn record_builtin_metrics(&self) {
        let health = self.get_health().await;

        self.record_metric("system_cpu_usage_percent", health.cpu_usage_percent, HashMap::new())
            .await;
        self.record_metric(
            "system_memory_usage_percent",
            health.memory_usage_percent,
            HashMap::new(),
        )
        .await;
        self.record_metric(
            "system_disk_usage_percent",
            health.disk_usage_percent,
            HashMap::new(),
        )
        .await;
        self.record_metric(
            "system_requests_per_second",
            health.requests_per_second,
            HashMap::new(),
        )
        .await;
        self.record_metric(
            "system_error_rate_percent",
            health.error_rate_percent,
            HashMap::new(),
        )
        .await;
        self.record_metric(
            "system_average_latency_ms",
            health.average_latency_ms,
            HashMap::new(),
        )
        .await;
    }

    pub async fn setup_default_alert_rules(&self) {
        let rules = vec![
            AlertRule {
                id: Uuid::new_v4(),
                name: "High CPU Usage".to_string(),
                description: "CPU usage exceeds 90%".to_string(),
                metric_name: "system_cpu_usage_percent".to_string(),
                condition: AlertCondition::GreaterThan(90.0),
                severity: AlertSeverity::Warning,
                duration_seconds: 300,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "Critical CPU Usage".to_string(),
                description: "CPU usage exceeds 95%".to_string(),
                metric_name: "system_cpu_usage_percent".to_string(),
                condition: AlertCondition::GreaterThan(95.0),
                severity: AlertSeverity::Critical,
                duration_seconds: 60,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "High Memory Usage".to_string(),
                description: "Memory usage exceeds 85%".to_string(),
                metric_name: "system_memory_usage_percent".to_string(),
                condition: AlertCondition::GreaterThan(85.0),
                severity: AlertSeverity::Warning,
                duration_seconds: 300,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "High Error Rate".to_string(),
                description: "Error rate exceeds 5%".to_string(),
                metric_name: "system_error_rate_percent".to_string(),
                condition: AlertCondition::GreaterThan(5.0),
                severity: AlertSeverity::Error,
                duration_seconds: 120,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "High Latency".to_string(),
                description: "Average latency exceeds 1000ms".to_string(),
                metric_name: "system_average_latency_ms".to_string(),
                condition: AlertCondition::GreaterThan(1000.0),
                severity: AlertSeverity::Warning,
                duration_seconds: 300,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        for rule in rules {
            self.add_alert_rule(rule).await;
        }
    }
}
