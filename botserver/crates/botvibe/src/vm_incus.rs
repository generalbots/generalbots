//! #744 — Incus CLI driver for the VM lifecycle (host-side container ops).
//! Runs the `incus` CLI via the harness command guard so every invocation
//! is allowlisted and argument-validated; containers are visible through
//! `incus list` (Verifiable per #744 DoD).
//!
//! Dual-platform: on Linux the `incus` binary runs directly; on Windows the
//! same CLI runs inside an explicit WSL2 distro, with automatic first-run
//! provisioning of WSL2 + Debian + the Incus package. Set
//! `GBO_WSL_DISTRO` to select another installed distro.

use std::sync::OnceLock;

use crate::harness::cmd::{run, spawn_persistent, GuardError, RunOutput};
use crate::vm_lifecycle::VmLifecycle;

const VM_UNAVAILABLE: &str = "vm-skip: incus binary unavailable";

/// Working directory for driver commands: `/tmp` on Linux, the OS temp dir
/// on Windows (where `/tmp` does not exist).
fn driver_cwd() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::temp_dir()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::path::PathBuf::from("/tmp")
    }
}

fn checked_run(
    program: &str,
    args: &[String],
    cwd: &std::path::Path,
    timeout: u64,
) -> Result<RunOutput, GuardError> {
    let output = run(program, args, cwd, timeout)?;
    if output.exit_code == Some(0) {
        return Ok(output);
    }
    let detail = output
        .stderr
        .lines()
        .chain(output.stdout.lines())
        .find(|line| !line.trim().is_empty())
        .unwrap_or("command failed")
        .chars()
        .take(300)
        .collect::<String>();
    Err(GuardError::Io(format!(
        "{program} exited with {:?}: {detail}",
        output.exit_code
    )))
}

#[cfg(target_os = "windows")]
fn wsl_distro() -> String {
    std::env::var("GBO_WSL_DISTRO")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Debian".to_string())
}

#[cfg(target_os = "windows")]
fn wsl_exec_args(command: &[String]) -> Vec<String> {
    let mut args = vec![
        "-d".to_string(),
        wsl_distro(),
        "-u".to_string(),
        "root".to_string(),
        "--".to_string(),
    ];
    args.extend(command.iter().cloned());
    args
}

#[cfg(target_os = "windows")]
fn ensure_wsl_keepalive() -> Result<(), String> {
    static KEEPALIVE: OnceLock<std::sync::Mutex<Option<std::process::Child>>> = OnceLock::new();
    let mut child = KEEPALIVE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .map_err(|_| "WSL keepalive lock poisoned".to_string())?;
    if let Some(existing) = child.as_mut() {
        match existing.try_wait() {
            Ok(None) => return Ok(()),
            Ok(Some(_)) => *child = None,
            Err(e) => return Err(format!("inspect WSL keepalive: {e}")),
        }
    }
    let command = ["/bin/sleep".to_string(), "infinity".to_string()];
    let spawned = spawn_persistent("wsl", &wsl_exec_args(&command), &driver_cwd())
        .map_err(|e| format!("start Debian WSL keepalive: {e}"))?;
    *child = Some(spawned);
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_path_to_wsl(path: &std::path::Path) -> Result<String, String> {
    let text = path.to_string_lossy().replace('\\', "/");
    let bytes = text.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'/' {
        return Err(format!("cannot map Windows path into WSL: {text}"));
    }
    let drive = (bytes[0] as char).to_ascii_lowercase();
    Ok(format!("/mnt/{drive}/{}", &text[3..]))
}

impl VmLifecycle {
    /// Run an `incus` invocation on the current platform.
    fn incus_run(&self, args: &[String], timeout: u64) -> Result<RunOutput, GuardError> {
        #[cfg(target_os = "windows")]
        {
            let mut command = vec!["incus".to_string()];
            command.extend(args.iter().cloned());
            checked_run("wsl", &wsl_exec_args(&command), &driver_cwd(), timeout)
        }
        #[cfg(not(target_os = "windows"))]
        {
            checked_run("incus", args, &driver_cwd(), timeout)
        }
    }

    /// Provision WSL2 + the selected distro + Incus automatically (Windows only).
    fn ensure_incus_wsl(&self) -> Result<(), String> {
        let cwd = driver_cwd();
        let distro = wsl_distro();

        if checked_run("wsl", &["--status".to_string()], &cwd, 30).is_err() {
            checked_run(
                "wsl",
                &["--set-default-version".to_string(), "2".to_string()],
                &cwd,
                30,
            )
            .ok();
        }

        if checked_run("wsl", &wsl_exec_args(&["true".to_string()]), &cwd, 30).is_err() {
            checked_run(
                "wsl",
                &[
                    "--install".to_string(),
                    "-d".to_string(),
                    distro.clone(),
                    "--no-launch".to_string(),
                ],
                &cwd,
                600,
            )
            .map_err(|e| {
                format!("wsl --install -d {distro} failed: {e} (a reboot may be required)")
            })?;
        }

        if checked_run(
            "wsl",
            &wsl_exec_args(&["incus".to_string(), "version".to_string()]),
            &cwd,
            30,
        )
        .is_ok()
        {
            return ensure_wsl_keepalive();
        }

        checked_run(
            "wsl",
            &wsl_exec_args(&["apt-get".to_string(), "update".to_string()]),
            &cwd,
            300,
        )
        .map_err(|e| format!("apt-get update inside {distro} failed: {e}"))?;
        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "apt-get".to_string(),
                "install".to_string(),
                "-y".to_string(),
                "incus".to_string(),
            ]),
            &cwd,
            600,
        )
        .map_err(|e| format!("apt-get install incus inside {distro} failed: {e}"))?;
        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "systemctl".to_string(),
                "enable".to_string(),
                "--now".to_string(),
                "incus".to_string(),
            ]),
            &cwd,
            120,
        )
        .map_err(|e| format!("starting Incus inside {distro} failed: {e}"))?;

        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "incus".to_string(),
                "admin".to_string(),
                "init".to_string(),
                "--minimal".to_string(),
            ]),
            &cwd,
            300,
        )
        .map_err(|e| format!("incus admin init --minimal failed in {distro}: {e}"))?;
        ensure_wsl_keepalive()
    }

    pub(crate) fn linux_available(&self) -> bool {
        if std::env::var("VIBE_INCUS_FORCE_UNAVAILABLE").as_deref() == Ok("1") {
            return false;
        }
        #[cfg(target_os = "windows")]
        {
            static WSL_INCUS_READY: OnceLock<bool> = OnceLock::new();
            *WSL_INCUS_READY.get_or_init(|| {
                let _ = self.ensure_incus_wsl();
                checked_run(
                    "wsl",
                    &wsl_exec_args(&["incus".to_string(), "version".to_string()]),
                    &driver_cwd(),
                    30,
                )
                .is_ok()
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            checked_run("incus", &["version".to_string()], &driver_cwd(), 5).is_ok()
        }
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

    /// Resolve the container's primary IPv4 address from `incus list` JSON
    /// (`state.network.eth0.addresses[].address` where family=inet). The host
    /// cannot resolve `{container}.incus` DNS names, so the health probe must
    /// dial the real IP. On Windows/WSL2 the address is only reachable from
    /// inside the WSL2 VM unless a proxy device is configured.
    pub(crate) fn linux_ip(&self, name: &str) -> Result<Option<String>, String> {
        self.skip_if_unavailable()?;
        let list = self.linux_list()?;
        Ok(list
            .as_array()
            .and_then(|arr| arr.iter().find(|i| i["name"].as_str() == Some(name)))
            .and_then(|i| i["state"]["network"]["eth0"]["addresses"].as_array())
            .and_then(|addrs| {
                addrs
                    .iter()
                    .find(|a| a["family"].as_str() == Some("inet"))
                    .and_then(|a| a["address"].as_str().map(str::to_string))
            }))
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
        Ok(())
    }

    /// Deploy a Node workspace into the WSL-hosted Incus container and expose
    /// it through a localhost port. Native Linux keeps using the ALM/CI path.
    #[cfg(target_os = "windows")]
    pub(crate) fn deploy_node_files(
        &self,
        name: &str,
        files: &[serde_json::Value],
        host_port: u16,
    ) -> Result<String, String> {
        self.skip_if_unavailable()?;
        let temp = std::env::temp_dir().join(format!("vibe-deploy-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).map_err(|e| format!("create deploy temp dir: {e}"))?;

        let result = (|| -> Result<String, String> {
            self.incus_run(
                &[
                    "exec".to_string(),
                    name.to_string(),
                    "--".to_string(),
                    "mkdir".to_string(),
                    "-p".to_string(),
                    "/opt/vibe/app".to_string(),
                ],
                30,
            )
            .map_err(|e| format!("prepare app directory: {e}"))?;

            for file in files {
                let rel = file
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "deployment file is missing path".to_string())?;
                let rel_path = std::path::Path::new(rel);
                if rel_path.is_absolute()
                    || rel_path
                        .components()
                        .any(|part| matches!(part, std::path::Component::ParentDir))
                {
                    return Err(format!("invalid deployment path '{rel}'"));
                }
                let bytes = file
                    .get("content")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| format!("deployment file '{rel}' has invalid content"))?
                    .iter()
                    .map(|v| {
                        v.as_u64()
                            .filter(|n| *n <= u8::MAX as u64)
                            .map(|n| n as u8)
                            .ok_or_else(|| format!("deployment file '{rel}' has invalid byte"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let source = temp.join(rel_path);
                if let Some(parent) = source.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("create deploy path for '{rel}': {e}"))?;
                }
                std::fs::write(&source, bytes)
                    .map_err(|e| format!("write deploy file '{rel}': {e}"))?;
                let source_wsl = windows_path_to_wsl(&source)?;
                let destination = format!("{name}/opt/vibe/app/{}", rel.replace('\\', "/"));
                self.incus_run(
                    &[
                        "file".to_string(),
                        "push".to_string(),
                        source_wsl,
                        destination,
                        "--create-dirs".to_string(),
                    ],
                    60,
                )
                .map_err(|e| format!("push deploy file '{rel}': {e}"))?;
            }

            let service = temp.join("vibe-app.service");
            std::fs::write(
                &service,
                "[Unit]\nDescription=Vibe application\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000\nExecStart=/usr/bin/node /opt/vibe/app/index.js\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n",
            )
            .map_err(|e| format!("write service unit: {e}"))?;
            self.incus_run(
                &[
                    "file".to_string(),
                    "push".to_string(),
                    windows_path_to_wsl(&service)?,
                    format!("{name}/etc/systemd/system/vibe-app.service"),
                    "--create-dirs".to_string(),
                ],
                60,
            )
            .map_err(|e| format!("push service unit: {e}"))?;

            if self
                .incus_run(
                    &[
                        "exec".to_string(),
                        name.to_string(),
                        "--".to_string(),
                        "node".to_string(),
                        "--version".to_string(),
                    ],
                    30,
                )
                .is_err()
            {
                for command in [
                    vec!["apt-get", "update"],
                    vec!["apt-get", "install", "-y", "nodejs", "npm"],
                ] {
                    let mut args = vec!["exec".to_string(), name.to_string(), "--".to_string()];
                    args.extend(command.into_iter().map(str::to_string));
                    self.incus_run(&args, 300)
                        .map_err(|e| format!("install Node runtime: {e}"))?;
                }
            }

            self.incus_run(
                &[
                    "exec".to_string(),
                    name.to_string(),
                    "--".to_string(),
                    "node".to_string(),
                    "/opt/vibe/app/test.js".to_string(),
                ],
                60,
            )
            .map_err(|e| format!("calculator tests failed: {e}"))?;
            for command in [
                vec!["systemctl", "daemon-reload"],
                vec!["systemctl", "enable", "vibe-app.service"],
                vec!["systemctl", "restart", "vibe-app.service"],
            ] {
                let mut args = vec!["exec".to_string(), name.to_string(), "--".to_string()];
                args.extend(command.into_iter().map(str::to_string));
                self.incus_run(&args, 60)
                    .map_err(|e| format!("start Vibe application: {e}"))?;
            }

            let devices = self
                .incus_run(
                    &[
                        "config".to_string(),
                        "device".to_string(),
                        "show".to_string(),
                        name.to_string(),
                    ],
                    30,
                )
                .map_err(|e| format!("inspect proxy device: {e}"))?;
            if devices.stdout.contains("vibe-http:") {
                self.incus_run(
                    &[
                        "config".to_string(),
                        "device".to_string(),
                        "remove".to_string(),
                        name.to_string(),
                        "vibe-http".to_string(),
                    ],
                    30,
                )
                .map_err(|e| format!("replace proxy device: {e}"))?;
            }
            self.incus_run(
                &[
                    "config".to_string(),
                    "device".to_string(),
                    "add".to_string(),
                    name.to_string(),
                    "vibe-http".to_string(),
                    "proxy".to_string(),
                    format!("listen=tcp:0.0.0.0:{host_port}"),
                    "connect=tcp:127.0.0.1:3000".to_string(),
                ],
                60,
            )
            .map_err(|e| format!("expose application port: {e}"))?;

            Ok(format!("http://localhost:{host_port}"))
        })();

        if let Err(e) = std::fs::remove_dir_all(&temp) {
            log::warn!("Vibe: failed to remove deploy temp dir: {e}");
        }
        result
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

    fn linux_list(&self) -> Result<serde_json::Value, String> {
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
