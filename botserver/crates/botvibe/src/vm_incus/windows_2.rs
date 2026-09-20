//! `vm_incus::windows_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

impl VmLifecycle {
    /// Deploy a Node workspace into the WSL-hosted Incus container and expose
    /// it through a localhost port. Native Linux keeps using the ALM/CI path.
    #[cfg(target_os = "windows")]
    pub(crate) fn deploy_node_files(
        &self,
        name: &str,
        files: &[serde_json::Value],
        host_port: u16,
        database_url: Option<&str>,
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
            let db_line = database_url
                .map(|url| format!("\nEnvironment=DATABASE_URL={url}"))
                .unwrap_or_default();
            std::fs::write(
                &service,
                format!(
                    "[Unit]\nDescription=Vibe application\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000{db_line}\nExecStart=/usr/bin/node /opt/vibe/app/index.js\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n"
                ),
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
        database_url: Option<&str>,
    ) -> Result<String, String> {
        self.deploy_node_files(name, files, host_port, database_url)
    }
}
