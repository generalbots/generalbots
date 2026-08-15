//! #771 — Ops: health probes and auto-restart for deployed apps.
//!
//! Probes a project VM: process state (Incus liveness) plus an HTTP health
//! check against the container's application endpoint. On failure the ops
//! driver can restart the container and re-probe; the result is a
//! `ProbeReport` the tools/API surface can act on. Backup restore (#773)
//! ends by running the same probe to prove recovery (DoD).

use std::time::Duration;

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::types::DbPool;
use crate::vm_lifecycle::{VmInstance, VmLifecycle, VALID_ENVS};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProbeReport {
    pub container: String,
    pub project_id: Uuid,
    pub env: String,
    pub running: bool,
    pub http_code: Option<u16>,
    pub ok: bool,
    pub error: Option<String>,
    pub checked_at: String,
}

impl ProbeReport {
    fn failing(container: &str, project_id: Uuid, env: &str, reason: String) -> Self {
        Self {
            container: container.to_string(),
            project_id,
            env: env.to_string(),
            running: false,
            http_code: None,
            ok: false,
            error: Some(reason),
            checked_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Probe URL for a container; env `VIBE_PROBE_URL_TEMPLATE` may contain
/// `{container}` (default `http://{container}.incus` — matches the Caddy
/// dial used by #770). When the container's IP is resolvable, the probe
/// dials `http://{ip}:{VIBE_APP_PORT}` instead (see `VmOps::probe`).
pub fn probe_url(container: &str) -> String {
    let tmpl = std::env::var("VIBE_PROBE_URL_TEMPLATE")
        .unwrap_or_else(|_| "http://{container}.incus".to_string());
    tmpl.replace("{container}", container)
}

#[derive(Clone)]
pub struct VmOps {
    pool: DbPool,
}

impl VmOps {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Run a single health probe: Incus liveness + HTTP app check.
    pub async fn probe(&self, vm: &VmInstance) -> Result<ProbeReport, String> {
        let lifecycle = VmLifecycle::new(self.pool.clone());
        let running = match lifecycle.linux_running(&vm.container_name) {
            Ok(r) => r,
            Err(e) => {
                return Ok(ProbeReport::failing(
                    &vm.container_name,
                    vm.project_id,
                    &vm.env,
                    format!("incus state: {e}"),
                ))
            }
        };
        let mut report = ProbeReport {
            container: vm.container_name.clone(),
            project_id: vm.project_id,
            env: vm.env.clone(),
            running,
            http_code: None,
            ok: false,
            error: None,
            checked_at: Utc::now().to_rfc3339(),
        };
        if !running {
            report.error = Some("container not running".to_string());
            return Ok(report);
        }
        // Prefer the container's real IPv4 (the host cannot resolve
        // `{container}.incus` DNS names); fall back to the name template.
        let url = match lifecycle.linux_ip(&vm.container_name) {
            Ok(Some(ip)) => {
                let tmpl = std::env::var("VIBE_PROBE_URL_TEMPLATE")
                    .unwrap_or_else(|_| "http://{container}:{port}".to_string());
                let port = std::env::var("VIBE_APP_PORT").unwrap_or_else(|_| "80".to_string());
                tmpl.replace("{container}", &ip).replace("{port}", &port)
            }
            _ => probe_url(&vm.container_name),
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        match client.get(&url).send().await {
            Ok(resp) => {
                let code = resp.status().as_u16();
                report.http_code = Some(code);
                report.ok = code < 400;
                if !report.ok {
                    report.error = Some(format!("http {code}"));
                }
            }
            Err(e) => report.error = Some(format!("http probe {url}: {e}")),
        }
        Ok(report)
    }

    /// Probe, and if unhealthy, optionally auto-restart (start if stopped,
    /// else `incus restart`); re-probes once after the action.
    pub async fn probe_and_recover(
        &self,
        project_id: Uuid,
        env: &str,
        auto_restart: bool,
    ) -> Result<ProbeReport, String> {
        if !VALID_ENVS.contains(&env) {
            return Err(format!("invalid env '{env}'"));
        }
        let lifecycle = VmLifecycle::new(self.pool.clone());
        let vm = self.lookup_env(&lifecycle, project_id, env)?;
        let mut report = self.probe(&vm).await?;
        if auto_restart && !report.ok {
            match lifecycle.linux_restart(&vm.container_name) {
                Ok(()) => report = self.probe(&vm).await?,
                Err(e) => report.error = Some(format!("restart: {e}")),
            }
            let _ = lifecycle.sync_status(vm.id);
        }
        Ok(report)
    }

    /// Verify a backup restore (#773 DoD): the VM must be running AND the
    /// HTTP probe must pass after restore.
    pub async fn verify_restore(&self, vm: &VmInstance) -> Result<ProbeReport, String> {
        let lifecycle = VmLifecycle::new(self.pool.clone());
        if !lifecycle.linux_running(&vm.container_name).unwrap_or(false) {
            return Ok(ProbeReport::failing(
                &vm.container_name,
                vm.project_id,
                &vm.env,
                "container not running after restore".to_string(),
            ));
        }
        self.probe(vm).await
    }

    fn lookup_env(
        &self,
        lifecycle: &VmLifecycle,
        project_id: Uuid,
        env: &str,
    ) -> Result<VmInstance, String> {
        let vms = lifecycle
            .list(project_id)
            .map_err(|e| format!("list vms: {e}"))?;
        vms.into_iter()
            .find(|v| v.env == env)
            .ok_or_else(|| format!("no VM for env '{env}' — publish the project first"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_url_template_defaults_then_accepts_env_override() {
        std::env::remove_var("VIBE_PROBE_URL_TEMPLATE");
        assert_eq!(
            probe_url("my-app-prod"),
            "http://my-app-prod.incus"
        );
        std::env::set_var("VIBE_PROBE_URL_TEMPLATE", "http://{container}.gb.solutions");
        assert_eq!(
            probe_url("my-app-prod"),
            "http://my-app-prod.gb.solutions"
        );
        std::env::remove_var("VIBE_PROBE_URL_TEMPLATE");
    }

    #[test]
    fn failing_report_is_not_ok() {
        let r = ProbeReport::failing("a-prod", Uuid::nil(), "production", "boom".into());
        assert!(!r.ok);
        assert!(!r.running);
        assert_eq!(r.error.as_deref(), Some("boom"));
    }
}