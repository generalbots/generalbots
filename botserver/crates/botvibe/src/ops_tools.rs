//! #771/#772/#773 — Ops tools: health probes (`vm/probe`, `vm/restart`),
//! deploy history + rollback (`publish/history`, `publish/rollback`), and
//! backups (`backup/snapshot`, `backup/export`, `backup/list`,
//! `backup/restore`). All follow the `Box::pin` with an owned `DbPool`
//! pattern so the returned future is `'static` (no borrowed state).

use std::sync::Arc;

use serde_json::{json, Value};
use uuid::Uuid;

use crate::ops::VmOps;
use crate::projects::ProjectRegistry;
use crate::tool_executor::{ToolHandler, ToolSchema, ToolSchemaExt};
use crate::types::{DbPool, VibeState, VibeToolResult, VibeUseCase};

fn handler<F>(f: F) -> ToolHandler
where
    F: Fn(DbPool, Value) -> futures::future::BoxFuture<'static, Result<Value, String>> + Send + Sync + 'static,
{
    Arc::new(move |args: Value, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        let args = args.clone();
        Box::pin(async move {
            let started = std::time::Instant::now();
            match f(pool, args).await {
                Ok(data) => VibeToolResult { success: true, data, error: None, latency_ms: started.elapsed().as_millis() as u64 },
                Err(e) => VibeToolResult { success: false, data: Value::Null, error: Some(e), latency_ms: started.elapsed().as_millis() as u64 },
            }
        })
    })
}

fn str_arg(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing '{key}'"))
}

async fn do_probe(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let env = str_arg(&args, "env").unwrap_or_else(|_| "production".to_string());
    let report = VmOps::new(pool).probe_and_recover(pid, &env, false).await?;
    Ok(json!({ "probe": report }))
}

async fn do_restart(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let env = str_arg(&args, "env").unwrap_or_else(|_| "production".to_string());
    let report = VmOps::new(pool).probe_and_recover(pid, &env, true).await?;
    if !report.ok {
        return Err(format!("app unhealthy after restart: {report:?}"));
    }
    Ok(json!({ "restarted": true, "probe": report }))
}

async fn do_history(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let env = args.get("env").and_then(|v| v.as_str());
    let rows = ProjectRegistry::new(pool).list_deployments(pid, env)?;
    Ok(json!({ "deployments": rows }))
}

async fn do_rollback(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let env = str_arg(&args, "env").unwrap_or_else(|_| "production".to_string());
    let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let registry = ProjectRegistry::new(pool.clone());
    let rows = registry.list_deployments(pid, Some(&env))?;
    let target = rows
        .get(index)
        .ok_or_else(|| format!("deployment index {index} not found (have {})", rows.len()))?;
    let domain = target.get("domain").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let _redeploy = crate::publish::do_publish(
        json!({ "project_id": pid.to_string(), "env": env, "domain": domain }),
        pool.clone(),
    )
    .await?;
    let note = json!({
        "env": env,
        "at": chrono::Utc::now().to_rfc3339(),
        "url": target.get("url").and_then(|v| v.as_str()).unwrap_or(""),
        "container": target.get("container").and_then(|v| v.as_str()).unwrap_or(""),
        "domain": domain,
        "track": "rollback",
    });
    registry.append_deployment(pid, &note).map_err(|e| format!("record rollback: {e}"))?;
    Ok(json!({
        "rolled_back": true,
        "to_index": index,
        "from": target,
        "redeployed": true,
    }))
}

async fn do_backup_snapshot(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let env = str_arg(&args, "env").unwrap_or_else(|_| "production".to_string());
    let rec = crate::backups::Backups::new(pool).create_snapshot(pid, &env)?;
    Ok(json!({ "backup": rec }))
}

async fn do_backup_export(pool: DbPool, args: Value) -> Result<Value, String> {
    let bid = Uuid::parse_str(&str_arg(&args, "backup_id")?)?;
    let rec = crate::backups::Backups::new(pool).export_snapshot(bid)?;
    Ok(json!({ "backup": rec, "exported": true }))
}

async fn do_backup_list(pool: DbPool, args: Value) -> Result<Value, String> {
    let pid = Uuid::parse_str(&str_arg(&args, "project_id")?)?;
    let rows = crate::backups::Backups::new(pool).list(pid)?;
    Ok(json!({ "backups": rows }))
}

async fn do_backup_restore(pool: DbPool, args: Value) -> Result<Value, String> {
    let bid = Uuid::parse_str(&str_arg(&args, "backup_id")?)?;
    let backups = crate::backups::Backups::new(pool.clone());
    let ops = VmOps::new(pool);
    let (rec, probe) = backups.restore(bid, &ops).await?;
    Ok(json!({ "backup": rec, "probe_after_restore": probe }))
}

fn params(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required })
}

fn project_env_params() -> Value {
    params(
        json!({
            "project_id": { "type": "string" },
            "env": { "type": "string", "enum": ["development", "staging", "production"], "default": "production" }
        }),
        &["project_id"],
    )
}

fn id_params() -> Value {
    params(json!({ "backup_id": { "type": "string" } }), &["backup_id"])
}

fn history_params() -> Value {
    params(
        json!({
            "project_id": { "type": "string" },
            "env": { "type": "string", "enum": ["development", "staging", "production"] }
        }),
        &["project_id"],
    )
}

fn rollback_params() -> Value {
    params(
        json!({
            "project_id": { "type": "string" },
            "env": { "type": "string", "enum": ["development", "staging", "production"], "default": "production" },
            "index": { "type": "integer", "description": "0 = latest deployment, 1 = previous (default)" }
        }),
        &["project_id"],
    )
}

/// All ops tools: (name, schema, handler) triples for the registry.
pub fn ops_tools() -> Vec<(&'static str, ToolSchema, ToolHandler)> {
    let mut out: Vec<(&'static str, ToolSchema, ToolHandler)> = Vec::new();
    out.push((
        "vm/probe",
        ToolSchema::new("vm/probe", "Probe the health of a project env VM (liveness + HTTP check)")
            .with_parameters(project_env_params())
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_probe(pool, args))),
    ));
    out.push((
        "vm/restart",
        ToolSchema::new("vm/restart", "Restart a project env VM after a failed probe and re-probe")
            .with_parameters(project_env_params())
            .with_approval_if(true)
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_restart(pool, args))),
    ));
    out.push((
        "publish/history",
        ToolSchema::new("publish/history", "List the deployment history of a project env")
            .with_parameters(history_params())
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_history(pool, args))),
    ));
    out.push((
        "publish/rollback",
        ToolSchema::new("publish/rollback", "Roll back a project env to a previous deployment")
            .with_parameters(rollback_params())
            .with_approval_if(true)
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_rollback(pool, args))),
    ));
    out.push((
        "backup/snapshot",
        ToolSchema::new("backup/snapshot", "Create an Incus snapshot backup of the project env VM")
            .with_parameters(project_env_params())
            .with_approval_if(true)
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_backup_snapshot(pool, args))),
    ));
    out.push((
        "backup/export",
        ToolSchema::new("backup/export", "Export a snapshot off-machine into VIBE_BACKUP_DIR")
            .with_parameters(id_params())
            .with_approval_if(true)
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_backup_export(pool, args))),
    ));
    out.push((
        "backup/list",
        ToolSchema::new("backup/list", "List backups of a project")
            .with_parameters(params(json!({ "project_id": { "type": "string" } }), &["project_id"]))
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_backup_list(pool, args))),
    ));
    out.push((
        "backup/restore",
        ToolSchema::new("backup/restore", "Restore a snapshot onto the VM and verify via health probe")
            .with_parameters(id_params())
            .with_approval_if(true)
            .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
        handler(|pool, args| Box::pin(do_backup_restore(pool, args))),
    ));
    out
}