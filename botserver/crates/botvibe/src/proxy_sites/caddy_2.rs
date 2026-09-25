//! `proxy_sites::caddy_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Render the block for an arbitrary target (production or test). The test
/// python port differs from production's so both services can run at once.
pub(crate) fn site_block_for_target(target: &SiteTarget, slug: &str, python: bool, tls_internal: bool) -> String {
    let site_host = target.host.clone();
    let tls = tls_directive(tls_internal);
    if python {
        let port = python_port_for(target, slug);
        format!("{site_host} {{\n{tls}\treverse_proxy 127.0.0.1:{port}\n}}\n")
    } else {
        format!(
            "{site_host} {{\n{tls}\troot * {dir}\n\tfile_server\n\tencode zstd gzip\n}}\n",
            dir = target.dir
        )
    }
}

/// Pull the proxy Caddyfile to a local temp file. Caller owns the temp file.
pub(crate) fn pull_proxy_config() -> Result<std::path::PathBuf, String> {
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

pub(crate) fn push_proxy_config(local: &Path) -> Result<(), String> {
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
pub(crate) fn validate_candidate(candidate: &Path) -> Result<(), String> {
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

/// Reload caddy against the LIVE config path. The harness runs commands
/// with a cleared environment, so `CADDY_ADMIN` is passed explicitly as an
/// argv env assignment. The admin endpoint binds `0.0.0.0:2019`; Caddy's
/// origin enforcement only accepts the dial host `localhost` (the CLI
/// normalizes `127.0.0.1` to Host `0.0.0.0:2019`, which is rejected with
/// HTTP 403 — verified against the dev proxy).
pub(crate) fn reload_caddy() -> Result<(), String> {
    must_run(
        "caddy reload",
        &[
            "env".to_string(),
            "CADDY_ADMIN=localhost:2019".to_string(),
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
pub(crate) fn backup_proxy_config() -> Result<(), String> {
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

/// Drop `site_host`'s block from a section, keeping every other site block
/// untouched. The section key is the FULL HOST (the block header) — the old
/// slug-vs-host comparison never matched and left duplicate blocks (Caddy
/// "ambiguous site definition").
pub(crate) fn drop_site_block(existing_section: &str, site_host: &str) -> String {
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
pub(crate) fn drop_foreign_domain_blocks(section: &str) -> String {
    let suffix = format!(".{}", crate::publish::published_domain());
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
pub(crate) fn upsert_site_config(site_host: &str, block: &str) -> Result<(), String> {
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
pub(crate) fn remove_site_config(site_host: &str) -> Result<(), String> {
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
