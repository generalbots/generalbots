//! #744 — Per-project VM lifecycle.
//!
//! Models the lifecycle of project VMs: a dev VM exists from project start,
//! a prod VM is raised on publish, a CI runner can live on the dev VM, and
//! each env is tiered (small/medium/large). VM records are persisted in
//! `vm_instances` (scoped by branch_id like the rest of the SaaS surface);
//! the driver layer talks to the host Incus so `incus list` reflects the
//! real containers (dev `...-dev[-runner]` / prod `...-prod` naming).
//!
//! ALM mapping: ALM org = branch short id, ALM repo = project name
//! (mirrors g.tmp #744: ALM org=branch, repo=project).
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::DbPool;

mod vm_incus;

pub const VM_INSTANCES_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS vm_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    project_name VARCHAR(255) NOT NULL DEFAULT '',
    env VARCHAR(16) NOT NULL DEFAULT 'development',
    tier VARCHAR(16) NOT NULL DEFAULT 'small',
    status VARCHAR(20) NOT NULL DEFAULT 'created',
    container_name VARCHAR(255) NOT NULL,
    runner_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_vm_instances_project_env ON vm_instances(project_id, env);
CREATE INDEX IF NOT EXISTS idx_vm_instances_branch ON vm_instances(branch_id);
CREATE INDEX IF NOT EXISTS idx_vm_instances_status ON vm_instances(status);
";

pub const VALID_ENVS: &[&str] = &["development", "staging", "production"];
pub const VALID_TIERS: &[&str] = &["small", "medium", "large"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInstance {
    pub id: Uuid,
    pub project_id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub project_name: String,
    pub env: String,
    pub tier: String,
    pub status: String,
    pub container_name: String,
    pub runner_enabled: bool,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub alm_org: String,
    pub alm_repo: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateVmRequest {
    pub env: String,
    pub tier: String,
    #[serde(default)]
    pub runner_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vm: Option<VmInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vms: Option<Vec<VmInstance>>,
    pub error: Option<String>,
}

impl VmResult {
    pub fn ok(vm: VmInstance) -> Self {
        Self { success: true, vm: Some(vm), vms: None, error: None }
    }
    pub fn ok_list(vms: Vec<VmInstance>) -> Self {
        Self { success: true, vm: None, vms: Some(vms), error: None }
    }
    pub fn err(msg: String) -> Self {
        Self { success: false, vm: None, vms: None, error: Some(msg) }
    }
    pub fn deleted() -> Self {
        Self { success: true, vm: None, vms: None, error: None }
    }
}

/// Per-project VM lifecycle driver (DB records + host Incus operations).
#[derive(Clone)]
pub struct VmLifecycle {
    pool: DbPool,
}

impl VmLifecycle {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, String>
    {
        self.pool.get().map_err(|e| format!("db pool: {e}"))
    }

    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(VM_INSTANCES_SCHEMA)
            .execute(&mut conn)
            .map_err(|e| format!("vm schema: {e}"))?;
        Ok(())
    }

    pub fn validate(req: &CreateVmRequest) -> Result<(String, String), String> {
        let env = req.env.trim().to_lowercase();
        if !VALID_ENVS.contains(&env.as_str()) {
            return Err(format!("invalid env '{env}', expected {}", VALID_ENVS.join("/")));
        }
        let tier = req.tier.trim().to_lowercase();
        if !VALID_TIERS.contains(&tier.as_str()) {
            return Err(format!("invalid tier '{tier}', expected {}", VALID_TIERS.join("/")));
        }
        Ok((env, tier))
    }

    /// ALM (Forgejo) org name for a branch (g.tmp #744 — org=branch).
    pub fn alm_org(branch_id: Uuid) -> String {
        branch_id
            .to_string()
            .split('-')
            .next()
            .unwrap_or("default")
            .to_string()
    }

    /// ALM repo name from the project name (g.tmp #744 — repo=project).
    pub fn alm_repo(project_name: &str) -> String {
        sanitize_part(project_name)
    }

    pub fn container_name(project_name: &str, env: &str, runner: bool) -> String {
        let env = match env {
            "production" => "prod",
            other => other,
        };
        let suffix = if runner { "-runner" } else { "" };
        format!("{}-{}{}", sanitize_part(project_name), env, suffix)
    }

    /// Ensure the project has a VM for `env`; if missing, insert the row,
    /// create (or start) the Incus container, and mark it running.
    pub fn create_project_vm(
        &self,
        project_id: Uuid,
        branch_id: Uuid,
        project_name: &str,
        req: &CreateVmRequest,
    ) -> Result<VmInstance, String> {
        let (env, tier) = Self::validate(req)?;
        let container = Self::container_name(project_name, &env, req.runner_enabled);

        match self.lookup(&project_id, &env) {
            Ok(existing) => {
                let state = self.linux_running(&existing.container_name)?;
                if !state {
                    self.linux_start(&existing.container_name)?;
                }
                self.set_status(&existing.id, "running")?;
                Ok(existing)
            }
            Err(_) => {
                let mut conn = self.conn()?;
                diesel::sql_query(
                    "INSERT INTO vm_instances (project_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, 'created', $6, $7, NOW(), NOW())",
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

                if !self.linux_exists(&container)? {
                    self.linux_create(&container, &tier)?;
                }
                if !self.linux_running(&container)? {
                    self.linux_start(&container)?;
                }
                let inst = self.lookup(&project_id, &env)?;
                self.set_status(&inst.id, "running")?;
                Ok(inst)
            }
        }
    }

    pub fn stop(&self, vm_id: Uuid) -> Result<VmInstance, String> {
        let inst = self.lookup_by_id(&vm_id)?;
        self.linux_stop(&inst.container_name)?;
        self.set_status(&inst.id, "stopped")?;
        Ok(VmInstance { status: "stopped".into(), ..inst })
    }

    pub fn delete(&self, vm_id: Uuid) -> Result<(), String> {
        let inst = self.lookup_by_id(&vm_id)?;
        if self.linux_exists(&inst.container_name)? {
            self.linux_delete(&inst.container_name)?;
        }
        let mut conn = self.conn()?;
        diesel::sql_query("DELETE FROM vm_instances WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(vm_id)
            .execute(&mut conn)
            .map_err(|e| format!("delete vm: {e}"))?;
        Ok(())
    }

    pub fn list(&self, project_id: Uuid) -> Result<Vec<VmInstance>, String> {
        let mut conn = self.conn()?;
        let rows = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, error, created_at, updated_at
             FROM vm_instances WHERE project_id = $1 ORDER BY env",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .load::<VmRow>(&mut conn)
        .map_err(|e| format!("list vms: {e}"))?;
        Ok(rows.into_iter().map(|r| r.into_vm()).collect())
    }

    pub fn sync_status(&self, vm_id: Uuid) -> Result<VmInstance, String> {
        let inst = self.lookup_by_id(&vm_id)?;
        let running = self.linux_running(&inst.container_name)?;
        let wanted = if running { "running" } else { "stopped" };
        if inst.status != wanted {
            self.set_status(&inst.id, wanted)?;
            return self.lookup_by_id(&vm_id);
        }
        Ok(inst)
    }

    pub fn get(&self, vm_id: Uuid) -> Result<VmInstance, String> {
        self.lookup_by_id(&vm_id)
    }

    fn set_status(&self, id: &Uuid, status: &str) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query("UPDATE vm_instances SET status = $2, error = NULL, updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(*id)
            .bind::<diesel::sql_types::Text, _>(status)
            .execute(&mut conn)
            .map_err(|e| format!("update vm: {e}"))?;
        Ok(())
    }

    fn lookup(&self, project_id: &Uuid, env: &str) -> Result<VmInstance, String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, error, created_at, updated_at
             FROM vm_instances WHERE project_id = $1 AND env = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(env)
        .get_result::<VmRow>(&mut conn)
        .map_err(|e| format!("lookup vm: {e}"))?;
        Ok(row.into_vm())
    }

    fn lookup_by_id(&self, id: &Uuid) -> Result<VmInstance, String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, error, created_at, updated_at
             FROM vm_instances WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(*id)
        .get_result::<VmRow>(&mut conn)
        .map_err(|e| format!("vm lookup: {e}"))?;
        Ok(row.into_vm())
    }

}

#[derive(diesel::QueryableByName)]
struct VmRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    org_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    branch_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    env: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    tier: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    container_name: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    runner_enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: DateTime<Utc>,
}

impl VmRow {
    fn into_vm(self) -> VmInstance {
        let branch_id = Uuid::parse_str(&self.branch_id).unwrap_or_default();
        VmInstance {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            project_id: Uuid::parse_str(&self.project_id).unwrap_or_default(),
            org_id: Uuid::parse_str(&self.org_id).unwrap_or_default(),
            branch_id,
            project_name: self.project_name.clone(),
            env: self.env,
            tier: self.tier,
            status: self.status,
            container_name: self.container_name,
            runner_enabled: self.runner_enabled,
            error: self.error,
            created_at: self.created_at,
            updated_at: self.updated_at,
            alm_org: VmLifecycle::alm_org(branch_id),
            alm_repo: VmLifecycle::alm_repo(&self.project_name),
        }
    }
}

fn sanitize_part(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars().take(32) {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        }
    }
    if out.is_empty() {
        "app".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_names_are_env_scoped() {
        assert_eq!(VmLifecycle::container_name("My App", "development", false), "my-app-development");
        assert_eq!(VmLifecycle::container_name("My App", "production", false), "my-app-prod");
        assert_eq!(VmLifecycle::container_name("Web", "development", true), "web-development-runner");
    }

    #[test]
    fn validate_accepts_known_envs_and_tiers() {
        let ok_req = CreateVmRequest { env: "production".into(), tier: "medium".into(), runner_enabled: false };
        assert!(VmLifecycle::validate(&ok_req).is_ok());
        let bad = CreateVmRequest { env: "moon".into(), tier: "small".into(), runner_enabled: false };
        assert!(VmLifecycle::validate(&bad).is_err());
        let bad_tier = CreateVmRequest { env: "dev".into(), tier: "huge".into(), runner_enabled: false };
        assert!(VmLifecycle::validate(&bad_tier).is_err());
    }

    #[test]
    fn alm_mapping_org_is_branch_short() {
        let id = Uuid::new_v4();
        let org = VmLifecycle::alm_org(id);
        assert_eq!(org.len(), 8);
        assert_eq!(VmLifecycle::alm_repo("My Web App"), "my-web-app");
    }
}