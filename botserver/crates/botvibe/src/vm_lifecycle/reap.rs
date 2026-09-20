//! `vm_lifecycle::reap` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Containers already reported missing by the prod-VM guard. The guard runs on
/// a timer, so without this it repeats the same warning for the same rows on
/// every cycle; the warning is emitted once per missing container instead.
pub(crate) fn missing_container_reports() -> &'static std::sync::Mutex<std::collections::HashSet<String>> {
    static REPORTS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    REPORTS.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

impl VmResult {
    pub fn ok(vm: VmInstance) -> Self {
        Self {
            success: true,
            vm: Some(vm),
            vms: None,
            error: None,
        }
    }
    pub fn ok_list(vms: Vec<VmInstance>) -> Self {
        Self {
            success: true,
            vm: None,
            vms: Some(vms),
            error: None,
        }
    }
    pub fn err(msg: String) -> Self {
        Self {
            success: false,
            vm: None,
            vms: None,
            error: Some(msg),
        }
    }
    pub fn deleted() -> Self {
        Self {
            success: true,
            vm: None,
            vms: None,
            error: None,
        }
    }
}

impl VmLifecycle {
    /// Expose the DB pool so asset cleanup (domain unbind etc.) can share
    /// the same connection pool instead of opening its own.
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub(crate) fn conn(
        &self,
    ) -> Result<
        diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
        String,
    > {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    /// Ensure the project has a VM for `env`; if missing, insert the row,
    /// create (or start) the Incus container, and mark it running.
    ///
    /// #924 — a lookup error must not be mistaken for "not found" (which would
    /// attempt a duplicate insert), the insert is idempotent against the
    /// unique `(project_id, env)` index, and a provisioning failure marks the
    /// row `failed` with the underlying error instead of leaving it `created`.
    pub fn create_project_vm(
        &self,
        project_id: Uuid,
        branch_id: Uuid,
        project_name: &str,
        req: &CreateVmRequest,
    ) -> Result<VmInstance, String> {
        let (env, tier) = Self::validate(req)?;
        let container = Self::container_name(project_name, &env, req.runner_enabled);
        if Self::is_protected_container_name(project_name) {
            return Err(format!(
                "project name '{}' resolves to a protected platform container name; choose another name",
                project_name
            ));
        }

        if let Some(existing) = self.lookup_opt(&project_id, &env)? {
            if self.linux_available() {
                if !self.linux_exists(&existing.container_name)? {
                    self.provision_container(&existing.container_name, &existing.tier)?;
                } else if !self.linux_running(&existing.container_name)? {
                    self.linux_start(&existing.container_name)?;
                }
                self.set_status(&existing.id, "running")?;
            } else {
                self.set_status(&existing.id, "skipped")?;
            }
            return self.lookup(&project_id, &env);
        }

        // Idempotent upsert against the unique `(project_id, env)` index so
        // concurrent create requests converge on one row instead of racing a
        // check-then-act insert.
        let mut conn = self.conn()?;
        diesel::sql_query(
            "INSERT INTO vm_instances (project_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'created', $6, $7, NOW(), NOW())
             ON CONFLICT (project_id, env) DO NOTHING",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(project_name)
        .bind::<diesel::sql_types::Text, _>(&env)
        .bind::<diesel::sql_types::Text, _>(&tier)
        .bind::<diesel::sql_types::Text, _>(&container)
        .bind::<diesel::sql_types::Bool, _>(req.runner_enabled)
        .execute(&mut conn)
        .map_err(|e| format!("insert vm: {e}"))?;

        let inst = self.lookup(&project_id, &env)?;

        if !self.linux_available() {
            self.set_status(&inst.id, "skipped")?;
            return self.lookup(&project_id, &env);
        }

        if let Err(e) = self.provision_container(&container, &tier) {
            self.set_failed(&inst.id, &e)?;
            return Err(e);
        }
        self.set_status(&inst.id, "running")?;
        self.lookup(&project_id, &env)
    }

    pub fn delete(&self, vm_id: Uuid) -> Result<(), String> {
        let inst = self.lookup_by_id(&vm_id)?;
        if self.linux_available() && self.linux_exists(&inst.container_name)? {
            self.linux_delete(&inst.container_name)?;
        }
        let mut conn = self.conn()?;
        diesel::sql_query("DELETE FROM vm_instances WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(vm_id)
            .execute(&mut conn)
            .map_err(|e| format!("delete vm: {e}"))?;
        Ok(())
    }

    /// Deletes every VM (row + Incus container) belonging to a project.
    /// Called when the project itself is removed so no orphaned containers
    /// or `vm_instances` rows survive the project delete (#1266).
    pub fn delete_all_for_project(&self, project_id: Uuid) -> Result<usize, String> {
        let vms = self.list(project_id)?;
        let mut deleted = 0usize;
        for vm in &vms {
            if self.linux_available() && self.linux_exists(&vm.container_name)? {
                if let Err(e) = self.linux_delete(&vm.container_name) {
                    log::error!(
                        "Vibe: delete container {} for project {project_id} failed: {e}",
                        vm.container_name
                    );
                }
            }
            let mut conn = self.conn()?;
            diesel::sql_query("DELETE FROM vm_instances WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(vm.id)
                .execute(&mut conn)
                .map_err(|e| format!("delete vm for project {project_id}: {e}"))?;
            deleted += 1;
        }
        Ok(deleted)
    }

    /// Prod-VM guard (#1271): production VMs stay running forever once
    /// deployed. Starts every `production` VM whose container exists but is
    /// not running (host reboot, manual stop, crash) and syncs the row back
    /// to `running`. Missing containers are reported so the operator can
    /// redeploy. Returns the container names that were (re)started.
    pub fn ensure_prod_running(&self) -> Result<Vec<String>, String> {
        let vms = self.list_all()?;
        if vms.is_empty() || !self.linux_available() {
            return Ok(Vec::new());
        }
        let mut started: Vec<String> = Vec::new();
        for vm in vms {
            if vm.env != "production" {
                continue;
            }
            match self.linux_exists(&vm.container_name) {
                Ok(false) => {
                    // #1444 M3 — a pruned prod container must fail honestly:
                    // the row used to stay `running` forever (the reaper
                    // skips production), so the published URL 502'd until a
                    // manual redeploy. The row is marked `failed` once with
                    // the reason; the UI/proxy stop showing running and the
                    // operator redeploys (which recreates + resyncs).
                    if vm.status != "failed" {
                        let already_reported = missing_container_reports()
                            .lock()
                            .map(|mut reported| !reported.insert(vm.container_name.clone()))
                            .unwrap_or(false);
                        if !already_reported {
                            log::warn!(
                                "Vibe prod-VM guard: {} row exists but container is missing — marking failed (redeploy to recreate)",
                                vm.container_name
                            );
                        }
                        if let Err(e) = self.set_failed(
                            &vm.id,
                            "production container is missing (pruned?) — redeploy to recreate",
                        ) {
                            log::error!(
                                "Vibe prod-VM guard: mark {} failed: {e}",
                                vm.container_name
                            );
                        }
                    }
                }
                Ok(true) => {
                    // The container is back: allow a future disappearance to
                    // warn again instead of staying suppressed.
                    if let Ok(mut reported) = missing_container_reports().lock() {
                        reported.remove(&vm.container_name);
                    }
                    match self.linux_running(&vm.container_name) {
                        Ok(true) => {
                            // #1444 M3 — heal a row a previous guard marked
                            // failed once the container is genuinely back.
                            if vm.status == "failed" {
                                if let Err(e) = self.set_status(&vm.id, "running") {
                                    log::error!(
                                        "Vibe prod-VM guard: heal status for {} failed: {e}",
                                        vm.container_name
                                    );
                                }
                            }
                        }
                        Ok(false) => match self.linux_start(&vm.container_name) {
                            Ok(()) => {
                                if let Err(e) = self.set_status(&vm.id, "running") {
                                    log::error!(
                                        "Vibe prod-VM guard: sync status for {} failed: {e}",
                                        vm.container_name
                                    );
                                }
                                started.push(vm.container_name.clone());
                            }
                            Err(e) => log::error!(
                                "Vibe prod-VM guard: start {} failed: {e}",
                                vm.container_name
                            ),
                        },
                        Err(e) => log::error!(
                            "Vibe prod-VM guard: check {} failed: {e}",
                            vm.container_name
                        ),
                    }
                }
                Err(e) => log::error!(
                    "Vibe prod-VM guard: probe {} failed: {e}",
                    vm.container_name
                ),
            }
        }
        Ok(started)
    }

    /// Lists every VM across all projects (reaper / admin sweep).
    pub fn list_all(&self) -> Result<Vec<VmInstance>, String> {
        let mut conn = self.conn()?;
        let rows = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, project_name, env, tier, status, \
             container_name, runner_enabled, error, created_at, updated_at \
             FROM vm_instances ORDER BY created_at",
        )
        .load::<VmRow>(&mut conn)
        .map_err(|e| format!("list all vms: {e}"))?;
        Ok(rows.into_iter().map(|row| row.into_vm()).collect())
    }

    /// Idle reaper + expiry sweep (#1181 / #1167). Stops VMs that have been
    /// idle (no `updated_at` change) longer than `idle_secs` and deletes VMs
    /// older than `max_age_secs`. Returns the container names affected so the
    /// caller can log them. Running/awaiting VMs are never touched unless they
    /// exceed the max age (expiry), which also forces a stop.
    pub fn reap(&self, idle_secs: i64, max_age_secs: i64) -> Result<Vec<String>, String> {
        let vms = self.list_all()?;
        let now = chrono::Utc::now();
        let mut reaped: Vec<String> = Vec::new();
        for vm in vms {
            if vm.status == "stopped" {
                continue;
            }
            // #1271 — production VMs are never reaped: they run forever once
            // deployed (the prod-VM guard keeps them up). Only dev/staging
            // VMs are subject to idle-stop and expiry.
            if vm.env == "production" {
                continue;
            }
            let idle = now.signed_duration_since(vm.updated_at).num_seconds();
            let age = now.signed_duration_since(vm.created_at).num_seconds();
            if age > max_age_secs {
                // Expired: delete outright (containers are disposable).
                match self.delete(vm.id) {
                    Ok(()) => reaped.push(format!("{} (expired, deleted)", vm.container_name)),
                    Err(e) => log::error!("Vibe reaper: expire {} failed: {e}", vm.container_name),
                }
            } else if idle > idle_secs {
                // Idle: stop the container, keep the record for restart.
                match self.stop(vm.id) {
                    Ok(_) => reaped.push(format!("{} (idle, stopped)", vm.container_name)),
                    Err(e) => log::error!("Vibe reaper: idle-stop {} failed: {e}", vm.container_name),
                }
            }
        }
        Ok(reaped)
    }
}
