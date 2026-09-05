use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::ensure_schema_sql;
pub use crate::metering_schema::{
    EnforceResult, LimitRow, MeterKind, MeterPlan, UsageRow, UsageSummary, METERING_SCHEMA,
};
use crate::types::DbPool;

pub type VMeteringRef = std::sync::Arc<VMetering>;

type Conn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(diesel::QueryableByName)]
struct F64Cell {
    #[diesel(sql_type = diesel::sql_types::Float8)]
    value: f64,
}

#[derive(diesel::QueryableByName)]
struct I64Cell {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    value: i64,
}

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
        ensure_schema_sql(&mut conn, METERING_SCHEMA, "metering schema")?;
        Ok(())
    }

    fn project_scope(&self, conn: &mut Conn, project_id: Uuid) -> Result<(Uuid, Uuid), String> {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: Uuid,
        }
        let row = diesel::sql_query(
            "SELECT org_id, branch_id FROM vibe_projects WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .get_result::<Row>(conn)
        .map_err(|e| format!("project lookup: {e}"))?;
        Ok((row.org_id, row.branch_id))
    }

    fn plan_for(&self, conn: &mut Conn, branch_id: Uuid) -> Result<MeterPlan, String> {
        if branch_id.is_nil() {
            return Ok(MeterPlan::Free);
        }
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            plan_name: String,
        }
        // The plan is persisted in `billing_recurring.description`
        // ('Free Plan', 'shared - 14 Day Trial', 'private-cloud', ...);
        // there is no `plan_name` column in the table schema.
        let plan: Option<Row> = diesel::sql_query(
            "SELECT description AS plan_name FROM billing_recurring \
             WHERE branch_id = $1 AND status = 'active' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<Row>(conn)
        .optional()
        .map_err(|e| format!("plan lookup: {e}"))?;
        Ok(plan.map(|r| r.plan_name).as_deref().map(MeterPlan::parse).unwrap_or(MeterPlan::Free))
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
        let cap: Option<F64Cell> = diesel::sql_query(
            "SELECT hard_limit AS value FROM metering_limits \
             WHERE org_id = $1 AND scope = $2 AND meter = $3",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(plan.as_str())
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .get_result::<F64Cell>(&mut conn)
        .optional()
        .map_err(|e| format!("cap lookup: {e}"))?;
        let cap = cap.map(|c| c.value).unwrap_or(0.0);
        if cap <= 0.0 {
            return Ok(EnforceResult { allowed: true, metered: meter.as_str().to_string() });
        }
        let used: F64Cell = diesel::sql_query(
            "SELECT COALESCE(SUM(amount), 0)::float8 AS value FROM metering_usage \
             WHERE org_id = $1 AND meter = $2 \
             AND period_start >= NOW() - make_interval(secs => $3)",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(meter.as_str())
        .bind::<diesel::sql_types::BigInt, _>(Self::window_seconds())
        .get_result::<F64Cell>(&mut conn)
        .map_err(|e| format!("usage sum: {e}"))?;
        let used_val = used.value;
        if used_val >= cap {
            return Err(format!(
                "metering '{}' cap reached: {used_val:.2} of {cap:.2} ({} plan) — upgrade to continue",
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

    /// #769 — Pending VM hours for a running VM: elapsed hours capped to the
    /// metering window, minus what this environment already recorded in that
    /// window. Idempotent under repeated sampling.
    fn pending_vm_hours(&self, project_id: Uuid, env: &str, vm_created_at: DateTime<Utc>) -> Result<f64, String> {
        let mut conn = self.conn()?;
        let window = Self::window_seconds();
        let elapsed = (Utc::now() - vm_created_at).num_seconds().max(0) as f64 / 3600.0;
        let window_hours = window as f64 / 3600.0;
        let recorded: f64 = diesel::sql_query(
            "SELECT COALESCE(SUM(amount), 0)::float8 AS value FROM metering_usage \
             WHERE project_id = $1 AND env = $2 AND meter = $3 \
             AND period_start >= NOW() - make_interval(secs => $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(env)
        .bind::<diesel::sql_types::Text, _>(MeterKind::VmHours.as_str())
        .bind::<diesel::sql_types::BigInt, _>(window)
        .get_result::<F64Cell>(&mut conn)
        .map(|c| c.value)
        .map_err(|e| format!("recorded usage sum: {e}"))?;
        Ok((elapsed.min(window_hours) - recorded).max(0.0))
    }

    /// Idempotent tick: accrue the pending VM hours of every running VM.
    pub fn sample_running_vm_hours(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        #[derive(diesel::QueryableByName)]
        struct RunningVm {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            project_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            env: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }
        let vms = diesel::sql_query(
            "SELECT project_id, env, MAX(created_at) AS created_at \
             FROM vm_instances WHERE status = 'running' \
             GROUP BY project_id, env",
        )
        .load::<RunningVm>(&mut conn)
        .map_err(|e| format!("running vm list: {e}"))?;
        drop(conn);
        for vm in vms {
            let pending = match self.pending_vm_hours(vm.project_id, &vm.env, vm.created_at) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("metering sampler pending hours: {e}");
                    continue;
                }
            };
            if pending <= 0.001 {
                continue;
            }
            if let Err(e) = self.add_for_project(vm.project_id, &vm.env, MeterKind::VmHours, pending) {
                log::warn!("metering sampler accrual failed: {e}");
            }
        }
        Ok(())
    }

    /// Background sampler service (auto_service pattern): every `every_secs`
    /// accrues the pending VM hours of the running project VMs.
    pub fn spawn_vm_hours_sampler(metering: VMeteringRef, every_secs: u64) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(every_secs)).await;
                if let Err(e) = metering.sample_running_vm_hours() {
                    log::warn!("metering sampler tick failed: {e}");
                }
            }
        });
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
            "SELECT COUNT(*) AS value FROM vibe_projects \
             WHERE org_id = $1 AND project_type = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(project_type)
        .get_result::<I64Cell>(&mut conn)
        .map(|c| c.value)
        .map_err(|e| format!("project count: {e}"))
    }

    pub fn enforce_project_creation(
        &self,
        org_id: Uuid,
        branch_id: Uuid,
        project_type: &str,
    ) -> Result<(), String> {
        // Dev/testing escape hatch; mirrors the flag the cloud signup path
        // honors (botcloud::api::handle_signup). Never set in production.
        if std::env::var("SAAS_DISABLE_CAPACITY_CHECK").as_deref() == Ok("1") {
            return Ok(());
        }
        if org_id.is_nil() {
            return Ok(());
        }
        let mut conn = self.conn()?;
        let plan = self.plan_for(&mut conn, branch_id)?;
        match plan {
            MeterPlan::PrivateCloud => Ok(()),
            MeterPlan::Free => {
                // #1291 — kind renamed custom→apps; legacy rows keep "custom".
                let is_apps = project_type.eq_ignore_ascii_case("apps")
                    || project_type.eq_ignore_ascii_case("custom");
                if is_apps {
                    return Err("apps projects require the private-cloud plan".to_string());
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
