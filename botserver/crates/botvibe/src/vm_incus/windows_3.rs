//! `vm_incus::windows_3` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

impl VmLifecycle {
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
        database_url: Option<&str>,
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

            // #1276 — entry resolution (node/python/static precedence) is
            // extracted into `resolve_web_entry` so the clobber rules stay
            // unit-tested without a container.
            let resolved = resolve_web_entry(files);
            let has_node_entry = !resolved.is_python && !resolved.needs_static_fallback;
            let has_python_entry = resolved.is_python;
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

            // Pick the web entry + runtime (extracted + unit-tested — #1276).
            // This drives the systemd ExecStart and the runtime bootstrap
            // below.
            let is_python = resolved.is_python;
            let entry = resolved.entry;
            let service = temp.join("vibe-app.service");
            // #1386 — the project's own database URL is injected into the
            // unit so app code reaches its per-environment database through
            // the standard `DATABASE_URL` convention.
            let db_line = database_url
                .map(|url| format!("\nEnvironment=DATABASE_URL={url}"))
                .unwrap_or_default();
            std::fs::write(
                &service,
                format!(
                    "[Unit]\nDescription=Vibe application\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000{db_line}\nExecStart={} /opt/vibe/app/{entry}\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n",
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

            // Install project dependencies when a package.json exists (the
            // same trigger the python branch uses for requirements.txt) so a
            // Run serves the real app even when the agent never ran npm
            // install itself. Tolerated failure: the static fallback probe
            // below still rescues projects whose deps cannot resolve.
            // #1273 — skipped when both manifests are byte-identical to the
            // previous Run: the hash file written after a successful install
            // turns every subsequent Run into a no-op instead of re-paying
            // the npm tax on slow networks.
            if files
                .iter()
                .any(|f| f.get("path").and_then(|v| v.as_str()) == Some("package.json"))
            {
                let mut hasher_input = Vec::new();
                for manifest in ["package.json", "package-lock.json"] {
                    if let Some(f) = files
                        .iter()
                        .find(|f| f.get("path").and_then(|v| v.as_str()) == Some(manifest))
                    {
                        if let Some(content) = f.get("content").and_then(|v| v.as_array()) {
                            for b in content {
                                hasher_input.push(b.as_u64().unwrap_or(0) as u8);
                            }
                        }
                    }
                    hasher_input.push(b'\n');
                }
                let deps_hash = format!("{:x}", Sha256::digest(&hasher_input));
                let check = self.incus_run(
                    &[
                        "exec".to_string(),
                        name.to_string(),
                        "--".to_string(),
                        "cat".to_string(),
                        "/opt/vibe/app/.vibe-deps-hash".to_string(),
                    ],
                    15,
                );
                let unchanged = check
                    .map(|out| out.stdout.trim() == deps_hash)
                    .unwrap_or(false);
                if unchanged {
                    log::info!("Vibe: {name} dependencies unchanged — skipping npm install");
                } else {
                    let args = vec![
                        "exec".to_string(),
                        name.to_string(),
                        "--env".to_string(),
                        "DEBIAN_FRONTEND=noninteractive".to_string(),
                        "--".to_string(),
                        "npm".to_string(),
                        "install".to_string(),
                        "--prefix".to_string(),
                        "/opt/vibe/app".to_string(),
                        "--no-audit".to_string(),
                        "--no-fund".to_string(),
                    ];
                    if let Err(e) = self.incus_run(&args, 300) {
                        log::warn!("Vibe: {name} npm install failed: {e}");
                    } else {
                        // Record the hash only after a successful install so
                        // a failed attempt retries on the next Run.
                        let _ = self.incus_run(
                            &[
                                "exec".to_string(),
                                name.to_string(),
                                "--".to_string(),
                                "sh".to_string(),
                                "-c".to_string(),
                                format!(
                                    "printf %s {} > /opt/vibe/app/.vibe-deps-hash",
                                    deps_hash
                                ),
                            ],
                            15,
                        );
                    }
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
            let probe_path = temp.join(format!("vibe-{probe_file}"));
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
                let db_line = database_url
                    .map(|url| format!("\nEnvironment=DATABASE_URL={url}"))
                    .unwrap_or_default();
                let unit = format!(
                    "[Unit]\nDescription=Vibe application (static)\nAfter=network.target\n\n[Service]\nType=simple\nWorkingDirectory=/opt/vibe/app\nEnvironment=PORT=3000{db_line}\nExecStart=/usr/bin/node /opt/vibe/app/server.js\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n"
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
}
