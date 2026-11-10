use super::*;

pub use botlib::db_pool::DbPool;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAlertNotification {
    pub alert_id: Uuid,
    pub organization_id: Uuid,
    pub severity: String,
    pub alert_type: String,
    pub title: String,
    pub message: String,
    pub metric: String,
    pub percentage: f64,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductConfig {
    pub branding: BrandingConfig,
    pub plans: HashMap<String, PlanConfig>,
    pub features: Vec<String>,
    pub stripe: Option<StripeConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub name: String,
    pub logo: Option<String>,
    pub primary_color: String,
    pub secondary_color: Option<String>,
    pub favicon: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub name: String,
    pub description: Option<String>,
    pub price: PlanPrice,
    pub limits: PlanLimits,
    pub features: Vec<String>,
    pub stripe_price_id: Option<String>,
    pub trial_days: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlanPrice {
    Free,
    Fixed { amount: u64, currency: String, period: BillingPeriod },
    Custom,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BillingPeriod {
    Monthly,
    Yearly,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimits {
    pub messages_per_day: LimitValue,
    pub storage_mb: LimitValue,
    pub bots: LimitValue,
    pub users: LimitValue,
    pub api_calls_per_day: LimitValue,
    pub signups_per_day: Option<LimitValue>,
    pub kb_documents: LimitValue,
    pub apps: LimitValue,
    pub emails_per_day: LimitValue,
    pub builds_per_day: LimitValue,
    pub uploads_per_hour: LimitValue,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LimitValue {
    Limited(u64),
    Unlimited,
}
impl LimitValue {
    pub fn is_unlimited(&self) -> bool {
        matches!(self, Self::Unlimited)
    }

    pub fn value(&self) -> Option<u64> {
        match self {
            Self::Limited(v) => Some(*v),
            Self::Unlimited => None,
        }
    }

    pub fn check(&self, current: u64) -> bool {
        match self {
            Self::Limited(limit) => current < *limit,
            Self::Unlimited => true,
        }
    }
}
impl Default for LimitValue {
    fn default() -> Self {
        Self::Limited(0)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    pub publishable_key: String,
    pub webhook_secret: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: chrono::DateTime<chrono::Utc>,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Unpaid,
    Incomplete,
    IncompleteExpired,
    Paused,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub organization_id: Uuid,
    pub metric: UsageMetric,
    pub value: u64,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UsageMetric {
    #[default]
    Messages,
    StorageBytes,
    ApiCalls,
    Bots,
    Users,
    KbDocuments,
    Apps,
    EmailsPerDay,
    BuildsPerDay,
    UploadsPerHour,
}
pub struct BillingService {
    config: Arc<RwLock<ProductConfig>>,
    saas_enabled: bool,
}
impl BillingService {
    pub fn new(config: ProductConfig, saas_enabled: bool) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            saas_enabled,
        }
    }

    pub async fn load_from_file(path: &Path) -> Result<ProductConfig, BillingError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| BillingError::ConfigLoad(e.to_string()))?;

        let config: ProductConfig = serde_json::from_str(&content)
            .map_err(|e| BillingError::ConfigParse(e.to_string()))?;

        Ok(config)
    }

    pub fn is_saas_enabled(&self) -> bool {
        self.saas_enabled
    }

    pub async fn get_plan(&self, plan_id: &str) -> Option<PlanConfig> {
        let config = self.config.read().await;
        config.plans.get(plan_id).cloned()
    }

    pub async fn get_all_plans(&self) -> HashMap<String, PlanConfig> {
        let config = self.config.read().await;
        config.plans.clone()
    }

    pub async fn check_limit(
        &self,
        plan_id: &str,
        metric: UsageMetric,
        current_usage: u64,
    ) -> Result<bool, BillingError> {
        let plan = self.get_plan(plan_id).await
            .ok_or_else(|| BillingError::PlanNotFound(plan_id.to_string()))?;

        let limit = match metric {
            UsageMetric::Messages => plan.limits.messages_per_day,
            UsageMetric::StorageBytes => LimitValue::Limited(plan.limits.storage_mb.value().unwrap_or(0) * 1024 * 1024),
            UsageMetric::ApiCalls => plan.limits.api_calls_per_day,
            UsageMetric::Bots => plan.limits.bots,
            UsageMetric::Users => plan.limits.users,
            UsageMetric::KbDocuments => plan.limits.kb_documents,
            UsageMetric::Apps => plan.limits.apps,
            UsageMetric::EmailsPerDay => plan.limits.emails_per_day,
            UsageMetric::BuildsPerDay => plan.limits.builds_per_day,
            UsageMetric::UploadsPerHour => plan.limits.uploads_per_hour,
        };

        Ok(limit.check(current_usage))
    }

    pub async fn get_branding(&self) -> BrandingConfig {
        let config = self.config.read().await;
        config.branding.clone()
    }
}
#[derive(Debug, Clone)]
pub enum BillingError {
    ConfigLoad(String),
    ConfigParse(String),
    PlanNotFound(String),
    QuotaExceeded(UsageMetric),
    StripeError(String),
    SubscriptionNotFound,
    InvalidOperation(String),
}
impl std::fmt::Display for BillingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigLoad(e) => write!(f, "Failed to load billing config: {e}"),
            Self::ConfigParse(e) => write!(f, "Failed to parse billing config: {e}"),
            Self::PlanNotFound(id) => write!(f, "Plan not found: {id}"),
            Self::QuotaExceeded(metric) => write!(f, "Quota exceeded for {metric:?}"),
            Self::StripeError(e) => write!(f, "Stripe error: {e}"),
            Self::SubscriptionNotFound => write!(f, "Subscription not found"),
            Self::InvalidOperation(e) => write!(f, "Invalid operation: {e}"),
        }
    }
}
impl std::error::Error for BillingError {}

#[cfg(test)]
mod limit_tests {
    use super::*;

    fn limits(messages: LimitValue) -> PlanLimits {
        PlanLimits {
            messages_per_day: messages,
            storage_mb: LimitValue::Limited(100),
            bots: LimitValue::Limited(2),
            users: LimitValue::Limited(5),
            api_calls_per_day: LimitValue::Limited(1000),
            signups_per_day: Some(LimitValue::Limited(10)),
            kb_documents: LimitValue::Limited(50),
            apps: LimitValue::Limited(3),
            emails_per_day: LimitValue::Limited(200),
            builds_per_day: LimitValue::Limited(20),
            uploads_per_hour: LimitValue::Limited(30),
        }
    }

    fn config(limits: PlanLimits) -> ProductConfig {
        let plan = PlanConfig {
            name: "Starter".to_string(),
            description: None,
            price: PlanPrice::Free,
            limits,
            features: vec!["chat".to_string()],
            stripe_price_id: None,
            trial_days: None,
        };
        let mut plans = HashMap::new();
        plans.insert("starter".to_string(), plan);
        ProductConfig {
            branding: BrandingConfig {
                name: "Acme".to_string(),
                logo: None,
                primary_color: "#000000".to_string(),
                secondary_color: None,
                favicon: None,
            },
            plans,
            features: Vec::new(),
            stripe: None,
        }
    }

    #[test]
    fn a_limited_value_allows_usage_below_the_limit_only() {
        let limit = LimitValue::Limited(10);
        assert!(limit.check(0));
        assert!(limit.check(9));
        assert!(!limit.check(10));
        assert!(!limit.check(11));
    }

    #[test]
    fn an_unlimited_value_always_allows() {
        let limit = LimitValue::Unlimited;
        assert!(limit.is_unlimited());
        assert_eq!(limit.value(), None);
        assert!(limit.check(0));
        assert!(limit.check(u64::MAX));
    }

    #[test]
    fn a_limited_value_reports_its_number() {
        assert_eq!(LimitValue::Limited(42).value(), Some(42));
        assert!(!LimitValue::Limited(42).is_unlimited());
        // The default is a hard zero: deny everything until configured.
        assert!(!LimitValue::default().check(0));
    }

    #[tokio::test]
    async fn check_limit_maps_each_metric_to_its_field() {
        let service = BillingService::new(config(limits(LimitValue::Limited(100))), true);

        assert!(service
            .check_limit("starter", UsageMetric::Messages, 99)
            .await
            .expect("messages"));
        assert!(!service
            .check_limit("starter", UsageMetric::Messages, 100)
            .await
            .expect("messages"));
        assert!(!service
            .check_limit("starter", UsageMetric::Bots, 2)
            .await
            .expect("bots"));
        assert!(service
            .check_limit("starter", UsageMetric::Users, 4)
            .await
            .expect("users"));
    }

    #[tokio::test]
    async fn storage_is_compared_in_bytes() {
        let service = BillingService::new(config(limits(LimitValue::Limited(1))), true);
        // The plan allows 100 MB, so 100 MB of usage is the first refused value.
        let hundred_mb = 100 * 1024 * 1024;
        assert!(service
            .check_limit("starter", UsageMetric::StorageBytes, hundred_mb - 1)
            .await
            .expect("under"));
        assert!(!service
            .check_limit("starter", UsageMetric::StorageBytes, hundred_mb)
            .await
            .expect("at limit"));
    }

    #[tokio::test]
    async fn an_unknown_plan_is_reported() {
        let service = BillingService::new(config(limits(LimitValue::Limited(1))), true);
        assert!(service
            .check_limit("missing", UsageMetric::Messages, 0)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn plans_are_readable_by_id() {
        let service = BillingService::new(config(limits(LimitValue::Limited(1))), false);
        assert!(service.get_plan("starter").await.is_some());
        assert!(service.get_plan("missing").await.is_none());
        assert_eq!(service.get_all_plans().await.len(), 1);
        assert!(!service.is_saas_enabled());
    }

    #[tokio::test]
    async fn an_unlimited_plan_never_refuses() {
        let service = BillingService::new(config(limits(LimitValue::Unlimited)), true);
        assert!(service
            .check_limit("starter", UsageMetric::Messages, u64::MAX)
            .await
            .expect("unlimited"));
    }
}
