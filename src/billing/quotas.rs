use crate::billing::{BillingError, LimitValue, UsageMetric};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct QuotaManager {
    usage_cache: Arc<RwLock<HashMap<Uuid, OrganizationQuotas>>>,
    alert_thresholds: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct OrganizationQuotas {
    pub organization_id: Uuid,
    pub plan_id: String,
    pub limits: QuotaLimits,
    pub usage: QuotaUsage,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct QuotaLimits {
    pub messages_per_day: LimitValue,
    pub storage_bytes: LimitValue,
    pub api_calls_per_day: LimitValue,
    pub bots: LimitValue,
    pub users: LimitValue,
    pub kb_documents: LimitValue,
    pub apps: LimitValue,
}

#[derive(Debug, Clone, Default)]
pub struct QuotaUsage {
    pub messages_today: u64,
    pub storage_bytes: u64,
    pub api_calls_today: u64,
    pub bots: u64,
    pub users: u64,
    pub kb_documents: u64,
    pub apps: u64,
    pub last_reset: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub metric: UsageMetric,
    pub current: u64,
    pub limit: Option<u64>,
    pub remaining: Option<u64>,
    pub percentage_used: f64,
    pub alerts: Vec<QuotaAlert>,
}

#[derive(Debug, Clone)]
pub struct QuotaAlert {
    pub metric: UsageMetric,
    pub threshold: f64,
    pub current_percentage: f64,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum QuotaAction {
    Allow,
    Warn { message: String, percentage: f64 },
    Block { message: String },
}

impl QuotaManager {
    pub fn new() -> Self {
        Self {
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_thresholds: vec![80.0, 90.0, 100.0],
        }
    }

    pub fn with_thresholds(thresholds: Vec<f64>) -> Self {
        Self {
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
            alert_thresholds: thresholds,
        }
    }

    pub async fn set_quotas(&self, quotas: OrganizationQuotas) {
        let mut cache = self.usage_cache.write().await;
        cache.insert(quotas.organization_id, quotas);
    }

    pub async fn get_quotas(&self, organization_id: Uuid) -> Option<OrganizationQuotas> {
        let cache = self.usage_cache.read().await;
        cache.get(&organization_id).cloned()
    }

    pub async fn check_quota(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        increment: u64,
    ) -> Result<QuotaCheckResult, BillingError> {
        let cache = self.usage_cache.read().await;
        let quotas = cache
            .get(&organization_id)
            .ok_or(BillingError::SubscriptionNotFound)?;

        let (current, limit) = self.get_metric_values(quotas, metric);
        let new_value = current + increment;

        let (allowed, remaining, percentage) = match limit {
            LimitValue::Unlimited => (true, None, 0.0),
            LimitValue::Limited(max) => {
                let allowed = new_value <= max;
                let remaining = if new_value > max { 0 } else { max - new_value };
                let percentage = (new_value as f64 / max as f64) * 100.0;
                (allowed, Some(remaining), percentage)
            }
        };

        let alerts = self.generate_alerts(metric, percentage);

        Ok(QuotaCheckResult {
            allowed,
            metric,
            current: new_value,
            limit: limit.value(),
            remaining,
            percentage_used: percentage,
            alerts,
        })
    }

    pub async fn increment_usage(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        amount: u64,
    ) -> Result<QuotaCheckResult, BillingError> {
        let check_result = self.check_quota(organization_id, metric, amount).await?;

        if check_result.allowed {
            let mut cache = self.usage_cache.write().await;
            if let Some(quotas) = cache.get_mut(&organization_id) {
                self.apply_increment(&mut quotas.usage, metric, amount);
            }
        }

        Ok(check_result)
    }

    pub async fn decrement_usage(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        amount: u64,
    ) -> Result<(), BillingError> {
        let mut cache = self.usage_cache.write().await;
        let quotas = cache
            .get_mut(&organization_id)
            .ok_or(BillingError::SubscriptionNotFound)?;

        self.apply_decrement(&mut quotas.usage, metric, amount);
        Ok(())
    }

    pub async fn set_usage(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
        value: u64,
    ) -> Result<(), BillingError> {
        let mut cache = self.usage_cache.write().await;
        let quotas = cache
            .get_mut(&organization_id)
            .ok_or(BillingError::SubscriptionNotFound)?;

        self.set_metric_value(&mut quotas.usage, metric, value);
        Ok(())
    }

    pub async fn reset_daily_quotas(&self, organization_id: Uuid) -> Result<(), BillingError> {
        let mut cache = self.usage_cache.write().await;
        let quotas = cache
            .get_mut(&organization_id)
            .ok_or(BillingError::SubscriptionNotFound)?;

        quotas.usage.messages_today = 0;
        quotas.usage.api_calls_today = 0;
        quotas.usage.last_reset = Some(chrono::Utc::now());

        Ok(())
    }

    pub async fn reset_all_daily_quotas(&self) {
        let mut cache = self.usage_cache.write().await;
        let now = chrono::Utc::now();

        for quotas in cache.values_mut() {
            quotas.usage.messages_today = 0;
            quotas.usage.api_calls_today = 0;
            quotas.usage.last_reset = Some(now);
        }
    }

    pub async fn get_usage_summary(&self, organization_id: Uuid) -> Result<UsageSummary, BillingError> {
        let cache = self.usage_cache.read().await;
        let quotas = cache
            .get(&organization_id)
            .ok_or(BillingError::SubscriptionNotFound)?;

        let metrics = vec![
            UsageMetric::Messages,
            UsageMetric::StorageBytes,
            UsageMetric::ApiCalls,
            UsageMetric::Bots,
            UsageMetric::Users,
            UsageMetric::KbDocuments,
            UsageMetric::Apps,
        ];

        let items: Vec<UsageSummaryItem> = metrics
            .into_iter()
            .map(|metric| {
                let (current, limit) = self.get_metric_values(quotas, metric);
                let (remaining, percentage) = match limit {
                    LimitValue::Unlimited => (None, 0.0),
                    LimitValue::Limited(max) => {
                        let remaining = if current > max { 0 } else { max - current };
                        let percentage = (current as f64 / max as f64) * 100.0;
                        (Some(remaining), percentage)
                    }
                };

                UsageSummaryItem {
                    metric,
                    current,
                    limit: limit.value(),
                    remaining,
                    percentage_used: percentage,
                    is_unlimited: limit.is_unlimited(),
                }
            })
            .collect();

        Ok(UsageSummary {
            organization_id,
            plan_id: quotas.plan_id.clone(),
            items,
            period_start: quotas.period_start,
            period_end: quotas.period_end,
        })
    }

    pub async fn check_action(&self, organization_id: Uuid, metric: UsageMetric) -> QuotaAction {
        match self.check_quota(organization_id, metric, 1).await {
            Ok(result) => {
                if !result.allowed {
                    QuotaAction::Block {
                        message: format!(
                            "Quota exceeded for {:?}. Current: {}, Limit: {:?}",
                            metric, result.current, result.limit
                        ),
                    }
                } else if result.percentage_used >= 90.0 {
                    QuotaAction::Warn {
                        message: format!(
                            "Approaching quota limit for {:?}. {}% used.",
                            metric, result.percentage_used as u32
                        ),
                        percentage: result.percentage_used,
                    }
                } else {
                    QuotaAction::Allow
                }
            }
            Err(_) => QuotaAction::Block {
                message: "Unable to verify quota".to_string(),
            },
        }
    }

    fn get_metric_values(&self, quotas: &OrganizationQuotas, metric: UsageMetric) -> (u64, LimitValue) {
        match metric {
            UsageMetric::Messages => (quotas.usage.messages_today, quotas.limits.messages_per_day),
            UsageMetric::StorageBytes => (quotas.usage.storage_bytes, quotas.limits.storage_bytes),
            UsageMetric::ApiCalls => (quotas.usage.api_calls_today, quotas.limits.api_calls_per_day),
            UsageMetric::Bots => (quotas.usage.bots, quotas.limits.bots),
            UsageMetric::Users => (quotas.usage.users, quotas.limits.users),
            UsageMetric::KbDocuments => (quotas.usage.kb_documents, quotas.limits.kb_documents),
            UsageMetric::Apps => (quotas.usage.apps, quotas.limits.apps),
        }
    }

    fn apply_increment(&self, usage: &mut QuotaUsage, metric: UsageMetric, amount: u64) {
        match metric {
            UsageMetric::Messages => usage.messages_today += amount,
            UsageMetric::StorageBytes => usage.storage_bytes += amount,
            UsageMetric::ApiCalls => usage.api_calls_today += amount,
            UsageMetric::Bots => usage.bots += amount,
            UsageMetric::Users => usage.users += amount,
            UsageMetric::KbDocuments => usage.kb_documents += amount,
            UsageMetric::Apps => usage.apps += amount,
        }
    }

    fn apply_decrement(&self, usage: &mut QuotaUsage, metric: UsageMetric, amount: u64) {
        match metric {
            UsageMetric::Messages => usage.messages_today = usage.messages_today.saturating_sub(amount),
            UsageMetric::StorageBytes => usage.storage_bytes = usage.storage_bytes.saturating_sub(amount),
            UsageMetric::ApiCalls => usage.api_calls_today = usage.api_calls_today.saturating_sub(amount),
            UsageMetric::Bots => usage.bots = usage.bots.saturating_sub(amount),
            UsageMetric::Users => usage.users = usage.users.saturating_sub(amount),
            UsageMetric::KbDocuments => usage.kb_documents = usage.kb_documents.saturating_sub(amount),
            UsageMetric::Apps => usage.apps = usage.apps.saturating_sub(amount),
        }
    }

    fn set_metric_value(&self, usage: &mut QuotaUsage, metric: UsageMetric, value: u64) {
        match metric {
            UsageMetric::Messages => usage.messages_today = value,
            UsageMetric::StorageBytes => usage.storage_bytes = value,
            UsageMetric::ApiCalls => usage.api_calls_today = value,
            UsageMetric::Bots => usage.bots = value,
            UsageMetric::Users => usage.users = value,
            UsageMetric::KbDocuments => usage.kb_documents = value,
            UsageMetric::Apps => usage.apps = value,
        }
    }

    fn generate_alerts(&self, metric: UsageMetric, percentage: f64) -> Vec<QuotaAlert> {
        self.alert_thresholds
            .iter()
            .filter(|&&threshold| percentage >= threshold)
            .map(|&threshold| QuotaAlert {
                metric,
                threshold,
                current_percentage: percentage,
                message: self.alert_message(metric, threshold, percentage),
            })
            .collect()
    }

    fn alert_message(&self, metric: UsageMetric, threshold: f64, current: f64) -> String {
        let metric_name = match metric {
            UsageMetric::Messages => "messages",
            UsageMetric::StorageBytes => "storage",
            UsageMetric::ApiCalls => "API calls",
            UsageMetric::Bots => "bots",
            UsageMetric::Users => "users",
            UsageMetric::KbDocuments => "KB documents",
            UsageMetric::Apps => "apps",
        };

        if current >= 100.0 {
            format!("You have reached your {} quota limit.", metric_name)
        } else {
            format!(
                "You have used {}% of your {} quota (threshold: {}%).",
                current as u32, metric_name, threshold as u32
            )
        }
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct UsageSummary {
    pub organization_id: Uuid,
    pub plan_id: String,
    pub items: Vec<UsageSummaryItem>,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UsageSummaryItem {
    pub metric: UsageMetric,
    pub current: u64,
    pub limit: Option<u64>,
    pub remaining: Option<u64>,
    pub percentage_used: f64,
    pub is_unlimited: bool,
}

pub struct QuotaMiddleware {
    quota_manager: Arc<QuotaManager>,
}

impl QuotaMiddleware {
    pub fn new(quota_manager: Arc<QuotaManager>) -> Self {
        Self { quota_manager }
    }

    pub async fn check_and_increment(
        &self,
        organization_id: Uuid,
        metric: UsageMetric,
    ) -> Result<QuotaCheckResult, BillingError> {
        self.quota_manager.increment_usage(organization_id, metric, 1).await
    }

    pub async fn check_storage(
        &self,
        organization_id: Uuid,
        bytes: u64,
    ) -> Result<QuotaCheckResult, BillingError> {
        self.quota_manager.check_quota(organization_id, UsageMetric::StorageBytes, bytes).await
    }

    pub async fn add_storage(
        &self,
        organization_id: Uuid,
        bytes: u64,
    ) -> Result<QuotaCheckResult, BillingError> {
        self.quota_manager.increment_usage(organization_id, UsageMetric::StorageBytes, bytes).await
    }

    pub async fn remove_storage(
        &self,
        organization_id: Uuid,
        bytes: u64,
    ) -> Result<(), BillingError> {
        self.quota_manager.decrement_usage(organization_id, UsageMetric::StorageBytes, bytes).await
    }
}

pub async fn daily_quota_reset_job(quota_manager: Arc<QuotaManager>) {
    loop {
        let now = chrono::Utc::now();
        let tomorrow = (now + chrono::Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));

        if let Some(next_reset) = tomorrow {
            let duration = next_reset - now;
            if let Ok(std_duration) = duration.to_std() {
                tokio::time::sleep(std_duration).await;
            }
        }

        quota_manager.reset_all_daily_quotas().await;
        tracing::info!("Daily quotas reset completed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::LimitValue;

    fn create_test_quotas(org_id: Uuid, plan_id: &str) -> OrganizationQuotas {
        let now = chrono::Utc::now();
        OrganizationQuotas {
            organization_id: org_id,
            plan_id: plan_id.to_string(),
            limits: QuotaLimits {
                messages_per_day: LimitValue::Limited(100),
                storage_bytes: LimitValue::Limited(1024 * 1024 * 100),
                api_calls_per_day: LimitValue::Limited(1000),
                bots: LimitValue::Limited(5),
                users: LimitValue::Limited(10),
                kb_documents: LimitValue::Limited(50),
                apps: LimitValue::Limited(10),
            },
            usage: QuotaUsage::default(),
            period_start: now,
            period_end: now + chrono::Duration::days(30),
        }
    }

    fn create_unlimited_quotas(org_id: Uuid) -> OrganizationQuotas {
        let now = chrono::Utc::now();
        OrganizationQuotas {
            organization_id: org_id,
            plan_id: "enterprise".to_string(),
            limits: QuotaLimits {
                messages_per_day: LimitValue::Unlimited,
                storage_bytes: LimitValue::Unlimited,
                api_calls_per_day: LimitValue::Unlimited,
                bots: LimitValue::Unlimited,
                users: LimitValue::Unlimited,
                kb_documents: LimitValue::Unlimited,
                apps: LimitValue::Unlimited,
            },
            usage: QuotaUsage::default(),
            period_start: now,
            period_end: now + chrono::Duration::days(30),
        }
    }



    #[tokio::test]
    async fn test_set_and_get_quotas() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");

        manager.set_quotas(quotas.clone()).await;

        let retrieved = manager.get_quotas(org_id).await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.organization_id, org_id);
        assert_eq!(retrieved.plan_id, "business");
    }

    #[tokio::test]
    async fn test_get_quotas_nonexistent() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();

        let result = manager.get_quotas(org_id).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_check_quota_within_limit() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager
            .check_quota(org_id, UsageMetric::Messages, 50)
            .await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);
        assert_eq!(check.current, 50);
        assert_eq!(check.limit, Some(100));
        assert_eq!(check.remaining, Some(50));
        assert_eq!(check.percentage_used, 50.0);
    }

    #[tokio::test]
    async fn test_check_quota_at_limit() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager
            .check_quota(org_id, UsageMetric::Messages, 100)
            .await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);
        assert_eq!(check.remaining, Some(0));
        assert_eq!(check.percentage_used, 100.0);
    }

    #[tokio::test]
    async fn test_check_quota_exceeds_limit() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager
            .check_quota(org_id, UsageMetric::Messages, 101)
            .await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed);
        assert_eq!(check.remaining, Some(0));
    }

    #[tokio::test]
    async fn test_check_quota_unlimited() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_unlimited_quotas(org_id);
        manager.set_quotas(quotas).await;

        let result = manager
            .check_quota(org_id, UsageMetric::Messages, 1_000_000)
            .await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);
        assert_eq!(check.limit, None);
        assert_eq!(check.remaining, None);
        assert_eq!(check.percentage_used, 0.0);
    }

    #[tokio::test]
    async fn test_check_quota_subscription_not_found() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();

        let result = manager
            .check_quota(org_id, UsageMetric::Messages, 1)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BillingError::SubscriptionNotFound));
    }

    #[tokio::test]
    async fn test_increment_usage() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager
            .increment_usage(org_id, UsageMetric::Messages, 10)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().allowed);

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.messages_today, 10);
    }

    #[tokio::test]
    async fn test_increment_usage_blocked_when_exceeded() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager
            .increment_usage(org_id, UsageMetric::Messages, 150)
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap().allowed);

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.messages_today, 0);
    }

    #[tokio::test]
    async fn test_decrement_usage() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        manager.increment_usage(org_id, UsageMetric::Bots, 3).await.unwrap();

        let result = manager.decrement_usage(org_id, UsageMetric::Bots, 1).await;
        assert!(result.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.bots, 2);
    }

    #[tokio::test]
    async fn test_decrement_usage_saturating() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager.decrement_usage(org_id, UsageMetric::Bots, 100).await;
        assert!(result.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.bots, 0);
    }

    #[tokio::test]
    async fn test_set_usage() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager.set_usage(org_id, UsageMetric::StorageBytes, 5000).await;
        assert!(result.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.storage_bytes, 5000);
    }

    #[tokio::test]
    async fn test_reset_daily_quotas() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        manager.increment_usage(org_id, UsageMetric::Messages, 50).await.unwrap();
        manager.increment_usage(org_id, UsageMetric::ApiCalls, 100).await.unwrap();
        manager.increment_usage(org_id, UsageMetric::Bots, 2).await.unwrap();

        let result = manager.reset_daily_quotas(org_id).await;
        assert!(result.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.messages_today, 0);
        assert_eq!(quotas.usage.api_calls_today, 0);
        assert_eq!(quotas.usage.bots, 2);
        assert!(quotas.usage.last_reset.is_some());
    }

    #[tokio::test]
    async fn test_reset_all_daily_quotas() {
        let manager = QuotaManager::new();
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();

        manager.set_quotas(create_test_quotas(org_id1, "business")).await;
        manager.set_quotas(create_test_quotas(org_id2, "personal")).await;

        manager.increment_usage(org_id1, UsageMetric::Messages, 50).await.unwrap();
        manager.increment_usage(org_id2, UsageMetric::Messages, 30).await.unwrap();

        manager.reset_all_daily_quotas().await;

        let q1 = manager.get_quotas(org_id1).await.unwrap();
        let q2 = manager.get_quotas(org_id2).await.unwrap();
        assert_eq!(q1.usage.messages_today, 0);
        assert_eq!(q2.usage.messages_today, 0);
    }

    #[tokio::test]
    async fn test_get_usage_summary() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        manager.increment_usage(org_id, UsageMetric::Messages, 50).await.unwrap();
        manager.increment_usage(org_id, UsageMetric::Bots, 2).await.unwrap();

        let result = manager.get_usage_summary(org_id).await;
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.organization_id, org_id);
        assert_eq!(summary.plan_id, "business");
        assert_eq!(summary.items.len(), 7);

        let messages_item = summary.items.iter().find(|i| i.metric == UsageMetric::Messages).unwrap();
        assert_eq!(messages_item.current, 50);
        assert_eq!(messages_item.limit, Some(100));
        assert_eq!(messages_item.remaining, Some(50));
        assert_eq!(messages_item.percentage_used, 50.0);
        assert!(!messages_item.is_unlimited);
    }

    #[tokio::test]
    async fn test_check_action_allow() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let action = manager.check_action(org_id, UsageMetric::Messages).await;
        assert!(matches!(action, QuotaAction::Allow));
    }

    #[tokio::test]
    async fn test_check_action_warn() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        manager.set_usage(org_id, UsageMetric::Messages, 91).await.unwrap();

        let action = manager.check_action(org_id, UsageMetric::Messages).await;
        assert!(matches!(action, QuotaAction::Warn { .. }));
    }

    #[tokio::test]
    async fn test_check_action_block() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        manager.set_usage(org_id, UsageMetric::Messages, 100).await.unwrap();

        let action = manager.check_action(org_id, UsageMetric::Messages).await;
        assert!(matches!(action, QuotaAction::Block { .. }));
    }

    #[tokio::test]
    async fn test_alerts_generated_at_thresholds() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let result = manager.check_quota(org_id, UsageMetric::Messages, 85).await.unwrap();
        assert_eq!(result.alerts.len(), 1);
        assert_eq!(result.alerts[0].threshold, 80.0);

        let result = manager.check_quota(org_id, UsageMetric::Messages, 95).await.unwrap();
        assert_eq!(result.alerts.len(), 2);

        let result = manager.check_quota(org_id, UsageMetric::Messages, 100).await.unwrap();
        assert_eq!(result.alerts.len(), 3);
    }

    #[tokio::test]
    async fn test_quota_middleware_check_and_increment() {
        let manager = Arc::new(QuotaManager::new());
        let middleware = QuotaMiddleware::new(manager.clone());
        let org_id = Uuid::new_v4();

        manager.set_quotas(create_test_quotas(org_id, "business")).await;

        let result = middleware.check_and_increment(org_id, UsageMetric::ApiCalls).await;
        assert!(result.is_ok());
        assert!(result.unwrap().allowed);

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.api_calls_today, 1);
    }

    #[tokio::test]
    async fn test_quota_middleware_storage_operations() {
        let manager = Arc::new(QuotaManager::new());
        let middleware = QuotaMiddleware::new(manager.clone());
        let org_id = Uuid::new_v4();

        manager.set_quotas(create_test_quotas(org_id, "business")).await;

        let check = middleware.check_storage(org_id, 1000).await;
        assert!(check.is_ok());
        assert!(check.unwrap().allowed);

        let add = middleware.add_storage(org_id, 1000).await;
        assert!(add.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.storage_bytes, 1000);

        let remove = middleware.remove_storage(org_id, 500).await;
        assert!(remove.is_ok());

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.storage_bytes, 500);
    }

    #[test]
    fn test_quota_usage_default() {
        let usage = QuotaUsage::default();
        assert_eq!(usage.messages_today, 0);
        assert_eq!(usage.storage_bytes, 0);
        assert_eq!(usage.api_calls_today, 0);
        assert_eq!(usage.bots, 0);
        assert_eq!(usage.users, 0);
        assert_eq!(usage.kb_documents, 0);
        assert_eq!(usage.apps, 0);
        assert!(usage.last_reset.is_none());
    }

    #[tokio::test]
    async fn test_all_metric_types() {
        let manager = QuotaManager::new();
        let org_id = Uuid::new_v4();
        let quotas = create_test_quotas(org_id, "business");
        manager.set_quotas(quotas).await;

        let metrics = vec![
            (UsageMetric::Messages, 10),
            (UsageMetric::StorageBytes, 1000),
            (UsageMetric::ApiCalls, 5),
            (UsageMetric::Bots, 1),
            (UsageMetric::Users, 2),
            (UsageMetric::KbDocuments, 3),
            (UsageMetric::Apps, 1),
        ];

        for (metric, amount) in metrics {
            let result = manager.increment_usage(org_id, metric, amount).await;
            assert!(result.is_ok(), "Failed for metric {:?}", metric);
            assert!(result.unwrap().allowed, "Not allowed for metric {:?}", metric);
        }

        let quotas = manager.get_quotas(org_id).await.unwrap();
        assert_eq!(quotas.usage.messages_today, 10);
        assert_eq!(quotas.usage.storage_bytes, 1000);
        assert_eq!(quotas.usage.api_calls_today, 5);
        assert_eq!(quotas.usage.bots, 1);
        assert_eq!(quotas.usage.users, 2);
        assert_eq!(quotas.usage.kb_documents, 3);
        assert_eq!(quotas.usage.apps, 1);
    }
}
