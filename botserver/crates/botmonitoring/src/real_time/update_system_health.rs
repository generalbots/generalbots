use super::*;
#[cfg(target_os = "linux")]
use botlib::security::SafeCommand;

impl MetricsCollector {
    pub async fn update_system_health(&self) {
        let now = Utc::now();

        let cpu_usage = self.collect_cpu_usage().await;
        let memory_usage = self.collect_memory_usage().await;
        let disk_usage = self.collect_disk_usage().await;

        let request_count = self.request_counter.load(Ordering::Relaxed);
        let error_count = self.error_counter.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);

        let uptime = (now - self.start_time).num_seconds() as u64;

        let stats = crate::policy::request_stats(request_count, error_count, total_latency, uptime);
        let error_rate = stats.error_rate_percent;
        let avg_latency = stats.average_latency_ms;
        let rps = stats.requests_per_second;

        let components = self.check_component_health().await;

        let overall_status = crate::policy::calculate_overall_status(
            cpu_usage,
            memory_usage,
            error_rate,
            &components,
            &crate::policy::HealthThresholds::default(),
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
            if let Ok(output) = SafeCommand::new("df")
                .and_then(|c| c.args(&["-h", "/"]))
                .and_then(|c| c.execute())
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
}
