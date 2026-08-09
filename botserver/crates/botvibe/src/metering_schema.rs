use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const METERING_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS metering_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    project_id UUID,
    env VARCHAR(16) NOT NULL DEFAULT 'production',
    meter VARCHAR(32) NOT NULL,
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_metering_usage_lookup
    ON metering_usage(org_id, meter, period_start);
CREATE INDEX IF NOT EXISTS idx_metering_usage_project
    ON metering_usage(project_id) WHERE project_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS metering_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    scope VARCHAR(16) NOT NULL DEFAULT 'free',
    meter VARCHAR(32) NOT NULL,
    hard_limit DOUBLE PRECISION,
    soft_limit DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT metering_limits_org_meter UNIQUE (org_id, scope, meter)
);
CREATE INDEX IF NOT EXISTS idx_metering_limits_scope ON metering_limits(org_id, scope);

CREATE TABLE IF NOT EXISTS metering_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    actor_user_id UUID NOT NULL,
    reason VARCHAR(255) NOT NULL,
    until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_metering_overrides_org ON metering_overrides(org_id, until);
";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterPlan {
    Free,
    Shared,
    PrivateCloud,
}

impl MeterPlan {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "shared" => Self::Shared,
            "private" | "private-cloud" | "privatecloud" | "custom" => Self::PrivateCloud,
            _ => Self::Free,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Shared => "shared",
            Self::PrivateCloud => "private-cloud",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterKind {
    VmHours,
    BuildMinutes,
    StorageGbHours,
    EgressGb,
    SnapshotCount,
    DomainBindings,
}

impl MeterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VmHours => "vm_hours",
            Self::BuildMinutes => "build_minutes",
            Self::StorageGbHours => "storage_gb_hours",
            Self::EgressGb => "egress_gb",
            Self::SnapshotCount => "snapshot_count",
            Self::DomainBindings => "domain_bindings",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageRow {
    pub meter: String,
    pub amount: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub project_id: Uuid,
    pub org_id: Uuid,
    pub plan: String,
    pub window_seconds: i64,
    pub rows: Vec<UsageRow>,
}

#[derive(Debug, Clone, Serialize, diesel::QueryableByName)]
pub struct LimitRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub scope: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub meter: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    pub hard_limit: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    pub soft_limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnforceResult {
    pub allowed: bool,
    pub metered: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_plan_parses_known_and_unknown() {
        assert_eq!(MeterPlan::parse("free"), MeterPlan::Free);
        assert_eq!(MeterPlan::parse("shared"), MeterPlan::Shared);
        assert_eq!(MeterPlan::parse("private-cloud"), MeterPlan::PrivateCloud);
        assert_eq!(MeterPlan::parse("PrivateCloud"), MeterPlan::PrivateCloud);
        assert_eq!(MeterPlan::parse("SHARED"), MeterPlan::Shared);
        assert_eq!(MeterPlan::parse("nonsense"), MeterPlan::Free);
        assert_eq!(MeterPlan::parse(""), MeterPlan::Free);
    }

    #[test]
    fn meter_plan_as_str_round_trip() {
        assert_eq!(MeterPlan::Free.as_str(), "free");
        assert_eq!(MeterPlan::Shared.as_str(), "shared");
        assert_eq!(MeterPlan::PrivateCloud.as_str(), "private-cloud");
        assert_eq!(MeterPlan::parse(MeterPlan::PrivateCloud.as_str()), MeterPlan::PrivateCloud);
    }

    #[test]
    fn meter_kind_as_str() {
        assert_eq!(MeterKind::VmHours.as_str(), "vm_hours");
        assert_eq!(MeterKind::BuildMinutes.as_str(), "build_minutes");
        assert_eq!(MeterKind::StorageGbHours.as_str(), "storage_gb_hours");
        assert_eq!(MeterKind::EgressGb.as_str(), "egress_gb");
        assert_eq!(MeterKind::SnapshotCount.as_str(), "snapshot_count");
        assert_eq!(MeterKind::DomainBindings.as_str(), "domain_bindings");
    }

    #[test]
    fn schema_contains_core_tables() {
        assert!(METERING_SCHEMA.contains("metering_usage"));
        assert!(METERING_SCHEMA.contains("metering_limits"));
        assert!(METERING_SCHEMA.contains("metering_overrides"));
    }
}
