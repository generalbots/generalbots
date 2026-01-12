use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod alerts;
pub mod billing_ui;
pub mod invoice;
pub mod lifecycle;
pub mod meters;
pub mod middleware;
pub mod plans;
pub mod quotas;
pub mod stripe_integration;
pub mod testing;

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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

pub fn default_product_config() -> ProductConfig {
    let mut plans = HashMap::new();

    plans.insert("free".to_string(), PlanConfig {
        name: "Free".to_string(),
        description: Some("Get started with basic features".to_string()),
        price: PlanPrice::Free,
        limits: PlanLimits {
            messages_per_day: LimitValue::Limited(5),
            storage_mb: LimitValue::Limited(50),
            bots: LimitValue::Limited(1),
            users: LimitValue::Limited(1),
            api_calls_per_day: LimitValue::Limited(100),
            signups_per_day: Some(LimitValue::Limited(1)),
            kb_documents: LimitValue::Limited(10),
            apps: LimitValue::Limited(1),
        },
        features: vec!["basic_chat".to_string()],
        stripe_price_id: None,
        trial_days: None,
    });

    plans.insert("personal".to_string(), PlanConfig {
        name: "Personal".to_string(),
        description: Some("For individual users and small projects".to_string()),
        price: PlanPrice::Fixed {
            amount: 900,
            currency: "usd".to_string(),
            period: BillingPeriod::Monthly,
        },
        limits: PlanLimits {
            messages_per_day: LimitValue::Limited(100),
            storage_mb: LimitValue::Limited(1024),
            bots: LimitValue::Limited(5),
            users: LimitValue::Limited(1),
            api_calls_per_day: LimitValue::Limited(1000),
            signups_per_day: None,
            kb_documents: LimitValue::Limited(100),
            apps: LimitValue::Limited(5),
        },
        features: vec![
            "basic_chat".to_string(),
            "file_upload".to_string(),
            "email_support".to_string(),
        ],
        stripe_price_id: None,
        trial_days: Some(14),
    });

    plans.insert("business".to_string(), PlanConfig {
        name: "Business".to_string(),
        description: Some("For teams and growing businesses".to_string()),
        price: PlanPrice::Fixed {
            amount: 4900,
            currency: "usd".to_string(),
            period: BillingPeriod::Monthly,
        },
        limits: PlanLimits {
            messages_per_day: LimitValue::Limited(1000),
            storage_mb: LimitValue::Limited(10240),
            bots: LimitValue::Limited(25),
            users: LimitValue::Limited(10),
            api_calls_per_day: LimitValue::Limited(10000),
            signups_per_day: None,
            kb_documents: LimitValue::Limited(1000),
            apps: LimitValue::Limited(25),
        },
        features: vec![
            "basic_chat".to_string(),
            "file_upload".to_string(),
            "priority_support".to_string(),
            "custom_branding".to_string(),
            "api_access".to_string(),
            "analytics".to_string(),
        ],
        stripe_price_id: None,
        trial_days: Some(14),
    });

    plans.insert("enterprise".to_string(), PlanConfig {
        name: "Enterprise".to_string(),
        description: Some("For large organizations with advanced needs".to_string()),
        price: PlanPrice::Custom,
        limits: PlanLimits {
            messages_per_day: LimitValue::Unlimited,
            storage_mb: LimitValue::Unlimited,
            bots: LimitValue::Unlimited,
            users: LimitValue::Unlimited,
            api_calls_per_day: LimitValue::Unlimited,
            signups_per_day: None,
            kb_documents: LimitValue::Unlimited,
            apps: LimitValue::Unlimited,
        },
        features: vec![
            "basic_chat".to_string(),
            "file_upload".to_string(),
            "priority_support".to_string(),
            "custom_branding".to_string(),
            "api_access".to_string(),
            "analytics".to_string(),
            "sso_saml".to_string(),
            "sla_guarantee".to_string(),
            "dedicated_support".to_string(),
            "on_premise".to_string(),
            "audit_logs".to_string(),
        ],
        stripe_price_id: None,
        trial_days: None,
    });

    ProductConfig {
        branding: BrandingConfig {
            name: "General Bots".to_string(),
            logo: None,
            primary_color: "#1976d2".to_string(),
            secondary_color: None,
            favicon: None,
        },
        plans,
        features: vec![
            "basic_chat".to_string(),
            "file_upload".to_string(),
            "email_support".to_string(),
            "priority_support".to_string(),
            "custom_branding".to_string(),
            "api_access".to_string(),
            "analytics".to_string(),
            "sso_saml".to_string(),
            "sla_guarantee".to_string(),
            "dedicated_support".to_string(),
            "on_premise".to_string(),
            "audit_logs".to_string(),
        ],
        stripe: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_product_config() -> ProductConfig {
        default_product_config()
    }

    #[test]
    fn test_limit_value_limited() {
        let limit = LimitValue::Limited(100);
        assert!(!limit.is_unlimited());
        assert_eq!(limit.value(), Some(100));
        assert!(limit.check(50));
        assert!(limit.check(99));
        assert!(!limit.check(100));
        assert!(!limit.check(101));
    }

    #[test]
    fn test_limit_value_unlimited() {
        let limit = LimitValue::Unlimited;
        assert!(limit.is_unlimited());
        assert_eq!(limit.value(), None);
        assert!(limit.check(0));
        assert!(limit.check(1_000_000));
        assert!(limit.check(u64::MAX));
    }

    #[test]
    fn test_limit_value_default() {
        let limit = LimitValue::default();
        assert_eq!(limit.value(), Some(0));
        assert!(!limit.check(0));
        assert!(!limit.check(1));
    }

    #[test]
    fn test_default_product_config_has_required_plans() {
        let config = test_product_config();
        assert!(config.plans.contains_key("free"));
        assert!(config.plans.contains_key("personal"));
        assert!(config.plans.contains_key("business"));
        assert!(config.plans.contains_key("enterprise"));
    }

    #[test]
    fn test_free_plan_limits() {
        let config = test_product_config();
        let free = config.plans.get("free").unwrap();

        assert_eq!(free.name, "Free");
        assert!(matches!(free.price, PlanPrice::Free));

        assert_eq!(free.limits.messages_per_day.value(), Some(5));
        assert_eq!(free.limits.storage_mb.value(), Some(50));
        assert_eq!(free.limits.bots.value(), Some(1));
        assert_eq!(free.limits.users.value(), Some(1));
    }

    #[test]
    fn test_enterprise_plan_unlimited() {
        let config = test_product_config();
        let enterprise = config.plans.get("enterprise").unwrap();

        assert_eq!(enterprise.name, "Enterprise");
        assert!(matches!(enterprise.price, PlanPrice::Custom));

        assert!(enterprise.limits.messages_per_day.is_unlimited());
        assert!(enterprise.limits.storage_mb.is_unlimited());
        assert!(enterprise.limits.bots.is_unlimited());
        assert!(enterprise.limits.users.is_unlimited());
        assert!(enterprise.limits.api_calls_per_day.is_unlimited());
        assert!(enterprise.limits.kb_documents.is_unlimited());
        assert!(enterprise.limits.apps.is_unlimited());
    }

    #[test]
    fn test_business_plan_pricing() {
        let config = test_product_config();
        let business = config.plans.get("business").unwrap();

        match &business.price {
            PlanPrice::Fixed { amount, currency, period } => {
                assert_eq!(*amount, 4900);
                assert_eq!(currency, "usd");
                assert_eq!(*period, BillingPeriod::Monthly);
            }
            _ => panic!("Business plan should have fixed pricing"),
        }
    }

    #[test]
    fn test_personal_plan_has_trial() {
        let config = test_product_config();
        let personal = config.plans.get("personal").unwrap();
        assert_eq!(personal.trial_days, Some(14));
    }

    #[test]
    fn test_free_plan_no_trial() {
        let config = test_product_config();
        let free = config.plans.get("free").unwrap();
        assert_eq!(free.trial_days, None);
    }

    #[test]
    fn test_branding_config() {
        let config = test_product_config();
        assert_eq!(config.branding.name, "General Bots");
        assert_eq!(config.branding.primary_color, "#1976d2");
    }

    #[test]
    fn test_features_list() {
        let config = test_product_config();
        assert!(config.features.contains(&"basic_chat".to_string()));
        assert!(config.features.contains(&"api_access".to_string()));
        assert!(config.features.contains(&"sso_saml".to_string()));
    }

    #[test]
    fn test_enterprise_has_all_features() {
        let config = test_product_config();
        let enterprise = config.plans.get("enterprise").unwrap();

        assert!(enterprise.features.contains(&"basic_chat".to_string()));
        assert!(enterprise.features.contains(&"sso_saml".to_string()));
        assert!(enterprise.features.contains(&"audit_logs".to_string()));
        assert!(enterprise.features.contains(&"on_premise".to_string()));
    }

    #[tokio::test]
    async fn test_billing_service_get_plan() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let free = service.get_plan("free").await;
        assert!(free.is_some());
        assert_eq!(free.unwrap().name, "Free");

        let nonexistent = service.get_plan("nonexistent").await;
        assert!(nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_billing_service_get_all_plans() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let plans = service.get_all_plans().await;
        assert_eq!(plans.len(), 4);
        assert!(plans.contains_key("free"));
        assert!(plans.contains_key("personal"));
        assert!(plans.contains_key("business"));
        assert!(plans.contains_key("enterprise"));
    }

    #[tokio::test]
    async fn test_billing_service_check_limit_within_quota() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let result = service
            .check_limit("free", UsageMetric::Messages, 3)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_billing_service_check_limit_exceeded() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let result = service
            .check_limit("free", UsageMetric::Messages, 10)
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_billing_service_check_limit_unlimited() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let result = service
            .check_limit("enterprise", UsageMetric::Messages, 1_000_000)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_billing_service_check_limit_plan_not_found() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let result = service
            .check_limit("nonexistent", UsageMetric::Messages, 1)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BillingError::PlanNotFound(_)));
    }

    #[tokio::test]
    async fn test_billing_service_get_branding() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let branding = service.get_branding().await;
        assert_eq!(branding.name, "General Bots");
    }

    #[test]
    fn test_billing_service_saas_enabled() {
        let config = test_product_config();

        let service_enabled = BillingService::new(config.clone(), true);
        assert!(service_enabled.is_saas_enabled());

        let service_disabled = BillingService::new(config, false);
        assert!(!service_disabled.is_saas_enabled());
    }

    #[test]
    fn test_subscription_status_variants() {
        let statuses = vec![
            SubscriptionStatus::Active,
            SubscriptionStatus::Trialing,
            SubscriptionStatus::PastDue,
            SubscriptionStatus::Canceled,
            SubscriptionStatus::Unpaid,
            SubscriptionStatus::Incomplete,
            SubscriptionStatus::IncompleteExpired,
            SubscriptionStatus::Paused,
        ];

        for status in statuses {
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: SubscriptionStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_usage_metric_variants() {
        let metrics = vec![
            UsageMetric::Messages,
            UsageMetric::StorageBytes,
            UsageMetric::ApiCalls,
            UsageMetric::Bots,
            UsageMetric::Users,
            UsageMetric::KbDocuments,
            UsageMetric::Apps,
        ];

        for metric in metrics {
            let serialized = serde_json::to_string(&metric).unwrap();
            let deserialized: UsageMetric = serde_json::from_str(&serialized).unwrap();
            assert_eq!(metric, deserialized);
        }
    }

    #[test]
    fn test_billing_period_equality() {
        assert_eq!(BillingPeriod::Monthly, BillingPeriod::Monthly);
        assert_eq!(BillingPeriod::Yearly, BillingPeriod::Yearly);
        assert_ne!(BillingPeriod::Monthly, BillingPeriod::Yearly);
    }

    #[test]
    fn test_billing_error_display() {
        let errors = vec![
            (BillingError::ConfigLoad("test".to_string()), "Failed to load billing config: test"),
            (BillingError::ConfigParse("test".to_string()), "Failed to parse billing config: test"),
            (BillingError::PlanNotFound("test".to_string()), "Plan not found: test"),
            (BillingError::QuotaExceeded(UsageMetric::Messages), "Quota exceeded for Messages"),
            (BillingError::StripeError("test".to_string()), "Stripe error: test"),
            (BillingError::SubscriptionNotFound, "Subscription not found"),
            (BillingError::InvalidOperation("test".to_string()), "Invalid operation: test"),
        ];

        for (error, expected_msg) in errors {
            assert_eq!(error.to_string(), expected_msg);
        }
    }

    #[tokio::test]
    async fn test_billing_service_storage_limit_conversion() {
        let config = test_product_config();
        let service = BillingService::new(config, true);

        let result = service
            .check_limit("free", UsageMetric::StorageBytes, 50 * 1024 * 1024 - 1)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        let result_exceeded = service
            .check_limit("free", UsageMetric::StorageBytes, 51 * 1024 * 1024)
            .await;
        assert!(result_exceeded.is_ok());
        assert!(!result_exceeded.unwrap());
    }

    #[test]
    fn test_plan_price_serialization() {
        let free = PlanPrice::Free;
        let free_json = serde_json::to_string(&free).unwrap();
        assert!(free_json.contains("Free") || free_json == "\"Free\"");

        let fixed = PlanPrice::Fixed {
            amount: 1000,
            currency: "usd".to_string(),
            period: BillingPeriod::Monthly,
        };
        let fixed_json = serde_json::to_string(&fixed).unwrap();
        assert!(fixed_json.contains("1000"));
        assert!(fixed_json.contains("usd"));

        let custom = PlanPrice::Custom;
        let custom_json = serde_json::to_string(&custom).unwrap();
        assert!(custom_json.contains("Custom") || custom_json == "\"Custom\"");
    }

    #[test]
    fn test_plan_limits_all_metrics() {
        let config = test_product_config();
        let personal = config.plans.get("personal").unwrap();

        assert!(personal.limits.messages_per_day.value().is_some());
        assert!(personal.limits.storage_mb.value().is_some());
        assert!(personal.limits.bots.value().is_some());
        assert!(personal.limits.users.value().is_some());
        assert!(personal.limits.api_calls_per_day.value().is_some());
        assert!(personal.limits.kb_documents.value().is_some());
        assert!(personal.limits.apps.value().is_some());
    }

    #[test]
    fn test_subscription_struct() {
        let now = chrono::Utc::now();
        let subscription = Subscription {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            plan_id: "business".to_string(),
            status: SubscriptionStatus::Active,
            current_period_start: now,
            current_period_end: now + chrono::Duration::days(30),
            stripe_subscription_id: Some("sub_123".to_string()),
            stripe_customer_id: Some("cus_123".to_string()),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(subscription.plan_id, "business");
        assert_eq!(subscription.status, SubscriptionStatus::Active);
        assert!(subscription.stripe_subscription_id.is_some());
    }

    #[test]
    fn test_usage_record_struct() {
        let now = chrono::Utc::now();
        let record = UsageRecord {
            organization_id: Uuid::new_v4(),
            metric: UsageMetric::Messages,
            value: 100,
            period_start: now,
            period_end: now + chrono::Duration::days(1),
        };

        assert_eq!(record.metric, UsageMetric::Messages);
        assert_eq!(record.value, 100);
    }
}
