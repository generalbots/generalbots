//! `proxy_sites::caddy_deploy` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

// #2654 — private staging for the project DB URL lives in its own module so
// this file stays under the 450-line limit (AGENTS.md).
#[path = "secure_env.rs"]
mod secure_env;
use secure_env::{harden_proxy_env_file, PrivateTemp, SecureEnvFile};

/// Install (or refresh) the per-site python systemd service inside the
/// proxy: venv + dependency install + unit + start. Idempotent. The unit
/// name and payload dir come from the (slug, env) target so prod and dev
/// services of the same site run side by side on distinct ports.
pub(crate) fn ensure_python_service_for(
    slug: &str,
    site_dir: &str,
    port: u16,
    env: SiteEnv,
    database_url: Option<&str>,
) -> Result<u16, String> {
    check_python_runtime()?;
    let unit_name = site_unit_name(slug, env);

    // venv + deps (idempotent; pip resolves the locked set every publish).
    must_run(
        "python venv",
        &[
            "python3".to_string(),
            "-m".to_string(),
            "venv".to_string(),
            format!("{site_dir}/.venv"),
        ],
        180,
    )?;
    let requirements = format!("{site_dir}/requirements.txt");
    let has_req = proxy_exec(&["test".to_string(), "-f".to_string(), requirements.clone()], 15)?;
    if has_req.exit_code == Some(0) {
        must_run(
            "pip install",
            &[
                format!("{site_dir}/.venv/bin/python3"),
                "-m".to_string(),
                "pip".to_string(),
                "install".to_string(),
                "--no-input".to_string(),
                "--quiet".to_string(),
                "-r".to_string(),
                requirements,
            ],
            600,
        )?;
    }

    // systemd unit — pushed as a file (no shell needed). Unit name is
    // env-suffixed for the test twin (gb-vibe-{slug}-test) so it never
    // clashes with the production service of the same site.
    // #1386 — the project's database URL lives in a persistent
    // EnvironmentFile (`/etc/gb-vibe/{unit_name}.env`) written only by the
    // publish/promote path. Rollback rewrites the unit but NOT the env file,
    // so a rolled-back release keeps the last-published database URL without
    // needing database access during the rollback.
    let env_file = format!("/etc/gb-vibe/{unit_name}.env");
    if let Some(url) = database_url {
        let env_content = format!("DATABASE_URL={url}\n");
        // #2654 — never stage the DB URL (with its password) in a predictable,
        // world-readable /tmp file. The private 0700 dir + 0600 O_EXCL file are
        // owned by `env_tmp`; its Drop removes them on every exit path.
        let env_tmp = SecureEnvFile::create(&env_content)?;
        if let Err(e) = proxy_exec(
            &["mkdir".to_string(), "-p".to_string(), "/etc/gb-vibe".to_string()],
            15,
        ) {
            log::warn!("Vibe python service: create /etc/gb-vibe failed: {e}");
        }
        if let Err(e) = proxy_exec(
            &["rm".to_string(), "-f".to_string(), env_file.to_string()],
            15,
        ) {
            log::warn!("Vibe python service: remove stale env file failed: {e}");
        }
        let pushed_env = crate::harness::cmd::run(
            "incus",
            &[
                "file".to_string(),
                "push".to_string(),
                env_tmp.path().to_string_lossy().to_string(),
                format!("proxy{env_file}"),
            ],
            Path::new("."),
            30,
        )
        .map_err(|e| format!("incus file push env file: {e}"))?;
        if pushed_env.exit_code != Some(0) {
            // Generic message: never echo tool output that could carry the URL.
            return Err("env file push failed".to_string());
        }
        harden_proxy_env_file(&env_file)?;
    }
    let unit = format!(
        "[Unit]\nDescription=GB vibe site {unit_name}\nAfter=network.target\n\n[Service]\nWorkingDirectory={site_dir}\nEnvironment=PORT={port}\nEnvironmentFile=-{env_file}\nExecStart={site_dir}/.venv/bin/python {site_dir}/app.py\nRestart=always\nRestartSec=3\n\n[Install]\nWantedBy=multi-user.target\n"
    );
    let unit_tmp = PrivateTemp::create(&format!("{unit_name}.service"), unit.as_bytes())?;
    let unit_target = format!("proxy/etc/systemd/system/gb-vibe-{unit_name}.service");
    // Pre-delete the unit: `incus file push` cannot overwrite a file owned
    // by another uid (EACCES) — seen when a previous botserver ran as a
    // different user.
    if let Err(e) = proxy_exec(
        &["rm".to_string(), "-f".to_string(), format!("/etc/systemd/system/gb-vibe-{unit_name}.service")],
        15,
    ) {
        log::warn!("Vibe python service: pre-delete systemd unit failed: {e}");
    }
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            unit_tmp.path.to_string_lossy().to_string(),
            // no-colon form — see stage_payload.
            unit_target,
        ],
        Path::new("."),
        30,
    )
    .map_err(|e| format!("incus file push unit: {e}"))?;
    if pushed.exit_code != Some(0) {
        return Err(format!("unit push failed: {}", pushed.stderr.trim()));
    }
    must_run("daemon-reload", &["systemctl".to_string(), "daemon-reload".to_string()], 30)?;
    must_run(
        "service restart",
        &["systemctl".to_string(), "restart".to_string(), format!("gb-vibe-{unit_name}")],
        60,
    )?;
    Ok(port)
}

/// Probe the python service inside the proxy. A TCP accept + HTTP response
/// of ANY code proves the service is up (API scaffolds 404 on `/`), so
/// urllib's HTTPError counts as success; only connection failures retry.
/// The probe script is pushed as a file — the harness guard rejects `;` and
/// newlines in arguments, so a `-c` one-liner is impossible.
pub(crate) fn probe_python_service(port: u16) -> Result<(), String> {
    let script = format!(
        "import urllib.request, urllib.error\ntry:\n    r = urllib.request.urlopen('http://127.0.0.1:{port}/', timeout=3)\n    print(r.status)\nexcept urllib.error.HTTPError as e:\n    print('http', e.code)\nexcept Exception as e:\n    print('down', e)\n    raise SystemExit(1)\n"
    );
    let tmp = std::env::temp_dir().join(format!("gb-probe-{}.py", std::process::id()));
    std::fs::write(&tmp, script).map_err(|e| format!("write probe: {e}"))?;
    let proxy_path = "/tmp/gb-vibe-probe.py";
    if let Err(e) = proxy_exec(&["rm".to_string(), "-f".to_string(), proxy_path.to_string()], 15) {
        log::warn!("Vibe proxy probe: remove stale script failed: {e}");
    }
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            tmp.to_string_lossy().to_string(),
            format!("proxy{proxy_path}"),
        ],
        Path::new("."),
        30,
    );
    if let Err(e) = std::fs::remove_file(&tmp) {
        log::warn!("Vibe proxy probe: remove local temp script failed: {e}");
    }
    if let Ok(p) = pushed {
        if p.exit_code != Some(0) {
            return Err(format!("probe push failed: {}", p.stderr.trim()));
        }
    } else {
        return Err("probe push error".to_string());
    }
    for attempt in 1..=6 {
        let out = proxy_exec(&["python3".to_string(), proxy_path.to_string()], 20);
        if let Ok(o) = &out {
            if o.exit_code == Some(0) {
                return Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1500 * attempt));
    }
    Err(format!("python service on :{port} did not come up inside proxy"))
}

/// Post-publish verification THROUGH Caddy: the site host must complete a
/// TLS handshake with SNI = the site host against Caddy's local :443 and
/// return ANY HTTP response (status proves the route matches; handshake
/// failure / connection reset means Caddy has no route for this host).
/// The dial is to 127.0.0.1 directly — a fresh site has no DNS, and the
/// probe must never depend on the public resolver.
pub(crate) fn verify_route_serving(site_host: &str) -> Result<(), String> {
    let script = format!(
        "import socket, ssl\nctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)\nctx.check_hostname = False\nctx.verify_mode = ssl.CERT_NONE\ntry:\n    raw = socket.create_connection(('127.0.0.1', 443), timeout=5)\n    tls = ctx.wrap_socket(raw, server_hostname='{site_host}')\n    tls.sendall((\n        'GET / HTTP/1.1\\r\\n'\n        'Host: {site_host}\\r\\n'\n        'Connection: close\\r\\n\\r\\n'\n    ).encode())\n    data = tls.recv(200)\n    tls.close()\nexcept Exception as e:\n    print('down', e)\n    raise SystemExit(1)\nif not data.startswith(b'HTTP/'):\n    print('nohttp', data[:80])\n    raise SystemExit(1)\nprint('ok', data.split(b'\\\\r\\\\n')[0].decode(errors='replace'))\n"
    );
    let tmp = std::env::temp_dir().join(format!("gb-verify-{}.py", std::process::id()));
    std::fs::write(&tmp, script).map_err(|e| format!("write verify probe: {e}"))?;
    let proxy_path = "/tmp/gb-vibe-verify.py";
    if let Err(e) = proxy_exec(&["rm".to_string(), "-f".to_string(), proxy_path.to_string()], 15) {
        log::warn!("Vibe proxy probe: remove stale script failed: {e}");
    }
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            tmp.to_string_lossy().to_string(),
            format!("proxy{proxy_path}"),
        ],
        Path::new("."),
        30,
    );
    if let Err(e) = std::fs::remove_file(&tmp) {
        log::warn!("Vibe proxy probe: remove local temp script failed: {e}");
    }
    if let Ok(p) = pushed {
        if p.exit_code != Some(0) {
            return Err(format!("verify probe push failed: {}", p.stderr.trim()));
        }
    } else {
        return Err("verify probe push error".to_string());
    }
    for attempt in 1..=8 {
        let out = proxy_exec(&["python3".to_string(), proxy_path.to_string()], 25);
        if let Ok(o) = &out {
            if o.exit_code == Some(0) {
                return Ok(());
            }
        }
        // Generous window: on production a brand-new host triggers an ACME
        // issuance before the first successful TLS handshake.
        std::thread::sleep(std::time::Duration::from_millis(1500 * attempt));
    }
    Err(format!("route for {site_host} does not serve through Caddy yet"))
}

/// Blocking core of [`deploy_site_to_proxy_env`] — runs under the publish lock.
/// #1290 — parameterized on the environment target (production keeps the
/// legacy `{slug}` dir/host; test stages into `{slug}-test` at
/// `{slug}-test.{domain}`).
pub(crate) fn deploy_site_to_target_sync(
    project: &crate::projects::Project,
    pool: &crate::types::DbPool,
    python: bool,
    verify: bool,
    target: &SiteTarget,
    env: SiteEnv,
    deploy_rev: Option<&str>,
) -> Result<(String, String), String> {
    let _guard = lock_publish();
    let slug = site_slug(&project.name, project.branch_id);
    validate_slug(&slug)?;
    validate_project_site_name(&project.name)?;
    if env == SiteEnv::Test {
        // The test suffix must itself be a legal host label.
        validate_slug(&format!("{slug}-test"))?;
    }
    let site_dir = target.dir.clone();
    if !dir_is_vibe_owned(&site_dir)? {
        return Err(format!(
            "refusing to publish: {site_dir} already exists and is not vibe-managed \
             (missing {MARKER_FILE}) — move it or choose another project name"
        ));
    }
    // #1505 — the deployable payload comes from the git revision (HEAD of
    // the pushed state), not the live workspace. Native projects fall back
    // to the workspace walk inside `materialize_files`.
    let files = crate::deploy_source::materialize_files(project, deploy_rev)?;
    check_serveability(&files, python)?;
    stage_payload(project, &site_dir, deploy_rev)?;
    let mut service_note = String::new();
    if python {
        let port = python_port_for(target, &slug);
        // #1386 — provision the project's own database for this environment
        // and inject its URL into the service. A failure is fatal for the
        // publish: a python site without its database would 500 on boot.
        let database_url = crate::project_db::ensure_project_database(
            pool,
            project.branch_id,
            &project.name,
            env.as_str(),
        )?;
        ensure_python_service_for(&slug, &site_dir, port, env, Some(&database_url))?;
        probe_python_service(port)?;
        let unit_slug = site_unit_name(&slug, env);
        service_note = format!("gb-vibe-{unit_slug}@127.0.0.1:{port}");
    }
    upsert_site_config(&target.host, &site_block_for_target(target, &slug, python, tls_internal_from_env()))?;
    if verify {
        verify_route_serving(&target.host)?;
    }
    let url = format!("https://{}/", target.host);
    log::info!(
        "Vibe publish {} ({}): staged site to proxy {site_dir} host {} {}",
        project.name,
        env.as_str(),
        target.host,
        service_note
    );
    Ok((url, service_note))
}

/// #1290 — env-aware deploy: `production` keeps the legacy `{slug}` target,
/// `test` publishes `{slug}-test.{domain}` from `websites/{slug}-test`
/// with its own release ring and python service.
pub async fn deploy_site_to_proxy_env(
    project: &crate::projects::Project,
    python: bool,
    env: SiteEnv,
    pool: &crate::types::DbPool,
    deploy_rev: Option<String>,
) -> Result<(String, String), String> {
    let p = project.clone();
    let domain = crate::publish::published_domain();
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let slug = site_slug(&p.name, p.branch_id);
        let target = SiteTarget::new(&slug, env, &domain);
        deploy_site_to_target_sync(&p, &pool, python, true, &target, env, deploy_rev.as_deref())
    })
    .await
    .map_err(|e| format!("publish task: {e}"))?
}
