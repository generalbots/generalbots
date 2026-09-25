//! `vm_incus::linux` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const VM_UNAVAILABLE: &str = "vm-skip: incus binary unavailable";

impl VmLifecycle {
    pub(crate) fn skip_if_unavailable(&self) -> Result<(), String> {
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
                    .map(|i| {
                        // `incus list --format json` exposes the plain status
                        // as the top-level `status` field; `state` is a nested
                        // object (state.status), not a string.
                        i["status"]
                            .as_str()
                            .unwrap_or("")
                            .eq_ignore_ascii_case("running")
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false))
    }

    pub(crate) fn linux_create(&self, name: &str, tier: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        let image =
            std::env::var("VIBE_VM_IMAGE").unwrap_or_else(|_| "images:debian/13".to_string());
        let (cpu, mem) = match tier {
            "medium" => ("2", "2GiB"),
            "large" => ("4", "4GiB"),
            _ => ("1", "1GiB"),
        };
        // `incus launch` takes config as `--config key=value` flags; passing
        // bare `key=value` positionals is rejected ("Invalid number of
        // arguments"), which silently left VMs marked running without a
        // container.
        let args = [
            "launch".to_string(),
            image,
            name.to_string(),
            "--config".to_string(),
            format!("limits.cpu={cpu}"),
            "--config".to_string(),
            format!("limits.memory={mem}"),
            "--config".to_string(),
            "environment.VIBE_PROJECT=1".to_string(),
        ];
        self.incus_run(&args, 120)
            .map_err(|e| format!("incus launch {name} (tier {tier}): {e}"))?;
        // Attach the container to the managed bridge. `incus launch` only
        // applies the default profile, which typically carries a root disk
        // but no NIC — leaving the VM with loopback only and no DNS, so the
        // deployment API (bot.incus) is unreachable from inside the VM.
        // Prefer the default managed bridge (Incus names it `incusbr0`);
        // fall back to `incusbr0` when detection yields nothing.
        let bridge = self
            .incus_run(
                &[
                    "network".to_string(),
                    "list".to_string(),
                    "--format".to_string(),
                    "csv".to_string(),
                ],
                30,
            )
            .ok()
            .and_then(|out| {
                let rows: Vec<Vec<String>> = out
                    .stdout
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| {
                        line.split(',')
                            .map(|col| col.trim().to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect();
                // CSV columns: name,type,managed,ipv4,ipv6,description,usedby,state
                // Prefer the default managed bridge (`incusbr0`), then any
                // managed bridge; never a physical NIC (`type=physical`).
                rows.iter()
                    .find(|r| r.first().map(String::as_str) == Some("incusbr0"))
                    .or_else(|| {
                        rows.iter().find(|r| {
                            r.get(1).map(String::as_str) == Some("bridge")
                        })
                    })
                    .and_then(|r| r.first().cloned())
            })
            .unwrap_or_else(|| "incusbr0".to_string());
        let nic_args = [
            "config".to_string(),
            "device".to_string(),
            "add".to_string(),
            name.to_string(),
            "eth0".to_string(),
            "nic".to_string(),
            format!("network={bridge}"),
            "name=eth0".to_string(),
        ];
        self.incus_run(&nic_args, 30).map_err(|e| {
            format!("incus config device add eth0 to {name} (bridge {bridge}): {e}")
        })?;
        // Pre-create the app working directory so the project terminal
        // (`incus exec --cwd /opt/vibe/app`) and the publish flow never hit a
        // missing-directory error on a fresh container.
        let mkdir_args = [
            "exec".to_string(),
            name.to_string(),
            "--".to_string(),
            "mkdir".to_string(),
            "-p".to_string(),
            "/opt/vibe/app".to_string(),
        ];
        if let Err(e) = self.incus_run(&mkdir_args, 60) {
            log::warn!("Vibe: pre-create /opt/vibe/app in {name} failed (will retry later): {e}");
        }
        Ok(())
    }

    pub(crate) fn linux_start(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        self.incus_run(&["start".to_string(), name.to_string()], 60)
            .map_err(|e| format!("incus start {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_stop(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        self.incus_run(&["stop".to_string(), name.to_string()], 60)
            .map_err(|e| format!("incus stop {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_delete(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        self.incus_run(
            &[
                "delete".to_string(),
                "--force".to_string(),
                name.to_string(),
            ],
            60,
        )
        .map_err(|e| format!("incus delete {name}: {e}"))?;
        Ok(())
    }

    pub(crate) fn linux_restart(&self, name: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        self.incus_run(&["restart".to_string(), name.to_string()], 120)
            .map_err(|e| format!("incus restart {name}: {e}"))?;
        Ok(())
    }

    /// `incus snapshot create {name} {tag}` — point-in-time VM backup (#773).
    pub(crate) fn linux_snapshot(&self, name: &str, tag: &str) -> Result<(), String> {
        self.skip_if_unavailable()?;
        self.incus_run(
            &[
                "snapshot".to_string(),
                "create".to_string(),
                format!("{name}/{tag}"),
            ],
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
        self.incus_run(
            &["restore".to_string(), name.to_string(), tag.to_string()],
            180,
        )
        .map_err(|e| format!("incus restore {name}/{tag}: {e}"))?;
        Ok(())
    }

    /// `incus export {name}/{tag} {path}` — off-machine copy of a snapshot
    /// into `VIBE_BACKUP_DIR` (#773); returns the export target path.
    pub(crate) fn linux_export(
        &self,
        name: &str,
        tag: &str,
        target: &str,
    ) -> Result<String, String> {
        self.skip_if_unavailable()?;
        self.incus_run(
            &[
                "export".to_string(),
                format!("{name}/{tag}"),
                target.to_string(),
            ],
            300,
        )
        .map_err(|e| format!("incus export {name}/{tag}: {e}"))?;
        Ok(target.to_string())
    }

    pub(crate) fn linux_list(&self) -> Result<serde_json::Value, String> {
        let out = self
            .incus_run(
                &[
                    "list".to_string(),
                    "--format".to_string(),
                    "json".to_string(),
                ],
                30,
            )
            .map_err(|e| format!("incus list: {e}"))?;
        serde_json::from_str(&out.stdout).map_err(|e| format!("incus list parse: {e}"))
    }
}
