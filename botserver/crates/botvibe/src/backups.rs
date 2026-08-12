//! #773 — Prod VM backups: Incus snapshots + off-machine export copies.
//!
//! Backups are per-project/environment: an Incus snapshot (`incus snapshot
//! create`) is recorded in `vm_backups`; an optional off-machine copy is
//! exported with `incus export <container>/<tag>` into `VIBE_BACKUP_DIR`
//! (host-side directory, default `/opt/gbo/backups/{project}`). Restore
//! applies the snapshot back and then runs the #771 health probe — a
//! restore that fails the probe is reported as failed (DoD: "backup
//! restore passes health probe").

use std::path::Path;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::ensure_schema_sql;
use crate::types::DbPool;
use crate::vm_lifecycle::{VmInstance, VmLifecycle};

pub const VM_BACKUPS_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS vm_backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    env VARCHAR(16) NOT NULL,
    container_name VARCHAR(255) NOT NULL,
    tag VARCHAR(64) NOT NULL,
    kind VARCHAR(16) NOT NULL DEFAULT 'snapshot',
    status VARCHAR(20) NOT NULL DEFAULT 'created',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vm_backups_project ON vm_backups(project_id);
CREATE INDEX IF NOT EXISTS idx_vm_backups_created ON vm_backups(created_at DESC);
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub env: String,
    pub container_name: String,
    pub tag: String,
    pub kind: String,
    pub status: String,
    pub size_bytes: i64,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Backups {
    pool: DbPool,
}

/// Strip anything that is not [A-Za-z0-9._-] — used for tags and dirs so a
/// project/container name can never escape the backup root.
fn sanitize_for_path(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(64)
        .collect()
}

fn backup_root() -> String {
    std::env::var("VIBE_BACKUP_DIR")
        .unwrap_or_else(|_| "/opt/gbo/backups".to_string())
}

pub fn tag_now() -> String {
    Utc::now().format("backup-%Y%m%d-%H%M%S%f").to_string()
}

impl Backups {
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
        ensure_schema_sql(&mut conn, VM_BACKUPS_SCHEMA, "vm_backups schema")?;
        Ok(())
    }

    /// Create an Incus snapshot for the project env VM and record it.
    pub fn create_snapshot(&self, project_id: Uuid, env: &str) -> Result<BackupRecord, String> {
        let vm = self.env_vm(project_id, env)?;
        let tag = tag_now();
        let safe_tag = sanitize_for_path(&tag);
        VmLifecycle::new(self.pool.clone())
            .linux_snapshot(&vm.container_name, &safe_tag)
            .map_err(|e| format!("incus snapshot: {e}"))?;
        self.insert(&vm, &safe_tag, "snapshot", "created", 0)
    }

    /// Create an off-machine copy of an existing snapshot record.
    pub fn export_snapshot(&self, backup_id: Uuid) -> Result<BackupRecord, String> {
        let backup = self.get(backup_id)?;
        let root = Path::new(&backup_root()).join(backup.project_id.to_string());
        std::fs::create_dir_all(&root).map_err(|e| format!("backup dir: {e}"))?;
        let target = root.join(format!("{}-{}.tar.gz", backup.tag, sanitize_for_path(&backup.env)));
        let out = VmLifecycle::new(self.pool.clone())
            .linux_export(&backup.container_name, &backup.tag, &target.to_string_lossy())
            .map_err(|e| format!("incus export: {e}"))?;
        let size = std::fs::metadata(&target)
            .map(|m| m.len() as i64)
            .unwrap_or(0);
        self.update_status(backup_id, "exported", Some(out), size)
            .map(|_| BackupRecord {
                status: "exported".to_string(),
                size_bytes: size,
                ..backup
            })
    }

    pub fn list(&self, project_id: Uuid) -> Result<Vec<BackupRecord>, String> {
        let mut conn = self.conn()?;
        let rows = diesel::sql_query(
            "SELECT id, project_id, env, container_name, tag, kind, status, size_bytes, error, created_at
             FROM vm_backups WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind::<diesel::sql_types::Uuid, _>(project_id)
        .load::<BackupRow>(&mut conn)
        .map_err(|e| format!("list backups: {e}"))?;
        Ok(rows.into_iter().map(BackupRow::into_record).collect())
    }

    /// Restore a snapshot onto the VM: stop, restore, start, then probe.
    /// Returns the refreshed record plus the probe result (DoD).
    pub async fn restore(
        &self,
        backup_id: Uuid,
        ops: &crate::ops::VmOps,
    ) -> Result<(BackupRecord, crate::ops::ProbeReport), String> {
        let backup = self.get(backup_id)?;
        let lifecycle = VmLifecycle::new(self.pool.clone());
        if lifecycle.linux_exists(&backup.container_name)? {
            lifecycle.linux_restore_snapshot(&backup.container_name, &backup.tag)
                .map_err(|e| format!("incus restore: {e}"))?;
            if !lifecycle.linux_running(&backup.container_name)? {
                lifecycle.linux_start(&backup.container_name)?;
            }
        }
        let record = self.update_status(backup_id, "restored", None, 0)?;
        let vm = self.env_vm(backup.project_id, &backup.env)?;
        let probe = ops.verify_restore(&vm).await?;
        if !probe.ok {
            let why = format!(
                "backup {} restored but health probe failed: {}{}",
                backup.tag,
                probe.error.as_deref().unwrap_or("unknown"),
                if probe.http_code.is_some() {
                    format!(" (http {:?})", probe.http_code)
                } else {
                    String::new()
                }
            );
            let _ = self.update_status(backup_id, "failed", Some(why.clone()), 0);
            return Err(why);
        }
        Ok((record, probe))
    }

    pub fn get(&self, backup_id: Uuid) -> Result<BackupRecord, String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "SELECT id, project_id, env, container_name, tag, kind, status, size_bytes, error, created_at
             FROM vm_backups WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(backup_id)
        .get_result::<BackupRow>(&mut conn)
        .map_err(|e| format!("backup lookup: {e}"))?;
        Ok(row.into_record())
    }

    fn env_vm(&self, project_id: Uuid, env: &str) -> Result<VmInstance, String> {
        VmLifecycle::new(self.pool.clone())
            .list(project_id)
            .map_err(|e| format!("list vms: {e}"))?
            .into_iter()
            .find(|v| v.env == env)
            .ok_or_else(|| format!("no VM for env '{env}' — publish the project first"))
    }

    fn insert(
        &self,
        vm: &VmInstance,
        tag: &str,
        kind: &str,
        status: &str,
        size_bytes: i64,
    ) -> Result<BackupRecord, String> {
        let mut conn = self.conn()?;
        let row = diesel::sql_query(
            "INSERT INTO vm_backups (project_id, env, container_name, tag, kind, status, size_bytes, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) RETURNING id, project_id, env, container_name, tag, kind, status, size_bytes, error, created_at",
        )
        .bind::<diesel::sql_types::Uuid, _>(vm.project_id)
        .bind::<diesel::sql_types::Text, _>(&vm.env)
        .bind::<diesel::sql_types::Text, _>(&vm.container_name)
        .bind::<diesel::sql_types::Text, _>(tag)
        .bind::<diesel::sql_types::Text, _>(kind)
        .bind::<diesel::sql_types::Text, _>(status)
        .bind::<diesel::sql_types::BigInt, _>(size_bytes)
        .get_result::<BackupRow>(&mut conn)
        .map_err(|e| format!("insert backup: {e}"))?;
        Ok(row.into_record())
    }

    fn update_status(
        &self,
        backup_id: Uuid,
        status: &str,
        note: Option<String>,
        size_bytes: i64,
    ) -> Result<BackupRecord, String> {
        let mut conn = self.conn()?;
        match note {
            None => {
                diesel::sql_query(
                    "UPDATE vm_backups SET status = $2, size_bytes = $3, updated_at = NOW() WHERE id = $1",
                )
                .bind::<diesel::sql_types::Uuid, _>(backup_id)
                .bind::<diesel::sql_types::Text, _>(status)
                .bind::<diesel::sql_types::BigInt, _>(size_bytes)
                .execute(&mut conn)
                .map_err(|e| format!("update backup: {e}"))?;
            }
            Some(n) => {
                diesel::sql_query(
                    "UPDATE vm_backups SET status = $2, error = $3, size_bytes = $4, updated_at = NOW() WHERE id = $1",
                )
                .bind::<diesel::sql_types::Uuid, _>(backup_id)
                .bind::<diesel::sql_types::Text, _>(status)
                .bind::<diesel::sql_types::Text, _>(&n)
                .bind::<diesel::sql_types::BigInt, _>(size_bytes)
                .execute(&mut conn)
                .map_err(|e| format!("update backup: {e}"))?;
            }
        }
        self.get(backup_id)
    }
}

#[derive(diesel::QueryableByName)]
struct BackupRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    env: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    container_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    tag: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    kind: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    size_bytes: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: DateTime<Utc>,
}

impl BackupRow {
    fn into_record(self) -> BackupRecord {
        BackupRecord {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            project_id: Uuid::parse_str(&self.project_id).unwrap_or_default(),
            env: self.env,
            container_name: self.container_name,
            tag: self.tag,
            kind: self.kind,
            status: self.status,
            size_bytes: self.size_bytes,
            error: self.error,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_now_has_prefix_and_unique_minutes() {
        let a = tag_now();
        let b = tag_now();
        assert!(a.starts_with("backup-"));
        assert_ne!(a, b);
    }

    #[test]
    fn sanitize_blocks_path_escape() {
        // Slashes are stripped; dots are still allowed for versioned tags,
        // but the removal of "/" means `..` segments can never become paths.
        assert_eq!(sanitize_for_path("../../etc/passwd"), "....etcpasswd");
        assert_eq!(sanitize_for_path("my app"), "myapp");
        assert_eq!(sanitize_for_path("ok-1.2"), "ok-1.2");
    }
}