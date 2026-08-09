//! #744 — Incus CLI driver for the VM lifecycle (host-side container ops).
//! Runs the `incus` CLI via the harness command guard so every invocation
//! is allowlisted and argument-validated; containers are visible through
//! `incus list` (Verifiable per #744 DoD).

use std::path::Path;

use crate::harness::cmd::run;
use crate::vm_lifecycle::VmLifecycle;

const VM_UNAVAILABLE: &str = "vm-skip: incus binary unavailable";

impl VmLifecycle {
    pub(crate) fn linux_available(&self) -> bool {
        if std::env::var("VIBE_INCUS_FORCE_UNAVAILABLE").as_deref() == Ok("1") {
            return false;
        }
        run("incus", &["version".to_string()], Path::new("/tmp"), 5).is_ok()
    }

    fn skip_if_unavailable(&self) -> Result<(), String> {
        if self.linux_available() {
            Ok(())
        } else {
            Err(VM_UNAVAILABLE.to_string())
        }
    }

    pub(crate) fn linux_exists(&self, name: &str) -> Result<bool, String> {
        self.skip_if_unavailable()?;
        let list = self.linux_list()?;
        Ok(list
            .as_array()
            .map(|arr| arr.iter().any(|i| i["name"].as_str() == Some(name)))
            .unwrap_or(false))
    }

    pub(crate) fn linux_running(&self, name: &str) -> Result<bool, String> {
        self.skip_if_unavailable()?;
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
        self.skip_if_unavailable()?;
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
        self.skip_if_unavailable()?;
        run("incus", &["start".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus start {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_stop(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        run("incus", &["stop".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus stop {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_delete(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        run("incus", &["delete".to_string(), "--force".to_string(), name.to_string()], Path::new("/tmp"), 60)
            .map_err(|e| format!("incus delete {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_restart(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        run("incus", &["restart".to_string(), name.to_string()], Path::new("/tmp"), 120)
            .map_err(|e| format!("incus restart {name}: {e}"))?;
        Ok(())
    }

    /// `incus snapshot create {name} {tag}` — point-in-time VM backup (#773).
    pub(crate) fn linux_snapshot(&self, name: &str, tag: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        run(
            "incus",
            &["snapshot".to_string(), "create".to_string(), format!("{name}/{tag}")],
            Path::new("/tmp"),
            180,
        )
        .map_err(|e| format!("incus snapshot {name}/{tag}: {e}"))?;
        Ok(())
    }

    /// `incus restore {name} {tag}` — applies a snapshot; the container must
    /// be stopped, so stop first and let the caller restart it.
    pub(crate) fn linux_restore_snapshot(&self, name: &str, tag: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        if self.linux_running(name)? {
            self.linux_stop(name)?;
        }
        run(
            "incus",
            &["restore".to_string(), name.to_string(), tag.to_string()],
            Path::new("/tmp"),
            180,
        )
        .map_err(|e| format!("incus restore {name}/{tag}: {e}"))?;
        Ok(())
    }

    /// `incus export {name}/{tag} {path}` — off-machine copy of a snapshot
    /// into `VIBE_BACKUP_DIR` (#773); returns the export target path.
    pub(crate) fn linux_export(&self, name: &str, tag: &str, target: &str) -> Result<String, String> {
        self.skip_if_unavailable()?;
        run(
            "incus",
            &["export".to_string(), format!("{name}/{tag}"), target.to_string()],
            Path::new("/tmp"),
            300,
        )
        .map_err(|e| format!("incus export {name}/{tag}: {e}"))?;
        Ok(target.to_string())
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