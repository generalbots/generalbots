use crate::agent_loop::AgentLoop;
use crate::pipeline::{
    PipelineEngine, PipelineRunContext, PipelineStageReport, RunPipeline, StageStatus,
};
use crate::prompt_manager::VibePromptManager;
use crate::projects::{CreateProjectRequest, ProjectRegistryRef};
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::{ToolDescriptor, VibeToolExecutor};
use crate::types::{VibeProgressEvent, VibeRun, VibeRunConfig, VibeRunState, VibeState, VibeUseCase};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use diesel::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

/// Upper bound on a single run's intent text (#925). Rejects oversized
/// requests with 413 instead of feeding unbounded text into prompts/DB JSONB.
const MAX_INTENT_CHARS: usize = 4000;
/// Upper bound on tool-call loops per run (#925).
const MAX_TOOL_CALLS: u32 = 500;
/// Upper bound on a single run's wall-clock timeout (#925).
const MAX_TIMEOUT_SECONDS: u64 = 3600;

/// Heuristic for #1286: is this intent a MODifying turn (tools that write to
/// the project workspace) or a read-only query? Only modifying turns take
/// the project's exclusive edit lock; read-only queries stay parallel.
fn is_modifying_intent(intent: &str) -> bool {
    const MODIFY_HINTS: &[&str] = &[
        "create", "make", "build", "write", "generate", "develop", "add",
        "change", "update", "edit", "refactor", "fix", "remove", "delete",
        "rename", "deploy", "run", "modify", "implement", "replace", "move",
        "crie", "criar", "mude", "mudar", "atualize", "atualizar", "edite",
        "editar", "adicione", "adiccionar", "corrija", "corrigir", "remova",
        "remover", "implemente", "implementar",
    ];
    let lower = intent.to_lowercase();
    // Bounded scan: the intent is already capped at MAX_INTENT_CHARS.
    MODIFY_HINTS.iter().any(|h| {
        lower
            .split(|c: char| !c.is_ascii_alphanumeric())
            .any(|w| w == *h)
    })
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub intent: String,
    /// Bot this run operates on. When absent, falls back to the default bot.
    /// The frontend passes the authenticated session's bot so runs are scoped
    /// to the user's tenant (not the global default).
    pub bot_id: Option<Uuid>,
    pub use_case: Option<String>,
    pub lang: Option<String>,
    pub auto_approve: Option<bool>,
    pub max_tool_calls: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub model: Option<String>,
    pub budget_cents: Option<u64>,
    /// vibe33 #811 — when "deploy", the run executes through the graph
    /// (PipelineEngine, approval-gated deploy pipeline) instead of the
    /// agent loop.
    pub pipeline_mode: Option<String>,
    /// Vibe project this run operates on (uuid string). When set, the
    /// deploy pipeline and the agent's harness tools resolve the project
    /// workspace instead of guessing from the intent text.
    pub project_id: Option<String>,
    /// Project name (workspace key) — the value the agent passes to
    /// file/run/git tools as `project`.
    pub project_name: Option<String>,
    /// Source-control mode for a project auto-created by this run:
    /// `native` (default), `git` (Forgejo-backed) or `github` (clone an
    /// external repository — see `clone_url`).
    pub source_control: Option<String>,
    /// External repository URL for `source_control = "github"` projects
    /// auto-created by this run.
    pub clone_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub success: bool,
    pub run_id: Uuid,
    pub state: String,
    pub use_case: String,
    pub system_prompt: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetRunResponse {
    pub run_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: Uuid,
    pub state: String,
    pub use_case: String,
    pub intent: String,
    pub tool_call_count: usize,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    /// Run budget in cents (from `VibeRunConfig`) so the Run Dock budget
    /// meter survives a page reload / persisted-run re-focus (#930).
    pub budget_cents: u64,
    pub lang: String,
    pub model: Option<String>,
    pub max_tool_calls: u32,
    pub auto_approve: bool,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    /// "deploy" when the run executed the production pipeline; None for
    /// development runs. Lets the UI skip dev-only actions (like opening
    /// the dev browser) after a deploy run (#1271).
    pub pipeline_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ToolDescriptor>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub state: Option<String>,
    pub use_case: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Resolves a nil bot id to the caller's own org bot (when the user belongs to
/// an organization) or the default bot otherwise, so vibe runs resolve the
/// correct LLM config (Vault + config.csv) instead of always hitting `default`.
fn resolve_effective_bot_id(pool: &crate::types::DbPool, user: &AuthenticatedUser) -> Uuid {
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
fn bot_accessible_to_user(pool: &crate::types::DbPool, user: &AuthenticatedUser, bid: &Uuid) -> bool {
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
fn resolve_project(
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
fn derive_project_name(intent: &str) -> String {
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
pub struct ActionResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub success: bool,
    pub metrics: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub success: bool,
    pub capabilities: Vec<crate::capability_registry::Capability>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    pub success: bool,
    pub pipeline: crate::pipeline::RunPipeline,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelRunRequest {
    pub reason: Option<String>,
}

pub(crate) struct VibeApiInner {
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    permissions: crate::permissions::PermissionEngineRef,
    skills: Arc<crate::skills::SkillStore>,
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
    runs_store: crate::run_store::VibeRunStore,
    project_registry: ProjectRegistryRef,
    project_rbac: crate::rbac::ProjectRbac,
    /// #1286 — per-project edit locks: one exclusive write slot per project
    /// so parallel multi-chat sessions queue instead of interleaving writes.
    project_locks: Arc<crate::project_locks::ProjectLockRegistry>,
}

impl crate::knowledge_graph::GraphDataSource for VibeApiInner {
    fn snapshot_runs(
        &self,
    ) -> crate::knowledge_graph::GraphFuture<Vec<crate::knowledge_graph::RunNodeInfo>> {
        let runs = Arc::clone(&self.runs);
        let runs_store = self.runs_store.clone();
        Box::pin(async move {
            let mut all: Vec<crate::knowledge_graph::RunNodeInfo> = Vec::new();
            // Persisted runs (survive restarts, issue #799).
            for r in runs_store.list_runs(500) {
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: r.run_id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                    project_id: r.config.project_id.clone(),
                });
            }
            // In-memory runs not yet flushed to Postgres.
            for r in runs.read().await.values() {
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: r.run_id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                    project_id: r.config.project_id.clone(),
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

/// Bundled security dependencies (permissions + skills) wired into the Vibe
/// API, grouped so the router keeps a readable signature.
pub struct VibeSecurityDeps {
    pub permissions: crate::permissions::PermissionEngineRef,
    pub skills: Arc<crate::skills::SkillStore>,
}

pub fn router(
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    security: VibeSecurityDeps,
    pool: crate::types::DbPool,
    project_registry: ProjectRegistryRef,
    project_rbac: crate::rbac::ProjectRbac,
) -> axum::Router {
    let api = Arc::new(VibeApiInner {
        state,
        prompt_manager,
        tool_executor,
        telemetry,
        permissions: security.permissions,
        skills: security.skills,
        runs: Arc::new(RwLock::new(HashMap::new())),
        runs_store: crate::run_store::VibeRunStore::new(pool),
        project_registry,
        project_rbac,
        project_locks: Arc::new(crate::project_locks::ProjectLockRegistry::new()),
    });
    axum::Router::new()
        .route("/api/vibe/run", axum::routing::post(create_run))
        .route("/api/vibe/run/:run_id", axum::routing::get(get_run))
        .route("/api/vibe/run/:run_id/cancel", axum::routing::post(cancel_run))
        .route("/api/vibe/run/:run_id/approve", axum::routing::post(approve_run))
        .route("/api/vibe/runs", axum::routing::get(list_runs))
        .route("/api/vibe/tools", axum::routing::get(list_tools))
        .route("/api/vibe/tools/:use_case", axum::routing::get(list_tools_for_use_case))
        .route("/api/vibe/metrics", axum::routing::get(get_global_metrics))
        .route("/api/vibe/metrics/:run_id", axum::routing::get(get_run_metrics))
        .route("/api/vibe/events/:run_id", axum::routing::get(get_run_events))
        .route("/api/vibe/run/:run_id/grounding", axum::routing::get(crate::grounding::get_run_grounding))
        .route("/api/vibe/run/:run_id/execute", axum::routing::post(execute_run))
        // #1290 — direct single-tool execution: the UI/REST callers invoke
        // `publish/project` etc. without spinning up a run. RBAC route rules
        // open POST /api/vibe/tools/** to authenticated users; deploy-grade
        // tools enforce roles + metering inside their handlers. The tool
        // name is a QUERY param (`?name=publish/project`) because axum's
        // matchit params never match `/`, and every vibe tool name contains
        // one (e.g. `publish/project`, `git/log`).
        .route("/api/vibe/tools/call", axum::routing::post(execute_tool_direct))
        .route("/api/vibe/graph/:use_case", axum::routing::get(crate::knowledge_graph::get_knowledge_graph))
        .route("/api/vibe/graph/run/:run_id", axum::routing::get(crate::knowledge_graph::get_run_graph))
        .route(
            "/api/vibe/projects/:project_id/conversation",
            axum::routing::get(export_project_conversation),
        )
        .route("/api/vibe/capabilities", axum::routing::get(list_capabilities))
        .route("/api/vibe/capabilities/:use_case", axum::routing::get(list_capabilities_for_use_case))
        .route("/api/vibe/pipeline/:use_case", axum::routing::get(get_pipeline))
        // #1288 — enterprise site lifecycle on the proxy container.
        .route("/api/vibe/projects/:project_id/site", axum::routing::delete(unpublish_project_site))
        .route("/api/vibe/projects/:project_id/site/rollback", axum::routing::post(rollback_project_site))
        // #1290 — promote the current DEV release to PROD; `?env=development`
        // rolls the DEV site back instead.
        .route("/api/vibe/projects/:project_id/site/promote", axum::routing::post(promote_project_site))
        .layer(axum::Extension(api))
}

#[derive(Debug, Deserialize)]
struct DirectToolRequest {
    #[serde(flatten)]
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct DirectToolParams {
    name: String,
}

/// #1290 — POST /api/vibe/tools/call?name=publish/project — execute a single
/// registered vibe tool directly (no run). The payload body is passed to the
/// tool as its arguments. Tools flagged `requires_approval` are refused here:
/// the direct path has no approval surface — callers go through the run flow
/// instead.
async fn execute_tool_direct(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<DirectToolParams>,
    Json(body): Json<DirectToolRequest>,
) -> Response {
    let tool_name = params.name;
    info!("Vibe direct tool call '{tool_name}' by user {}", user.user_id);
    // Honor the global permission mode exactly like the agent loop: in
    // Bypass mode destructive/deploy tools run without the approval gate;
    // otherwise they are refused here (no approval surface on this path).
    let mode = api.permissions.mode().await;
    let needs_approval = api.permissions.requires_approval(false, &tool_name, mode);
    let mut arguments = body.arguments;
    // #1280/#1291 — publish is deploy-role gated downstream; the tool runs
    // server-side without a session, so stamp the CALLING user's id into the
    // arguments exactly like the agent loop does (never silently privileged:
    // the deployment handler enforces the same RBAC for this id).
    if tool_name == "publish/project" {
        if let Some(args) = arguments.as_object_mut() {
            args.insert(
                "on_behalf_of_user".to_string(),
                serde_json::Value::String(user.user_id.to_string()),
            );
        }
    }
    let mut call = crate::types::VibeToolCall::new(
        Uuid::nil(),
        tool_name.clone(),
        arguments,
        needs_approval,
    );
    if matches!(mode, crate::permissions::PermissionMode::Bypass) {
        call.approved = true;
    }
    let result = api
        .tool_executor
        .execute(&mut call, crate::types::VibeUseCase::SoftwareDevelopment, api.state.as_ref())
        .await;
    match result {
        Ok(()) => {
            let payload = call
                .result
                .as_ref()
                .map(|r| r.data.clone())
                .unwrap_or_else(|| serde_json::json!({ "executed": true }));
            Json(serde_json::json!({ "success": true, "tool": tool_name, "result": payload }))
                .into_response()
        }
        Err(e) => {
            let needs_approval = call.requires_approval;
            warn!("Vibe direct tool '{tool_name}' failed: {e}");
            let status = if needs_approval {
                axum::http::StatusCode::ACCEPTED
            } else {
                axum::http::StatusCode::UNPROCESSABLE_ENTITY
            };
            (
                status,
                Json(serde_json::json!({
                    "success": false,
                    "tool": tool_name,
                    "error": e,
                    "requires_approval": needs_approval,
                })),
            )
                .into_response()
        }
    }
}

async fn create_run(
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
                            auto_approve: run.config.auto_approve,
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

async fn get_run(
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

fn run_to_response(run: &VibeRun) -> GetRunResponse {
    GetRunResponse {
        run_id: run.run_id,
        bot_id: run.bot_id,
        session_id: run.session_id,
        state: run.state.to_string(),
        use_case: run.use_case.to_string(),
        intent: run.intent.clone(),
        tool_call_count: run.tool_calls.len(),
        created_at: run.created_at.to_rfc3339(),
        completed_at: run.completed_at.map(|t| t.to_rfc3339()),
        error: run.error.clone(),
        budget_cents: run.config.budget_cents,
        lang: run.config.lang.clone(),
        model: run.config.model.clone(),
        max_tool_calls: run.config.max_tool_calls,
        auto_approve: run.config.auto_approve,
        project_id: run.config.project_id.clone(),
        project_name: run.config.project_name.clone(),
        pipeline_mode: run.config.pipeline_mode.clone(),
    }
}

/// Cancels a run without regressing a terminal state. The run must already
/// be removed from the in-memory map's borrow scope before this is called.
fn cancel_run_inner(run: &mut VibeRun) {
    // A late cancel must not regress an already-finished run back into
    // Cancelled (same terminal-clobber class as approve_run below).
    if !run.state.is_terminal() {
        run.transition(VibeRunState::Cancelled);
    }
}

/// Approves all pending tool calls and, unless the run already finished,
/// resumes it. Returns the user-facing message (the frontend keys off
/// "already finished" to refresh to the terminal state).
fn approve_run_inner(run: &mut VibeRun) -> String {
    for tool_call in &mut run.tool_calls {
        if tool_call.requires_approval && !tool_call.approved {
            tool_call.approved = true;
        }
    }
    let was_terminal = run.state.is_terminal();
    if !was_terminal {
        run.transition(VibeRunState::Running);
    }
    if was_terminal {
        "Run already finished — approval recorded, state unchanged".to_string()
    } else {
        "Pending tool calls approved and run resumed".to_string()
    }
}

async fn cancel_run(
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

async fn approve_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    if let Some(tx) = api.state.run_signal_sender() {
        let _ = tx.send(crate::types::VibeRunSignal::Approved(run_id));
    }
    let mut runs = api.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        let msg = approve_run_inner(run);
        info!("Vibe run approved: {run_id}");
        let snapshot = run.clone();
        drop(runs);
        {
            let mut state_runs = api.state.active_runs().write().await;
            if let Some(state_run) = state_runs.get_mut(&run_id) {
                approve_run_inner(state_run);
            }
        }
        if let Err(e) = api.runs_store.save_run(&snapshot) {
            error!("Vibe: persist approved run {run_id} failed: {e}");
        }
        Json(ActionResponse {
            success: true,
            message: Some(msg),
            error: None,
        })
    } else {
        drop(runs);
        // After a restart the in-memory map is empty; fall back to the
        // persisted store so a stored run can still be approved.
        match api.runs_store.get_run(run_id) {
            Some(mut run) => {
                let msg = approve_run_inner(&mut run);
                if let Err(e) = api.runs_store.save_run(&run) {
                    error!("Vibe: persist approved run {run_id} failed: {e}");
                }
                info!("Vibe run approved (persisted): {run_id}");
                Json(ActionResponse {
                    success: true,
                    message: Some(msg),
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

/// #1190 — conversation export: every run for a project plus its tool-event
/// timeline, as one JSON document (chat/WhatsApp can attach it to a reply).
async fn export_project_conversation(
    Extension(api): Extension<Arc<VibeApiInner>>,
    axum::extract::Path(project_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let want = project_id.to_string();
    let (live, persisted) = {
        let runs = api.runs.read().await;
        (runs.clone(), api.runs_store.list_runs(200))
    };
    let mut merged: Vec<VibeRun> = persisted;
    for run in live.into_values() {
        if let Some(existing) = merged.iter_mut().find(|r| r.run_id == run.run_id) {
            *existing = run;
        } else {
            merged.push(run);
        }
    }
    merged.sort_by_key(|r| r.created_at);

    let mut conversation: Vec<serde_json::Value> = Vec::new();
    for run in merged {
        if run.config.project_id.as_deref() != Some(&want) {
            continue;
        }
        let events = api.telemetry.get_events_for_run(run.run_id, 200).await;
        let run_json = match serde_json::to_value(&run) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Vibe conversation export: serialize run: {e}");
                continue;
            }
        };
        let events_json = match serde_json::to_value(&events) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Vibe conversation export: serialize events: {e}");
                serde_json::Value::Array(Vec::new())
            }
        };
        conversation.push(serde_json::json!({
            "run": run_json,
            "events": events_json,
        }));
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "project_id": project_id,
            "conversation": conversation,
        })),
    )
}

async fn list_runs(
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

async fn list_tools(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let tools = api.tool_executor.registry().list_tools().await;
    Json(ListToolsResponse { tools })
}

async fn list_capabilities(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let tools = api.tool_executor.registry().list_tools().await;
    let capabilities = crate::capability_registry::build_capabilities(&tools);
    Json(CapabilitiesResponse {
        success: true,
        capabilities,
        error: None,
    })
}

async fn list_capabilities_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools().await;
    let capabilities = crate::capability_registry::build_capabilities(&tools);
    let filtered = crate::capability_registry::capabilities_for(&capabilities, uc);
    Json(CapabilitiesResponse {
        success: true,
        capabilities: filtered,
        error: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct PipelineQuery {
    pub mode: Option<String>,
}

async fn get_pipeline(
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

async fn list_tools_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools_for_use_case(uc).await;
    Json(ListToolsResponse { tools })
}

async fn get_global_metrics(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
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

async fn get_run_metrics(
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

async fn get_run_events(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let events = api.telemetry.get_events_for_run(run_id, 100).await;
    Json(events)
}


async fn execute_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.runs.read().await;
    let run = match runs.get(&run_id) {
        Some(r) => r,
        None => {
            return Json(ActionResponse {
                success: false,
                message: None,
                error: Some("Run not found".to_string()),
            });
        }
    };

    let use_case = run.use_case;
    let tool_calls = run.tool_calls.clone();
    drop(runs);
    let state_clone = api.state.clone();

    for tool_call in &tool_calls {
        if !tool_call.approved && tool_call.requires_approval {
            return Json(ActionResponse {
                success: false,
                message: Some("Approval required".to_string()),
                error: None,
            });
        }

        let mut owned = tool_call.clone();
        let result = api
            .tool_executor
            .execute(&mut owned, use_case, state_clone.as_ref())
            .await;

        match result {
            Ok(_) => info!("Tool executed successfully"),
            Err(e) => {
                return Json(ActionResponse {
                    success: false,
                    message: None,
                    error: Some(format!("Execution error: {e}")),
                });
            }
        }
    }

    Json(ActionResponse {
        success: true,
        message: Some("Run executed".to_string()),
        error: None,
    })
}

fn parse_use_case(s: &str) -> Option<VibeUseCase> {
    match s {
        "software_development" => Some(VibeUseCase::SoftwareDevelopment),
        "customer_support" => Some(VibeUseCase::CustomerSupport),
        "financial_analysis" => Some(VibeUseCase::FinancialAnalysis),
        _ => None,
    }
}

/// Cuts `text` on a character boundary (not a byte boundary) so logging a
/// multi-byte UTF-8 intent can never panic (#925).
fn truncate_chars(text: &str, limit: usize) -> &str {
    if text.len() <= limit {
        return text;
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_never_panics_on_multibyte_boundary() {
        // 79 ASCII chars + a 2-byte 'é' at the boundary would panic under
        // byte slicing; the char-safe cut must not.
        let s = format!("{}é", "a".repeat(79));
        let out = truncate_chars(&s, 80);
        assert!(s.starts_with(out));
        assert!(out.len() <= 80);
        assert_eq!(truncate_chars("short", 80), "short");
        assert_eq!(truncate_chars("", 80), "");
    }

    #[test]
    fn truncate_chars_handles_portuguese_and_emoji() {
        // 'coração' has multi-byte 'ç'/'ã'; '🚀' is a 4-byte emoji. Both must
        // survive a cut that lands inside them without panicking.
        let pt = "eu quero agendar um batizado na catedral da sé, coração".repeat(4);
        assert!(truncate_chars(&pt, 80).len() <= 80);
        let emoji = format!("{}🚀", "x".repeat(79));
        assert!(truncate_chars(&emoji, 80).len() <= 80);
    }

    #[test]
    fn derive_project_name_strips_stopwords_and_slugs() {
        assert_eq!(
            derive_project_name("Create a calculator web app with + - * / buttons"),
            "calculator-web-app"
        );
        assert_eq!(
            derive_project_name("Build a new landing page"),
            "landing-page"
        );
        assert_eq!(derive_project_name("refactor the auth module"), "refactor-auth-module");
        assert_eq!(derive_project_name(""), "app");
        assert_eq!(derive_project_name("   "), "app");
        // #1272 — deictic phrases never become project names.
        assert_eq!(
            derive_project_name("Deploy the selected project to production"),
            "deploy-to-production"
        );
        assert_eq!(
            derive_project_name("Update this project settings"),
            "update-settings"
        );
    }
}

// ============================================================================
// #1288 — proxy site lifecycle (unpublish / rollback)
// ============================================================================

/// Shared guard: the caller must hold at least project Admin (or be a global
/// admin), and the project must exist. Returns the project on success.
fn guard_site_admin(
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

fn site_error_response(e: String) -> Response {
    // Site-operation errors carry an actionable message (refusals, missing
    // releases, proxy issues); they are operator-facing, not secrets.
    (
        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "success": false, "error": e })),
    )
        .into_response()
}

#[derive(Deserialize, Default)]
struct UnpublishSiteRequest {
    /// Drop the retained `.prev-*` releases and the retired payload too.
    #[serde(default)]
    purge: bool,
    /// #1290 — "development" targets the DEV site ({slug}-dev).
    #[serde(default)]
    env: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SiteEnvQuery {
    #[serde(default)]
    env: Option<String>,
}

/// Resolve the site environment from `?env=` (query wins over body).
fn parse_site_env_param(
    query: &Option<String>,
    body: Option<&str>,
) -> Option<crate::site_env::SiteEnv> {
    let raw = query.as_deref().or(body)?;
    crate::site_env::SiteEnv::parse(raw)
}

/// DELETE /api/vibe/projects/:project_id/site — take a published site off
/// the proxy: route removed immediately, python service stopped, payload
/// retired (or purged with `?purge=true` / body `{"purge": true}`).
async fn unpublish_project_site(
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
    // #1290 — `?env=development` (or the JSON body) targets the DEV site;
    // production stays the default.
    let env = parse_site_env_param(&env_q.env, body_env.as_deref());
    let result = match env {
        Some(crate::site_env::SiteEnv::Dev) => {
            crate::proxy_sites::unpublish_site_dev(&slug, purge).await
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
async fn rollback_project_site(
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
        Some(crate::site_env::SiteEnv::Dev) => crate::proxy_sites::rollback_site_dev(&slug).await,
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
/// current DEV release of the site to the PROD target (route + service
/// refreshed exactly like a direct prod deploy).
async fn promote_project_site(
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
    match crate::proxy_sites::promote_site_dev_to_prod(&project, is_python).await {
        Ok(url) => Json(serde_json::json!({
            "success": true,
            "message": "dev release promoted to production",
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
