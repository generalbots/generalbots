//! #744 — Incus CLI driver for the VM lifecycle (host-side container ops).
//! Runs the `incus` CLI via the harness command guard so every invocation
//! is allowlisted and argument-validated; containers are visible through
//! `incus list` (Verifiable per #744 DoD).
//!
//! Dual-platform: on Linux the `incus` binary runs directly; on Windows the
//! same CLI runs inside an explicit WSL2 distro, with automatic first-run
//! provisioning of WSL2 + Debian + the Incus package. Set
//! `GBO_WSL_DISTRO` to select another installed distro.

#[cfg(target_os = "windows")]
use std::sync::OnceLock;

use crate::harness::cmd::{run, GuardError, RunOutput};
#[cfg(target_os = "windows")]
use crate::harness::cmd::spawn_persistent;
use crate::vm_lifecycle::VmLifecycle;

const VM_UNAVAILABLE: &str = "vm-skip: incus binary unavailable";

/// Health-check probe pushed into the dev container and run with `node`.
/// Exits 0 as soon as something listens on 127.0.0.1:3000, otherwise after
/// `attempts` tries with a 1s pause each. Kept as a FILE (not a `bash -c`
/// string) because the command guard rejects shell metacharacters in
/// arguments: `;`, `$`, `>` and `&` would make an inline probe fail to
/// spawn and wrongly classify every app as "not listening".
const HEALTH_PROBE_JS: &str = r#"'use strict';
const net = require('net');
const attempts = Number(process.argv[2] || 20);
let tried = 0;
function probe() {
  if (tried >= attempts) { process.exit(1); }
  tried += 1;
  const sock = net.connect(3000, '127.0.0.1');
  sock.on('connect', () => { sock.destroy(); process.exit(0); });
  sock.on('error', () => { sock.destroy(); setTimeout(probe, 1000); });
}
probe();
"#;

/// Same TCP :3000 liveness probe in Python for runtime-python projects (the
/// base image may not have node installed, so the node probe would always
/// fail and wrongly trigger a static fallback).
const HEALTH_PROBE_PYTHON: &str = r#"import socket, sys, time
attempts = int(sys.argv[1] if len(sys.argv) > 1 else 20)
tried = 0
while tried < attempts:
    try:
        s = socket.create_connection(('127.0.0.1', 3000), timeout=1)
    except OSError:
        tried += 1
        time.sleep(1)
        continue
    s.close()
    sys.exit(0)
sys.exit(1)
"#;

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
    #[cfg(target_os = "windows")]
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
                // Fresh containers have no TTY; debconf's Dialog frontend
                // aborts with exit 100 unless the frontend is noninteractive.
                for command in [
                    vec!["apt-get", "update"],
                    vec!["apt-get", "install", "-y", "nodejs", "npm"],
                ] {
                    let mut args = vec!["exec".to_string(), name.to_string()];
                    args.push("--env".to_string());
                    args.push("DEBIAN_FRONTEND=noninteractive".to_string());
                    args.push("--".to_string());
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
            // host_port == 0: prod publish path — no proxy device is attached
            // (the app is served through the Caddy domain route → container
            // IP:3000). Attaching a device with the previous default of 80
            // failed with `bind: address already in use` on prod.
            if host_port != 0 {
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
                if host_port == 80 {
                    Ok("http://localhost".to_string())
                } else {
                    Ok(format!("http://localhost:{host_port}"))
                }
            } else {
                Ok("http://localhost:3000".to_string())
            }
        })();

        if let Err(e) = std::fs::remove_dir_all(&temp) {
            log::warn!("Vibe: failed to remove deploy temp dir: {e}");
        }
        result
    }

    /// Windows variant: the existing `deploy_node_files` flow already runs
    /// the app in the WSL-hosted Incus container; `run_dev_app` is the
    /// platform-neutral entry point used by the projects API.
    #[cfg(target_os = "windows")]
    pub(crate) fn run_dev_app(
        &self,
        name: &str,
        files: &[serde_json::Value],
        host_port: u16,
    ) -> Result<String, String> {
        self.deploy_node_files(name, files, host_port)
    }

    /// Run the project's own app inside the dev container as a REAL process
    /// (visible in the project terminal's `ps`), exposed through a host proxy
    /// device. This is the Linux equivalent of the Windows `deploy_node_files`
    /// flow: workspace files are pushed into `/opt/vibe/app`, node is
    /// installed when missing, the app is started as a systemd service, and a
    /// `vibe-http` proxy device maps `host_port` → 127.0.0.1:3000.
    ///
    /// The entry point is `index.js` when present; otherwise a minimal node
    /// static file server is generated so a node process always runs (the
    /// user's `ps` complaint: the browser showed the app but nothing was
    /// running on the VM).
    #[cfg(not(target_os = "windows"))]
    pub(crate) fn run_dev_app(
        &self,
        name: &str,
        files: &[serde_json::Value],
        host_port: u16,
    ) -> Result<String, String> {
        self.skip_if_unavailable()?;
        if !self.linux_running(name)? {
            self.linux_start(name)?;
        }
        let temp = std::env::temp_dir().join(format!("vibe-run-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).map_err(|e| format!("create run temp dir: {e}"))?;

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

            let has_index_js = files.iter().any(|f| {
                f.get("path")
                    .and_then(|v| v.as_str())
                    .map(|p| p == "index.js" || p.ends_with("/index.js"))
                    .unwrap_or(false)
            });
            // A node-framework project (framework type "node") may designate
            // its web entry point as `server.js` instead of `index.js` (or a
            // package.json "start" script). Treat a user-provided `server.js`
            // as a REAL web entrypoint so it is not clobbered by the generated
            // static-only fallback server below — otherwise an Express/Node
            // app silently degrades to the "No web app in this project yet"
            // page instead of running.
            let has_server_js = files.iter().any(|f| {
                f.get("path")
                    .and_then(|v| v.as_str())
                    .map(|p| p == "server.js" || p.ends_with("/server.js"))
                    .unwrap_or(false)
            });
            let has_node_entry = has_index_js || has_server_js;
            // A python-framework project (framework type "python") designates
            // its web entry point with a python module (app.py / server.py /
            // main.py) instead of index.js. Treat a user-provided python
            // entry as a REAL web entrypoint too, so a Flask/FastAPI app is
            // started and kept alive (Restart=always) rather than being
            // clobbered by the node static-only fallback below.
            let has_python_entry = files.iter().any(|f| {
                let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let name = path.rsplit('/').next().unwrap_or(path);
                matches!(
                    name,
                    "app.py" | "server.py" | "main.py" | "api.py" | "run.py"
                )
            });
            // Static fallback server: a node process that serves the workspace
            // files over HTTP. Used when the project has no index.js AND when
            // the real entry is not a web server (a CLI that exits, or a crash)
            // so the browser never opens against a dead port.
            let server_js = concat!(
                "const http=require('http'),fs=require('fs'),path=require('path');\n",
                "const root='/opt/vibe/app';\n",
                "const mime={'.html':'text/html','.css':'text/css','.js':'text/javascript','.mjs':'text/javascript','.json':'application/json','.svg':'image/svg+xml','.png':'image/png','.jpg':'image/jpeg','.gif':'image/gif','.ico':'image/x-icon','.woff2':'font/woff2','.woff':'font/woff','.ttf':'font/ttf','.txt':'text/plain'};\n",
                "http.createServer((req,res)=>{\n",
                "  let p=path.join(root,decodeURIComponent((req.url||'/').split('?')[0]));\n",
                "  if(p.endsWith('/'))p=path.join(p,'index.html');\n",
                "  fs.readFile(p,(e,d)=>{ if(e){\n",
                "    // No web entry point: show a helpful page instead of a dead 404 so\n",
                "    // Run never opens an empty browser for an app-less project.\n",
                "    const ents=fs.existsSync(root)?fs.readdirSync(root).map(f=>'<li>'+f+'</li>').join(''):'<li>(empty workspace)</li>';\n",
                "    const html='<!doctype html><html><head><meta charset=\"utf-8\"><title>No web app yet</title><style>body{font:16px system-ui;background:#0d1117;color:#e6edf3;display:grid;place-items:center;min-height:100vh;margin:0}main{max-width:560px;padding:28px;border:1px solid #30363d;border-radius:16px;background:#161b22}h1{color:#84d669}p{color:#8b949e}code{background:#0d1117;padding:2px 6px;border-radius:6px}ul{color:#8b949e}</style></head><body><main><h1>No web app in this project yet</h1><p>This project has no <code>index.html</code> entry point. Ask the Vibe agent in Chat to build one, then press Run again.</p><ul>'+ents+'</ul></main></body></html>';\n",
                "    res.writeHead(404,{'Content-Type':'text/html; charset=utf-8'});res.end(html);return;} \n",
                "    res.writeHead(200,{'Content-Type':mime[path.extname(p)]||'application/octet-stream'});res.end(d);});\n",
                "}).listen(3000);\n",
            );
            for file in files {
                let rel = file
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "run file is missing path".to_string())?;
                let rel_path = std::path::Path::new(rel);
                if rel_path.is_absolute()
                    || rel_path
                        .components()
                        .any(|part| matches!(part, std::path::Component::ParentDir))
                {
                    return Err(format!("invalid run path '{rel}'"));
                }
                let bytes = file
                    .get("content")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| format!("run file '{rel}' has invalid content"))?
                    .iter()
                    .map(|v| {
                        v.as_u64()
                            .filter(|n| *n <= u8::MAX as u64)
                            .map(|n| n as u8)
                            .ok_or_else(|| format!("run file '{rel}' has invalid byte"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let source = temp.join(rel_path);
                if let Some(parent) = source.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("create run path for '{rel}': {e}"))?;
                }
                std::fs::write(&source, bytes)
                    .map_err(|e| format!("write run file '{rel}': {e}"))?;
                let destination = format!("{name}/opt/vibe/app/{}", rel.replace('\\', "/"));
                self.incus_run(
                    &[
                        "file".to_string(),
                        "push".to_string(),
                        source.to_string_lossy().into_owned(),
                        destination,
                        "--create-dirs".to_string(),
                    ],
                    60,
                )
                .map_err(|e| format!("push run file '{rel}': {e}"))?;
            }

            // Static apps (pure htmx/html) with no node entrypoint get a
            // generated node server so a node process is what shows up in the
            // terminal's `ps`. When the project ships its own server.js (node)
            // or a python entry it is honored (has_node_entry /
            // has_python_entry), not overwritten.
            if !has_node_entry && !has_python_entry {
                let server_path = temp.join("server.js");
                std::fs::write(&server_path, server_js)
                    .map_err(|e| format!("write static server: {e}"))?;
                self.incus_run(
                    &[
                        "file".to_string(),
                        "push".to_string(),
                        server_path.to_string_lossy().into_owned(),
                        format!("{name}/opt/vibe/app/server.js"),
                        "--create-dirs".to_string(),
                    ],
                    60,
                )
                .map_err(|e| format!("push static server: {e}"))?;
            }

            // Pick the web entry + runtime. Node projects run index.js (or a
            // custom server.js) with /usr/bin/node; python projects run their
            // detected module with /usr/bin/python3. Pure-static projects get
            // the generated server.js (node). This drives the systemd ExecStart
            // and the runtime bootstrap below.
            let is_python = has_python_entry && !has_node_entry;
            let entry = if is_python {
                files
                    .iter()
                    .filter_map(|f| {
                        let path = f.get("path").and_then(|v| v.as_str())?;
                        let name = path.rsplit('/').next().unwrap_or(path);
                        (matches!(name, "app.py" | "server.py" | "main.py" | "api.py" | "run.py")).then_some(name)
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .next()
                    .unwrap_or("app.py")
            } else if has_index_js {
                "index.js"
            } else {
                "server.js"
            };
            let service = temp.join("vibe-app.service");
            std::fs::write(
                &service,
                format!(
                    "[Unit]\nDescription=Vibe application\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000\nExecStart={} /opt/vibe/app/{entry}\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n",
                    if is_python { "/usr/bin/python3" } else { "/usr/bin/node" }
                ),
            )
            .map_err(|e| format!("write service unit: {e}"))?;
            self.incus_run(
                &[
                    "file".to_string(),
                    "push".to_string(),
                    service.to_string_lossy().into_owned(),
                    format!("{name}/etc/systemd/system/vibe-app.service"),
                    "--create-dirs".to_string(),
                ],
                60,
            )
            .map_err(|e| format!("push service unit: {e}"))?;

            // Bootstrap the project runtime. Node apps check for nodejs/npm;
            // python apps check for python3 (and pip so `requirements.txt`
            // deps resolve before the service starts). Static fallbacks run
            // with node, so a python-only VM still gets node for the generated
            // static server.
            if is_python {
                if self
                    .incus_run(
                        &[
                            "exec".to_string(),
                            name.to_string(),
                            "--".to_string(),
                            "python3".to_string(),
                            "--version".to_string(),
                        ],
                        30,
                    )
                    .is_err()
                {
                    // Fresh containers have no TTY; debconf's Dialog frontend
                    // aborts with exit 100 unless the frontend is noninteractive.
                    for command in [
                        vec!["apt-get", "update"],
                        vec!["apt-get", "install", "-y", "python3", "python3-pip"],
                    ] {
                        let mut args = vec!["exec".to_string(), name.to_string()];
                        args.push("--env".to_string());
                        args.push("DEBIAN_FRONTEND=noninteractive".to_string());
                        args.push("--".to_string());
                        args.extend(command.into_iter().map(str::to_string));
                        self.incus_run(&args, 300)
                            .map_err(|e| format!("install Python runtime: {e}"))?;
                    }
                }
                // Resolve project dependencies before the service starts.
                if files.iter().any(|f| {
                    let p = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    p == "requirements.txt" || p.ends_with("/requirements.txt")
                }) {
                    let mut args = vec![
                        "exec".to_string(),
                        name.to_string(),
                        "--".to_string(),
                        "python3".to_string(),
                        "-m".to_string(),
                        "pip".to_string(),
                        "install".to_string(),
                        "-r".to_string(),
                        "/opt/vibe/app/requirements.txt".to_string(),
                    ];
                    // Some base images are externally-managed (PEP 668) and
                    // refuse system installs; --break-system-packages keeps a
                    // few-line python app running on Ubuntu 24.04 VMs.
                    args.push("--break-system-packages".to_string());
                    let _ = self.incus_run(&args, 300);
                }
            } else if self
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
                // Fresh containers have no TTY; debconf's Dialog frontend
                // aborts with exit 100 unless the frontend is noninteractive.
                for command in [
                    vec!["apt-get", "update"],
                    vec!["apt-get", "install", "-y", "nodejs", "npm"],
                ] {
                    let mut args = vec!["exec".to_string(), name.to_string()];
                    args.push("--env".to_string());
                    args.push("DEBIAN_FRONTEND=noninteractive".to_string());
                    args.push("--".to_string());
                    args.extend(command.into_iter().map(str::to_string));
                    self.incus_run(&args, 300)
                        .map_err(|e| format!("install Node runtime: {e}"))?;
                }
            }

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

            // Health check: if the project entry is not a web server (a CLI
            // that prints usage and exits, a crash, or a long build step)
            // nothing listens on :3000 and the browser would open against a
            // dead port. Fall back to the generated static server so the
            // browser always shows the workspace and `ps` shows a live node
            // process instead of a crash-loop.
            // The probe must live in a FILE: the command guard rejects shell
            // metacharacters in arguments, so `incus exec -- bash -c "..."`
            // (with `$`, `;`, `>`) is refused and the probe always fails,
            // which wrongly swapped every custom/node app to the static
            // server. A pushed probe script runs with clean arguments.
            let probe_file = if is_python { "healthcheck.py" } else { "healthcheck.js" };
            let probe_path = temp.join(&format!("vibe-{probe_file}"));
            let probe_src = if is_python { HEALTH_PROBE_PYTHON } else { HEALTH_PROBE_JS };
            std::fs::write(&probe_path, probe_src)
                .map_err(|e| format!("write health probe: {e}"))?;
            self.incus_run(
                &[
                    "file".to_string(),
                    "push".to_string(),
                    probe_path.to_string_lossy().into_owned(),
                    format!("{name}/opt/vibe/{probe_file}"),
                    "--create-dirs".to_string(),
                ],
                60,
            )
            .map_err(|e| format!("push health probe: {e}"))?;
            let probe_interp = if is_python { "python3" } else { "node" };
            let probe_dest = format!("/opt/vibe/{probe_file}");
            let listening = |attempts: u32| -> bool {
                self.incus_run(
                    &[
                        "exec".to_string(),
                        name.to_string(),
                        "--".to_string(),
                        probe_interp.to_string(),
                        probe_dest.clone(),
                        attempts.to_string(),
                    ],
                    60,
                )
                .is_ok()
            };
            if !listening(20) && has_node_entry {
                log::info!(
                    "Vibe: {name} entry is not a web server (nothing on :3000) — serving workspace statically"
                );
                let server_path = temp.join("server.js");
                std::fs::write(&server_path, server_js)
                    .map_err(|e| format!("write static fallback server: {e}"))?;
                self.incus_run(
                    &[
                        "file".to_string(),
                        "push".to_string(),
                        server_path.to_string_lossy().into_owned(),
                        format!("{name}/opt/vibe/app/server.js"),
                        "--create-dirs".to_string(),
                    ],
                    60,
                )
                .map_err(|e| format!("push static fallback server: {e}"))?;
                let unit = format!(
                    "[Unit]\nDescription=Vibe application (static)\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000\nExecStart=/usr/bin/node /opt/vibe/app/server.js\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n"
                );
                let unit_path = temp.join("vibe-app-static.service");
                std::fs::write(&unit_path, unit).map_err(|e| format!("write static unit: {e}"))?;
                self.incus_run(
                    &[
                        "file".to_string(),
                        "push".to_string(),
                        unit_path.to_string_lossy().into_owned(),
                        format!("{name}/etc/systemd/system/vibe-app.service"),
                        "--create-dirs".to_string(),
                    ],
                    60,
                )
                .map_err(|e| format!("push static unit: {e}"))?;
                for command in [
                    vec!["systemctl", "daemon-reload"],
                    vec!["systemctl", "restart", "vibe-app.service"],
                ] {
                    let mut args = vec!["exec".to_string(), name.to_string(), "--".to_string()];
                    args.extend(command.into_iter().map(str::to_string));
                    self.incus_run(&args, 60)
                        .map_err(|e| format!("start static fallback: {e}"))?;
                }
                if !listening(15) {
                    log::warn!("Vibe: {name} static fallback is not serving on :3000");
                }
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
            // host_port == 0: prod publish path — no proxy device is attached
            // (the app is served through the Caddy domain route → container
            // IP:3000). Attaching a device with the previous default of 80
            // failed with `bind: address already in use` on prod.
            if host_port != 0 {
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
                if host_port == 80 {
                    Ok("http://localhost".to_string())
                } else {
                    Ok(format!("http://localhost:{host_port}"))
                }
            } else {
                Ok("http://localhost:3000".to_string())
            }
        })();

        if let Err(e) = std::fs::remove_dir_all(&temp) {
            log::warn!("Vibe: failed to remove run temp dir: {e}");
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
