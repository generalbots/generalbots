//! `api::metrics_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) async fn get_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let state_runs = api.state.active_runs().read().await;
    if let Some(run) = state_runs.get(&run_id) {
        return Json(run_to_response(run));
    }
    drop(state_runs);
    let runs = api.runs.read().await;
    if let Some(run) = runs.get(&run_id) {
        return Json(run_to_response(run));
    }
    drop(runs);
    // Fall back to the persisted store (Issue #793): runs survive restarts.
    match api.runs_store.get_run(run_id) {
        Some(run) => Json(run_to_response(&run)),
        None => Json(GetRunResponse {
            run_id,
            bot_id: Uuid::nil(),
            session_id: Uuid::nil(),
            state: "not_found".to_string(),
            use_case: String::new(),
            intent: String::new(),
            tool_call_count: 0,
            last_tool_name: None,
            created_at: String::new(),
            completed_at: None,
            error: Some("Run not found".to_string()),
            budget_cents: 0,
            lang: String::new(),
            model: None,
            max_tool_calls: 0,
            auto_approve: false,
            project_id: None,
            project_name: None,
            pipeline_mode: None,
        }),
    }
}

/// Cancels a run without regressing a terminal state. The run must already
/// be removed from the in-memory map's borrow scope before this is called.
pub(crate) fn cancel_run_inner(run: &mut VibeRun) {
    // A late cancel must not regress an already-finished run back into
    // Cancelled (same terminal-clobber class as approve_run below).
    if !run.state.is_terminal() {
        run.transition(VibeRunState::Cancelled);
    }
}

pub(crate) async fn cancel_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
    Json(_req): Json<CancelRunRequest>,
) -> impl IntoResponse {
    if let Some(tx) = api.state.run_signal_sender() {
        let _ = tx.send(crate::types::VibeRunSignal::Cancelled(run_id));
    }
    let mut runs = api.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        cancel_run_inner(run);
        let snapshot = run.clone();
        drop(runs);
        {
            let mut state_runs = api.state.active_runs().write().await;
            if let Some(state_run) = state_runs.get_mut(&run_id) {
                cancel_run_inner(state_run);
            }
        }
        if let Err(e) = api.runs_store.save_run(&snapshot) {
            error!("Vibe: persist cancelled run {run_id} failed: {e}");
        }
        info!("Vibe run cancelled: {run_id}");
        Json(ActionResponse {
            success: true,
            message: Some("Run cancelled".to_string()),
            error: None,
        })
    } else {
        drop(runs);
        // After a restart the in-memory map is empty; fall back to the
        // persisted store so a stored run can still be resolved.
        match api.runs_store.get_run(run_id) {
            Some(mut run) => {
                cancel_run_inner(&mut run);
                if let Err(e) = api.runs_store.save_run(&run) {
                    error!("Vibe: persist cancelled run {run_id} failed: {e}");
                }
                info!("Vibe run cancelled (persisted): {run_id}");
                Json(ActionResponse {
                    success: true,
                    message: Some("Run cancelled".to_string()),
                    error: None,
                })
            }
            None => Json(ActionResponse {
                success: false,
                message: None,
                error: Some("Run not found".to_string()),
            }),
        }
    }
}

pub(crate) async fn get_global_metrics(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let mut metrics = api.telemetry.get_global_metrics().await;

    let mut runs: Vec<VibeRun> = api.runs_store.list_runs(1000);
    for run in api.runs.read().await.values() {
        match runs.iter_mut().find(|r| r.run_id == run.run_id) {
            Some(existing) => *existing = run.clone(),
            None => runs.push(run.clone()),
        }
    }
    runs.sort_by_key(|r| r.created_at);

    metrics.total_runs = 0;
    metrics.completed_runs = 0;
    metrics.failed_runs = 0;
    for m in metrics.by_use_case.values_mut() {
        m.total_runs = 0;
        m.completed_runs = 0;
        m.failed_runs = 0;
    }
    for run in runs.iter().rev() {
        metrics.total_runs += 1;
        let m = metrics.by_use_case.entry(run.use_case).or_default();
        m.total_runs += 1;
        match run.state {
            VibeRunState::Completed => {
                metrics.completed_runs += 1;
                m.completed_runs += 1;
            }
            VibeRunState::Failed | VibeRunState::Cancelled => {
                metrics.failed_runs += 1;
                m.failed_runs += 1;
            }
            _ => {}
        }
    }

    Json(MetricsResponse {
        success: true,
        metrics: Some(serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null)),
        error: None,
    })
}

pub(crate) async fn get_run_metrics(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    match api.telemetry.get_run_metrics(run_id).await {
        Some(metrics) => Json(MetricsResponse {
            success: true,
            metrics: Some(serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null)),
            error: None,
        }),
        None => Json(MetricsResponse {
            success: false,
            metrics: None,
            error: Some("No metrics found for run".to_string()),
        }),
    }
}

// ============================================================================
// #1288 — proxy site lifecycle (unpublish / rollback)
// ============================================================================
/// Shared guard: the caller must hold at least project Admin (or be a global
/// admin), and the project must exist. Returns the project on success.
pub(crate) fn guard_site_admin(
    api: &Arc<VibeApiInner>,
    user: &AuthenticatedUser,
    project_id: Uuid,
) -> Result<crate::projects::Project, (axum::http::StatusCode, String)> {
    if user.user_id.is_nil() {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "authentication required".to_string(),
        ));
    }
    let project = api
        .project_registry
        .get(project_id)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "project not found".to_string(),
            )
        })?;
    if !user.is_admin() {
        if let Err(e) = api.project_rbac.require_role(
            user.user_id,
            project_id,
            crate::rbac::ProjectRole::Admin,
        ) {
            return Err((axum::http::StatusCode::FORBIDDEN, e));
        }
    }
    Ok(project)
}

pub(crate) fn site_error_response(e: String) -> Response {
    // Site-operation errors carry an actionable message (refusals, missing
    // releases, proxy issues); they are operator-facing, not secrets.
    (
        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "success": false, "error": e })),
    )
        .into_response()
}

/// DELETE /api/vibe/projects/:project_id/site — take a published site off
/// the proxy: route removed immediately, python service stopped, payload
/// retired (or purged with `?purge=true` / body `{"purge": true}`).
pub(crate) async fn unpublish_project_site(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(env_q): Query<SiteEnvQuery>,
    body: Option<Json<UnpublishSiteRequest>>,
) -> Response {
    info!("Vibe site unpublish requested: project {project_id}");
    let project = match guard_site_admin(&api, &user, project_id) {
        Ok(p) => p,
        Err((status, msg)) => return (status, msg).into_response(),
    };
    let (purge, body_env) = match body {
        Some(Json(b)) => (b.purge, b.env),
        None => (false, None),
    };
    let slug = crate::proxy_sites::site_slug(&project.name);
    // #1290 — `?env=test` (or the legacy `development`) targets the test
    // twin; production stays the default.
    let env = parse_site_env_param(&env_q.env, body_env.as_deref());
    let result = match env {
        Some(crate::site_env::SiteEnv::Test) => {
            crate::proxy_sites::unpublish_site_test(&slug, purge).await
        }
        _ => crate::proxy_sites::unpublish_site(&slug, purge).await,
    };
    match result {
        Ok(()) => Json(serde_json::json!({
            "success": true,
            "message": format!("site '{slug}' unpublished (purge={purge})"),
            "site": slug,
        }))
        .into_response(),
        Err(e) => {
            error!("Vibe site unpublish failed for {slug}: {e}");
            site_error_response(e)
        }
    }
}

/// POST /api/vibe/projects/:project_id/site/rollback — reactivate the
/// previous retained release of the project's site on the proxy.
pub(crate) async fn rollback_project_site(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(env_q): Query<SiteEnvQuery>,
) -> Response {
    info!("Vibe site rollback requested: project {project_id}");
    let project = match guard_site_admin(&api, &user, project_id) {
        Ok(p) => p,
        Err((status, msg)) => return (status, msg).into_response(),
    };
    let slug = crate::proxy_sites::site_slug(&project.name);
    // #1290 — `?env=development` targets the DEV site's release ring.
    let env = parse_site_env_param(&env_q.env, None);
    let result = match env {
        Some(crate::site_env::SiteEnv::Test) => crate::proxy_sites::rollback_site_test(&slug).await,
        _ => crate::proxy_sites::rollback_site(&slug).await,
    };
    match result {
        Ok(url) => Json(serde_json::json!({
            "success": true,
            "message": format!("site '{slug}' rolled back to the previous release"),
            "site": slug,
            "env": env.map(|e| e.as_str()).unwrap_or("production"),
            "url": url,
        }))
        .into_response(),
        Err(e) => {
            error!("Vibe site rollback failed for {slug}: {e}");
            site_error_response(e)
        }
    }
}

/// #1290 — POST /api/vibe/projects/:project_id/site/promote — copy the
/// current TEST release of the site to the PRODUCTION target (route + service
/// refreshed exactly like a direct production deploy). This is the sanctioned
/// way to move a change from `{slug}-test.{domain}` to `{slug}.{domain}`
/// without editing the public payload in place.
pub(crate) async fn promote_project_site(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Response {
    info!("Vibe site promote requested: project {project_id}");
    let project = match guard_site_admin(&api, &user, project_id) {
        Ok(p) => p,
        Err((status, msg)) => return (status, msg).into_response(),
    };
    let is_python = crate::proxy_sites::looks_like_python(
        &crate::publish::collect_workspace_files(&project).unwrap_or_default(),
    );
    match crate::proxy_sites::promote_site_test_to_prod(
        &project,
        is_python,
        api.project_registry.pool(),
    )
    .await
    {
        Ok(url) => Json(serde_json::json!({
            "success": true,
            "message": "test release promoted to production",
            "site": crate::proxy_sites::site_slug(&project.name),
            "url": url,
        }))
        .into_response(),
        Err(e) => {
            error!("Vibe site promote failed for {}: {e}", project.name);
            site_error_response(e)
        }
    }
}
