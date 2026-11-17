use crate::{LimitValue, UsageMetric};
use uuid::Uuid;

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
            Self::Allowed {
                percentage_used, ..
            } => *percentage_used >= threshold,
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
                write!(
                    f,
                    "Too many bots: {current} (limit: {limit}). Please delete some bots."
                )
            }
            Self::TooManyUsers { current, limit } => {
                write!(
                    f,
                    "Too many users: {current} (limit: {limit}). Please remove some users."
                )
            }
            Self::TooMuchStorage {
                current_mb,
                limit_mb,
            } => {
                write!(f, "Too much storage: {current_mb}MB (limit: {limit_mb}MB). Please delete some files.")
            }
            Self::TooManyApps { current, limit } => {
                write!(
                    f,
                    "Too many apps: {current} (limit: {limit}). Please delete some apps."
                )
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
