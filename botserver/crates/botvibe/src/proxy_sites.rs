//! #1288 — published vibe sites served from the proxy container, not from a
//! dedicated prod VM. Enterprise-grade operation:
//!
//! - **Concurrency**: a process-wide publish mutex serializes every site
//!   mutation (payload swap + Caddy config rewrite) so two simultaneous
//!   publishes can never interleave config pulls/pushes and corrupt the
//!   shared Caddyfile section. Blocking incus work runs via
//!   `spawn_blocking` — never on the async executor.
//! - **Input hardening**: slugs are validated against a strict pattern and a
//!   reserved-name list (no `proxy`, `caddy`, `grafana`, …, no config
//!   collision); payload size and file counts are capped before any bytes
//!   reach the proxy; python payloads must contain `app.py`, static ones an
//!   `index.html` (no publishing a site that can only 404).
//! - **Zero-downtime config**: the new Caddyfile is validated as a
//!   *candidate file* BEFORE the swap; the current config is backed up
//!   (rotated, 10 kept); a failed reload auto-restores the backup.
//! - **Verification**: after publish, the route is probed through Caddy
//!   itself (Host header + `--resolve`-style dial inside the proxy), not
//!   just the backend — a route that never serves is a failed publish.
//! - **Releases & rollback**: each payload swap keeps the previous release
//!   in `<site>.prev-N` (10 retained); `rollback_site` re-activates one and
//!   `unpublish_site` removes route + service + payload.
//!
//! Transport: the bot container drives the proxy via nested
//! `incus exec proxy -- …` / `incus file push|pull` (verified on prod).
//! Every invocation goes through the harness command guard (`incus` is on
//! the allowlist; `sh` is deliberately not — inner commands are passed as
//! direct argv, never through a shell).

use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

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

/// How many previous-release payload dirs and config backups to retain.
const RELEASE_RETENTION: usize = 10;

/// Enterprise limits — a publish must never be able to fill the proxy disk
/// or wedge the command guard with thousands of tiny files.
const MAX_TOTAL_BYTES: usize = 50 * 1024 * 1024; // 50 MiB per site
const MAX_FILES: usize = 2_000;
const MAX_SINGLE_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB per file

/// Names a project can never take: proxy/infra hostnames and Caddyfile
/// artifact names. These either collide with infra or with the managed
/// config artifacts (`*.prev-*` dirs would be matched by site listing).
const RESERVED_SLUGS: [&str; 10] = [
    "proxy", "caddy", "bot", "api", "www", "mail", "smtp", "vault", "grafana", "admin",
];

/// Process-wide serialization for every site mutation. A std Mutex is fine:
/// contention is publish-frequency (human scale) and holders do incus IO.
static PUBLISH_LOCK: LazyLock<std::sync::Mutex<()>> =
    LazyLock::new(|| std::sync::Mutex::new(()));

/// Serialize all site mutations; the guard is released on scope exit (also
/// on error paths). Poisoning is tolerated: a panicked publisher must not
/// wedge every future publish — recover and continue.
fn lock_publish() -> std::sync::MutexGuard<'static, ()> {
    match PUBLISH_LOCK.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

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

/// Site directory slug: same sanitizer as the ALM repo name, plus the
/// enterprise hardening (reserved names, charset, length).
pub fn site_slug(project_name: &str) -> String {
    crate::vm_lifecycle::VmLifecycle::alm_repo(project_name)
}

/// Validate a slug against the enterprise rules. Separate from [`site_slug`]
/// so the sanitizer stays compatible with ALM repo names while publish
/// refuses anything unsafe to expose as `{slug}.{domain}`.
pub fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.len() < 3 || slug.len() > 63 {
        return Err(format!(
            "site name '{slug}' must be 3-63 characters (got {})",
            slug.len()
        ));
    }
    let ok = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !ok {
        return Err(format!(
            "site name '{slug}' may only contain lowercase letters, digits and hyphens"
        ));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(format!("site name '{slug}' may not start or end with a hyphen"));
    }
    if RESERVED_SLUGS.contains(&slug) {
        return Err(format!("site name '{slug}' is reserved for platform infrastructure"));
    }
    Ok(())
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

/// Decode a workspace file entry's content. The canonical producer
/// (`walk_workspace`) serializes raw bytes (a JSON array of numbers), but
/// accept a plain string too so callers passing text payloads work.
fn payload_bytes(entry: &serde_json::Value) -> Result<Vec<u8>, String> {
    let content = &entry["content"];
    if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(content.clone()) {
        return Ok(bytes);
    }
    if let Some(text) = content.as_str() {
        return Ok(text.as_bytes().to_vec());
    }
    Err(format!(
        "payload decode: unsupported content for '{}'",
        entry["path"].as_str().unwrap_or_default()
    ))
}

/// Enterprise payload caps: enforce size/file limits BEFORE transferring so
/// a runaway agent can never fill the proxy disk.
fn check_payload_limits(files: &[serde_json::Value]) -> Result<(), String> {
    if files.len() > MAX_FILES {
        return Err(format!(
            "payload has {} files; the limit is {MAX_FILES} — trim the workspace or publish to a VM",
            files.len()
        ));
    }
    let mut total: usize = 0;
    for f in files {
        let content = payload_bytes(f)?;
        if content.len() > MAX_SINGLE_FILE_BYTES {
            return Err(format!(
                "file '{}' is {} bytes; the per-file limit is {MAX_SINGLE_FILE_BYTES}",
                f["path"].as_str().unwrap_or_default(),
                content.len()
            ));
        }
        total = total.saturating_add(content.len());
    }
    if total > MAX_TOTAL_BYTES {
        return Err(format!(
            "payload is {total} bytes; the per-site limit is {MAX_TOTAL_BYTES} — publish to a VM instead"
        ));
    }
    Ok(())
}

/// Serveability precheck: a static site without index.html can only 404 at
/// `/`; a python site without app.py has no entrypoint. Refuse early with
/// actionable errors.
fn check_serveability(files: &[serde_json::Value], python: bool) -> Result<(), String> {
    let names: HashSet<String> = files
        .iter()
        .filter_map(|f| f["path"].as_str().map(String::from))
        .collect();
    if python {
        if !names.contains("app.py") {
            return Err(
                "python publish requires an app.py at the workspace root (the proxy runs it directly)"
                    .to_string(),
            );
        }
    } else if !names.contains("index.html") {
        return Err(
            "static publish requires an index.html at the workspace root (the proxy serves it at /)"
                .to_string(),
        );
    }
    Ok(())
}

/// Tar the workspace payload on the bot side (limits already checked) and
/// extract it into a fresh directory inside the proxy, keeping the previous
/// release as `<site>.prev-N` (rotated) for rollback. Then atomically swap.
fn stage_payload(project: &crate::projects::Project, site_dir: &str) -> Result<(), String> {
    let files = super::publish::collect_workspace_files(project)?;
    if files.is_empty() {
        return Err("workspace is empty — nothing to publish".to_string());
    }
    check_payload_limits(&files)?;

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
            let content = payload_bytes(f)?;
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
    let _ = std::fs::remove_file(&tmp);
    if pushed.exit_code != Some(0) {
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

    // Release retention: rotate the current payload into `<site>.prev-N`
    // (newest is .prev-1; oldest is dropped) so rollback is always possible.
    rotate_release(site_dir)?;

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
    Ok(())
}

/// Rotate the live payload dir into the `.prev-N` ring before a new release
/// replaces it. `.prev-1` is the most recent previous release.
fn rotate_release(site_dir: &str) -> Result<(), String> {
    let exists = proxy_exec(&["test".to_string(), "-d".to_string(), site_dir.to_string()], 15)?;
    if exists.exit_code != Some(0) {
        return Ok(()); // first publish — nothing to retain
    }
    // Drop the oldest slot to make room.
    let oldest = format!("{site_dir}.prev-{RELEASE_RETENTION}");
    let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), oldest], 60);
    for n in (1..RELEASE_RETENTION).rev() {
        let from = format!("{site_dir}.prev-{n}");
        let to = format!("{site_dir}.prev-{}", n + 1);
        let has = proxy_exec(&["test".to_string(), "-d".to_string(), from.clone()], 15)?;
        if has.exit_code == Some(0) {
            let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), to.clone()], 60);
            let _ = proxy_exec(&["mv".to_string(), from, to], 20);
        }
    }
    let _ = proxy_exec(
        &[
            "mv".to_string(),
            site_dir.to_string(),
            format!("{site_dir}.prev-1"),
        ],
        20,
    );
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

/// Pull the proxy Caddyfile to a local temp file. Caller owns the temp file.
fn pull_proxy_config() -> Result<std::path::PathBuf, String> {
    let tmp = std::env::temp_dir().join(format!(
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
            tmp.to_string_lossy().to_string(),
        ],
        Path::new("."),
        60,
    )
    .map_err(|e| format!("incus file pull: {e}"))?;
    if pulled.exit_code != Some(0) {
        return Err(format!("incus file pull failed: {}", pulled.stderr.trim()));
    }
    Ok(tmp)
}

fn push_proxy_config(local: &Path) -> Result<(), String> {
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            local.to_string_lossy().to_string(),
            // no-colon form — see stage_payload; `proxy:/path` is a REMOTE.
            format!("proxy{PROXY_CADDY_CONFIG}"),
        ],
        Path::new("."),
        60,
    )
    .map_err(|e| format!("incus file push: {e}"))?;
    if pushed.exit_code != Some(0) {
        return Err(format!("incus file push failed: {}", pushed.stderr.trim()));
    }
    Ok(())
}

/// Validate a CANDIDATE config file inside the proxy (before swapping it
/// into place) — the safest possible ordering: the live config is never
/// replaced by anything that fails validation.
fn validate_candidate(candidate: &Path) -> Result<(), String> {
    let proxy_tmp = format!(
        "/tmp/candidate-{}.conf",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            candidate.to_string_lossy().to_string(),
            format!("proxy{proxy_tmp}"),
        ],
        Path::new("."),
        60,
    )
    .map_err(|e| format!("candidate push: {e}"))?;
    if pushed.exit_code != Some(0) {
        return Err(format!("candidate push failed: {}", pushed.stderr.trim()));
    }
    let res = proxy_exec(
        &[
            "caddy".to_string(),
            "validate".to_string(),
            "--adapter".to_string(),
            "caddyfile".to_string(),
            "--config".to_string(),
            proxy_tmp.clone(),
        ],
        60,
    );
    let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), proxy_tmp], 20);
    let out = res?;
    if out.exit_code != Some(0) {
        return Err(format!(
            "candidate config rejected by caddy validate: {}",
            out.stderr.trim()
        ));
    }
    Ok(())
}

/// Reload caddy against the LIVE config path.
fn reload_caddy() -> Result<(), String> {
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

/// Rotate the config backup ring inside the proxy: `config.prev-N`
/// (`.prev-1` = most recent), oldest dropped.
fn backup_proxy_config() -> Result<(), String> {
    let oldest = format!("{PROXY_CADDY_CONFIG}.prev-{RELEASE_RETENTION}");
    let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), oldest], 30);
    for n in (1..RELEASE_RETENTION).rev() {
        let from = format!("{PROXY_CADDY_CONFIG}.prev-{n}");
        let to = format!("{PROXY_CADDY_CONFIG}.prev-{}", n + 1);
        let has = proxy_exec(&["test".to_string(), "-f".to_string(), from.clone()], 15)?;
        if has.exit_code == Some(0) {
            let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), to.clone()], 30);
            let _ = proxy_exec(&["mv".to_string(), from, to], 20);
        }
    }
    let _ = proxy_exec(
        &[
            "cp".to_string(),
            PROXY_CADDY_CONFIG.to_string(),
            format!("{PROXY_CADDY_CONFIG}.prev-1"),
        ],
        20,
    )?;
    Ok(())
}

/// Slice out the current managed section (everything between markers, or
/// empty when markers are absent — fresh setups append at the tail).
fn extract_section(original: &str) -> String {
    match (original.find(SECTION_BEGIN), original.find(SECTION_END)) {
        (Some(b), Some(e)) if e >= b => original[b + SECTION_BEGIN.len()..e].to_string(),
        _ => String::new(),
    }
}

/// Drop `site_host`'s block from a section, keeping every other site block
/// untouched. The section key is the FULL HOST (the block header) — the old
/// slug-vs-host comparison never matched and left duplicate blocks (Caddy
/// "ambiguous site definition").
fn drop_site_block(existing_section: &str, site_host: &str) -> String {
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
    kept
}

/// Drop blocks whose host does not belong to the CURRENT platform domain.
/// A domain change (or an earlier run under a different `GB_PLATFORM_DOMAIN`)
/// leaves stale blocks behind; they would accumulate forever and serve dead
/// hostnames. The managed section only ever contains
/// `{slug}.{published_domain()}` blocks, so anything else is garbage.
fn drop_foreign_domain_blocks(section: &str) -> String {
    let suffix = format!(".{}", super::publish::published_domain());
    let mut kept = String::new();
    let mut current: Option<(String, String)> = None; // (host, accumulated block)
    let flush = |current: &mut Option<(String, String)>, kept: &mut String| {
        if let Some((host, acc)) = current.take() {
            if host.ends_with(&suffix) {
                kept.push_str(&acc);
                kept.push('\n');
            }
        }
    };
    for line in section.lines() {
        let trimmed = line.trim();
        let is_header = trimmed.ends_with('{') && !trimmed.starts_with('#');
        if is_header {
            flush(&mut current, &mut kept);
            current = Some((
                trimmed.trim_end_matches('{').trim().to_string(),
                line.to_string(),
            ));
        } else if let Some((_open_host, acc)) = current.as_mut() {
            acc.push('\n');
            acc.push_str(line);
            if trimmed == "}" {
                flush(&mut current, &mut kept);
            }
        } else if !trimmed.is_empty() {
            kept.push_str(line);
            kept.push('\n');
        }
    }
    flush(&mut current, &mut kept);
    kept
}

/// Update the managed section so it holds `block` for `site_host` while
/// preserving every other vibe site's block. Zero-downtime ordering:
/// candidate file validated FIRST, then backup, then swap, then reload; a
/// failed reload auto-restores the backup and re-validates.
fn upsert_site_config(site_host: &str, block: &str) -> Result<(), String> {
    let tmp_pull = pull_proxy_config()?;
    let result = (|| -> Result<(), String> {
        let original = std::fs::read_to_string(&tmp_pull)
            .map_err(|e| format!("read pulled config: {e}"))?;
        let existing_section = extract_section(&original);
        let kept = drop_foreign_domain_blocks(&drop_site_block(&existing_section, site_host));
        let blocks = format!("{kept}{block}");
        let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
        let updated = match (original.find(SECTION_BEGIN), original.find(SECTION_END)) {
            (Some(b), Some(e)) if e >= b => format!(
                "{}{}{}",
                &original[..b],
                managed,
                original[e + SECTION_END.len()..].trim_start_matches('\n')
            ),
            _ => format!("{}\n{managed}", original.trim_end()),
        };
        std::fs::write(&tmp_pull, updated).map_err(|e| format!("write updated config: {e}"))?;

        // 1. The candidate must validate BEFORE the live config is touched.
        validate_candidate(&tmp_pull)?;
        // 2. Backup ring (restore point).
        backup_proxy_config()?;
        // 3. Swap + reload; on reload failure, restore the backup and
        //    reload again so the proxy never stays in a broken state.
        push_proxy_config(&tmp_pull)?;
        if let Err(e) = reload_caddy() {
            log::error!("caddy reload failed after publish ({e}); restoring backup");
            let restore = pull_proxy_config()?;
            // The live config is now the NEW (bad) one; grab the backup.
            let b1 = format!("{PROXY_CADDY_CONFIG}.prev-1");
            let got = crate::harness::cmd::run(
                "incus",
                &[
                    "file".to_string(),
                    "pull".to_string(),
                    format!("proxy{b1}"),
                    restore.to_string_lossy().to_string(),
                ],
                Path::new("."),
                60,
            );
            if let Ok(g) = got {
                if g.exit_code == Some(0) {
                    let _ = push_proxy_config(&restore);
                    let _ = reload_caddy();
                }
            }
            let _ = std::fs::remove_file(&restore);
            return Err(format!("caddy reload failed (config restored): {e}"));
        }
        Ok(())
    })();
    let _ = std::fs::remove_file(&tmp_pull);
    result
}

/// Remove a site's block from the managed section entirely (unpublish).
fn remove_site_config(site_host: &str) -> Result<(), String> {
    let tmp_pull = pull_proxy_config()?;
    let result = (|| -> Result<(), String> {
        let original = std::fs::read_to_string(&tmp_pull)
            .map_err(|e| format!("read pulled config: {e}"))?;
        let existing_section = extract_section(&original);
        let kept = drop_foreign_domain_blocks(&drop_site_block(&existing_section, site_host));
        let managed = format!("{SECTION_BEGIN}\n{kept}{SECTION_END}\n");
        let updated = match (original.find(SECTION_BEGIN), original.find(SECTION_END)) {
            (Some(b), Some(e)) if e >= b => format!(
                "{}{}{}",
                &original[..b],
                managed,
                original[e + SECTION_END.len()..].trim_start_matches('\n')
            ),
            _ => original.clone(),
        };
        std::fs::write(&tmp_pull, updated).map_err(|e| format!("write updated config: {e}"))?;
        validate_candidate(&tmp_pull)?;
        backup_proxy_config()?;
        push_proxy_config(&tmp_pull)?;
        reload_caddy()?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&tmp_pull);
    result
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
    let unit_target = format!("proxy/etc/systemd/system/gb-vibe-{slug}.service");
    // Pre-delete the unit: `incus file push` cannot overwrite a file owned
    // by another uid (EACCES) — seen when a previous botserver ran as a
    // different user.
    let _ = proxy_exec(
        &["rm".to_string(), "-f".to_string(), format!("/etc/systemd/system/gb-vibe-{slug}.service")],
        15,
    );
    let pushed = crate::harness::cmd::run(
        "incus",
        &[
            "file".to_string(),
            "push".to_string(),
            unit_tmp.to_string_lossy().to_string(),
            // no-colon form — see stage_payload.
            unit_target,
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
        &["systemctl".to_string(), "restart".to_string(), format!("gb-vibe-{slug}")],
        60,
    )?;
    Ok(port)
}

/// Probe the python service inside the proxy. A TCP accept + HTTP response
/// of ANY code proves the service is up (API scaffolds 404 on `/`), so
/// urllib's HTTPError counts as success; only connection failures retry.
/// The probe script is pushed as a file — the harness guard rejects `;` and
/// newlines in arguments, so a `-c` one-liner is impossible.
fn probe_python_service(port: u16) -> Result<(), String> {
    let script = format!(
        "import urllib.request, urllib.error\ntry:\n    r = urllib.request.urlopen('http://127.0.0.1:{port}/', timeout=3)\n    print(r.status)\nexcept urllib.error.HTTPError as e:\n    print('http', e.code)\nexcept Exception as e:\n    print('down', e)\n    raise SystemExit(1)\n"
    );
    let tmp = std::env::temp_dir().join(format!("gb-probe-{}.py", std::process::id()));
    std::fs::write(&tmp, script).map_err(|e| format!("write probe: {e}"))?;
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
fn verify_route_serving(site_host: &str) -> Result<(), String> {
    let script = format!(
        "import socket, ssl\nctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)\nctx.check_hostname = False\nctx.verify_mode = ssl.CERT_NONE\ntry:\n    raw = socket.create_connection(('127.0.0.1', 443), timeout=5)\n    tls = ctx.wrap_socket(raw, server_hostname='{site_host}')\n    tls.sendall((\n        'GET / HTTP/1.1\\r\\n'\n        'Host: {site_host}\\r\\n'\n        'Connection: close\\r\\n\\r\\n'\n    ).encode())\n    data = tls.recv(200)\n    tls.close()\nexcept Exception as e:\n    print('down', e)\n    raise SystemExit(1)\nif not data.startswith(b'HTTP/'):\n    print('nohttp', data[:80])\n    raise SystemExit(1)\nprint('ok', data.split(b'\\\\r\\\\n')[0].decode(errors='replace'))\n"
    );
    let tmp = std::env::temp_dir().join(format!("gb-verify-{}.py", std::process::id()));
    std::fs::write(&tmp, script).map_err(|e| format!("write verify probe: {e}"))?;
    let proxy_path = "/tmp/gb-vibe-verify.py";
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
            return Err(format!("verify probe push failed: {}", p.stderr.trim()));
        }
    } else {
        return Err("verify probe push error".to_string());
    }
    for attempt in 1..=4 {
        let out = proxy_exec(&["python3".to_string(), proxy_path.to_string()], 25);
        if let Ok(o) = &out {
            if o.exit_code == Some(0) {
                return Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1200 * attempt));
    }
    Err(format!("route for {site_host} does not serve through Caddy yet"))
}

/// Blocking core of [`deploy_site_to_proxy`] — runs under the publish lock.
fn deploy_site_to_proxy_sync(
    project: &crate::projects::Project,
    python: bool,
    verify: bool,
) -> Result<(String, String), String> {
    let _guard = lock_publish();
    let slug = site_slug(&project.name);
    validate_slug(&slug)?;
    let site_dir = format!("{PROXY_SITES_ROOT}/{slug}");
    if !dir_is_vibe_owned(&site_dir)? {
        return Err(format!(
            "refusing to publish: {site_dir} already exists and is not vibe-managed \
             (missing {MARKER_FILE}) — move it or choose another project name"
        ));
    }
    let files = super::publish::collect_workspace_files(project)?;
    check_serveability(&files, python)?;
    stage_payload(project, &site_dir)?;
    let mut service_note = String::new();
    if python {
        let port = ensure_python_service(&slug)?;
        probe_python_service(port)?;
        service_note = format!("gb-vibe-{slug}@127.0.0.1:{port}");
    }
    upsert_site_config(&format!("{slug}.{}", super::publish::published_domain()), &site_block(&slug, python))?;
    if verify {
        verify_route_serving(&format!("{slug}.{}", super::publish::published_domain()))?;
    }
    let host = format!("{slug}.{}", super::publish::published_domain());
    let url = format!("https://{host}/");
    log::info!(
        "Vibe publish {}: staged site to proxy {site_dir} host {host} {}",
        project.name,
        service_note
    );
    Ok((url, service_note))
}

/// Entry point used by `do_publish` for `website` and python `custom`
/// projects in production. Stages files into the proxy's websites tree and
/// registers the Caddyfile site block. Returns the public URL.
pub async fn deploy_site_to_proxy(
    project: &crate::projects::Project,
    python: bool,
) -> Result<(String, String), String> {
    let p = project.clone();
    tokio::task::spawn_blocking(move || deploy_site_to_proxy_sync(&p, python, true))
        .await
        .map_err(|e| format!("publish task: {e}"))?
}

/// Blocking core of [`rollback_site`].
fn rollback_site_sync(slug: &str) -> Result<String, String> {
    let _guard = lock_publish();
    validate_slug(slug)?;
    let site_dir = format!("{PROXY_SITES_ROOT}/{slug}");
    // `.prev-1` must exist and be vibe-owned.
    let prev = format!("{site_dir}.prev-1");
    let marker = proxy_exec(&["test".to_string(), "-f".to_string(), format!("{prev}/{MARKER_FILE}")], 15)?;
    if marker.exit_code != Some(0) {
        return Err(format!("no previous release retained for '{slug}' — nothing to roll back"));
    }
    // Current payload becomes .prev-1 (the target) → swap through .old.
    let old_dir = format!("{site_dir}.old");
    let _ = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir.clone()], 30);
    let _ = proxy_exec(&["mv".to_string(), site_dir.to_string(), old_dir.clone()], 20);
    must_run("promote previous", &["mv".to_string(), prev.clone(), site_dir.to_string()], 20)?;
    let _ = proxy_exec(&["mv".to_string(), old_dir, prev], 20);
    // Determine whether the payload is python (app.py present) and refresh
    // the service + route accordingly.
    let py = proxy_exec(
        &["test".to_string(), "-f".to_string(), format!("{site_dir}/app.py")],
        15,
    )?
    .exit_code == Some(0);
    if py {
        let port = ensure_python_service(slug)?;
        probe_python_service(port)?;
    }
    upsert_site_config(
        &format!("{slug}.{}", super::publish::published_domain()),
        &site_block(slug, py),
    )?;
    verify_route_serving(&format!("{slug}.{}", super::publish::published_domain()))?;
    let url = format!("https://{slug}.{}/", super::publish::published_domain());
    log::info!("Vibe rollback {slug}: previous release reactivated");
    Ok(url)
}

/// Reactivate the previous release of a site (`<site>.prev-1` → live) and
/// refresh its Caddy route / python service. Serialized with publishes.
pub async fn rollback_site(slug: &str) -> Result<String, String> {
    let slug = slug.to_string();
    tokio::task::spawn_blocking(move || rollback_site_sync(&slug))
        .await
        .map_err(|e| format!("rollback task: {e}"))?
}

/// Blocking core of [`unpublish_site`].
fn unpublish_site_sync(slug: &str, purge: bool) -> Result<(), String> {
    let _guard = lock_publish();
    validate_slug(slug)?;
    let site_dir = format!("{PROXY_SITES_ROOT}/{slug}");
    // Guard: only vibe-managed sites may be unpublished.
    let marker = proxy_exec(
        &["test".to_string(), "-f".to_string(), format!("{site_dir}/{MARKER_FILE}")],
        15,
    )?;
    if marker.exit_code != Some(0) {
        return Err(format!(
            "refusing to unpublish: {site_dir} is not vibe-managed (missing {MARKER_FILE})"
        ));
    }
    // 1. Route out first (the public face goes away immediately).
    remove_site_config(&format!("{slug}.{}", super::publish::published_domain()))?;
    // 2. Stop + disable + remove the python service when present.
    let unit = format!("/etc/systemd/system/gb-vibe-{slug}.service");
    let has_unit = proxy_exec(&["test".to_string(), "-f".to_string(), unit.clone()], 15)?;
    if has_unit.exit_code == Some(0) {
        let _ = proxy_exec(
            &[
                "systemctl".to_string(),
                "stop".to_string(),
                format!("gb-vibe-{slug}"),
            ],
            30,
        );
        let _ = proxy_exec(
            &[
                "systemctl".to_string(),
                "disable".to_string(),
                format!("gb-vibe-{slug}"),
            ],
            30,
        );
        let _ = proxy_exec(&["rm".to_string(), "-f".to_string(), unit], 15);
        let _ = proxy_exec(&["systemctl".to_string(), "daemon-reload".to_string()], 30);
    }
    // 3. Payload: keep `.prev-*` history by default, purge everything on
    //    explicit request.
    if purge {
        let _ = proxy_exec(
            &["rm".to_string(), "-rf".to_string(), format!("{site_dir}.prev-*")],
            120,
        );
    }
    let _ = proxy_exec(
        &[
            "mv".to_string(),
            site_dir.clone(),
            format!("{site_dir}.unpublished"),
        ],
        20,
    );
    log::info!("Vibe unpublish {slug}: route removed, payload retired (purge={purge})");
    Ok(())
}

/// Remove a site from the proxy: route first, then service, then payload
/// (retained as `<site>.unpublished` unless `purge` is set).
pub async fn unpublish_site(slug: &str, purge: bool) -> Result<(), String> {
    let slug = slug.to_string();
    tokio::task::spawn_blocking(move || unpublish_site_sync(&slug, purge))
        .await
        .map_err(|e| format!("unpublish task: {e}"))?
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
        assert!(b.contains("tls internal"));
    }

    #[test]
    fn site_block_python_has_reverse_proxy() {
        let b = site_block("pysite", true);
        assert!(b.contains(&format!("reverse_proxy 127.0.0.1:{}", python_port("pysite"))));
    }

    #[test]
    fn validate_slug_enforces_enterprise_rules() {
        assert!(validate_slug("mysite").is_ok());
        assert!(validate_slug("my-site-2").is_ok());
        // too short / too long
        assert!(validate_slug("ab").is_err());
        assert!(validate_slug(&"a".repeat(64)).is_err());
        // charset
        assert!(validate_slug("MySite").is_err());
        assert!(validate_slug("my_site").is_err());
        assert!(validate_slug(".hidden").is_err());
        // hyphen edges
        assert!(validate_slug("-lead").is_err());
        assert!(validate_slug("trail-").is_err());
        // reserved infrastructure names
        for r in RESERVED_SLUGS {
            assert!(validate_slug(r).is_err(), "{r} must be reserved");
        }
    }

    #[test]
    fn check_payload_limits_enforce_caps() {
        let small = serde_json::json!([{ "path": "index.html", "content": [104, 105] }]);
        assert!(check_payload_limits(small.as_array().unwrap()).is_ok());
        // string content is accepted as utf8 bytes
        let textual = serde_json::json!([{ "path": "index.html", "content": "hi" }]);
        assert!(check_payload_limits(textual.as_array().unwrap()).is_ok());
        // per-file cap: 11 MiB single file
        let big = serde_json::json!([{
            "path": "big.bin",
            "content": "x".repeat(MAX_SINGLE_FILE_BYTES + 1)
        }]);
        assert!(check_payload_limits(big.as_array().unwrap()).is_err());
        // total cap: many files just under the per-file cap
        let file = "x".repeat(MAX_SINGLE_FILE_BYTES - 1);
        let many: Vec<serde_json::Value> = (0..6)
            .map(|i| serde_json::json!({ "path": format!("f{i}.bin"), "content": file }))
            .collect();
        assert!(check_payload_limits(&many).is_err());
        // file count cap
        let too_many: Vec<serde_json::Value> = (0..(MAX_FILES + 1))
            .map(|i| serde_json::json!({ "path": format!("f{i}.txt"), "content": "x" }))
            .collect();
        assert!(check_payload_limits(&too_many).is_err());
    }

    #[test]
    fn check_serveability_requires_entry_files() {
        let no_index = serde_json::json!([{ "path": "page.html", "content": "x" }]);
        assert!(check_serveability(no_index.as_array().unwrap(), false).is_err());
        let with_index =
            serde_json::json!([{ "path": "index.html", "content": "x" }]);
        assert!(check_serveability(with_index.as_array().unwrap(), false).is_ok());
        let no_app = serde_json::json!([{ "path": "main.py", "content": "x" }]);
        assert!(check_serveability(no_app.as_array().unwrap(), true).is_err());
        let with_app = serde_json::json!([{ "path": "app.py", "content": "x" }]);
        assert!(check_serveability(with_app.as_array().unwrap(), true).is_ok());
    }

    #[test]
    fn drop_site_block_keeps_peers_and_removes_target() {
        let d = super::super::publish::published_domain();
        let section = format!(
            "a.{d} {{\n\ttls internal\n}}\nb.{d} {{\n\ttls internal\n}}\n"
        );
        let kept = drop_site_block(&section, &format!("a.{d}"));
        assert!(!kept.contains(&format!("a.{d}")));
        assert!(kept.contains(&format!("b.{d}")));
        // Unknown host → section unchanged.
        let kept2 = drop_site_block(&section, &format!("zz.{d}"));
        assert!(kept2.contains(&format!("a.{d}")));
        assert!(kept2.contains(&format!("b.{d}")));
    }

    #[test]
    fn drop_foreign_domain_blocks_removes_stale_domains() {
        let d = super::super::publish::published_domain();
        let stale_domain = if d == "generalbots.org" { "gb.solutions" } else { "generalbots.org" };
        let section = format!(
            "site.{d} {{\n\tfile_server\n}}\nstale.{stale_domain} {{\n\tfile_server\n}}\ncustom.example.com {{\n\tfile_server\n}}\n"
        );
        let kept = drop_foreign_domain_blocks(&section);
        assert!(kept.contains(&format!("site.{d}")));
        assert!(!kept.contains("stale."));
        assert!(!kept.contains("custom.example.com"));
    }

    #[test]
    fn extract_section_handles_missing_markers() {
        assert_eq!(extract_section("no markers here"), "");
        let doc = format!("head\n{SECTION_BEGIN}\nb.{}.com {{}}\n{SECTION_END}\ntail", "x");
        assert!(extract_section(&doc).contains("b."));
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
