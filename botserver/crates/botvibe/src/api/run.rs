//! `api::run` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Upper bound on a single run's intent text (#925). Rejects oversized
/// requests with 413 instead of feeding unbounded text into prompts/DB JSONB.
pub(crate) const MAX_INTENT_CHARS: usize = 4000;

/// Upper bound on a single run's wall-clock timeout (#925).
pub(crate) const MAX_TIMEOUT_SECONDS: u64 = 3600;

/// Resolves a nil bot id to the caller's own org bot (when the user belongs to
/// an organization) or the default bot otherwise, so vibe runs resolve the
/// correct LLM config (Vault + config.csv) instead of always hitting `default`.
pub(crate) fn resolve_effective_bot_id(pool: &crate::types::DbPool, user: &AuthenticatedUser) -> Uuid {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Uuid::nil(),
    };
    #[derive(diesel::QueryableByName)]
    struct BotIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    // A member of an org runs against an active bot of that org.
    if let Some(org_id) = user.organization_id {
        if let Ok(Some(row)) = diesel::sql_query(
            "SELECT id FROM bots WHERE org_id = $1 AND is_active = true LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .get_result::<BotIdRow>(&mut conn)
        .optional()
        {
            return row.id;
        }
    }
    diesel::sql_query("SELECT id FROM bots WHERE name = 'default' AND is_active = true LIMIT 1")
        .get_result::<BotIdRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.id)
        .unwrap_or(Uuid::nil())
}

/// #918 — a caller may run against a bot they hold an explicit grant for, the
/// bot their session authenticated against, or an active bot of their own
/// organization. Dev/SSO users without Zitadel `bot:` grants must still be able
/// to run vibe on the domain bot they logged in through.
pub(crate) fn bot_accessible_to_user(pool: &crate::types::DbPool, user: &AuthenticatedUser, bid: &Uuid) -> bool {
    if user.bot_access.contains_key(bid) {
        return true;
    }
    if user.current_bot_id.as_ref() == Some(bid) {
        return true;
    }
    let Some(org_id) = user.organization_id else {
        return false;
    };
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return false,
    };
    #[derive(diesel::QueryableByName)]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    diesel::sql_query("SELECT org_id FROM bots WHERE id = $1 AND is_active = true")
        .bind::<diesel::sql_types::Uuid, _>(bid)
        .get_result::<OrgRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.org_id == org_id)
        .unwrap_or(false)
}

/// Resolves the `(project_id, project_name)` a run operates on. An explicit
/// project in the request is honored as-is; otherwise a stable name is derived
/// from the intent and a project auto-created in the registry (scoped to the
/// caller's organization) so the run's output shows up in the sidebar project
/// list instead of landing in an untracked workspace directory.
pub(crate) fn resolve_project(
    registry: &ProjectRegistryRef,
    rbac: &crate::rbac::ProjectRbac,
    user: &AuthenticatedUser,
    req: &CreateRunRequest,
) -> (Option<String>, Option<String>) {
    match (req.project_id.as_deref(), req.project_name.as_deref()) {
        (Some(pid), name) => {
            // The UUID is authoritative. Resolve the canonical registry name
            // instead of trusting a stale display label from the browser; the
            // name is the workspace key used by all file/shell tools.
            let canonical = Uuid::parse_str(pid)
                .ok()
                .and_then(|id| registry.get(id).ok().flatten())
                .map(|project| crate::vm_lifecycle::VmLifecycle::alm_repo(&project.name));
            (Some(pid.to_string()), canonical.or_else(|| name.map(String::from)))
        }
        (None, Some(name)) => (None, Some(crate::vm_lifecycle::VmLifecycle::alm_repo(name))),
        (None, None) => {
            // "Deploy/Run the selected|current project" without a picker
            // selection is not a NEW project: minting one from the intent slug
            // produced junk workspaces like `deploy-the-selected`. Resolve the
            // caller's most recent real project instead — the UI always lists
            // newest-first, so this is exactly what "the selected project"
            // refers to when the combo state was lost (reload, restore).
            let intent_lower = req.intent.to_ascii_lowercase();
            if req.pipeline_mode.as_deref() == Some("deploy")
                || intent_lower.contains("the selected project")
                || intent_lower.contains("the current project")
            {
                let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
                if let Ok(Some(existing)) = registry.list(&crate::projects::ListProjectsQuery {
                    branch_id: Some(org_id),
                    limit: Some(1),
                    project_type: None,
                    status: None,
                    offset: None,
                })
                .map(|mut v| v.drain(..).next())
                {
                    let key = crate::vm_lifecycle::VmLifecycle::alm_repo(&existing.name);
                    return (Some(existing.id.to_string()), Some(key));
                }
            }
            let name = derive_project_name(&req.intent);
            let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
            let create = CreateProjectRequest {
                name: name.clone(),
                project_type: Some("apps".to_string()),
                repository: Some(name.clone()),
                framework: None,
                custom_domain: None,
                environment: None,
                source_control: Some(req.source_control.clone().unwrap_or_else(|| "native".to_string())),
                clone_url: req.clone_url.clone(),
                // #1312 — the run intent is the LLM scaffold prompt for
                // chat-created projects.
                description: Some(req.intent.clone()),
                org_id: Some(org_id),
                branch_id: None,
            };
            match registry.create(&create) {
                Ok(p) => {
                    // #1271 — an auto-created project must ship with starter
                    // content so a Run against it never opens an empty
                    // "No web app yet" VM. #1312 — that starter is LLM-
                    // generated from the run intent ("create a calculator
                    // app" → the LLM writes calc.js, not a hardcoded
                    // template). This resolver is synchronous, so the
                    // scaffold runs on a spawned thread (same bridge as
                    // daily_briefing); `run_project_app` re-seeds an empty
                    // workspace on Run, so the guarantee holds even if the
                    // scaffold is slow. (github-mode clones the caller's
                    // repo instead — wired async in create_run below.)
                    let key = crate::vm_lifecycle::VmLifecycle::alm_repo(&p.name);
                    let scaffold_key = key.clone();
                    let scaffold_name = p.name.clone();
                    let scaffold_framework = p.framework.clone();
                    let scaffold_intent = req.intent.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build();
                        if let Ok(rt) = rt {
                            let _ = rt.block_on(
                                crate::scaffold::scaffold_project_workspace(
                                    &scaffold_key,
                                    &scaffold_name,
                                    "apps",
                                    scaffold_framework.as_deref(),
                                    Some(&scaffold_intent),
                                ),
                            );
                        }
                    });
                    // #1271 — an auto-created project must grant the caller
                    // ownership exactly like explicit creation (which calls
                    // `rbac.set_user_role`), otherwise the workspace-files API
                    // returns "role viewer forbidden" and the run owner cannot
                    // list/edit the project's files in the UI.
                    if let Err(e) =
                        rbac.set_user_role(p.id, user.user_id, crate::rbac::ProjectRole::Owner)
                    {
                        error!(
                            "Vibe: grant owner on auto-created project '{name}' failed: {e}"
                        );
                    }
                    (Some(p.id.to_string()), Some(p.name))
                }
                Err(e) => {
                    error!("Vibe: auto-create project '{name}' failed: {e}");
                    (None, Some(name))
                }
            }
        }
    }
}

/// Derives a short, stable project name (workspace slug) from a run intent.
/// Leading verbs/articles are stripped so "Create a calculator web app"
/// becomes "calculator-web-app" rather than a noisy stop-word slug.
pub(crate) fn derive_project_name(intent: &str) -> String {
    const STOP: &[&str] = &[
        "create", "make", "build", "write", "generate", "develop", "add", "a", "an", "the",
        "my", "new", "simple", "basic", "me", "please",
    ];
    // #1272 — generic deictic phrases must never become project names: a run
    // like "Deploy the selected project to production" minted the junk
    // workspace `deploy-the-selected`. Such intents refer to an EXISTING
    // project; the caller must pass project_id/project_name instead.
    const FORBIDDEN: &[&str] = &["selected", "current", "this", "that", "project", "it"];
    // Articles add nothing to a slug and read badly mid-name ("deploy-the-to").
    const ARTICLES: &[&str] = &["the", "a", "an"];
    let mut words: Vec<String> = Vec::new();
    for raw in intent.split(|c: char| !c.is_ascii_alphanumeric()) {
        let word = raw.to_ascii_lowercase();
        if word.is_empty() {
            continue;
        }
        if words.is_empty() && STOP.contains(&word.as_str()) {
            continue;
        }
        if FORBIDDEN.contains(&word.as_str()) || ARTICLES.contains(&word.as_str()) {
            continue;
        }
        words.push(word);
        if words.len() >= 3 {
            break;
        }
    }
    if words.is_empty() {
        "app".to_string()
    } else {
        words.join("-")
    }
}

#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    pub success: bool,
    pub pipeline: crate::pipeline::RunPipeline,
    pub error: Option<String>,
}

impl crate::knowledge_graph::GraphDataSource for VibeApiInner {
    fn snapshot_runs(
        &self,
    ) -> crate::knowledge_graph::GraphFuture<Vec<crate::knowledge_graph::RunNodeInfo>> {
        let runs = Arc::clone(&self.runs);
        let runs_store = self.runs_store.clone();
        Box::pin(async move {
            let mut all: Vec<crate::knowledge_graph::RunNodeInfo> = Vec::new();
            // #1446 — dedup by run_id: a run that was flushed to Postgres and
            // is still in memory must appear once, not twice.
            let mut seen: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
            // Persisted runs (survive restarts, issue #799).
            for r in runs_store.list_runs(500) {
                let id = r.run_id;
                if !seen.insert(id) {
                    continue;
                }
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                    project_id: r.config.project_id.clone(),
                    created_at: r.created_at,
                });
            }
            // In-memory runs not yet flushed to Postgres.
            for r in runs.read().await.values() {
                if !seen.insert(r.run_id) {
                    continue;
                }
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: r.run_id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                    project_id: r.config.project_id.clone(),
                    created_at: r.created_at,
                });
            }
            all
        })
    }
}

impl VibeApiInner {
    pub(crate) async fn grounding_for(&self, run_id: Uuid) -> Vec<crate::grounding::GroundingSource> {
        let live_run = {
            let runs = self.runs.read().await;
            runs.get(&run_id).cloned()
        };
        let run = live_run.or_else(|| self.runs_store.get_run(run_id));
        let events = self.telemetry.get_events_for_run(run_id, 100).await;
        crate::grounding::build_grounding(run.as_ref(), &events)
    }
}

pub(crate) async fn list_runs(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Query(query): Query<ListRunsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let (live, persisted) = {
        let runs = api.runs.read().await;
        (runs.clone(), api.runs_store.list_runs((limit + offset) as i64))
    };

    let mut merged: Vec<VibeRun> = persisted;
    for run in live.into_values() {
        if let Some(existing) = merged.iter_mut().find(|r| r.run_id == run.run_id) {
            *existing = run;
        } else {
            merged.push(run);
        }
    }
    merged.sort_by_key(|r| std::cmp::Reverse(r.created_at));

    let filtered: Vec<GetRunResponse> = merged
        .iter()
        .skip(offset)
        .take(limit)
        .filter(|r| {
            query
                .state
                .as_ref()
                .is_none_or(|f| r.state.to_string() == *f)
        })
        .filter(|r| {
            query
                .use_case
                .as_ref()
                .is_none_or(|f| r.use_case.to_string() == *f)
        })
        .map(run_to_response)
        .collect();

    Json(filtered)
}

pub(crate) async fn get_pipeline(
    Path(use_case): Path<String>,
    Query(query): Query<PipelineQuery>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let pipeline = if query.mode.as_deref() == Some("deploy") {
        crate::pipeline::RunPipeline::deploy_pipeline(uc)
    } else {
        crate::pipeline::RunPipeline::for_use_case(uc)
    };
    Json(PipelineResponse {
        success: true,
        pipeline,
        error: None,
    })
}

pub(crate) async fn get_run_events(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let events = api.telemetry.get_events_for_run(run_id, 100).await;
    Json(events)
}

/// Cuts `text` on a character boundary (not a byte boundary) so logging a
/// multi-byte UTF-8 intent can never panic (#925).
pub(crate) fn truncate_chars(text: &str, limit: usize) -> &str {
    if text.len() <= limit {
        return text;
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[derive(Deserialize, Default)]
pub(crate) struct UnpublishSiteRequest {
    /// Drop the retained `.prev-*` releases and the retired payload too.
    #[serde(default)]
    pub(crate) purge: bool,
    /// #1290 — `test` (or the legacy `development`) targets the -test site.
    #[serde(default)]
    pub(crate) env: Option<String>,
}
