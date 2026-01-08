use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::billing::{BillingError, PlanConfig, Subscription, SubscriptionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionLifecycleEvent {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub organization_id: Uuid,
    pub event_type: LifecycleEventType,
    pub from_plan: Option<String>,
    pub to_plan: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub processed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleEventType {
    Created,
    Activated,
    Upgraded,
    Downgraded,
    Renewed,
    Paused,
    Resumed,
    CancellationRequested,
    Cancelled,
    Expired,
    PaymentFailed,
    PaymentRecovered,
    TrialStarted,
    TrialEnded,
    GracePeriodStarted,
    GracePeriodEnded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub organization_id: Uuid,
    pub plan_id: String,
    pub payment_method_id: Option<String>,
    pub trial_days: Option<u32>,
    pub coupon_code: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDowngradeRequest {
    pub subscription_id: Uuid,
    pub new_plan_id: String,
    pub prorate: bool,
    pub immediate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelSubscriptionRequest {
    pub subscription_id: Uuid,
    pub reason: Option<String>,
    pub feedback: Option<String>,
    pub cancel_immediately: bool,
    pub offer_retention: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionOffer {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub offer_type: RetentionOfferType,
    pub discount_percent: Option<u32>,
    pub free_months: Option<u32>,
    pub expires_at: DateTime<Utc>,
    pub accepted: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetentionOfferType {
    Discount,
    FreeMonth,
    PlanDowngrade,
    FeatureUnlock,
    PersonalSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChange {
    pub subscription_id: Uuid,
    pub change_type: ChangeType,
    pub effective_date: DateTime<Utc>,
    pub from_plan: String,
    pub to_plan: String,
    pub proration_amount: Option<i64>,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Upgrade,
    Downgrade,
    PlanChange,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Pending,
    Scheduled,
    Applied,
    Cancelled,
    Failed,
}

pub struct SubscriptionLifecycleService {
    subscriptions: Arc<RwLock<HashMap<Uuid, Subscription>>>,
    events: Arc<RwLock<Vec<SubscriptionLifecycleEvent>>>,
    pending_changes: Arc<RwLock<HashMap<Uuid, SubscriptionChange>>>,
    retention_offers: Arc<RwLock<HashMap<Uuid, RetentionOffer>>>,
    plans: Arc<RwLock<HashMap<String, PlanConfig>>>,
}

impl SubscriptionLifecycleService {
    pub fn new(plans: HashMap<String, PlanConfig>) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
            retention_offers: Arc::new(RwLock::new(HashMap::new())),
            plans: Arc::new(RwLock::new(plans)),
        }
    }

    pub async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> Result<Subscription, LifecycleError> {
        let plans = self.plans.read().await;
        let plan = plans
            .get(&request.plan_id)
            .ok_or_else(|| LifecycleError::PlanNotFound(request.plan_id.clone()))?;

        let now = Utc::now();
        let trial_days = request.trial_days.or(plan.trial_days).unwrap_or(0);

        let (status, period_start, period_end) = if trial_days > 0 {
            (
                SubscriptionStatus::Trialing,
                now,
                now + Duration::days(trial_days as i64),
            )
        } else {
            (
                SubscriptionStatus::Active,
                now,
                now + Duration::days(30),
            )
        };

        let subscription = Subscription {
            id: Uuid::new_v4(),
            organization_id: request.organization_id,
            plan_id: request.plan_id.clone(),
            status,
            current_period_start: period_start,
            current_period_end: period_end,
            stripe_subscription_id: None,
            stripe_customer_id: None,
            created_at: now,
            updated_at: now,
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id, subscription.clone());

        drop(subscriptions);

        let event_type = if trial_days > 0 {
            LifecycleEventType::TrialStarted
        } else {
            LifecycleEventType::Created
        };

        self.record_event(
            subscription.id,
            request.organization_id,
            event_type,
            None,
            Some(request.plan_id),
            request.metadata.unwrap_or_default(),
        )
        .await;

        Ok(subscription)
    }

    pub async fn get_subscription(&self, subscription_id: Uuid) -> Option<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(&subscription_id).cloned()
    }

    pub async fn get_subscription_by_org(&self, organization_id: Uuid) -> Option<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions
            .values()
            .find(|s| s.organization_id == organization_id && s.status != SubscriptionStatus::Canceled)
            .cloned()
    }

    pub async fn upgrade_subscription(
        &self,
        request: UpgradeDowngradeRequest,
    ) -> Result<SubscriptionChange, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&request.subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        let plans = self.plans.read().await;
        let current_plan = plans
            .get(&subscription.plan_id)
            .ok_or_else(|| LifecycleError::PlanNotFound(subscription.plan_id.clone()))?;
        let new_plan = plans
            .get(&request.new_plan_id)
            .ok_or_else(|| LifecycleError::PlanNotFound(request.new_plan_id.clone()))?;

        let is_upgrade = self.is_upgrade(current_plan, new_plan);
        if !is_upgrade {
            return Err(LifecycleError::InvalidOperation(
                "Use downgrade for moving to a lower plan".to_string(),
            ));
        }

        let proration_amount = if request.prorate {
            self.calculate_proration(subscription, current_plan, new_plan)
        } else {
            None
        };

        let change = SubscriptionChange {
            subscription_id: subscription.id,
            change_type: ChangeType::Upgrade,
            effective_date: if request.immediate {
                Utc::now()
            } else {
                subscription.current_period_end
            },
            from_plan: subscription.plan_id.clone(),
            to_plan: request.new_plan_id.clone(),
            proration_amount,
            status: if request.immediate {
                ChangeStatus::Applied
            } else {
                ChangeStatus::Scheduled
            },
        };

        if request.immediate {
            let old_plan = subscription.plan_id.clone();
            subscription.plan_id = request.new_plan_id.clone();
            subscription.updated_at = Utc::now();

            drop(subscriptions);
            drop(plans);

            self.record_event(
                change.subscription_id,
                subscription.organization_id,
                LifecycleEventType::Upgraded,
                Some(old_plan),
                Some(request.new_plan_id),
                HashMap::new(),
            )
            .await;
        } else {
            drop(subscriptions);
            drop(plans);

            let mut pending = self.pending_changes.write().await;
            pending.insert(change.subscription_id, change.clone());
        }

        Ok(change)
    }

    pub async fn downgrade_subscription(
        &self,
        request: UpgradeDowngradeRequest,
    ) -> Result<SubscriptionChange, LifecycleError> {
        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions
            .get(&request.subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        let plans = self.plans.read().await;
        let current_plan = plans
            .get(&subscription.plan_id)
            .ok_or_else(|| LifecycleError::PlanNotFound(subscription.plan_id.clone()))?;
        let new_plan = plans
            .get(&request.new_plan_id)
            .ok_or_else(|| LifecycleError::PlanNotFound(request.new_plan_id.clone()))?;

        let is_upgrade = self.is_upgrade(current_plan, new_plan);
        if is_upgrade {
            return Err(LifecycleError::InvalidOperation(
                "Use upgrade for moving to a higher plan".to_string(),
            ));
        }

        let change = SubscriptionChange {
            subscription_id: subscription.id,
            change_type: ChangeType::Downgrade,
            effective_date: subscription.current_period_end,
            from_plan: subscription.plan_id.clone(),
            to_plan: request.new_plan_id.clone(),
            proration_amount: None,
            status: ChangeStatus::Scheduled,
        };

        let org_id = subscription.organization_id;
        drop(subscriptions);
        drop(plans);

        let mut pending = self.pending_changes.write().await;
        pending.insert(change.subscription_id, change.clone());

        self.record_event(
            change.subscription_id,
            org_id,
            LifecycleEventType::Downgraded,
            Some(change.from_plan.clone()),
            Some(change.to_plan.clone()),
            HashMap::from([("scheduled".to_string(), "true".to_string())]),
        )
        .await;

        Ok(change)
    }

    pub async fn cancel_subscription(
        &self,
        request: CancelSubscriptionRequest,
    ) -> Result<CancellationResult, LifecycleError> {
        if request.offer_retention {
            let offer = self.create_retention_offer(request.subscription_id).await?;
            return Ok(CancellationResult::RetentionOffered(offer));
        }

        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&request.subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        let org_id = subscription.organization_id;

        if request.cancel_immediately {
            subscription.status = SubscriptionStatus::Canceled;
            subscription.updated_at = Utc::now();

            drop(subscriptions);

            self.record_event(
                request.subscription_id,
                org_id,
                LifecycleEventType::Cancelled,
                Some(subscription.plan_id.clone()),
                None,
                HashMap::from([
                    ("immediate".to_string(), "true".to_string()),
                    ("reason".to_string(), request.reason.unwrap_or_default()),
                ]),
            )
            .await;

            Ok(CancellationResult::CancelledImmediately)
        } else {
            drop(subscriptions);

            self.record_event(
                request.subscription_id,
                org_id,
                LifecycleEventType::CancellationRequested,
                None,
                None,
                HashMap::from([
                    ("reason".to_string(), request.reason.unwrap_or_default()),
                    ("feedback".to_string(), request.feedback.unwrap_or_default()),
                ]),
            )
            .await;

            let subscriptions = self.subscriptions.read().await;
            let subscription = subscriptions.get(&request.subscription_id);
            let end_date = subscription.map(|s| s.current_period_end).unwrap_or_else(Utc::now);

            Ok(CancellationResult::ScheduledForEndOfPeriod { end_date })
        }
    }

    pub async fn reactivate_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<Subscription, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        if subscription.status != SubscriptionStatus::Canceled {
            return Err(LifecycleError::InvalidOperation(
                "Subscription is not cancelled".to_string(),
            ));
        }

        let now = Utc::now();
        subscription.status = SubscriptionStatus::Active;
        subscription.current_period_start = now;
        subscription.current_period_end = now + Duration::days(30);
        subscription.updated_at = now;

        let result = subscription.clone();
        let org_id = subscription.organization_id;
        let plan_id = subscription.plan_id.clone();

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::Resumed,
            None,
            Some(plan_id),
            HashMap::new(),
        )
        .await;

        Ok(result)
    }

    pub async fn pause_subscription(&self, subscription_id: Uuid) -> Result<Subscription, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        if subscription.status != SubscriptionStatus::Active {
            return Err(LifecycleError::InvalidOperation(
                "Only active subscriptions can be paused".to_string(),
            ));
        }

        subscription.status = SubscriptionStatus::Paused;
        subscription.updated_at = Utc::now();

        let result = subscription.clone();
        let org_id = subscription.organization_id;

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::Paused,
            None,
            None,
            HashMap::new(),
        )
        .await;

        Ok(result)
    }

    pub async fn resume_subscription(&self, subscription_id: Uuid) -> Result<Subscription, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        if subscription.status != SubscriptionStatus::Paused {
            return Err(LifecycleError::InvalidOperation(
                "Only paused subscriptions can be resumed".to_string(),
            ));
        }

        subscription.status = SubscriptionStatus::Active;
        subscription.updated_at = Utc::now();

        let result = subscription.clone();
        let org_id = subscription.organization_id;

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::Resumed,
            None,
            None,
            HashMap::new(),
        )
        .await;

        Ok(result)
    }

    pub async fn renew_subscription(&self, subscription_id: Uuid) -> Result<Subscription, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        let now = Utc::now();
        subscription.current_period_start = now;
        subscription.current_period_end = now + Duration::days(30);
        subscription.status = SubscriptionStatus::Active;
        subscription.updated_at = now;

        let result = subscription.clone();
        let org_id = subscription.organization_id;

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::Renewed,
            None,
            None,
            HashMap::new(),
        )
        .await;

        Ok(result)
    }

    pub async fn handle_payment_failure(&self, subscription_id: Uuid) -> Result<(), LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        subscription.status = SubscriptionStatus::PastDue;
        subscription.updated_at = Utc::now();

        let org_id = subscription.organization_id;

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::PaymentFailed,
            None,
            None,
            HashMap::new(),
        )
        .await;

        Ok(())
    }

    pub async fn handle_payment_recovery(&self, subscription_id: Uuid) -> Result<Subscription, LifecycleError> {
        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions
            .get_mut(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        subscription.status = SubscriptionStatus::Active;
        subscription.updated_at = Utc::now();

        let result = subscription.clone();
        let org_id = subscription.organization_id;

        drop(subscriptions);

        self.record_event(
            subscription_id,
            org_id,
            LifecycleEventType::PaymentRecovered,
            None,
            None,
            HashMap::new(),
        )
        .await;

        Ok(result)
    }

    pub async fn accept_retention_offer(&self, offer_id: Uuid) -> Result<Subscription, LifecycleError> {
        let mut offers = self.retention_offers.write().await;
        let offer = offers
            .get_mut(&offer_id)
            .ok_or(LifecycleError::OfferNotFound)?;

        if offer.expires_at < Utc::now() {
            return Err(LifecycleError::OfferExpired);
        }

        if offer.accepted {
            return Err(LifecycleError::OfferAlreadyAccepted);
        }

        offer.accepted = true;
        let subscription_id = offer.subscription_id;

        drop(offers);

        let subscriptions = self.subscriptions.read().await;
        let subscription = subscriptions
            .get(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?
            .clone();

        Ok(subscription)
    }

    pub async fn get_events(&self, subscription_id: Uuid) -> Vec<SubscriptionLifecycleEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.subscription_id == subscription_id)
            .cloned()
            .collect()
    }

    pub async fn get_pending_change(&self, subscription_id: Uuid) -> Option<SubscriptionChange> {
        let pending = self.pending_changes.read().await;
        pending.get(&subscription_id).cloned()
    }

    pub async fn apply_pending_changes(&self) -> Vec<SubscriptionChange> {
        let now = Utc::now();
        let mut applied = Vec::new();

        let pending = self.pending_changes.read().await;
        let due_changes: Vec<SubscriptionChange> = pending
            .values()
            .filter(|c| c.effective_date <= now && c.status == ChangeStatus::Scheduled)
            .cloned()
            .collect();
        drop(pending);

        for change in due_changes {
            let mut subscriptions = self.subscriptions.write().await;
            if let Some(subscription) = subscriptions.get_mut(&change.subscription_id) {
                subscription.plan_id = change.to_plan.clone();
                subscription.updated_at = now;

                let mut pending = self.pending_changes.write().await;
                if let Some(pending_change) = pending.get_mut(&change.subscription_id) {
                    pending_change.status = ChangeStatus::Applied;
                }

                applied.push(change.clone());
            }
        }

        applied
    }

    async fn create_retention_offer(
        &self,
        subscription_id: Uuid,
    ) -> Result<RetentionOffer, LifecycleError> {
        let subscriptions = self.subscriptions.read().await;
        let _subscription = subscriptions
            .get(&subscription_id)
            .ok_or(LifecycleError::SubscriptionNotFound)?;

        let offer = RetentionOffer {
            id: Uuid::new_v4(),
            subscription_id,
            offer_type: RetentionOfferType::Discount,
            discount_percent: Some(20),
            free_months: None,
            expires_at: Utc::now() + Duration::days(7),
            accepted: false,
        };

        drop(subscriptions);

        let mut offers = self.retention_offers.write().await;
        offers.insert(offer.id, offer.clone());

        Ok(offer)
    }

    async fn record_event(
        &self,
        subscription_id: Uuid,
        organization_id: Uuid,
        event_type: LifecycleEventType,
        from_plan: Option<String>,
        to_plan: Option<String>,
        metadata: HashMap<String, String>,
    ) {
        let event = SubscriptionLifecycleEvent {
            id: Uuid::new_v4(),
            subscription_id,
            organization_id,
            event_type,
            from_plan,
            to_plan,
            timestamp: Utc::now(),
            metadata,
            processed: false,
        };

        let mut events = self.events.write().await;
        events.push(event);
    }

    fn is_upgrade(&self, current: &PlanConfig, new: &PlanConfig) -> bool {
        let current_value = self.plan_value(current);
        let new_value = self.plan_value(new);
        new_value > current_value
    }

    fn plan_value(&self, plan: &PlanConfig) -> u64 {
        match &plan.price {
            crate::billing::PlanPrice::Free => 0,
            crate::billing::PlanPrice::Fixed { amount, .. } => *amount,
            crate::billing::PlanPrice::Custom => u64::MAX,
        }
    }

    fn calculate_proration(
        &self,
        subscription: &Subscription,
        _current_plan: &PlanConfig,
        _new_plan: &PlanConfig,
    ) -> Option<i64> {
        let now = Utc::now();
        let period_total = (subscription.current_period_end - subscription.current_period_start)
            .num_days() as f64;
        let days_remaining = (subscription.current_period_end - now).num_days() as f64;

        if period_total > 0.0 {
            let ratio = days_remaining / period_total;
            Some((ratio * 100.0) as i64)
        } else {
            None
        }
    }
}

impl Default for SubscriptionLifecycleService {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CancellationResult {
    CancelledImmediately,
    ScheduledForEndOfPeriod { end_date: DateTime<Utc> },
    RetentionOffered(RetentionOffer),
}

#[derive(Debug, Clone)]
pub enum LifecycleError {
    SubscriptionNotFound,
    PlanNotFound(String),
    InvalidOperation(String),
    OfferNotFound,
    OfferExpired,
    OfferAlreadyAccepted,
    BillingError(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubscriptionNotFound => write!(f, "Subscription not found"),
            Self::PlanNotFound(id) => write!(f, "Plan not found: {id}"),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
            Self::OfferNotFound => write!(f, "Retention offer not found"),
            Self::OfferExpired => write!(f, "Retention offer has expired"),
            Self::OfferAlreadyAccepted => write!(f, "Retention offer already accepted"),
            Self::BillingError(msg) => write!(f, "Billing error: {msg}"),
        }
    }
}

impl std::error::Error for LifecycleError {}

impl From<BillingError> for LifecycleError {
    fn from(err: BillingError) -> Self {
        Self::BillingError(err.to_string())
    }
}

pub async fn process_expiring_subscriptions(
    service: Arc<SubscriptionLifecycleService>,
) -> Vec<Uuid> {
    let now = Utc::now();
    let subscriptions = service.subscriptions.read().await;

    let expiring: Vec<Uuid> = subscriptions
        .values()
        .filter(|s| {
            s.status == SubscriptionStatus::Active
                && s.current_period_end <= now + Duration::days(3)
                && s.current_period_end > now
        })
        .map(|s| s.id)
        .collect();

    expiring
}

pub async fn process_expired_trials(service: Arc<SubscriptionLifecycleService>) -> Vec<Uuid> {
    let now = Utc::now();
    let mut subscriptions = service.subscriptions.write().await;

    let mut expired = Vec::new();

    for subscription in subscriptions.values_mut() {
        if subscription.status == SubscriptionStatus::Trialing
            && subscription.current_period_end <= now
        {
            subscription.status = SubscriptionStatus::Active;
            subscription.current_period_start = now;
            subscription.current_period_end = now + Duration::days(30);
            subscription.updated_at = now;
            expired.push(subscription.id);
        }
    }

    expired
}
