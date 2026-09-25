//! `proxy_sites::caddy` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Websites root INSIDE the proxy container (matches prod layout). Production
/// site dirs stay `{root}/{slug}`; the test environment appends `-test`
/// (see [`crate::site_env::SiteTarget`]). Referenced by the unit tests that
/// pin the site-block shape; production code goes through SiteTarget.
#[cfg(test)]
pub(crate) const PROXY_SITES_ROOT: &str = "/opt/gbo/data/websites";

/// Caddyfile inside the proxy container (matches prod layout).
pub(crate) const PROXY_CADDY_CONFIG: &str = "/opt/gbo/conf/config";

/// Enterprise limits — a publish must never be able to fill the proxy disk
/// or wedge the command guard with thousands of tiny files.
pub(crate) const MAX_TOTAL_BYTES: usize = 50 * 1024 * 1024; // 50 MiB per site

/// Names a project can never take: proxy/infra hostnames and Caddyfile
/// artifact names. These either collide with infra or with the managed
/// config artifacts (`*.prev-*` dirs would be matched by site listing).
pub(crate) const RESERVED_SLUGS: [&str; 10] = [
    "proxy", "caddy", "bot", "api", "www", "mail", "smtp", "vault", "grafana", "admin",
];

/// `incus` runs on the host (or WSL); every proxy interaction is
/// `incus exec proxy -- <argv>` or `incus file push|pull`.
pub(crate) fn proxy_exec(args: &[String], timeout: u64) -> Result<crate::harness::cmd::RunOutput, String> {
    let mut full = vec!["exec".to_string(), "proxy".to_string(), "--".to_string()];
    full.extend_from_slice(args);
    crate::harness::cmd::run("incus", &full, Path::new("."), timeout)
        .map_err(|e| format!("incus exec proxy: {e}"))
}

pub(crate) fn must_run(
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
/// across restarts so the Caddy route never churns. The dev environment
/// hashes a DIFFERENT string (`{slug}-test`) so production and test services
/// of the same site never share a port.
pub fn python_port(slug: &str) -> u16 {
    let hash: u32 = slug
        .bytes()
        .fold(5381u32, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u32));
    (20000 + (hash % 9999)) as u16
}

/// Port for a (slug, env) pair — test sites derive from the test dir name so
/// they cannot collide with the production service of the same site.
pub(crate) fn python_port_for(target: &SiteTarget, slug: &str) -> u16 {
    let key = if target.host.starts_with(&format!("{slug}-test.")) {
        format!("{slug}-test")
    } else {
        slug.to_string()
    };
    python_port(&key)
}

/// `true` when `dir` is absent (free to create) or vibe-owned (marker file).
/// `Ok(false)` = exists but foreign → publish must refuse.
pub(crate) fn dir_is_vibe_owned(dir: &str) -> Result<bool, String> {
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
pub(crate) fn check_python_runtime() -> Result<(), String> {
    let out = proxy_exec(&["python3".to_string(), "--version".to_string()], 20)?;
    if out.exit_code != Some(0) {
        return Err(
            "python3 runtime not available in proxy container — publish the python project to a project VM instead"
                .to_string(),
        );
    }
    Ok(())
}

/// Enterprise payload caps: enforce size/file limits BEFORE transferring so
/// a runaway agent can never fill the proxy disk.
pub(crate) fn check_payload_limits(files: &[serde_json::Value]) -> Result<(), String> {
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
pub(crate) fn check_serveability(files: &[serde_json::Value], python: bool) -> Result<(), String> {
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
pub(crate) fn stage_payload(
    project: &crate::projects::Project,
    site_dir: &str,
    deploy_rev: Option<&str>,
) -> Result<(), String> {
    let files = crate::deploy_source::materialize_files(project, deploy_rev)?;
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
pub(crate) fn rotate_release(site_dir: &str) -> Result<(), String> {
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

/// Render the Caddyfile site block for one site. TLS mode is
/// environment-aware: production relies on Caddy's automatic ACME
/// (wildcard `*.{domain}` DNS already points there — a new site gets a
/// Let's Encrypt cert on demand, exactly like the platform sites), while
/// dev stacks set `GB_VIBE_TLS_INTERNAL=1` to serve locally-trusted certs
/// (ACME can never complete for a domain that doesn't resolve to dev).
pub(crate) fn tls_directive(internal: bool) -> String {
    if internal {
        "\ttls internal\n".to_string()
    } else {
        String::new()
    }
}

pub(crate) fn tls_internal_from_env() -> bool {
    std::env::var("GB_VIBE_TLS_INTERNAL")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// Test-mode block for the LEGACY prod target (kept for the unit tests that
/// pin the site-block shape against regressions).
#[cfg(test)]
pub(crate) fn site_block_with_mode(slug: &str, python: bool, tls_internal: bool) -> String {
    let target = SiteTarget::new(
        slug,
        SiteEnv::Production,
        &crate::publish::published_domain(),
    );
    site_block_for_target(&target, slug, python, tls_internal)
}
