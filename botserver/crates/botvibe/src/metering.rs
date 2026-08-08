use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::metering_schema::{
    EnforceResult, LimitRow, MeterKind, MeterPlan, UsageRow, UsageSummary, METERING_SCHEMA,
};
use crate::types::DbPool;

pub use crate::metering_schema::{
    EnforceResult, LimitRow, MeterKind, MeterPlan, UsageRow, UsageSummary,
};

pub type VMeteringRef = std::sync::Arc<VMetering>;

type Conn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

const WINDOW_ENV: &str = "VIBE_METER_WINDOW_SECONDS";
const DEFAULT_WINDOW_SECONDS: i64 = 30 * 24 * 3600;

#[derive(Clone)]
pub struct VMetering {
    pool: DbPool,
}

impl VMetering {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(&self) -> Result<Conn, String> {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(METERING_SCHEMA)
            .execute(&mut conn)
            .map_err(|e| format!("metering schema: {e}"))?;
        Ok(())
    }

    fn project_scope(&self, conn: &mut Conn, project_id: Uuid) -> Result<(Uuid, Uuid), String> {
        let row: serde_json::Value = diesel::sql_query(
            "SELECT org_id, branch_id FROM vibe_projects WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .get_result::<serde_json::Value>(conn)
        .map_err(|e| format!("project lookup: {e}"))?;
        let org = row
            .get("org_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);
        let branch = row
            .get("branch_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);
        Ok((org, branch))
    }

    fn plan_for(&self, conn: &mut Conn, branch_id: Uuid) -> Result<MeterPlan, String> {
        if branch_id.is_nil() {
            return Ok(MeterPlan::Free);
        }
        let plan: Option<String> = diesel::sql_query(
            "SELECT plan_name FROM billing_recurring \
             WHERE branch_id = $1 AND status = 'active' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<String>(conn)
        .optional()
        .map_err(|e| format!("plan lookup: {e}"))?;
        Ok(plan.as_deref().map(MeterPlan::parse).unwrap_or(MeterPlan::Free))
    }

    fn window_seconds() -> i64 {
        std::env::var(WINDOW_ENV)
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(DEFAULT_WINDOW_SECONDS)
    }

    pub fn add_usage(
        &self,
        org_id: Uuid,
        project_id: Option<Uuid>,
        env: &str,
        meter: MeterKind,
        amount: f64,
    ) -> Result<(), String> {
        let mut conn = self.conn()?;
        let window = Self::window_seconds();
        diesel::sql_query(
            "INSERT INTO metering_usage \
             (org_id, project_id, env, meter, amount, period_start, period_end) \
             VALUES ($1, $2, $3, $4, $5, NOW() - make_interval(secs => $6), NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(env)
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .bind::<diesel::sql_types::Float8, _>(amount)
        .bind::<diesel::sql_types::BigInt, _>(window)
        .execute(&mut conn)
        .map_err(|e| format!("meter add: {e}"))?;
        Ok(())
    }

    pub fn usage_by_project(&self, project_id: Uuid) -> Result<Vec<UsageRow>, String> {
        let mut conn = self.conn()?;
        let window = Self::window_seconds();
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            meter: String,
            #[diesel(sql_type = diesel::sql_types::Float8)]
            amount: f64,
        }
        let rows = diesel::sql_query(
            "SELECT meter, SUM(amount)::float8 AS amount FROM metering_usage \
             WHERE project_id = $1 AND period_start >= NOW() - make_interval(secs => $2) \
             GROUP BY meter ORDER BY meter",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::BigInt, _>(window)
        .load::<Row>(&mut conn)
        .map_err(|e| format!("usage query: {e}"))?;
        Ok(rows
            .into_iter()
            .map(|r| UsageRow {
                meter: r.meter,
                amount: r.amount,
                period_start: Utc::now() - chrono::Duration::seconds(window),
                period_end: Utc::now(),
            })
            .collect())
    }

    pub fn summary(&self, project_id: Uuid) -> Result<UsageSummary, String> {
        let mut conn = self.conn()?;
        let (org, branch) = self.project_scope(&mut conn, project_id)?;
        let plan = self.plan_for(&mut conn, branch)?;
        Ok(UsageSummary {
            project_id,
            org_id: org,
            plan: plan.as_str().to_string(),
            window_seconds: Self::window_seconds(),
            rows: self.usage_by_project(project_id)?,
        })
    }

    pub fn limits(&self, org_id: Uuid) -> Result<Vec<LimitRow>, String> {
        let mut conn = self.conn()?;
        let rows = diesel::sql_query(
            "SELECT scope, meter, hard_limit, soft_limit FROM metering_limits \
             WHERE org_id = $1 ORDER BY scope, meter",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .load::<LimitRow>(&mut conn)
        .map_err(|e| format!("limits query: {e}"))?;
        Ok(rows)
    }

    pub fn set_limit(
        &self,
        org_id: Uuid,
        scope: MeterPlan,
        meter: MeterKind,
        hard: Option<f64>,
        soft: Option<f64>,
    ) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "INSERT INTO metering_limits (org_id, scope, meter, hard_limit, soft_limit) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (org_id, scope, meter) DO UPDATE \
             SET hard_limit = EXCLUDED.hard_limit, soft_limit = EXCLUDED.soft_limit, \
                 updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(scope.as_str())
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Float8>, _>(hard)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Float8>, _>(soft)
        .execute(&mut conn)
        .map_err(|e| format!("set limit: {e}"))?;
        Ok(())
    }

    pub fn enforce_cap(&self, org_id: Uuid, branch_id: Uuid, meter: MeterKind) -> Result<EnforceResult, String> {
        let mut conn = self.conn()?;
        let plan = self.plan_for(&mut conn, branch_id)?;
        let cap: Option<f64> = diesel::sql_query(
            "SELECT hard_limit FROM metering_limits \
             WHERE org_id = $1 AND scope = $2 AND meter = $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(plan.as_str())
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .get_result::<f64>(&mut conn)
        .optional()
        .map_err(|e| format!("cap lookup: {e}"))?;
        let cap = cap.unwrap_or(0.0);
        if cap <= 0.0 {
            return Ok(EnforceResult { allowed: true, metered: meter.as_str().to_string() });
        }
        let used: f64 = diesel::sql_query(
            "SELECT COALESCE(SUM(amount), 0) FROM metering_usage \
             WHERE org_id = $1 AND meter = $2 \
             AND period_start >= NOW() - make_interval(secs => $3)",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .bind::<diesel::sql_types::BigInt, _>(Self::window_seconds())
        .get_result::<f64>(&mut conn)
        .map_err(|e| format!("usage sum: {e}"))?;
        if used >= cap {
            return Err(format!(
                "metering '{}' cap reached: {used:.2} of {cap:.2} ({} plan) — upgrade to continue",
                meter.as_str(),
                plan.as_str()
            ));
        }
        Ok(EnforceResult { allowed: true, metered: meter.as_str().to_string() })
    }

    pub fn record_override(
        &self,
        org_id: Uuid,
        actor_user_id: Uuid,
        reason: &str,
        until: DateTime<Utc>,
    ) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "INSERT INTO metering_overrides (org_id, actor_user_id, reason, until) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Uuid, _>(actor_user_id)
        .bind::<diesel::sql_types::Text, _>(reason)
        .bind::<diesel::sql_types::Timestamptz, _>(until)
        .execute(&mut conn)
        .map_err(|e| format!("override: {e}"))?;
        Ok(())
    }

    pub fn add_for_project(
        &self,
        project_id: Uuid,
        env: &str,
        meter: MeterKind,
        amount: f64,
    ) -> Result<(), String> {
        let mut conn = self.conn()?;
        let (org, _branch) = self.project_scope(&mut conn, project_id)?;
        drop(conn);
        self.add_usage(org, Some(project_id), env, meter, amount)
    }

    pub fn accrue_vm_hours(
        &self,
        project_id: Uuid,
        env: &str,
        vm_created_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let elapsed = (Utc::now() - vm_created_at).num_seconds().max(0) as f64 / 3600.0;
        let window = Self::window_seconds() as f64 / 3600.0;
        let hours = elapsed.min(window);
        if hours <= 0.0 {
            return Ok(());
        }
        self.add_for_project(project_id, env, MeterKind::VmHours, hours)
    }

    pub fn enforce_for_project(
        &self,
        project_id: Uuid,
        meter: MeterKind,
    ) -> Result<EnforceResult, String> {
        let mut conn = self.conn()?;
        let (org, branch) = self.project_scope(&mut conn, project_id)?;
        drop(conn);
        self.enforce_cap(org, branch, meter)
    }

    pub fn plan_of_project(&self, project_id: Uuid) -> Result<MeterPlan, String> {
        let mut conn = self.conn()?;
        let (_org, branch) = self.project_scope(&mut conn, project_id)?;
        self.plan_for(&mut conn, branch)
    }

    pub fn project_count(&self, org_id: Uuid, project_type: &str) -> Result<i64, String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "SELECT COUNT(*) FROM vibe_projects \
             WHERE org_id = $1 AND project_type = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(project_type)
        .get_result::<i64>(&mut conn)
        .map_err(|e| format!("project count: {e}"))
    }

    pub fn enforce_project_creation(
        &self,
        org_id: Uuid,
        branch_id: Uuid,
        project_type: &str,
    ) -> Result<(), String> {
        if org_id.is_nil() {
            return Ok(());
        }
        let mut conn = self.conn()?;
        let plan = self.plan_for(&mut conn, branch_id)?;
        match plan {
            MeterPlan::PrivateCloud => Ok(()),
            MeterPlan::Free => {
                let is_custom = project_type.to_ascii_lowercase() == "custom";
                if is_custom {
                    return Err("custom projects require the private-cloud plan".to_string());
                }
                let count = self.project_count(org_id, project_type)?;
                if count >= 1 {
                    return Err(format!(
                        "free plan allows at most 1 '{project_type}' project (currently {count})"
                    ));
                }
                Ok(())
            }
            MeterPlan::Shared => Ok(()),
        }
    }
}
