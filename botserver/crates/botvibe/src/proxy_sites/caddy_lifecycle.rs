//! `proxy_sites::caddy_lifecycle` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// #1290 — promote the current TEST release of a site to PRODUCTION. The test
/// live payload is archived (tar) and staged into the production target
/// through the same swap path as a normal publish, so the production
/// `.prev-N` ring, route and (for python) service are refreshed exactly like
/// a direct deploy.
pub async fn promote_site_test_to_prod(
    project: &crate::projects::Project,
    python: bool,
    pool: &crate::types::DbPool,
) -> Result<String, String> {
    let slug = site_slug(&project.name, project.branch_id);
    let domain = crate::publish::published_domain();
    let (prod, test) = crate::site_env::both_targets(&slug, &domain);
    let pool = pool.clone();
    let project = project.clone();
    let promote: Result<String, String> = tokio::task::spawn_blocking(move || {
        let _guard = lock_publish();
        validate_slug(&slug)?;
        validate_project_site_name(&project.name)?;
        // The test payload must exist and be vibe-owned.
        let marker = proxy_exec(
            &["test".to_string(), "-f".to_string(), format!("{dir}/{MARKER_FILE}", dir = test.dir)],
            15,
        )?;
        if marker.exit_code != Some(0) {
            return Err(format!(
                "no test release for '{slug}' — publish the project to its test site first"
            ));
        }
        // Prod target must be free or vibe-owned.
        if !dir_is_vibe_owned(&prod.dir)? {
            return Err(format!(
                "refusing to promote: {} exists and is not vibe-managed",
                prod.dir
            ));
        }
        // Stage: archive the test payload inside the proxy, extract into the
        // production staging dir, then reuse the standard swap.
        let arch = format!("/tmp/gb-promote-{}.tar", std::process::id());
        if let Err(e) = proxy_exec(&["rm".to_string(), "-f".to_string(), arch.clone()], 15) {
            log::warn!("Vibe promote: remove stale archive failed: {e}");
        }
        must_run(
            "archive test payload",
            &[
                "tar".to_string(),
                "-cf".to_string(),
                arch.clone(),
                "-C".to_string(),
                test.dir.clone(),
                ".".to_string(),
            ],
            120,
        )?;
        let new_dir = format!("{}.new", prod.dir);
        if let Err(e) = must_run("cleanup promote staging", &["rm".to_string(), "-rf".to_string(), new_dir.clone()], 30) {
            log::warn!("Vibe promote: clean staging dir failed: {e}");
        }
        must_run("mkdir promote staging", &["mkdir".to_string(), "-p".to_string(), new_dir.clone()], 20)?;
        let extract = must_run(
            "extract promote staging",
            &["tar".to_string(), "-xf".to_string(), arch.clone(), "-C".to_string(), new_dir.clone()],
            120,
        );
        if let Err(e) = proxy_exec(&["rm".to_string(), "-f".to_string(), arch.clone()], 20) {
            log::warn!("Vibe promote: remove promote archive failed: {e}");
        }
        extract?;
        // The marker must survive the copy; add it if the archive missed it.
        if let Err(e) = proxy_exec(&["touch".to_string(), format!("{new_dir}/{MARKER_FILE}")], 15) {
            log::warn!("Vibe promote: write ownership marker failed: {e}");
        }
        // Refresh the prod release ring and swap payloads.
        rotate_release(&prod.dir)?;
        let old_dir = format!("{}.old", prod.dir);
        if let Err(e) = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir.clone()], 30) {
            log::warn!("Vibe promote: remove stale old dir failed: {e}");
        }
        if let Err(e) = proxy_exec(&["mv".to_string(), prod.dir.clone(), old_dir.clone()], 20) {
            log::warn!("Vibe promote: move current payload to old dir failed: {e}");
        }
        must_run("promote test→prod", &["mv".to_string(), new_dir, prod.dir.clone()], 20)?;
        if let Err(e) = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir], 30) {
            log::warn!("Vibe promote: remove retired old dir failed: {e}");
        }
        // Service + route refresh identical to a direct prod deploy: the
        // caller's python flag decides the mode (the promoted payload carries
        // app.py exactly when the test site is python).
        let py = python;
        let mut service_note = String::new();
        if py {
            let port = python_port_for(&prod, &slug);
            // #1386 — production gets the project's public database (the dev
            // twin kept the `_dev` one), so promotion never repoints the
            // public site at test data.
            let database_url = crate::project_db::ensure_project_database(
                &pool,
                project.branch_id,
                &project.name,
                SiteEnv::Production.as_str(),
            )?;
            ensure_python_service_for(&slug, &prod.dir, port, SiteEnv::Production, Some(&database_url))?;
            probe_python_service(port)?;
            service_note = format!("gb-vibe-{slug}@127.0.0.1:{port}");
        }
        upsert_site_config(&prod.host, &site_block_for_target(&prod, &slug, py, tls_internal_from_env()))?;
        verify_route_serving(&prod.host)?;
        log::info!("Vibe promote {slug}: test release promoted to {}", prod.host);
        Ok(format!("https://{}/ ({service_note})", prod.host))
    })
    .await
    .map_err(|e| format!("promote task: {e}"))?;
    promote
}

/// Blocking core of the rollback entry points — #1290: env-aware via target.
pub(crate) fn rollback_site_for_sync(slug: &str, target: &SiteTarget, env: SiteEnv) -> Result<String, String> {
    let _guard = lock_publish();
    validate_slug(slug)?;
    let site_dir = target.dir.clone();
    // `.prev-1` must exist and be vibe-owned.
    let prev = format!("{site_dir}.prev-1");
    let marker = proxy_exec(&["test".to_string(), "-f".to_string(), format!("{prev}/{MARKER_FILE}")], 15)?;
    if marker.exit_code != Some(0) {
        return Err(format!(
            "no previous release retained for '{}' ({}) — nothing to roll back",
            slug,
            env.as_str()
        ));
    }
    // Current payload becomes .prev-1 (the target) → swap through .old.
    let old_dir = format!("{site_dir}.old");
    if let Err(e) = proxy_exec(&["rm".to_string(), "-rf".to_string(), old_dir.clone()], 30) {
        log::warn!("Vibe rollback: remove stale old dir failed: {e}");
    }
    if let Err(e) = proxy_exec(&["mv".to_string(), site_dir.to_string(), old_dir.clone()], 20) {
        log::warn!("Vibe rollback: move current payload to old dir failed: {e}");
    }
    must_run("promote previous", &["mv".to_string(), prev.clone(), site_dir.to_string()], 20)?;
    if let Err(e) = proxy_exec(&["mv".to_string(), old_dir, prev], 20) {
        log::warn!("Vibe rollback: restore previous release ring failed: {e}");
    }
    // Determine whether the payload is python (app.py present) and refresh
    // the service + route accordingly.
    let py = proxy_exec(
        &["test".to_string(), "-f".to_string(), format!("{site_dir}/app.py")],
        15,
    )?
    .exit_code == Some(0);
    if py {
        let port = python_port_for(target, slug);
        // #1386 — rollback rewrites the unit file; the database URL lives in
        // the persistent environment file written by the last publish, so no
        // database access is needed here.
        ensure_python_service_for(slug, &site_dir, port, env, None)?;
        probe_python_service(port)?;
    }
    upsert_site_config(&target.host, &site_block_for_target(target, slug, py, tls_internal_from_env()))?;
    verify_route_serving(&target.host)?;
    let url = format!("https://{}/", target.host);
    log::info!("Vibe rollback {slug} ({}): previous release reactivated", env.as_str());
    Ok(url)
}

/// Reactivate the previous release of a site (`<site>.prev-1` → live) and
/// refresh its Caddy route / python service. Serialized with publishes.
pub async fn rollback_site(slug: &str) -> Result<String, String> {
    let slug = slug.to_string();
    let target = SiteTarget::new(&slug, SiteEnv::Production, &crate::publish::published_domain());
    tokio::task::spawn_blocking(move || rollback_site_for_sync(&slug, &target, SiteEnv::Production))
        .await
        .map_err(|e| format!("rollback task: {e}"))?
}

/// #1290 — roll back the TEST release (`{slug}-test`).
pub async fn rollback_site_test(slug: &str) -> Result<String, String> {
    let slug = slug.to_string();
    let target = SiteTarget::new(&slug, SiteEnv::Test, &crate::publish::published_domain());
    tokio::task::spawn_blocking(move || rollback_site_for_sync(&slug, &target, SiteEnv::Test))
        .await
        .map_err(|e| format!("rollback task: {e}"))?
}

/// Blocking core of [`unpublish_site`] — #1290: env-aware via the target.
pub(crate) fn unpublish_site_for_sync(slug: &str, purge: bool, target: &SiteTarget, env: SiteEnv) -> Result<(), String> {
    let _guard = lock_publish();
    validate_slug(slug)?;
    let site_dir = target.dir.clone();
    // 1. Route out FIRST — the public face goes away immediately even when
    //    the payload dir is absent (VM-deployed sites have no payload dir;
    //    the early return below would otherwise leak the Caddy route).
    remove_site_config(&target.host)?;
    // Nothing published → nothing else to unpublish (idempotent for deletes
    // and project eviction; the caller logs a warning otherwise).
    let exists = proxy_exec(
        &["test".to_string(), "-d".to_string(), site_dir.clone()],
        15,
    )?;
    if exists.exit_code != Some(0) {
        return Ok(());
    }
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
    // 2. Stop + disable + remove the python service when present. The unit
    //    name is env-suffixed for the test twin so both services stay distinct.
    let unit_name = site_unit_name(slug, env);
    let unit = format!("/etc/systemd/system/gb-vibe-{unit_name}.service");
    let has_unit = proxy_exec(&["test".to_string(), "-f".to_string(), unit.clone()], 15)?;
    if has_unit.exit_code == Some(0) {
        if let Err(e) = proxy_exec(
            &[
                "systemctl".to_string(),
                "stop".to_string(),
                format!("gb-vibe-{unit_name}"),
            ],
            30,
        ) {
            log::warn!("Vibe unpublish: stop python service failed: {e}");
        }
        if let Err(e) = proxy_exec(
            &[
                "systemctl".to_string(),
                "disable".to_string(),
                format!("gb-vibe-{unit_name}"),
            ],
            30,
        ) {
            log::warn!("Vibe unpublish: disable python service failed: {e}");
        }
        if let Err(e) = proxy_exec(&["rm".to_string(), "-f".to_string(), unit], 15) {
            log::warn!("Vibe unpublish: remove python unit failed: {e}");
        }
        if let Err(e) = proxy_exec(&["systemctl".to_string(), "daemon-reload".to_string()], 30) {
            log::warn!("Vibe unpublish: systemd daemon-reload failed: {e}");
        }
    }
    // 3. Payload: keep `.prev-*` history by default, purge everything on
    //    explicit request.
    if purge {
        if let Err(e) = proxy_exec(
            &["rm".to_string(), "-rf".to_string(), format!("{site_dir}.prev-*")],
            120,
        ) {
            log::warn!("Vibe unpublish: purge release history failed: {e}");
        }
    }
    if let Err(e) = proxy_exec(
        &[
            "mv".to_string(),
            site_dir.clone(),
            format!("{site_dir}.unpublished"),
        ],
        20,
    ) {
        log::warn!("Vibe unpublish: retire payload dir failed: {e}");
    }
    log::info!("Vibe unpublish {slug} ({}): route removed, payload retired (purge={purge})", env.as_str());
    Ok(())
}

/// Remove a site from the proxy: route first, then service, then payload
/// (retained as `<site>.unpublished` unless `purge` is set).
pub async fn unpublish_site(slug: &str, purge: bool) -> Result<(), String> {
    let slug = slug.to_string();
    let target = SiteTarget::new(&slug, SiteEnv::Production, &crate::publish::published_domain());
    tokio::task::spawn_blocking(move || {
        unpublish_site_for_sync(&slug, purge, &target, SiteEnv::Production)
    })
    .await
    .map_err(|e| format!("unpublish task: {e}"))?
}

/// #1290 — take the TEST site (`{slug}-test`) off the proxy.
pub async fn unpublish_site_test(slug: &str, purge: bool) -> Result<(), String> {
    let slug = slug.to_string();
    let target = SiteTarget::new(&slug, SiteEnv::Test, &crate::publish::published_domain());
    tokio::task::spawn_blocking(move || {
        unpublish_site_for_sync(&slug, purge, &target, SiteEnv::Test)
    })
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
