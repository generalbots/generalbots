use crate::billing::{BillingError, LimitValue, PlanConfig, PlanPrice, UsageMetric};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PlanManager {
    plans: HashMap<String, PlanConfig>,
}

impl PlanManager {
    pub fn new(plans: HashMap<String, PlanConfig>) -> Self {
        Self { plans }
    }

    pub fn get_plan(&self, plan_id: &str) -> Option<&PlanConfig> {
        self.plans.get(plan_id)
    }

    pub fn list_plans(&self) -> Vec<(&String, &PlanConfig)> {
        self.plans.iter().collect()
    }

    pub fn list_public_plans(&self) -> Vec<(&String, &PlanConfig)> {
        self.plans
            .iter()
            .filter(|(id, _)| *id != "enterprise")
            .collect()
    }

    pub fn get_limit(&self, plan_id: &str, metric: UsageMetric) -> Option<LimitValue> {
        self.plans.get(plan_id).map(|plan| match metric {
            UsageMetric::Messages => plan.limits.messages_per_day,
            UsageMetric::StorageBytes => {
                match plan.limits.storage_mb {
                    LimitValue::Limited(mb) => LimitValue::Limited(mb * 1024 * 1024),
                    LimitValue::Unlimited => LimitValue::Unlimited,
                }
            }
            UsageMetric::ApiCalls => plan.limits.api_calls_per_day,
            UsageMetric::Bots => plan.limits.bots,
            UsageMetric::Users => plan.limits.users,
            UsageMetric::KbDocuments => plan.limits.kb_documents,
            UsageMetric::Apps => plan.limits.apps,
        })
    }

    pub fn check_limit(
        &self,
        plan_id: &str,
        metric: UsageMetric,
        current_usage: u64,
    ) -> Result<LimitCheckResult, BillingError> {
        let limit = self
            .get_limit(plan_id, metric)
            .ok_or_else(|| BillingError::PlanNotFound(plan_id.to_string()))?;

        match limit {
            LimitValue::Unlimited => Ok(LimitCheckResult::Allowed {
                remaining: None,
                percentage_used: 0.0,
            }),
            LimitValue::Limited(max) => {
                if current_usage >= max {
                    Ok(LimitCheckResult::Exceeded {
                        limit: max,
                        current: current_usage,
                    })
                } else {
                    let remaining = max - current_usage;
                    let percentage = (current_usage as f64 / max as f64) * 100.0;
                    Ok(LimitCheckResult::Allowed {
                        remaining: Some(remaining),
                        percentage_used: percentage,
                    })
                }
            }
        }
    }

    pub fn can_upgrade(&self, from_plan: &str, to_plan: &str) -> bool {
        let plan_order = ["free", "personal", "business", "enterprise"];

        let from_idx = plan_order.iter().position(|p| *p == from_plan);
        let to_idx = plan_order.iter().position(|p| *p == to_plan);

        match (from_idx, to_idx) {
            (Some(from), Some(to)) => to > from,
            _ => false,
        }
    }

    pub fn can_downgrade(&self, from_plan: &str, to_plan: &str) -> bool {
        let plan_order = ["free", "personal", "business", "enterprise"];

        let from_idx = plan_order.iter().position(|p| *p == from_plan);
        let to_idx = plan_order.iter().position(|p| *p == to_plan);

        match (from_idx, to_idx) {
            (Some(from), Some(to)) => to < from,
            _ => false,
        }
    }

    pub fn get_upgrade_options(&self, current_plan: &str) -> Vec<(&String, &PlanConfig)> {
        let plan_order = ["free", "personal", "business", "enterprise"];
        let current_idx = plan_order.iter().position(|p| *p == current_plan);

        match current_idx {
            Some(idx) => self
                .plans
                .iter()
                .filter(|(id, _)| {
                    plan_order
                        .iter()
                        .position(|p| *p == id.as_str())
                        .map(|plan_idx| plan_idx > idx)
                        .unwrap_or(false)
                })
                .collect(),
            None => vec![],
        }
    }

    pub fn calculate_proration(
        &self,
        from_plan: &str,
        to_plan: &str,
        days_remaining: u32,
        days_in_period: u32,
    ) -> Option<ProrationType> {
        let from_config = self.plans.get(from_plan)?;
        let to_config = self.plans.get(to_plan)?;

        let from_price = match &from_config.price {
            PlanPrice::Fixed { amount, .. } => *amount,
            PlanPrice::Free => 0,
            PlanPrice::Custom => return None,
        };

        let to_price = match &to_config.price {
            PlanPrice::Fixed { amount, .. } => *amount,
            PlanPrice::Free => 0,
            PlanPrice::Custom => return None,
        };

        let daily_from = from_price as f64 / days_in_period as f64;
        let daily_to = to_price as f64 / days_in_period as f64;

        let remaining_value_from = daily_from * days_remaining as f64;
        let remaining_cost_to = daily_to * days_remaining as f64;

        let difference = remaining_cost_to - remaining_value_from;

        if difference > 0.0 {
            Some(ProrationType::ChargeNow(difference.ceil() as u64))
        } else if difference < 0.0 {
            Some(ProrationType::Credit((-difference).ceil() as u64))
        } else {
            Some(ProrationType::NoChange)
        }
    }

    pub fn validate_downgrade(
        &self,
        to_plan: &str,
        current_usage: &OrganizationUsage,
    ) -> DowngradeValidation {
        let plan = match self.plans.get(to_plan) {
            Some(p) => p,
            None => {
                return DowngradeValidation {
                    allowed: false,
                    blockers: vec![DowngradeBlocker::PlanNotFound],
                }
            }
        };

        let mut blockers = Vec::new();

        if let LimitValue::Limited(max) = plan.limits.bots {
            if current_usage.bots > max {
                blockers.push(DowngradeBlocker::TooManyBots {
                    current: current_usage.bots,
                    limit: max,
                });
            }
        }

        if let LimitValue::Limited(max) = plan.limits.users {
            if current_usage.users > max {
                blockers.push(DowngradeBlocker::TooManyUsers {
                    current: current_usage.users,
                    limit: max,
                });
            }
        }

        if let LimitValue::Limited(max) = plan.limits.storage_mb {
            let max_bytes = max * 1024 * 1024;
            if current_usage.storage_bytes > max_bytes {
                blockers.push(DowngradeBlocker::TooMuchStorage {
                    current_mb: current_usage.storage_bytes / 1024 / 1024,
                    limit_mb: max,
                });
            }
        }

        if let LimitValue::Limited(max) = plan.limits.apps {
            if current_usage.apps > max {
                blockers.push(DowngradeBlocker::TooManyApps {
                    current: current_usage.apps,
                    limit: max,
                });
            }
        }

        DowngradeValidation {
            allowed: blockers.is_empty(),
            blockers,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LimitCheckResult {
    Allowed {
        remaining: Option<u64>,
        percentage_used: f64,
    },
    Exceeded {
        limit: u64,
        current: u64,
    },
}

impl LimitCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }

    pub fn is_warning_threshold(&self, threshold: f64) -> bool {
        match self {
            Self::Allowed { percentage_used, .. } => *percentage_used >= threshold,
            Self::Exceeded { .. } => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProrationType {
    ChargeNow(u64),
    Credit(u64),
    NoChange,
}

#[derive(Debug, Clone)]
pub struct OrganizationUsage {
    pub organization_id: Uuid,
    pub bots: u64,
    pub users: u64,
    pub storage_bytes: u64,
    pub apps: u64,
    pub kb_documents: u64,
    pub messages_today: u64,
    pub api_calls_today: u64,
}

impl Default for OrganizationUsage {
    fn default() -> Self {
        Self {
            organization_id: Uuid::nil(),
            bots: 0,
            users: 0,
            storage_bytes: 0,
            apps: 0,
            kb_documents: 0,
            messages_today: 0,
            api_calls_today: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DowngradeValidation {
    pub allowed: bool,
    pub blockers: Vec<DowngradeBlocker>,
}

#[derive(Debug, Clone)]
pub enum DowngradeBlocker {
    PlanNotFound,
    TooManyBots { current: u64, limit: u64 },
    TooManyUsers { current: u64, limit: u64 },
    TooMuchStorage { current_mb: u64, limit_mb: u64 },
    TooManyApps { current: u64, limit: u64 },
    TooManyKbDocuments { current: u64, limit: u64 },
}

impl std::fmt::Display for DowngradeBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlanNotFound => write!(f, "Target plan not found"),
            Self::TooManyBots { current, limit } => {
                write!(f, "Too many bots: {current} (limit: {limit}). Please delete some bots.")
            }
            Self::TooManyUsers { current, limit } => {
                write!(f, "Too many users: {current} (limit: {limit}). Please remove some users.")
            }
            Self::TooMuchStorage { current_mb, limit_mb } => {
                write!(f, "Too much storage: {current_mb}MB (limit: {limit_mb}MB). Please delete some files.")
            }
            Self::TooManyApps { current, limit } => {
                write!(f, "Too many apps: {current} (limit: {limit}). Please delete some apps.")
            }
            Self::TooManyKbDocuments { current, limit } => {
                write!(f, "Too many KB documents: {current} (limit: {limit}). Please delete some documents.")
            }
        }
    }
}

pub struct PlanComparison {
    pub from_plan: String,
    pub to_plan: String,
    pub limit_changes: Vec<LimitChange>,
    pub feature_changes: FeatureChanges,
    pub price_change: PriceChange,
}

#[derive(Debug, Clone)]
pub struct LimitChange {
    pub metric: UsageMetric,
    pub from: LimitValue,
    pub to: LimitValue,
    pub direction: ChangeDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDirection {
    Increase,
    Decrease,
    NoChange,
}

#[derive(Debug, Clone)]
pub struct FeatureChanges {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PriceChange {
    Increase { from: u64, to: u64 },
    Decrease { from: u64, to: u64 },
    NoChange,
    ToCustom,
    FromCustom,
}

impl PlanManager {
    pub fn compare_plans(&self, from_plan_id: &str, to_plan_id: &str) -> Option<PlanComparison> {
        let from = self.plans.get(from_plan_id)?;
        let to = self.plans.get(to_plan_id)?;

        let limit_changes = vec![
            Self::compare_limit(UsageMetric::Messages, from.limits.messages_per_day, to.limits.messages_per_day),
            Self::compare_limit(UsageMetric::StorageBytes, from.limits.storage_mb, to.limits.storage_mb),
            Self::compare_limit(UsageMetric::Bots, from.limits.bots, to.limits.bots),
            Self::compare_limit(UsageMetric::Users, from.limits.users, to.limits.users),
            Self::compare_limit(UsageMetric::ApiCalls, from.limits.api_calls_per_day, to.limits.api_calls_per_day),
            Self::compare_limit(UsageMetric::Apps, from.limits.apps, to.limits.apps),
        ];

        let added: Vec<String> = to
            .features
            .iter()
            .filter(|f| !from.features.contains(f))
            .cloned()
            .collect();

        let removed: Vec<String> = from
            .features
            .iter()
            .filter(|f| !to.features.contains(f))
            .cloned()
            .collect();

        let price_change = Self::compare_price(&from.price, &to.price);

        Some(PlanComparison {
            from_plan: from_plan_id.to_string(),
            to_plan: to_plan_id.to_string(),
            limit_changes,
            feature_changes: FeatureChanges { added, removed },
            price_change,
        })
    }

    fn compare_limit(metric: UsageMetric, from: LimitValue, to: LimitValue) -> LimitChange {
        let direction = match (from, to) {
            (LimitValue::Unlimited, LimitValue::Limited(_)) => ChangeDirection::Decrease,
            (LimitValue::Limited(_), LimitValue::Unlimited) => ChangeDirection::Increase,
            (LimitValue::Limited(f), LimitValue::Limited(t)) => {
                if t > f {
                    ChangeDirection::Increase
                } else if t < f {
                    ChangeDirection::Decrease
                } else {
                    ChangeDirection::NoChange
                }
            }
            (LimitValue::Unlimited, LimitValue::Unlimited) => ChangeDirection::NoChange,
        };

        LimitChange {
            metric,
            from,
            to,
            direction,
        }
    }

    fn compare_price(from: &PlanPrice, to: &PlanPrice) -> PriceChange {
        match (from, to) {
            (PlanPrice::Free, PlanPrice::Free) => PriceChange::NoChange,
            (PlanPrice::Free, PlanPrice::Fixed { amount, .. }) => {
                PriceChange::Increase { from: 0, to: *amount }
            }
            (PlanPrice::Fixed { amount, .. }, PlanPrice::Free) => {
                PriceChange::Decrease { from: *amount, to: 0 }
            }
            (PlanPrice::Fixed { amount: from_amt, .. }, PlanPrice::Fixed { amount: to_amt, .. }) => {
                if to_amt > from_amt {
                    PriceChange::Increase { from: *from_amt, to: *to_amt }
                } else if to_amt < from_amt {
                    PriceChange::Decrease { from: *from_amt, to: *to_amt }
                } else {
                    PriceChange::NoChange
                }
            }
            (_, PlanPrice::Custom) => PriceChange::ToCustom,
            (PlanPrice::Custom, _) => PriceChange::FromCustom,
        }
    }
}
