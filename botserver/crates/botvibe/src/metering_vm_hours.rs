//! VM-hour accrual and sampling for the metering engine.
//!
//! Split out of `metering.rs` to keep both files under the 450-line limit; the
//! functions live in an `impl VMetering` block and use the crate-visible pool
//! accessor.

use super::*;

impl VMetering {
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
}
