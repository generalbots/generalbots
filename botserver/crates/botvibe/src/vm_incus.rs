//! #744 — Incus CLI driver for the VM lifecycle (host-side container ops).
//! Runs the `incus` CLI via the harness command guard so every invocation
//! is allowlisted and argument-validated; containers are visible through
//! `incus list` (Verifiable per #744 DoD).

use std::path::Path;

use crate::harness::cmd::run;
use crate::vm_lifecycle::VmLifecycle;

impl VmLifecycle {
    pub(crate) fn linux_exists(&self, name: &str) -> Result<bool, String> {
        let list = self.linux_list()?;
        Ok(list
            .as_array()
            .map(|arr| arr.iter().any(|i| i["name"].as_str() == Some(name)))
            .unwrap_or(false))
    }

    pub(crate) fn linux_running(&self, name: &str) -> Result<bool, String> {
        let list = self.linux_list()?;
        Ok(list
            .as_array()
            .map(|arr| {
                arr.iter()
                    .find(|i| i["name"].as_str() == Some(name))
                    .map(|i| i["state"].as_str().unwrap_or("") == "Running")
                    .unwrap_or(false)
            })
            .unwrap_or(false))
    }

    pub(crate) fn linux_create(&self, name: &str, tier: &str) -> Result<(), String> {
        let image = std::env::var("VIBE_VM_IMAGE").unwrap_or_else(|_| "images:ubuntu/24.04".to_string());
        let (cpu, mem) = match tier {
            "medium" => ("2", "2GiB"),
            "large" => ("4", "4GiB"),
            _ => ("1", "1GiB"),
        };
        let args = [
            "launch".to_string(),
            image,
            name.to_string(),
            format!("limits.cpu={cpu}"),
            format!("limits.memory={mem}"),
            "environment.VIBE_PROJECT=1".to_string(),
        ];
        run("incus", &args, Path::new("/tmp"), 120)
            .map_err(|e| format!("incus launch {name} (tier {tier}): {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_start(&self, name: &str) -> Result<(), String> {
        run("incus", &["start".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus start {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_stop(&self, name: &str) -> Result<(), String> {
        run("incus", &["stop".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus stop {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_delete(&self, name: &str) -> Result<(), String> {
        run("incus", &["delete".to_string(), "--force".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus delete {name}: {e}"))?;
        Ok(())
    }

    fn linux_list(&self) -> Result<serde_json::Value, String> {
        let out = run(
            "incus",
            &["list".to_string(), "--format".to_string(), "json".to_string()],
            Path::new("/tmp"),
            30,
        )
        .map_err(|e| format!("incus list: {e}"))?;
        serde_json::from_str(&out.stdout).map_err(|e| format!("incus list parse: {e}"))
    }
}