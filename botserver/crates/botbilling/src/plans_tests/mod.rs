//! Offline unit tests for the plan manager (#2075).
//!
//! `PlanManager` is the whole billing decision surface that does not touch a
//! database: limit checks, upgrade/downgrade ordering, proration and plan
//! comparison. A wrong answer here charges a customer the wrong amount, so
//! every branch is pinned.

use super::*;
use crate::{
    BillingError, BillingPeriod, LimitValue, PlanConfig, PlanLimits, PlanPrice, UsageMetric,
};
use std::collections::HashMap;
use uuid::Uuid;

fn limits(
    bots: LimitValue,
    users: LimitValue,
    storage_mb: LimitValue,
    apps: LimitValue,
) -> PlanLimits {
    PlanLimits {
        messages_per_day: LimitValue::Limited(100),
        storage_mb,
        bots,
        users,
        api_calls_per_day: LimitValue::Limited(1000),
        signups_per_day: None,
        kb_documents: LimitValue::Limited(10),
        apps,
        emails_per_day: LimitValue::Limited(50),
        builds_per_day: LimitValue::Limited(5),
        uploads_per_hour: LimitValue::Limited(20),
    }
}

fn plan(name: &str, price: PlanPrice, limits: PlanLimits, features: &[&str]) -> PlanConfig {
    PlanConfig {
        name: name.to_string(),
        description: None,
        price,
        limits,
        features: features.iter().map(|f| f.to_string()).collect(),
        stripe_price_id: None,
        trial_days: None,
    }
}

fn free_limits() -> PlanLimits {
    limits(
        LimitValue::Limited(1),
        LimitValue::Limited(2),
        LimitValue::Limited(100),
        LimitValue::Limited(1),
    )
}

fn shared_limits() -> PlanLimits {
    limits(
        LimitValue::Limited(5),
        LimitValue::Limited(20),
        LimitValue::Limited(5000),
        LimitValue::Limited(10),
    )
}

fn private_limits() -> PlanLimits {
    limits(
        LimitValue::Unlimited,
        LimitValue::Unlimited,
        LimitValue::Unlimited,
        LimitValue::Unlimited,
    )
}

fn monthly(amount: u64) -> PlanPrice {
    PlanPrice::Fixed {
        amount,
        currency: "USD".to_string(),
        period: BillingPeriod::Monthly,
    }
}

fn manager() -> PlanManager {
    let mut plans = HashMap::new();
    plans.insert(
        "free".to_string(),
        plan("Free", PlanPrice::Free, free_limits(), &["chat"]),
    );
    plans.insert(
        "shared".to_string(),
        plan("Shared", monthly(399), shared_limits(), &["chat", "drive"]),
    );
    plans.insert(
        "private-cloud".to_string(),
        plan(
            "Private",
            monthly(9999),
            private_limits(),
            &["chat", "drive", "sso"],
        ),
    );
    plans.insert(
        "enterprise".to_string(),
        plan("Enterprise", PlanPrice::Custom, private_limits(), &[]),
    );
    PlanManager::new(plans)
}

fn usage(bots: u64, users: u64, storage_bytes: u64, apps: u64) -> OrganizationUsage {
    OrganizationUsage {
        organization_id: Uuid::nil(),
        bots,
        users,
        storage_bytes,
        apps,
        kb_documents: 0,
        messages_today: 0,
        api_calls_today: 0,
    }
}

mod comparison;
mod limits;
mod upgrade_downgrade;
