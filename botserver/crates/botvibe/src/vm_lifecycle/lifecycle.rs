//! `vm_lifecycle::lifecycle` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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

/// Valid deployment environments. `test` is the site twin of a
/// website/python project (`{slug}-test.{domain}`); `staging` stays available
/// for VM projects that need a third tier.
pub const VALID_ENVS: &[&str] = &["test", "development", "staging", "production"];

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

/// Per-project VM lifecycle driver (DB records + host Incus operations).
#[derive(Clone)]
pub struct VmLifecycle {
    pub(crate) pool: DbPool,
}

impl VmLifecycle {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn ensure_schema(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        ensure_schema_sql(&mut conn, VM_INSTANCES_SCHEMA, "vm_instances schema")?;
        Ok(())
    }

    pub fn validate(req: &CreateVmRequest) -> Result<(String, String), String> {
        let env = req.env.trim().to_lowercase();
        if !VALID_ENVS.contains(&env.as_str()) {
            return Err(format!(
                "invalid env '{env}', expected {}",
                VALID_ENVS.join("/")
            ));
        }
        let tier = req.tier.trim().to_lowercase();
        if !VALID_TIERS.contains(&tier.as_str()) {
            return Err(format!(
                "invalid tier '{tier}', expected {}",
                VALID_TIERS.join("/")
            ));
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

    /// #1503 — ALM org from the branch slug (readable org-branch org); falls
    /// back to the short-uuid form when no slug resolves. Kept next to the
    /// legacy derivation so all naming stays in one module.
    pub fn alm_org_from_slug(branch_slug: &str) -> String {
        let cleaned: String = branch_slug
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        let cleaned = cleaned.trim_matches('-');
        if cleaned.is_empty() {
            Self::alm_org(uuid::Uuid::nil())
        } else {
            cleaned.to_string()
        }
    }

    /// ALM repo name from the project name (g.tmp #744 — repo=project).
    pub fn alm_repo(project_name: &str) -> String {
        sanitize_part(project_name)
    }

    /// #1397 — container names that Vibe projects must never claim: they host
    /// platform services (botserver, database, drive, proxy, vault, ALM…).
    /// A malicious/accidental project name like "tables" or "system" would
    /// otherwise collide with a production container and break the host.
    pub fn is_protected_container_name(name: &str) -> bool {
        const PROTECTED: &[&str] = &[
            "bot", "system", "tables", "cache", "drive", "vault", "directory",
            "proxy", "dns", "email", "webmail", "meet", "vectordb", "llm", "alm",
            "alm-ci", "table-editor", "vibe", "vibe-runner", "host", "incus",
        ];
        let lower = name.to_ascii_lowercase();
        PROTECTED.contains(&lower.as_str())
    }

    pub fn container_name(project_name: &str, env: &str, runner: bool) -> String {
        let env = match env {
            "production" => "prod",
            other => other,
        };
        let suffix = if runner { "-runner" } else { "" };
        format!("{}-{}{}", sanitize_part(project_name), env, suffix)
    }

    /// Creates and starts the container if it does not already exist/run.
    pub(crate) fn provision_container(&self, container: &str, tier: &str) -> Result<(), String> {
        if !self.linux_exists(container)? {
            self.linux_create(container, tier)?;
        }
        if !self.linux_running(container)? {
            self.linux_start(container)?;
        }
        Ok(())
    }

    /// Marks a VM row `failed` and records the underlying provisioning error
    /// (#924) so a later request cannot mistake the dead row for a live VM.
    pub(crate) fn set_failed(&self, id: &Uuid, error: &str) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE vm_instances SET status = 'failed', error = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(*id)
        .bind::<diesel::sql_types::Text, _>(error)
        .execute(&mut conn)
        .map_err(|e| format!("mark vm failed: {e}"))?;
        Ok(())
    }

    pub fn stop(&self, vm_id: Uuid) -> Result<VmInstance, String> {
        let inst = self.lookup_by_id(&vm_id)?;
        if self.linux_available() {
            self.linux_stop(&inst.container_name)?;
        }
        self.set_status(&inst.id, "stopped")?;
        Ok(VmInstance {
            status: "stopped".into(),
            ..inst
        })
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
        if !self.linux_available() {
            return Ok(inst);
        }
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

    pub(crate) fn set_status(&self, id: &Uuid, status: &str) -> Result<(), String> {
        let mut conn = self.conn()?;
        diesel::sql_query(
            "UPDATE vm_instances SET status = $2, error = NULL, updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(*id)
        .bind::<diesel::sql_types::Text, _>(status)
        .execute(&mut conn)
        .map_err(|e| format!("update vm: {e}"))?;
        Ok(())
    }

    pub(crate) fn lookup(&self, project_id: &Uuid, env: &str) -> Result<VmInstance, String> {
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

    /// #924 — optional lookup that distinguishes "no row" (Ok(None)) from a
    /// real database failure (Err), so callers never treat a transient DB
    /// error as "not found".
    pub(crate) fn lookup_opt(&self, project_id: &Uuid, env: &str) -> Result<Option<VmInstance>, String> {
        let mut conn = self.conn()?;
        let result = diesel::sql_query(
            "SELECT id, project_id, org_id, branch_id, project_name, env, tier, status, container_name, runner_enabled, error, created_at, updated_at
             FROM vm_instances WHERE project_id = $1 AND env = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .bind::<diesel::sql_types::Text, _>(env)
        .get_result::<VmRow>(&mut conn);
        match result {
            Ok(row) => Ok(Some(row.into_vm())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(format!("lookup vm: {e}")),
        }
    }

    pub(crate) fn lookup_by_id(&self, id: &Uuid) -> Result<VmInstance, String> {
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
pub(crate) struct VmRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) project_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) project_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) env: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) tier: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) container_name: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub(crate) runner_enabled: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) created_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) updated_at: DateTime<Utc>,
}

impl VmRow {
    pub(crate) fn into_vm(self) -> VmInstance {
        VmInstance {
            id: self.id,
            project_id: self.project_id,
            org_id: self.org_id,
            branch_id: self.branch_id,
            project_name: self.project_name.clone(),
            env: self.env,
            tier: self.tier,
            status: self.status,
            container_name: self.container_name,
            runner_enabled: self.runner_enabled,
            error: self.error,
            created_at: self.created_at,
            updated_at: self.updated_at,
            alm_org: VmLifecycle::alm_org(self.branch_id),
            alm_repo: VmLifecycle::alm_repo(&self.project_name),
        }
    }
}

pub(crate) fn sanitize_part(s: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in s.chars().take(32) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if (ch == '-' || ch == '_' || ch.is_whitespace()) && !out.is_empty() && !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    if out.is_empty() {
        "app".to_string()
    } else {
        out
    }
}
