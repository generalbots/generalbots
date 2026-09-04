//! #1288 — published vibe sites served from the proxy container, not from a
//! dedicated prod VM.
//!
//! Static (`website`) and python (`custom`/python) projects are published to
//! the **proxy container's** shared websites directory
//! (`/opt/gbo/data/websites/<site>`), the same tree that already hosts the
//! platform's marketing sites. Each site is reachable at
//! `https://<site>.{GB_PLATFORM_DOMAIN}` exactly like a custom domain —
//! but served straight from Caddy's file_server (static) or a tiny
//! per-site systemd service inside the proxy (python), with **no prod VM
//! per site** (too expensive) and nothing on the bot container (the proxy
//! cannot see its filesystem).
//!
//! Transport: the bot container drives the proxy via nested
//! `incus exec proxy -- …` / `incus file push|pull` (verified on prod). Every
//! invocation goes through the harness command guard (`incus` is on the
//! allowlist; `sh` is deliberately not — inner commands are passed as direct
//! argv, never through a shell).
//!
//! Existing sites are never touched: the sites root keeps all pre-existing
//! directories; vibe manages (a) the per-site payload directory — created
//! only when absent, marker `.gb-vibe-site` proves ownership — and (b) the
//! marker-delimited `# BEGIN/END GB VIBE SITES` section of
//! `/opt/gbo/conf/config`, rewritten atomically on every publish. Anything
//! outside those markers is platform content and is left byte-identical.

use std::collections::HashSet;
use std::path::Path;

/// Websites root INSIDE the proxy container (matches prod layout).
const PROXY_SITES_ROOT: &str = "/opt/gbo/data/websites";

/// Caddyfile inside the proxy container (matches prod layout).
const PROXY_CADDY_CONFIG: &str = "/opt/gbo/conf/config";

/// Marker file recording that a site directory is vibe-managed. Presence in
/// an existing directory makes redeploys safe (they only replace the project
/// payload, never foreign files).
const MARKER_FILE: &str = ".gb-vibe-site";

const SECTION_BEGIN: &str = "# BEGIN GB VIBE SITES (auto-managed by botserver — do not edit)";
const SECTION_END: &str = "# END GB VIBE SITES";

/// `incus` runs on the host (or WSL); every proxy interaction is
/// `incus exec proxy -- <argv>` or `incus file push|pull`.
fn proxy_exec(args: &[String], timeout: u64) -> Result<crate::harness::cmd::RunOutput, String> {
    let mut full = vec!["exec".to_string(), "proxy".to_string(), "--".to_string()];
    full.extend_from_slice(args);
    crate::harness::cmd::run("incus", &full, Path::new("."), timeout)
        .map_err(|e| format!("incus exec proxy: {e}"))
}

fn must_run(
    label: &str,
    args: &[String],
    timeout: u64,
) -> Result<crate::harness::cmd::RunOutput, String> {
    let out = proxy_exec(args, timeout)?;
    if out.exit_code != Some(0) {
        return Err(format!("{label} failed: {}", out.stderr.trim()));
    }
    Ok(out)
}

/// Site directory slug: same sanitizer as the ALM repo name, so drive paths,
/// repo names and proxy dirs agree.
pub fn site_slug(project_name: &str) -> String {
    crate::vm_lifecycle::VmLifecycle::alm_repo(project_name)
}

/// Deterministic per-site port for python services (20000-29999). Stable
/// across restarts so the Caddy route never churns.
pub fn python_port(slug: &str) -> u16 {
    let hash: u32 = slug
        .bytes()
        .fold(5381u32, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u32));
    (20000 + (hash % 9999)) as u16
}

/// `true` when `dir` is absent (free to create) or vibe-owned (marker file).
/// `Ok(false)` = exists but foreign → publish must refuse.
fn dir_is_vibe_owned(dir: &str) -> Result<bool, String> {
    let exists = proxy_exec(&["test".to_string(), "-d".to_string(), dir.to_string()], 20)?;
    if exists.exit_code != Some(0) {
        return Ok(true);
    }
    let marker = proxy_exec(
        &["test".to_string(), "-f".to_string(), format!("{dir}/{MARKER_FILE}")],
        20,
    )?;
    Ok(marker.exit_code == Some(0))
}

/// Python sites need a runtime in the proxy. python3 stdlib is present on
/// the base image; venv support (`python3 -m venv`) is required for the
/// per-site dependency install.
fn check_python_runtime() -> Result<(), String> {
    let out = proxy_exec(&["python3".to_string(), "--version".to_string()], 20)?;
    if out.exit_code != Some(0) {
        return Err(
            "python3 runtime not available in proxy container — publish the python project to a project VM instead"
                .to_string(),
        );
    }
    Ok(())
}

/// Tar the workspace payload on the bot side and extract it into a fresh
/// directory inside the proxy, then atomically swap it into place. Only the
/// payload replaces; the marker is (re)written so ownership survives.
fn stage_payload(project: &crate::projects::Project, site_dir: &str) -> Result<(), String> {
    let files = super::publish::collect_workspace_files(project)?;
    if files.is_empty() {
        return Err("workspace is empty — nothing to publish".to_string());
    }
    let tmp = std::env::temp_dir().join(format!(
        "vibe-site-{}-{}.tar",
        project.id,
        chrono::Utc::now().timestamp_millis()
    ));
    {
        use tar::Builder;
        let fh = std::fs::File::create(&tmp).map_err(|e| format!("create staging tar: {e}"))?;
        let mut builder = Builder::new(fh);
        for f in &files {
            let rel = f["path"].as_str().unwrap_or_default().to_string();
            if rel.is_empty() || rel.contains("..") {
                continue;
            }
            let content: Vec<u8> = serde_json::from_value(f["content"].clone())
                .map_err(|e| format!("payload decode for {rel}: {e}"))?;
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, &rel, content.as_slice())
                .map_err(|e| format!("tar append {rel}: {e}"))?;
        }
        builder.finish().map_err(|e| format!("tar finish: {e}"))?;
    }

    // Push the tarball, extract into a staging dir, verify non-empty, swap.
    let proxy_tmp = format!("/tmp/{}", tmp.file_name().unwrap_or_default().to_string_lossy());
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            tmp.to_string_lossy().to_string(),
            // NOTE: the incus file syntax is `incus file push <local>
            // <container>/<abs-path>` (no colon — the colon form parses
            // `proxy` as a REMOTE name and fails with "remote doesn't exist").
            format!("proxy{proxy_tmp}"),
        ],
        Path::new("."),
        120,
    )
    .map_err(|e| format!("incus file push: {e}"))?;
    if pushed.exit_code != Some(0) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("incus file push failed: {}", pushed.stderr.trim()));
    }

    let new_dir = format!("{site_dir}.new");
    let old_dir = format!("{site_dir}.old");
    let _ = must_run("cleanup staging", &["rm".to_string(), "-rf".to_string(), new_dir.clone()], 30);
    must_run("mkdir staging", &["mkdir".to_string(), "-p".to_string(), new_dir.clone()], 20)?;
    must_run(
        "tar extract",
        &[
            "tar".to_string(),
            // -x auto-detects the format (plain tar here); -z would demand
            // gzip and the archive is uncompressed by design.
            "-xf".to_string(),
            proxy_tmp.clone(),
            "-C".to_string(),
            new_dir.clone(),
        ],
        60,
    )?;
    let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), proxy_tmp.clone()], 20);
    // Ownership marker travels with the payload.
    must_run(
        "write marker",
        &["touch".to_string(), format!("{new_dir}/{MARKER_FILE}")],
        20,
    )?;
    // Non-empty check: payload extraction must have produced files.
    let count = must_run(
        "verify payload",
        &[
            "find".to_string(),
            new_dir.clone(),
            "-type".to_string(),
            "f".to_string(),
        ],
        30,
    )?;
    if count.stdout.lines().filter(|l| !l.trim().is_empty()).count() < 2 {
        // less than marker + at least one payload file
        let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), new_dir.clone()], 30);
        return Err("payload extraction produced no files in proxy".to_string());
    }
    // Atomic-ish swap: current → .old, .new → current, drop .old.
    let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir.clone()], 30);
    let _ = proxy_exec(
        &["mv".to_string(), site_dir.to_string(), old_dir.clone()],
        20,
    );
    must_run(
        "promote staging",
        &["mv".to_string(), new_dir.clone(), site_dir.to_string()],
        20,
    )?;
    let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir], 30);
    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

/// Render the Caddyfile site block for one site.
fn site_block(slug: &str, python: bool) -> String {
    let site_host = format!("{slug}.{}", super::publish::published_domain());
    // `tls internal` keeps Caddy from attempting an ACME handshake for a
    // host whose DNS may not point here yet (staging/new sites): Caddy then
    // serves a locally-trusted cert immediately and upgrades to Let's
    // Encrypt once the directive is dropped and DNS/ACME are ready.
    if python {
        let port = python_port(slug);
        format!(
            "{site_host} {{\n\ttls internal\n\treverse_proxy 127.0.0.1:{port}\n}}\n"
        )
    } else {
        format!(
            "{site_host} {{\n\ttls internal\n\troot * {PROXY_SITES_ROOT}/{slug}\n\tfile_server\n\tencode zstd gzip\n}}\n"
        )
    }
}

/// Replace the marker-delimited section of the proxy Caddyfile so it holds
/// the block for `slug` while PRESERVING the blocks of all other vibe sites
/// already present in the section (the section is shared state: a single
/// project's publish must not drop its peers' routes).
fn upsert_site_config(slug: &str, block: &str) -> Result<(), String> {
    let tmp_pull = std::env::temp_dir().join(format!(
        "gb-proxy-config-{}.conf",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let pulled = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "pull".to_string(),
            // no-colon form — see stage_payload; `proxy:/path` is a REMOTE.
            format!("proxy{PROXY_CADDY_CONFIG}"),
            tmp_pull.to_string_lossy().to_string(),
        ],
        Path::new("."),
        60,
    )
    .map_err(|e| format!("incus file pull: {e}"))?;
    if pulled.exit_code != Some(0) {
        return Err(format!(
            "incus file pull failed: {}",
            pulled.stderr.trim()
        ));
    }
    let original = std::fs::read_to_string(&tmp_pull)
        .map_err(|e| format!("read pulled config: {e}"))?;

    // Slice out the current managed section (everything between markers, or
    // empty when markers are absent — fresh setups append at the tail).
    let existing_section = match (
        original.find(SECTION_BEGIN),
        original.find(SECTION_END),
    ) {
        (Some(b), Some(e)) if e >= b => original[b + SECTION_BEGIN.len()..e].to_string(),
        _ => String::new(),
    };
    // Drop this site's previous block (if any) and append the fresh one,
    // keeping every other site block in the section untouched. The section
    // key is the FULL HOST (`{slug}.{domain}`) — the block header — which is
    // what uniquely identifies a managed site.
    let site_host = format!("{slug}.{}", super::publish::published_domain());
    let mut kept = String::new();
    let mut current: Option<(String, String)> = None; // (host, accumulated block)
    for line in existing_section.lines() {
        let trimmed = line.trim();
        let is_header = trimmed.ends_with('{') && !trimmed.starts_with('#');
        if is_header {
            if let Some((_prev_host, acc)) = current.take() {
                kept.push_str(&acc);
                kept.push('\n');
            }
            current = Some((
                trimmed.trim_end_matches('{').trim().to_string(),
                line.to_string(),
            ));
        } else if let Some((_open_host, acc)) = current.as_mut() {
            acc.push('\n');
            acc.push_str(line);
            if trimmed == "}" {
                let (kept_host, acc) = current.take().unwrap();
                if kept_host != site_host {
                    kept.push_str(&acc);
                    kept.push('\n');
                }
            }
        } else if !trimmed.is_empty() {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    if let Some((kept_host, acc)) = current {
        if kept_host != site_host {
            kept.push_str(&acc);
            kept.push('\n');
        }
    }
    let blocks = format!("{kept}{block}");
    let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
    let updated = match (
        original.find(SECTION_BEGIN),
        original.find(SECTION_END),
    ) {
        (Some(b), Some(e)) if e >= b => format!(
            "{}{}{}",
            &original[..b],
            managed,
            original[e + SECTION_END.len()..].trim_start_matches('\n')
        ),
        _ => format!("{}\n{managed}", original.trim_end()),
    };
    std::fs::write(&tmp_pull, updated).map_err(|e| format!("write updated config: {e}"))?;

    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            tmp_pull.to_string_lossy().to_string(),
            // no-colon form — see stage_payload; `proxy:/path` is a REMOTE.
            format!("proxy{PROXY_CADDY_CONFIG}"),
        ],
        Path::new("."),
        60,
    )
    .map_err(|e| format!("incus file push: {e}"))?;
    let _ = std::fs::remove_file(&tmp_pull);
    if pushed.exit_code != Some(0) {
        return Err(format!("incus file push failed: {}", pushed.stderr.trim()));
    }

    // Validate + hot-reload. A syntax error leaves the old config running
    // and is reported to the caller (publish surfaces it to the agent).
    must_run(
        "caddy validate",
        &[
            "caddy".to_string(),
            "validate".to_string(),
            "--adapter".to_string(),
            "caddyfile".to_string(),
            "--config".to_string(),
            PROXY_CADDY_CONFIG.to_string(),
        ],
        60,
    )?;
    must_run(
        "caddy reload",
        &[
            "caddy".to_string(),
            "reload".to_string(),
            "--adapter".to_string(),
            "caddyfile".to_string(),
            "--config".to_string(),
            PROXY_CADDY_CONFIG.to_string(),
        ],
        60,
    )?;
    Ok(())
}

/// Install (or refresh) the per-site python systemd service inside the
/// proxy: venv + dependency install + unit + start. Idempotent.
fn ensure_python_service(slug: &str) -> Result<u16, String> {
    check_python_runtime()?;
    let site_dir = format!("{PROXY_SITES_ROOT}/{slug}");
    let port = python_port(slug);

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

    // systemd unit — pushed as a file (no shell needed).
    let unit = format!(
        "[Unit]\nDescription=GB vibe site {slug}\nAfter=network.target\n\n[Service]\nWorkingDirectory={site_dir}\nEnvironment=PORT={port}\nExecStart={site_dir}/.venv/bin/python {site_dir}/app.py\nRestart=always\nRestartSec=3\n\n[Install]\nWantedBy=multi-user.target\n"
    );
    let unit_tmp = std::env::temp_dir().join(format!("gb-vibe-{slug}.service"));
    std::fs::write(&unit_tmp, unit).map_err(|e| format!("write unit: {e}"))?;
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            unit_tmp.to_string_lossy().to_string(),
            // no-colon form — see stage_payload; `proxy:/path` is a REMOTE.
            format!("proxy/etc/systemd/system/gb-vibe-{slug}.service"),
        ],
        Path::new("."),
        30,
    )
    .map_err(|e| format!("incus file push unit: {e}"))?;
    let _ = std::fs::remove_file(&unit_tmp);
    if pushed.exit_code != Some(0) {
        return Err(format!("unit push failed: {}", pushed.stderr.trim()));
    }
    must_run("daemon-reload", &["systemctl".to_string(), "daemon-reload".to_string()], 30)?;
    must_run(
        "service restart",
        &[
            "systemctl".to_string(),
            "restart".to_string(),
            format!("gb-vibe-{slug}"),
        ],
        60,
    )?;
    Ok(port)
}

/// Probe the python service inside the proxy (urllib via python3 — curl is
/// not on the harness allowlist). Best-effort: a slow first boot must not
/// fail the publish, so only hard connection refusals after retries fail.
fn probe_python_service(port: u16) -> Result<(), String> {
    // Flask apps answer 200 on `/` but API-style scaffolds (e.g. /health
    // only) return 404 there — a TCP accept + HTTP response of ANY code
    // proves the service is up, so treat urllib's HTTPError (which carries
    // the code) as success and only connection failures as down.
    //
    // The harness guard rejects arguments containing `;` or newlines, so the
    // probe CANNOT be a `-c` one-liner: push a tiny probe script into the
    // proxy via `incus file push` (no shell) and run it.
    let script = format!(
        "import urllib.request, urllib.error\ntry:\n    r = urllib.request.urlopen('http://127.0.0.1:{port}/', timeout=3)\n    print(r.status)\nexcept urllib.error.HTTPError as e:\n    print('http', e.code)\nexcept Exception as e:\n    print('down', e)\n    raise SystemExit(1)\n"
    );
    let tmp = std::env::temp_dir().join(format!("gb-probe-{}.py", std::process::id()));
    std::fs::write(&tmp, script).map_err(|e| format!("write probe: {e}"))?;
    // Unique per-boot target + pre-delete: `incus file push` cannot overwrite
    // a file owned by another uid (EACCES) — seen when a previous botserver
    // ran as a different user. rm first, then push a fresh file.
    let proxy_path = "/tmp/gb-vibe-probe.py";
    let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), proxy_path.to_string()], 15);
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
    let _ = std::fs::remove_file(&tmp);
    if let Ok(p) = pushed {
        if p.exit_code != Some(0) {
            return Err(format!("probe push failed: {}", p.stderr.trim()));
        }
    } else {
        return Err("probe push error".to_string());
    }
    for attempt in 1..=6 {
        let out = proxy_exec(
            &["python3".to_string(), proxy_path.to_string()],
            20,
        );
        if let Ok(o) = &out {
            if o.exit_code == Some(0) {
                return Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1500 * attempt));
    }
    Err(format!("python service on :{port} did not come up inside proxy"))
}

/// Entry point used by `do_publish` for `website` and python `custom`
/// projects in production. Stages files into the proxy's websites tree and
/// registers the Caddyfile site block. Returns the public URL.
pub async fn deploy_site_to_proxy(
    project: &crate::projects::Project,
    python: bool,
) -> Result<(String, String), String> {
    let slug = site_slug(&project.name);
    let site_dir = format!("{PROXY_SITES_ROOT}/{slug}");
    if !dir_is_vibe_owned(&site_dir)? {
        return Err(format!(
            "refusing to publish: {site_dir} already exists and is not vibe-managed \
             (missing {MARKER_FILE}) — move it or choose another project name"
        ));
    }
    stage_payload(project, &site_dir)?;
    let mut service_note = String::new();
    if python {
        let port = ensure_python_service(&slug)?;
        probe_python_service(port)?;
        service_note = format!("gb-vibe-{slug}@127.0.0.1:{port}");
    }
    upsert_site_config(&slug, &site_block(&slug, python))?;
    let host = format!("{slug}.{}", super::publish::published_domain());
    let url = format!("https://{host}/");
    log::info!(
        "Vibe publish {}: staged site to proxy {site_dir} host {host} {}",
        project.name,
        service_note
    );
    Ok((url, service_note))
}

/// Which workspace filenames indicate a python project (used by publish to
/// route to the proxy-python path). Kept in sync with the scaffold prompt.
pub fn looks_like_python(files: &[serde_json::Value]) -> bool {
    let names: HashSet<String> = files
        .iter()
        .filter_map(|f| f["path"].as_str().map(String::from))
        .collect();
    names.contains("app.py") || names.contains("requirements.txt")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm_lifecycle::VmLifecycle;

    #[test]
    fn site_slug_matches_alm_repo_slug() {
        assert_eq!(site_slug("My Web App"), VmLifecycle::alm_repo("My Web App"));
        assert_eq!(site_slug("site1276"), "site1276");
    }

    #[test]
    fn marker_file_is_hidden() {
        assert!(MARKER_FILE.starts_with('.'));
    }

    #[test]
    fn looks_like_python_detects_entry_and_requirements() {
        let py = serde_json::json!([{ "path": "app.py", "content": "" }]);
        let req = serde_json::json!([{ "path": "requirements.txt", "content": "" }]);
        let other = serde_json::json!([{ "path": "index.html", "content": "" }]);
        assert!(looks_like_python(py.as_array().unwrap()));
        assert!(looks_like_python(req.as_array().unwrap()));
        assert!(!looks_like_python(other.as_array().unwrap()));
    }

    #[test]
    fn python_port_is_stable_and_in_range() {
        let a = python_port("site1276");
        assert_eq!(a, python_port("site1276"));
        assert!((20000..30000).contains(&a));
    }

    #[test]
    fn site_block_static_has_file_server() {
        let b = site_block("mysite", false);
        assert!(b.starts_with("mysite."));
        assert!(b.contains("file_server"));
        assert!(b.contains(&format!("root * {PROXY_SITES_ROOT}/mysite")));
    }

    #[test]
    fn site_block_python_has_reverse_proxy() {
        let b = site_block("pysite", true);
        assert!(b.contains(&format!("reverse_proxy 127.0.0.1:{}", python_port("pysite"))));
    }

    #[test]
    fn upsert_replaces_marker_section_only() {
        let original = format!(
            "pre existing site one {{\n\troot * /srv/one\n}}\n\n{SECTION_BEGIN}\nold.example.com {{}}\n{SECTION_END}\n\npost site two {{}}\n"
        );
        let blocks = "new.example.com {\n\tfile_server\n}\n";
        let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
        let (b, e) = (
            original.find(SECTION_BEGIN).unwrap(),
            original.find(SECTION_END).unwrap(),
        );
        let updated = format!(
            "{}{}{}",
            &original[..b],
            managed,
            original[e + SECTION_END.len()..].trim_start_matches('\n')
        );
        assert!(updated.contains("pre existing site one"));
        assert!(updated.contains("post site two"));
        assert!(updated.contains("new.example.com"));
        assert!(!updated.contains("old.example.com"));
        assert_eq!(updated.matches(SECTION_BEGIN).count(), 1);
    }

    #[test]
    fn upsert_dedups_by_full_host_not_slug() {
        // Regression: the old parser compared the block header (full host,
        // `site1276.generalbots.org`) against the bare slug, so re-publishing
        // a site left BOTH the old and the new block in the section — Caddy
        // then failed validation with "ambiguous site definition".
        let existing_section = format!(
            "site1276.{} {{\n\ttls internal\n\troot * /x\n}}\npy1276.{} {{\n\ttls internal\n}}\n",
            crate::publish::published_domain(),
            crate::publish::published_domain()
        );
        let site_host = format!("site1276.{}", crate::publish::published_domain());
        let mut kept = String::new();
        let mut current: Option<(String, String)> = None;
        for line in existing_section.lines() {
            let trimmed = line.trim();
            let is_header = trimmed.ends_with('{') && !trimmed.starts_with('#');
            if is_header {
                if let Some((_p, acc)) = current.take() {
                    kept.push_str(&acc);
                    kept.push('\n');
                }
                current = Some((trimmed.trim_end_matches('{').trim().to_string(), line.to_string()));
            } else if let Some((_o, acc)) = current.as_mut() {
                acc.push('\n');
                acc.push_str(line);
                if trimmed == "}" {
                    let (h, acc) = current.take().unwrap();
                    if h != site_host {
                        kept.push_str(&acc);
                        kept.push('\n');
                    }
                }
            } else if !trimmed.is_empty() {
                kept.push_str(line);
                kept.push('\n');
            }
        }
        if let Some((h, acc)) = current {
            if h != site_host {
                kept.push_str(&acc);
                kept.push('\n');
            }
        }
        assert_eq!(kept.matches(&site_host).count(), 0, "old block must be dropped: {kept}");
        assert!(kept.contains("py1276."), "peer block must survive: {kept}");
    }

    #[test]
    fn upsert_appends_section_when_missing() {
        let original = "admin {\n\tlocal\n}\n".to_string();
        let blocks = "x.example.com {\n}\n";
        let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
        let updated = format!("{}\n{managed}", original.trim_end());
        assert!(updated.starts_with("admin {"));
        assert!(updated.contains(SECTION_BEGIN));
        assert!(updated.trim_end().ends_with(SECTION_END));
    }
}
