//! `api::metrics` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub success: bool,
    pub run_id: Uuid,
    pub state: String,
    pub use_case: String,
    pub system_prompt: String,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub state: Option<String>,
    pub use_case: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub success: bool,
    pub metrics: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub(crate) async fn create_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateRunRequest>,
) -> Response {
    info!("Vibe create run: {}", truncate_chars(&req.intent, 80));

    // #925 — validate intent bounds up front so oversized/empty input returns
    // a structured error instead of reaching prompts, DB JSONB, or the agent.
    if req.intent.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "intent must not be empty" })),
        )
            .into_response();
    }
    if req.intent.chars().count() > MAX_INTENT_CHARS {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({ "success": false, "error": format!("intent exceeds {MAX_INTENT_CHARS} characters") })),
        )
            .into_response();
    }

    let use_case = req
        .use_case
        .as_deref()
        .and_then(parse_use_case)
        .unwrap_or(VibeUseCase::SoftwareDevelopment);

    // An explicit project wins; otherwise derive one from the intent and
    // auto-create it in the registry so the run's output lands in a tracked
    // project (visible in the sidebar) instead of an orphan workspace dir.
    let (project_id, project_name) =
        resolve_project(&api.project_registry, &api.project_rbac, &user, &req);

    // #1271 — git-mode projects auto-created by this run get their Forgejo
    // repo + origin remote wired (mirrors explicit creation); github-mode
    // projects clone the caller's external repository into the workspace
    // instead of the built-in seed. Non-fatal: native-mode projects and
    // ALM-unavailable cases keep working.
    if let Some(pid) = project_id.as_deref().and_then(|s| Uuid::parse_str(s).ok()) {
        if let Ok(Some(p)) = api.project_registry.get(pid) {
            match p.source_control.as_str() {
                "git" => {
                    if let Err(e) = crate::git_mode::ensure_git_repo(&p).await {
                        error!("Vibe: git-mode wiring for project {pid} failed: {e}");
                    }
                }
                "github" => {
                    if let Err(e) = crate::git_mode::ensure_github_clone(&p).await {
                        error!("Vibe: github-mode wiring for project {pid} failed: {e}");
                    }
                }
                _ => {}
            }
        }
    }

    let config = VibeRunConfig {
        use_case,
        lang: req.lang.unwrap_or_else(|| "en".to_string()),
        // #919/#1271 — auto-approval like freebuff: runs execute tools
        // without manual approval gates. The client may opt out explicitly;
        // otherwise every run (Run and Deploy) proceeds automatically.
        auto_approve: req.auto_approve.unwrap_or(true),
        max_tool_calls: req.max_tool_calls.unwrap_or(50).min(MAX_TOOL_CALLS),
        timeout_seconds: req.timeout_seconds.unwrap_or(600).min(MAX_TIMEOUT_SECONDS),
        model: req.model,
        llm_key: None,
        llm_url: None,
        agent: req.agent,
        budget_cents: req.budget_cents.unwrap_or(0),
        project_id,
        project_name: project_name.clone(),
        pipeline_mode: req.pipeline_mode.clone(),
    };

    let intent = match (project_name.as_deref(), req.intent.as_str()) {
        (Some(name), raw) if !name.is_empty() && !raw.to_lowercase().contains(name.to_lowercase().as_str()) => {
            format!("In project {name}: {raw}")
        }
        _ => req.intent,
    };
    // #918 — a caller may only run against a bot they have access to; only an
    // administrator may target an arbitrary bot. The vibe UI always sends the
    // site's own bot UUID, which for session users (whose auth cache may lack
    // org/role grants) would otherwise 403 legitimate runs on the very bot
    // they logged in through. When the explicit bot is not granted, fall back
    // to the session's effective bot instead of rejecting — the run still
    // executes, scoped to the caller's own domain bot.
    let bot_id = match req.bot_id {
        Some(bid) if !bid.is_nil() && !user.is_admin() && !bot_accessible_to_user(api.state.db_pool(), &user, &bid) => {
            let fallback = resolve_effective_bot_id(api.state.db_pool(), &user);
            log::info!(
                "Vibe create run: bot {bid} not granted to user {}, falling back to effective bot {fallback}",
                user.user_id
            );
            fallback
        }
        Some(bid) => bid,
        None => resolve_effective_bot_id(api.state.db_pool(), &user),
    };
    // #1280 — record the acting user on the run: the publish tool and the
    // deploy pipeline forward it to the deployment API's RBAC gate. A nil
    // user would make every REST-issued deploy fail as "anonymous".
    let run = VibeRun::new(bot_id, Uuid::nil(), user.user_id, intent, config);
    let run_id = run.run_id;
    let state_str = run.state.to_string();
    let uc_str = run.use_case.to_string();

    // #1286 — a MODifying turn on a project must not interleave writes with
    // an in-flight run on the same project. The run is created immediately
    // (state pending, fully observable) and the spawned executor waits on
    // the project's exclusive edit lock BEFORE running tools — a second
    // chat tab bound to the same project queues FIFO instead of racing.
    // Read-only queries and other projects are untouched: parallelism
    // across tabs/projects is the feature's core value.
    let needs_edit_lock =
        is_modifying_intent(&run.intent) && run.config.project_id.is_some();
    if needs_edit_lock {
        if let Some(pid) = run.config.project_id.as_deref() {
            if !api.project_locks.is_free(pid).await {
                info!(
                    "Vibe run {run_id} queued: another session is modifying project {pid}"
                );
            }
        }
    }

    // #921 — persist the run row *before* the first telemetry event so the
    // `vibe_telemetry.run_id` FK is satisfied and the run stays durable even
    // if the process dies mid-execution. save_run upserts, so the later
    // completion snapshot still wins.
    if let Err(e) = api.runs_store.save_run(&run) {
        error!("Vibe: persist run {run_id} failed: {e}");
    }

    let ctx = api.prompt_manager.build_context(
        run.use_case,
        &run.config.lang,
        &run.intent,
        &[],
    );
    let system_prompt = ctx.system_prompt.clone();

    api.telemetry.record_run_start(&run).await;

    let agent_loop = Arc::new(
        AgentLoop::new(
            api.prompt_manager.clone(),
            api.tool_executor.clone(),
            api.telemetry.clone(),
            api.state.clone(),
        )
        .with_security(
            api.permissions.clone(),
            api.skills.clone(),
        ),
    );

    {
        let mut runs = api.runs.write().await;
        runs.insert(run_id, run.clone());
    }
    {
        let mut runs = api.state.active_runs().write().await;
        runs.insert(run_id, run.clone());
    }

    api.state.broadcast_progress(
        VibeProgressEvent::started(run_id.to_string(), "Vibe run created", 3),
    );

    let pipeline_mode = req.pipeline_mode.clone();
    let api_clone = api.clone();
    // Slot for the acquired edit lock; held across the run, released at the
    // end of the spawned task (terminal state) via drop.
    let mut run_guard_slot: Option<crate::project_locks::ProjectLockGuard> = None;
    tokio::spawn(async move {
        // #827 — keep a "running" placeholder in the map so the run stays
        // queryable (GET /api/vibe/run/{id}) while the loop executes, instead
        // of vanishing to `not_found` until it finishes.
        let run_opt = {
            let mut runs = api_clone.runs.write().await;
            let taken = runs.remove(&run_id);
            if let Some(snap) = taken.as_ref() {
                let mut placeholder = snap.clone();
                placeholder.transition(VibeRunState::Running);
                runs.insert(run_id, placeholder);
            }
            taken
        };
        if let Some(mut run) = run_opt {
            // #1286 — modifying runs wait (FIFO, bounded) for the project's
            // exclusive edit slot while another session's run is executing.
            // The run stays observable as pending/running so the user's tab
            // shows the queued turn; timeout fails it explicitly instead of
            // dropping it (the #1275 lesson).
            if needs_edit_lock {
                let pid = run.config.project_id.clone().unwrap_or_default();
                let wait = tokio::time::timeout(
                    crate::project_locks::LOCK_WAIT_TIMEOUT,
                    api_clone.project_locks.acquire(&pid, run_id),
                )
                .await;
                match wait {
                    Ok(Ok(guard)) => {
                        // Hold the slot until end of scope: it releases when
                        // guard drops at the end of this block.
                        run_guard_slot = Some(guard);
                    }
                    Ok(Err(e)) => {
                        error!("Vibe: run {run_id} edit lock for project {pid} failed: {e}");
                        run.transition(VibeRunState::Failed);
                        run.error = Some(format!("project edit lock: {e}"));
                    }
                    Err(_) => {
                        info!("Vibe: run {run_id} timed out waiting for the edit lock of project {pid}");
                        run.transition(VibeRunState::Failed);
                        run.error = Some(
                            "another session is modifying this project — your turn waited too long; try again".to_string(),
                        );
                    }
                }
                if let Err(e) = api_clone.runs_store.save_run(&run) {
                    error!("Vibe: persist run {run_id} failed: {e}");
                }
            }
            let lock_is_failed = run.state == VibeRunState::Failed;
            if pipeline_mode.as_deref() == Some("deploy") && !lock_is_failed {
                // vibe33 #811 — graph execution path: the deploy pipeline
                // runs its stages through the tool executor with approval
                // gates and fail-fast (failed stage skips the rest).
                run.transition(VibeRunState::Running);
                let engine = PipelineEngine::new(api_clone.telemetry.clone());
                let pipeline = RunPipeline::deploy_pipeline(run.use_case);
                let project_id = run.config.project_id.clone();
                let project_name = run.config.project_name.clone();
                let report = engine
                    .run(
                        &pipeline,
                        &api_clone.tool_executor,
                        api_clone.state.as_ref(),
                        &PipelineRunContext {
                            run_id,
                            use_case: run.use_case,
                            intent: &run.intent,
                            project_id: project_id.as_deref(),
                            project_name: project_name.as_deref(),
                            user_id: run.user_id,
                        },
                    )
                    .await;
                // #1268 — a tolerated stage failure (continue_on_failure, e.g.
                // DNS verify or a TLS hiccup) must not fail the whole run:
                // only fail-fast stage failures abort the deploy.
                let blocking_failure = |report_stage: &PipelineStageReport| {
                    report_stage.status == StageStatus::Failed
                        && pipeline
                            .stage(&report_stage.stage_id)
                            .map(|st| !st.continue_on_failure)
                            .unwrap_or(true)
                };
                let failed = report.stages.iter().any(&blocking_failure);
                let skipped = report
                    .stages
                    .iter()
                    .filter(|s| s.status == StageStatus::Skipped)
                    .count();
                if failed {
                    run.transition(VibeRunState::Failed);
                    run.error = Some(
                        report
                            .stages
                            .iter()
                            .find(|s| blocking_failure(s))
                            .and_then(|s| s.error.clone())
                            .unwrap_or_else(|| "pipeline stage failed".to_string()),
                    );
                } else {
                    run.transition(VibeRunState::Completed);
                }
                info!(
                    "Vibe deploy pipeline {run_id}: {} stages, {skipped} skipped, failed={failed}",
                    report.stages.len()
                );
                api_clone
                    .telemetry
                    .record_run_completion(&run, 0, None, 0.0)
                    .await;
            } else if pipeline_mode.as_deref() != Some("deploy") && !lock_is_failed {
                agent_loop.execute_run(&mut run).await;
            } else if lock_is_failed {
                info!("Vibe run {run_id}: skipped execution — edit lock not acquired");
            }
            // Keep the sidebar project status truthful: a completed run marks
            // its project active, a failed/cancelled one marks it failed.
            if let Some(pid) = run.config.project_id.as_deref() {
                if let Ok(pid) = Uuid::parse_str(pid) {
                    let status = match run.state {
                        VibeRunState::Completed => "active",
                        VibeRunState::Failed | VibeRunState::Cancelled => "failed",
                        _ => "pending",
                    };
                    let update = crate::projects::UpdateProjectRequest {
                        name: None,
                        project_type: None,
                        repository: None,
                        framework: None,
                        custom_domain: None,
                        environment: None,
                        source_control: None,
                        status: Some(status.to_string()),
                        payload: None,
                    };
                    if let Err(e) = api_clone.project_registry.update(pid, &update) {
                        error!("Vibe: sync project {pid} status failed: {e}");
                    }
                }
            }
            if let Err(e) = api_clone.runs_store.save_run(&run) {
                error!("Vibe: persist run {run_id} failed: {e}");
            }
            // #1286 — the run reached a terminal state: release the project
            // edit lock so the queued session's turn can start. Dropping the
            // guard wakes the next FIFO waiter.
            drop(run_guard_slot);
            let final_run = run.clone();
            let mut runs = api_clone.runs.write().await;
            runs.insert(run_id, run);
            drop(runs);
            let mut state_runs = api_clone.state.active_runs().write().await;
            state_runs.insert(run_id, final_run);
        }
    });

    Json(CreateRunResponse {
        success: true,
        run_id,
        state: state_str,
        use_case: uc_str,
        system_prompt,
        error: None,
    })
    .into_response()
}
