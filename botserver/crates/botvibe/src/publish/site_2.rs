//! `publish::site_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) async fn do_publish(args: Value, pool: crate::types::DbPool) -> Result<Value, String> {
    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "publish/project requires 'project_id' (uuid string)".to_string())?;
    let env = args
        .get("env")
        .and_then(|v| v.as_str())
        .unwrap_or(PUBLISH_DEFAULT_ENV)
        .to_lowercase();
    if !crate::vm_lifecycle::VALID_ENVS.contains(&env.as_str()) {
        return Err(format!("invalid env '{env}'"));
    }
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    // #1280 — the acting user for the downstream deployment RBAC gate. The
    // agent loop stamps the initiating user's id into the tool arguments;
    // direct REST/ops callers have their session user enforced at the API
    // layer. Internal (X-Internal-Token) requests carry a nil session user,
    // so the deployment handler relies on this field to authorize.
    let on_behalf_of_user = args
        .get("on_behalf_of_user")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .filter(|id| !id.is_nil());
    // #1160 — desktop launch surface: `launcher` auto-pins the app to the
    // launcher; `widget` marks the app as an always-visible desktop widget.
    let launcher_requested = args
        .get("launcher")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let widget_requested = args
        .get("widget")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let registry = ProjectRegistry::new(pool.clone());
    let project = registry
        .get(project_id)?
        .ok_or_else(|| format!("project {project_id} not found"))?;

    let metering = crate::metering::VMetering::new(pool.clone());
    metering.enforce_for_project(project_id, crate::metering::MeterKind::BuildMinutes)?;

    // #1288 — website and python projects in production are served straight
    // from the proxy container's shared websites dir (Caddy file_server),
    // NOT from a dedicated prod VM (too expensive) and NOT from the bot
    // container (the proxy cannot see its filesystem). One early return
    // covers both kinds; every side effect below (deployment history,
    // launcher, domain binding) is shared with the normal path.
    let is_python_project = (project.project_type == "apps" || project.project_type == "custom")
        && project
            .framework
            .as_deref()
            .map(|f| f.eq_ignore_ascii_case("python") || f.eq_ignore_ascii_case("python3") || f.eq_ignore_ascii_case("flask"))
            .unwrap_or(false);
    // #1290 — site publishes go to the proxy's targets instead of raising an
    // expensive per-site VM: the working copy at `{slug}-test.{domain}`
    // (`websites/{slug}-test`) and the public page at `{slug}.{domain}`
    // (`websites/{slug}`).
    //
    // Two websites per project: a site publish defaults to the TEST twin, so
    // an edit under test can never blank the live page. The public slug is
    // written only when the deploy pipeline (or an admin ops rollback) stamps
    // the request as approved — an agent that asks for `env=production` on its
    // own still lands on the test twin, and the response says so.
    let wants_site = project.project_type == "website" || is_python_project;
    // #1505 — optional explicit revision to deploy (git tag, branch or SHA);
    // empty/absent means HEAD of the project checkout. Only meaningful for
    // git-mode projects.
    let deploy_rev: Option<String> = args
        .get("deploy_rev")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(ToString::to_string);
    let approved = production_approved(&args);
    let requested_production = matches!(
        crate::site_env::SiteEnv::parse(&env),
        Some(crate::site_env::SiteEnv::Production)
    );
    let site_env = if !wants_site {
        None
    } else if requested_production && approved {
        Some(crate::site_env::SiteEnv::Production)
    } else {
        Some(crate::site_env::SiteEnv::Test)
    };
    if let Some(site_env) = site_env {
        {
            // Sanctioned production publish: stage the workspace into the test
            // twin first, then promote THAT release to the public slug. Both
            // websites stay in step, the public page is always a payload that
            // has already been served (and verified) as the test site, and the
            // production `.prev-N` ring still holds the previous release.
            // #1505 — a sanctioned production deploy ships the pushed git
            // state: resolve the revision once (explicit arg > HEAD) and carry
            // it through both the test staging and the promote-to-prod swap.
            let deploy_rev = crate::deploy_source::deploy_revision(&project, deploy_rev.as_deref())?;
            let (proxy_url, route) = if site_env == crate::site_env::SiteEnv::Production {
                // The test twin's URL is intentionally discarded: the public
                // release is what this call reports, and the twin's own route
                // is refreshed by the staging deploy above.
                crate::proxy_sites::deploy_site_to_proxy_env(
                    &project,
                    is_python_project,
                    crate::site_env::SiteEnv::Test,
                    &pool,
                    deploy_rev.clone(),
                )
                .await?;
                let promoted = crate::proxy_sites::promote_site_test_to_prod(
                    &project,
                    is_python_project,
                    &pool,
                )
                .await?;
                (promoted, "promoted-from-test-twin".to_string())
            } else {
                crate::proxy_sites::deploy_site_to_proxy_env(
                    &project,
                    is_python_project,
                    site_env,
                    &pool,
                    deploy_rev.clone(),
                )
                .await?
            };
            // The recorded env is the canonical site env (test/production) so
            // the UI can resolve the right preview URL per environment.
            let site_env_name = site_env.as_str();
            let deployment = serde_json::json!({
                "env": site_env_name,
                "at": chrono::Utc::now().to_rfc3339(),
                "url": proxy_url,
                "container": "proxy",
                "deploy_target": "proxy-websites",
                "caddy_route": route,
                "domain": domain.clone().unwrap_or_default(),
                "track": "ok",
            });
        registry
            .append_deployment(project_id, &deployment)
            .map_err(|e| format!("record deployment: {e}"))?;
        let launch_info = if launcher_requested || widget_requested {
            let launch = serde_json::json!({
                "enabled": true,
                "kind": if widget_requested { "widget" } else { "app" },
                "at": chrono::Utc::now().to_rfc3339(),
                "env": site_env_name,
            });
            registry
                .set_launcher(project_id, &launch)
                .map_err(|e| format!("record launcher flag: {e}"))?;
            Some(launch)
        } else {
            None
        };
        let binding = match &domain {
            Some(d) => {
                let bind_req = BindDomainRequest {
                    domain: d.clone(),
                    env: site_env_name.to_string(),
                    access: None,
                    allowed_emails: None,
                };
                match ProjectDomains::new(pool).bind(project_id, &bind_req).await {
                    Ok(b) => serde_json::json!({ "bound": true, "id": b.id, "domain": b.domain, "env": b.env, "container": b.container, "verified": b.verified, "tls_status": b.tls_status }),
                    Err(e) => serde_json::json!({ "bound": false, "error": e }),
                }
            }
            None => serde_json::json!({ "bound": false, "error": "no domain provided" }),
        };
        // A production request that was not sanctioned by the deploy pipeline
        // is served by the test twin; report where the release actually went
        // so the agent (and the user) never assume the live page changed.
        let note = if requested_production && !approved {
            format!(
                "release published to the test site ({}) — the public site ({}) is only updated by the Deploy action",
                deployment["url"].as_str().unwrap_or_default(),
                format!("https://{}.{}", crate::proxy_sites::site_slug(&project.name), crate::publish::published_domain()),
            )
        } else {
            String::new()
        };
        return Ok(serde_json::json!({
            "published": true,
            "project": project.name,
            "env": site_env_name,
            "requested_env": env,
            "note": note,
            "container": "proxy",
            "deploy_target": "proxy-websites",
            "url": proxy_url,
            "domain": domain,
            "domain_bind": binding,
            "deployment": deployment,
            "launcher": launch_info,
            "history_key": "deployments",
        }));
        }
    }

    #[cfg(not(target_os = "windows"))]
    let repo_name = VmLifecycle::alm_repo(&project.name);
    #[cfg(not(target_os = "windows"))]
    let org = VmLifecycle::alm_org(project.branch_id);

    let vm_req = CreateVmRequest {
        env: env.clone(),
        tier: "small".to_string(),
        runner_enabled: false,
    };
    let vm = VmLifecycle::new(pool.clone()).create_project_vm(
        project_id,
        project.branch_id,
        &project.name,
        &vm_req,
    )?;

    // Self-hosted ALM (Forgejo): Vault `secret/gbo/alm` → env → localhost.
    #[cfg(not(target_os = "windows"))]
    let (alm_base, _, _) = botcoresecrets::alm_config();

    #[cfg(not(target_os = "windows"))]
    let target = match &domain {
        Some(d) => serde_json::json!({
            "External": {
                "repo_url": format!("{}/{}/{}", alm_base.trim_end_matches('/'), org, repo_name),
                "custom_domain": d,
                "ci_cd_enabled": true
            }
        }),
        None => serde_json::json!({
            "Internal": {
                "route": format!("{repo_name}-{env}.{}", published_domain()),
                "shared_resources": true
            }
        }),
    };

    let files = collect_workspace_files(&project)?;
    // #1444 M4 — an empty workspace must fail loudly: publishing `files: []`
    // used to record track:ok and bill BuildMinutes while the user saw
    // published:true for an empty repo. Billing happens only after this
    // guard, so a phantom deploy is never billed.
    if files.is_empty() {
        return Err(format!(
            "cannot publish an empty workspace for project '{}' — run the project first so the scaffold lands, then publish",
            project.name
        ));
    }
    let _ = metering.add_for_project(
        project_id,
        &env,
        crate::metering::MeterKind::BuildMinutes,
        1.0,
    );
    // The Vibe project registry uses the user-facing kinds `apps` and
    // `website`, while the deployment API accepts `app-*` and `site`.
    // Translate at the boundary so a calculator/apps app is deployable.
    // (#1291 — legacy rows keep the `custom` kind and translate identically.)
    let deployment_type = match project.project_type.as_str() {
        "website" => "site".to_string(),
        "apps" | "custom" => format!("app-{}", project.framework.as_deref().unwrap_or("node")),
        other if other == "site" || other == "bot" || other.starts_with("app-") => {
            other.to_string()
        }
        _ => "app-node".to_string(),
    };
    // #1386 — resolve the project's per-environment database ONCE here and
    // carry it through the deployment API → gateway → app.service, so every
    // redeploy of an environment keeps pointing at the same database (the
    // gateway no longer invents one). App-kind projects get the public DB in
    // production and the `_dev` twin in test/development; sites and bots
    // deploy without a DATABASE_URL.
    let deploy_db_url = if deployment_type.starts_with("app-") {
        let db_env = if env == "production" { "production" } else { "dev" };
        match crate::project_db::ensure_project_database(&pool, project.branch_id, &project.name, db_env) {
            Ok(url) => Some(url),
            Err(e) => {
                log::warn!(
                    "Vibe publish {}: per-env database unavailable ({db_env}), deploying without DATABASE_URL: {e}",
                    project.name
                );
                None
            }
        }
    } else {
        None
    };
    #[cfg(target_os = "windows")]
    let deployed: Value = {
        let host_port = std::env::var("VIBE_WSL_APP_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(80);
        let url = VmLifecycle::new(pool.clone()).deploy_node_files(
            &vm.container_name,
            &files,
            host_port,
            None, // Windows WSL path: no project database yet (#1386)
        )?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| format!("local deploy probe client: {e}"))?;
        let health_url = format!("{}/health", url.trim_end_matches('/'));
        let mut healthy = false;
        for _ in 0..20 {
            if client
                .get(&health_url)
                .send()
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false)
            {
                healthy = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        if !healthy {
            return Err(format!(
                "local WSL deployment did not become healthy at {health_url}"
            ));
        }
        serde_json::json!({
            "success": true,
            "url": url,
            "repository": "local-wsl",
            "project_type": deployment_type,
            "deploy_target": "incus-container",
            "status": "Deployed"
        })
    };

    #[cfg(not(target_os = "windows"))]
    let deployed: Value = {
        let body = serde_json::json!({
            "app_name": repo_name,
            "organization": org,
            "project_type": deployment_type,
            "environment": env,
            "target": target,
            "project_id": project_id,
            "on_behalf_of_user": on_behalf_of_user,
            "files": files,
            "database_url": deploy_db_url,
        });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let mut req = client
            .post(format!("{}/api/deployment/deploy", api_base()))
            .header("Content-Type", "application/json");
        // The deployment API is an internal endpoint guarded by the
        // INTERNAL_API_TOKEN (X-Internal-Token), same as the other internal
        // callers (invoke_action, vibe_agent keywords).
        let internal_token = botcoresecrets::internal_api_token();
        if !internal_token.is_empty() {
            req = req.header("X-Internal-Token", internal_token);
        }
        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("deployment API unreachable: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("deployment API returned {status}: {text}"));
        }
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }))
    };

    // #1271 — prod must keep the framework running always-on, not just raise
    // the container. Deploying to the prod VM reuses the same app-start as
    // dev Run (run_dev_app): workspace pushed into /opt/vibe/app and the node
    // (or python) service started with Restart=always, so the published URL
    // serves a live process instead of an empty VM. Only meaningful for
    // production (dev Run already goes through run_dev_app directly).
    #[cfg(not(target_os = "windows"))]
    if env == "production" {
        // 0 = no host proxy device (see run_dev_app): prod apps are served
        // through the Caddy domain route -> container IP:3000, and the old
        // default of 80 collided with the container's listeners.
        let host_port = std::env::var("VIBE_PROD_APP_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(0);
        // Start the app always-on. `run_dev_app` may return Err solely
        // because the host port-80 proxy device cannot bind (Caddy owns :80
        // on the host) — irrelevant for prod, which is served through the
        // domain route; the app service inside the container
        // (`vibe-app.service`, Restart=always) is what matters.
        // #1386 — production VMs get the project's PUBLIC database; the dev
        // twin keeps the `_dev` one (see run_project_app). Reuse the URL
        // already resolved for the deployment body (same env → same DB).
        match VmLifecycle::new(pool.clone()).run_dev_app(
            &vm.container_name,
            &files,
            host_port,
            deploy_db_url.as_deref(),
        ) {
            Ok(_) => log::info!(
                "Vibe publish {}: started app always-on in prod container {}",
                project.name,
                vm.container_name
            ),
            Err(e) => log::warn!(
                "Vibe publish {}: could not start prod app in {}: {e}",
                project.name,
                vm.container_name
            ),
        }
        // #1261 — the published app is reachable at `{repo}.{domain}` (the
        // same wildcard zone as bots). Register a Caddy reverse-proxy route
        // to the prod container unconditionally: the app listens on :3000
        // inside the container (`run_dev_app` pins PORT=3000) and the proxy
        // container cannot resolve `{container}.incus` names, so dial the
        // container's real IP. Route failure is logged but non-fatal.
        // (#1271 — this same host is the public URL recorded below.)
        let host = format!("{repo_name}.{}", published_domain());
        let dial = match VmLifecycle::new(pool.clone()).linux_ip(&vm.container_name) {
            Ok(Some(ip)) => format!("{ip}:3000"),
            Ok(None) => {
                log::warn!(
                    "Vibe publish {}: no IPv4 found for {} — published URL {} will not route",
                    project.name,
                    vm.container_name,
                    host
                );
                String::new()
            }
            Err(e) => {
                log::warn!(
                    "Vibe publish {}: could not resolve IP for {}: {e}",
                    project.name,
                    vm.container_name
                );
                String::new()
            }
        };
        if !dial.is_empty() {
            match crate::caddy::upsert_route_to(&host, &dial, "public").await {
                Ok(route) => log::info!(
                    "Vibe publish {}: Caddy route {} -> {dial}",
                    project.name,
                    route.route_id
                ),
                Err(e) => log::warn!(
                    "Vibe publish {}: Caddy route for {host} failed: {e}",
                    project.name
                ),
            }
        }
    }

    // #1271 — the deployment history row must carry a browser-openable URL:
    // the platform host route (https://{repo}.{platform-domain}) when the
    // Caddy route was installed, else whatever the inner deploy reported.
    // `ops_api::preview` reads this row to give the UI a real public URL;
    // an empty url forces the frontend to guess slugs from the hostname.
    let public_host = format!("{repo_name}.{}", published_domain());
    let public_url = if env == "production" {
        Some(format!("https://{public_host}/"))
    } else {
        None
    };
    let deployment = serde_json::json!({
        "env": env,
        "at": chrono::Utc::now().to_rfc3339(),
        "url": public_url
            .clone()
            .or_else(|| deployed.get("url").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_default(),
        "container": vm.container_name,
        "domain": domain.clone().unwrap_or_default(),
        "track": "ok",
    });
    registry
        .append_deployment(project_id, &deployment)
        .map_err(|e| format!("record deployment: {e}"))?;

    let launch_info = if launcher_requested || widget_requested {
        let launch = serde_json::json!({
            "enabled": true,
            "kind": if widget_requested { "widget" } else { "app" },
            "at": chrono::Utc::now().to_rfc3339(),
            "env": env,
        });
        registry
            .set_launcher(project_id, &launch)
            .map_err(|e| format!("record launcher flag: {e}"))?;
        Some(launch)
    } else {
        None
    };

    let binding = match &domain {
        Some(d) => {
            let bind_req = BindDomainRequest {
                domain: d.clone(),
                env: env.clone(),
                access: None,
                allowed_emails: None,
            };
            match ProjectDomains::new(pool).bind(project_id, &bind_req).await {
                Ok(b) => {
                    serde_json::json!({ "bound": true, "id": b.id, "domain": b.domain, "env": b.env, "container": b.container, "verified": b.verified, "tls_status": b.tls_status })
                }
                Err(e) => serde_json::json!({ "bound": false, "error": e }),
            }
        }
        None => serde_json::json!({ "bound": false, "error": "no domain provided" }),
    };

    Ok(serde_json::json!({
        "published": true,
        "project": project.name,
        "env": env,
        "container": vm.container_name,
        "domain": domain,
        "domain_bind": binding,
        "deployment": deployed,
        "launcher": launch_info,
        "history_key": "deployments"
    }))
}
