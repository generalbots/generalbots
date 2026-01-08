use crate::billing::{BillingError, UsageMetric};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct UsageMeteringService {
    storage: Arc<RwLock<MeteringStorage>>,
    aggregation_interval_secs: u64,
}

#[derive(Default)]
struct MeteringStorage {
    events: HashMap<Uuid, Vec<UsageEvent>>,
    aggregated: HashMap<Uuid, HashMap<UsageMetric, AggregatedUsage>>,
    daily_snapshots: HashMap<Uuid, Vec<DailySnapshot>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub metric: UsageMetric,
    pub value: i64,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedUsage {
    pub metric: UsageMetric,
    pub total: u64,
    pub count: u64,
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub average: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySnapshot {
    pub organization_id: Uuid,
    pub date: chrono::NaiveDate,
    pub metrics: HashMap<UsageMetric, u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringReport {
    pub organization_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub metrics: Vec<MetricReport>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    pub metric: UsageMetric,
    pub current_value: u64,
    pub period_total: u64,
    pub period_average: f64,
    pub peak_value: u64,
    pub trend: UsageTrend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageTrend {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

impl UsageMeteringService {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MeteringStorage::default())),
            aggregation_interval_secs: 3600,
        }
    }

    pub fn with_aggregation_interval(interval_secs: u64) -> Self {
        Self {
            storage: Arc::new(RwLock::new(MeteringStorage::default())),
            aggregation_interval_secs: interval_secs,
        }
    }

    pub async fn record_event(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        value: i64,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<UsageEvent, BillingError> {
        let event = UsageEvent {
            id: Uuid::new_v4(),
            organization_id,
            metric,
            value,
            timestamp: Utc::now(),
            metadata,
        };

        let mut storage = self.storage.write().await;
        storage
            .events
            .entry(organization_id)
            .or_default()
            .push(event.clone());

        self.update_aggregation(&mut storage, organization_id, metric, value);

        Ok(event)
    }

    pub async fn record_increment(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        amount: u64,
    ) -> Result<UsageEvent, BillingError> {
        self.record_event(organization_id, metric, amount as i64, None)
            .await
    }

    pub async fn record_decrement(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        amount: u64,
    ) -> Result<UsageEvent, BillingError> {
        self.record_event(organization_id, metric, -(amount as i64), None)
            .await
    }

    pub async fn get_current_usage(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
    ) -> Result<u64, BillingError> {
        let storage = self.storage.read().await;
        let aggregated = storage
            .aggregated
            .get(&organization_id)
            .and_then(|m| m.get(&metric));

        Ok(aggregated.map(|a| a.total).unwrap_or(0))
    }

    pub async fn get_aggregated_usage(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
    ) -> Result<Option<AggregatedUsage>, BillingError> {
        let storage = self.storage.read().await;
        let aggregated = storage
            .aggregated
            .get(&organization_id)
            .and_then(|m| m.get(&metric))
            .cloned();

        Ok(aggregated)
    }

    pub async fn get_all_metrics(
        &self,
        organization_id: Uuid,
    ) -> Result<HashMap<UsageMetric, AggregatedUsage>, BillingError> {
        let storage = self.storage.read().await;
        let metrics = storage
            .aggregated
            .get(&organization_id)
            .cloned()
            .unwrap_or_default();

        Ok(metrics)
    }

    pub async fn generate_report(
        &self,
        organization_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<MeteringReport, BillingError> {
        let storage = self.storage.read().await;
        let aggregated = storage.aggregated.get(&organization_id);

        let metrics = [
            UsageMetric::Messages,
            UsageMetric::StorageBytes,
            UsageMetric::ApiCalls,
            UsageMetric::Bots,
            UsageMetric::Users,
            UsageMetric::KbDocuments,
            UsageMetric::Apps,
        ];

        let metric_reports: Vec<MetricReport> = metrics
            .iter()
            .map(|&metric| {
                let agg = aggregated.and_then(|m| m.get(&metric));
                MetricReport {
                    metric,
                    current_value: agg.map(|a| a.total).unwrap_or(0),
                    period_total: agg.map(|a| a.total).unwrap_or(0),
                    period_average: agg.map(|a| a.average).unwrap_or(0.0),
                    peak_value: agg.and_then(|a| a.max).unwrap_or(0),
                    trend: self.calculate_trend(&storage, organization_id, metric),
                }
            })
            .collect();

        Ok(MeteringReport {
            organization_id,
            period_start,
            period_end,
            metrics: metric_reports,
            generated_at: Utc::now(),
        })
    }

    pub async fn create_daily_snapshot(
        &self,
        organization_id: Uuid,
    ) -> Result<DailySnapshot, BillingError> {
        let mut storage = self.storage.write().await;
        let metrics = storage
            .aggregated
            .get(&organization_id)
            .map(|m| m.iter().map(|(&k, v)| (k, v.total)).collect())
            .unwrap_or_default();

        let snapshot = DailySnapshot {
            organization_id,
            date: Utc::now().date_naive(),
            metrics,
            created_at: Utc::now(),
        };

        storage
            .daily_snapshots
            .entry(organization_id)
            .or_default()
            .push(snapshot.clone());

        Ok(snapshot)
    }

    pub async fn get_daily_snapshots(
        &self,
        organization_id: Uuid,
        days: usize,
    ) -> Result<Vec<DailySnapshot>, BillingError> {
        let storage = self.storage.read().await;
        let snapshots = storage
            .daily_snapshots
            .get(&organization_id)
            .map(|s| {
                let len = s.len();
                let start = if len > days { len - days } else { 0 };
                s[start..].to_vec()
            })
            .unwrap_or_default();

        Ok(snapshots)
    }

    pub async fn reset_daily_metrics(&self, organization_id: Uuid) -> Result<(), BillingError> {
        let mut storage = self.storage.write().await;

        if let Some(org_aggregated) = storage.aggregated.get_mut(&organization_id) {
            let now = Utc::now();
            for metric in [UsageMetric::Messages, UsageMetric::ApiCalls] {
                if let Some(agg) = org_aggregated.get_mut(&metric) {
                    agg.total = 0;
                    agg.count = 0;
                    agg.min = None;
                    agg.max = None;
                    agg.average = 0.0;
                    agg.period_start = now;
                    agg.last_updated = now;
                }
            }
        }

        Ok(())
    }

    pub async fn prune_old_events(&self, retention_days: i64) -> Result<usize, BillingError> {
        let mut storage = self.storage.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(retention_days);
        let mut pruned = 0;

        for events in storage.events.values_mut() {
            let original_len = events.len();
            events.retain(|e| e.timestamp > cutoff);
            pruned += original_len - events.len();
        }

        for snapshots in storage.daily_snapshots.values_mut() {
            let original_len = snapshots.len();
            snapshots.retain(|s| s.created_at > cutoff);
            pruned += original_len - snapshots.len();
        }

        Ok(pruned)
    }

    fn update_aggregation(
        &self,
        storage: &mut MeteringStorage,
        organization_id: Uuid,
        metric: UsageMetric,
        value: i64,
    ) {
        let now = Utc::now();
        let org_aggregated = storage.aggregated.entry(organization_id).or_default();
        let agg = org_aggregated.entry(metric).or_insert_with(|| AggregatedUsage {
            metric,
            total: 0,
            count: 0,
            min: None,
            max: None,
            average: 0.0,
            period_start: now,
            period_end: now,
            last_updated: now,
        });

        if value >= 0 {
            agg.total = agg.total.saturating_add(value as u64);
        } else {
            agg.total = agg.total.saturating_sub(value.unsigned_abs());
        }

        agg.count += 1;
        agg.average = agg.total as f64 / agg.count as f64;

        let abs_value = value.unsigned_abs();
        agg.min = Some(agg.min.map(|m| m.min(abs_value)).unwrap_or(abs_value));
        agg.max = Some(agg.max.map(|m| m.max(abs_value)).unwrap_or(abs_value));
        agg.period_end = now;
        agg.last_updated = now;
    }

    fn calculate_trend(
        &self,
        storage: &MeteringStorage,
        organization_id: Uuid,
        metric: UsageMetric,
    ) -> UsageTrend {
        let snapshots = match storage.daily_snapshots.get(&organization_id) {
            Some(s) if s.len() >= 2 => s,
            _ => return UsageTrend::Unknown,
        };

        let recent: Vec<u64> = snapshots
            .iter()
            .rev()
            .take(7)
            .filter_map(|s| s.metrics.get(&metric).copied())
            .collect();

        if recent.len() < 2 {
            return UsageTrend::Unknown;
        }

        let first_half_avg: f64 =
            recent[recent.len() / 2..].iter().sum::<u64>() as f64 / (recent.len() / 2) as f64;
        let second_half_avg: f64 =
            recent[..recent.len() / 2].iter().sum::<u64>() as f64 / (recent.len() / 2) as f64;

        let change_percent = if first_half_avg > 0.0 {
            ((second_half_avg - first_half_avg) / first_half_avg) * 100.0
        } else {
            0.0
        };

        if change_percent > 10.0 {
            UsageTrend::Increasing
        } else if change_percent < -10.0 {
            UsageTrend::Decreasing
        } else {
            UsageTrend::Stable
        }
    }

    pub fn aggregation_interval(&self) -> u64 {
        self.aggregation_interval_secs
    }
}

impl Default for UsageMeteringService {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn metering_aggregation_job(metering_service: Arc<UsageMeteringService>) {
    let interval = metering_service.aggregation_interval();
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(interval));

    loop {
        ticker.tick().await;

        if let Err(e) = metering_service.prune_old_events(30).await {
            tracing::warn!("Failed to prune old metering events: {e}");
        }
    }
}

pub async fn daily_snapshot_job(
    metering_service: Arc<UsageMeteringService>,
    organization_ids: Arc<RwLock<Vec<Uuid>>>,
) {
    loop {
        let now = Utc::now();
        let tomorrow = (now + chrono::Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

        if let Some(next_snapshot) = tomorrow {
            let duration = next_snapshot - now;
            if let Ok(std_duration) = duration.to_std() {
                tokio::time::sleep(std_duration).await;
            }
        }

        let org_ids = organization_ids.read().await.clone();
        for org_id in org_ids {
            if let Err(e) = metering_service.create_daily_snapshot(org_id).await {
                tracing::warn!("Failed to create daily snapshot for org {org_id}: {e}");
            }
            if let Err(e) = metering_service.reset_daily_metrics(org_id).await {
                tracing::warn!("Failed to reset daily metrics for org {org_id}: {e}");
            }
        }

        tracing::info!("Daily metering snapshots completed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metering_service_new() {
        let service = UsageMeteringService::new();
        assert_eq!(service.aggregation_interval(), 3600);
    }

    #[test]
    fn test_usage_metering_service_with_interval() {
        let service = UsageMeteringService::with_aggregation_interval(1800);
        assert_eq!(service.aggregation_interval(), 1800);
    }

    #[test]
    fn test_usage_metering_service_default() {
        let service = UsageMeteringService::default();
        assert_eq!(service.aggregation_interval(), 3600);
    }

    #[tokio::test]
    async fn test_record_event() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let result = service
            .record_event(org_id, UsageMetric::Messages, 10, None)
            .await;
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.organization_id, org_id);
        assert_eq!(event.metric, UsageMetric::Messages);
        assert_eq!(event.value, 10);
        assert!(event.metadata.is_none());
    }

    #[tokio::test]
    async fn test_record_event_with_metadata() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "api".to_string());

        let result = service
            .record_event(org_id, UsageMetric::ApiCalls, 1, Some(metadata.clone()))
            .await;
        assert!(result.is_ok());

        let event = result.unwrap();
        assert!(event.metadata.is_some());
        assert_eq!(event.metadata.unwrap().get("source"), Some(&"api".to_string()));
    }

    #[tokio::test]
    async fn test_record_increment() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let result = service.record_increment(org_id, UsageMetric::Messages, 5).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 5);
    }

    #[tokio::test]
    async fn test_record_decrement() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let result = service.record_decrement(org_id, UsageMetric::StorageBytes, 100).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, -100);
    }

    #[tokio::test]
    async fn test_get_current_usage() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 10).await.unwrap();
        service.record_increment(org_id, UsageMetric::Messages, 20).await.unwrap();

        let usage = service.get_current_usage(org_id, UsageMetric::Messages).await;
        assert!(usage.is_ok());
        assert_eq!(usage.unwrap(), 30);
    }

    #[tokio::test]
    async fn test_get_current_usage_nonexistent() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let usage = service.get_current_usage(org_id, UsageMetric::Messages).await;
        assert!(usage.is_ok());
        assert_eq!(usage.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_aggregated_usage() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::ApiCalls, 5).await.unwrap();
        service.record_increment(org_id, UsageMetric::ApiCalls, 15).await.unwrap();

        let result = service.get_aggregated_usage(org_id, UsageMetric::ApiCalls).await;
        assert!(result.is_ok());

        let agg = result.unwrap().unwrap();
        assert_eq!(agg.metric, UsageMetric::ApiCalls);
        assert_eq!(agg.total, 20);
        assert_eq!(agg.count, 2);
        assert_eq!(agg.average, 10.0);
        assert_eq!(agg.min, Some(5));
        assert_eq!(agg.max, Some(15));
    }

    #[tokio::test]
    async fn test_get_aggregated_usage_nonexistent() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let result = service.get_aggregated_usage(org_id, UsageMetric::Messages).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_all_metrics() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 10).await.unwrap();
        service.record_increment(org_id, UsageMetric::ApiCalls, 5).await.unwrap();
        service.record_increment(org_id, UsageMetric::StorageBytes, 1000).await.unwrap();

        let result = service.get_all_metrics(org_id).await;
        assert!(result.is_ok());

        let metrics = result.unwrap();
        assert_eq!(metrics.len(), 3);
        assert!(metrics.contains_key(&UsageMetric::Messages));
        assert!(metrics.contains_key(&UsageMetric::ApiCalls));
        assert!(metrics.contains_key(&UsageMetric::StorageBytes));
    }

    #[tokio::test]
    async fn test_generate_report() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        service.record_increment(org_id, UsageMetric::Messages, 50).await.unwrap();
        service.record_increment(org_id, UsageMetric::ApiCalls, 100).await.unwrap();

        let result = service
            .generate_report(org_id, now - chrono::Duration::days(1), now)
            .await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.organization_id, org_id);
        assert_eq!(report.metrics.len(), 7);

        let messages_report = report.metrics.iter().find(|m| m.metric == UsageMetric::Messages).unwrap();
        assert_eq!(messages_report.current_value, 50);

        let api_report = report.metrics.iter().find(|m| m.metric == UsageMetric::ApiCalls).unwrap();
        assert_eq!(api_report.current_value, 100);
    }

    #[tokio::test]
    async fn test_create_daily_snapshot() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 25).await.unwrap();
        service.record_increment(org_id, UsageMetric::Bots, 2).await.unwrap();

        let result = service.create_daily_snapshot(org_id).await;
        assert!(result.is_ok());

        let snapshot = result.unwrap();
        assert_eq!(snapshot.organization_id, org_id);
        assert_eq!(snapshot.metrics.get(&UsageMetric::Messages), Some(&25));
        assert_eq!(snapshot.metrics.get(&UsageMetric::Bots), Some(&2));
    }

    #[tokio::test]
    async fn test_get_daily_snapshots() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 10).await.unwrap();
        service.create_daily_snapshot(org_id).await.unwrap();

        service.record_increment(org_id, UsageMetric::Messages, 20).await.unwrap();
        service.create_daily_snapshot(org_id).await.unwrap();

        let result = service.get_daily_snapshots(org_id, 10).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_daily_snapshots_limited() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        for i in 0..5 {
            service.record_increment(org_id, UsageMetric::Messages, i * 10).await.unwrap();
            service.create_daily_snapshot(org_id).await.unwrap();
        }

        let result = service.get_daily_snapshots(org_id, 3).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_reset_daily_metrics() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 100).await.unwrap();
        service.record_increment(org_id, UsageMetric::ApiCalls, 50).await.unwrap();
        service.record_increment(org_id, UsageMetric::Bots, 3).await.unwrap();

        let result = service.reset_daily_metrics(org_id).await;
        assert!(result.is_ok());

        let messages = service.get_current_usage(org_id, UsageMetric::Messages).await.unwrap();
        let api_calls = service.get_current_usage(org_id, UsageMetric::ApiCalls).await.unwrap();
        let bots = service.get_current_usage(org_id, UsageMetric::Bots).await.unwrap();

        assert_eq!(messages, 0);
        assert_eq!(api_calls, 0);
        assert_eq!(bots, 3);
    }

    #[tokio::test]
    async fn test_prune_old_events() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::Messages, 10).await.unwrap();
        service.record_increment(org_id, UsageMetric::Messages, 20).await.unwrap();

        let result = service.prune_old_events(30).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_decrement_reduces_total() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::StorageBytes, 1000).await.unwrap();
        service.record_decrement(org_id, UsageMetric::StorageBytes, 300).await.unwrap();

        let usage = service.get_current_usage(org_id, UsageMetric::StorageBytes).await.unwrap();
        assert_eq!(usage, 700);
    }

    #[tokio::test]
    async fn test_decrement_saturating() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        service.record_increment(org_id, UsageMetric::StorageBytes, 100).await.unwrap();
        service.record_decrement(org_id, UsageMetric::StorageBytes, 500).await.unwrap();

        let usage = service.get_current_usage(org_id, UsageMetric::StorageBytes).await.unwrap();
        assert_eq!(usage, 0);
    }

    #[test]
    fn test_usage_trend_variants() {
        let trends = vec![
            UsageTrend::Increasing,
            UsageTrend::Decreasing,
            UsageTrend::Stable,
            UsageTrend::Unknown,
        ];

        for trend in trends {
            let serialized = serde_json::to_string(&trend).unwrap();
            let deserialized: UsageTrend = serde_json::from_str(&serialized).unwrap();
            assert_eq!(trend, deserialized);
        }
    }

    #[test]
    fn test_aggregated_usage_default() {
        let agg = AggregatedUsage::default();
        assert_eq!(agg.total, 0);
        assert_eq!(agg.count, 0);
        assert!(agg.min.is_none());
        assert!(agg.max.is_none());
        assert_eq!(agg.average, 0.0);
    }

    #[tokio::test]
    async fn test_multiple_organizations() {
        let service = UsageMeteringService::new();
        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();

        service.record_increment(org1, UsageMetric::Messages, 100).await.unwrap();
        service.record_increment(org2, UsageMetric::Messages, 50).await.unwrap();

        let usage1 = service.get_current_usage(org1, UsageMetric::Messages).await.unwrap();
        let usage2 = service.get_current_usage(org2, UsageMetric::Messages).await.unwrap();

        assert_eq!(usage1, 100);
        assert_eq!(usage2, 50);
    }

    #[tokio::test]
    async fn test_all_metric_types_metering() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();

        let metrics = vec![
            (UsageMetric::Messages, 10),
            (UsageMetric::StorageBytes, 1000),
            (UsageMetric::ApiCalls, 5),
            (UsageMetric::Bots, 1),
            (UsageMetric::Users, 2),
            (UsageMetric::KbDocuments, 3),
            (UsageMetric::Apps, 1),
        ];

        for (metric, amount) in &metrics {
            service.record_increment(org_id, *metric, *amount).await.unwrap();
        }

        for (metric, expected) in &metrics {
            let usage = service.get_current_usage(org_id, *metric).await.unwrap();
            assert_eq!(usage, *expected, "Failed for metric {:?}", metric);
        }
    }

    #[tokio::test]
    async fn test_report_trend_unknown_without_snapshots() {
        let service = UsageMeteringService::new();
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        service.record_increment(org_id, UsageMetric::Messages, 50).await.unwrap();

        let report = service
            .generate_report(org_id, now - chrono::Duration::days(1), now)
            .await
            .unwrap();

        let messages_report = report.metrics.iter().find(|m| m.metric == UsageMetric::Messages).unwrap();
        assert_eq!(messages_report.trend, UsageTrend::Unknown);
    }
}
