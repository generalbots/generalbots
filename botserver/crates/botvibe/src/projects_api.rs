//! #743 — REST surface for the Vibe project registry. Handlers are thin:
//! validation + delegation to `ProjectRegistry`; errors are sanitized.
//! #768 — every mutation is gated by per-project RBAC: create grants the
//! creator `owner`; update requires `developer+`; delete requires `owner`.

use std::sync::Arc;

use diesel::{OptionalExtension, RunQueryDsl};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::harness;
use crate::metering::VMetering;
use crate::projects::{
    CreateProjectRequest, ListProjectsQuery, Project, ProjectRegistryRef, UpdateProjectRequest,
};
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::vm_lifecycle::VmLifecycle;

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub success: bool,
    pub project: Option<Project>,
    pub projects: Option<Vec<Project>>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<ProjectResponse>);

/// Resolve the caller's active branch for an org so project creation and
/// metering run against the real tenant scope instead of the nil branch.
fn resolve_org_branch(registry: &ProjectRegistryRef, org_id: Uuid) -> Option<Uuid> {
    if org_id.is_nil() {
        return None;
    }
    match registry.conn() {
        Ok(mut conn) => {
            #[derive(diesel::QueryableByName)]
            struct BranchRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
            }
            diesel::sql_query(
                "SELECT id FROM branches WHERE org_id = $1 AND is_active = true \
                 ORDER BY created_at ASC LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(org_id)
            .get_result::<BranchRow>(&mut conn)
            .optional()
            .ok()
            .flatten()
            .map(|r| r.id)
        }
        Err(e) => {
            log::error!("Vibe: resolve branch for org {org_id} failed: {e}");
            None
        }
    }
}

fn ok_project(p: Project) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: Some(p),
            projects: None,
            error: None,
        }),
    )
}

fn ok_projects(list: Vec<Project>) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: Some(list),
            error: None,
        }),
    )
}

fn err_response(msg: String) -> ApiResult {
    log::error!("Vibe projects API error: {msg}");
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
        }),
    )
}

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe projects API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
        }),
    )
}

fn deleted() -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: None,
            error: None,
        }),
    )
}

async fn create_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<Arc<VMetering>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateProjectRequest>,
) -> ApiResult {
    if user.user_id.is_nil() {
        return forbidden("forbidden: anonymous users cannot create projects".into());
    }
    if req.name.trim().is_empty() {
        return err_response("project name must not be empty".into());
    }
    // #918 — tenant scope comes from the authenticated principal, never from
    // untrusted request fields; a caller cannot create a project under another
    // organization.
    let mut req = req;
    req.org_id = Some(user.organization_id.unwrap_or_else(Uuid::nil));
    let org_id = req.org_id.unwrap_or_else(Uuid::nil);
    // #1267 — resolve the caller's real org branch when the request does not
    // carry one, so metering sees the org's actual plan (a nil branch always
    // resolves to the Free plan and wrongly blocks custom projects even for
    // private-cloud subscribers) and the project lands in the right tenant
    // scope instead of the nil branch.
    let branch_id = match req.branch_id {
        Some(bid) => bid,
        None => resolve_org_branch(&registry, org_id).unwrap_or_else(Uuid::nil),
    };
    req.branch_id = Some(branch_id);
    if let Err(e) = metering.enforce_project_creation(
        org_id,
        branch_id,
        req.project_type.as_deref().unwrap_or("bot"),
    ) {
        return forbidden(e);
    }
    match registry.create(&req) {
        Ok(p) => {
            // Grant ownership BEFORE seeding: a project the caller cannot
            // administer must not be visible in their list (#931).
            if let Err(e) = rbac.set_user_role(p.id, user.user_id, ProjectRole::Owner) {
                log::error!("grant owner on project {} failed: {e}", p.id);
                if let Err(de) = registry.delete(p.id) {
                    log::error!("compensating delete for project {} failed: {de}", p.id);
                }
                return err_response(format!("grant project ownership failed: {e}"));
            }
            // Vibe owns its starter content: seed the built-in template
            // (calculator for calculator-style projects, README starter
            // otherwise) so the workspace is never empty and nothing depends
            // on a pre-seeded external tree. Never clobbers agent output.
            // A failed seed must not leave an orphan project row whose
            // workspace is permanently empty (#931).
            let key = workspace_key(&p);
            // #1271 — github-mode clones the caller's repository into the
            // workspace; seeding the built-in template first would make the
            // clone target non-empty and fail. Clone replaces the seed.
            let wants_seed = p.source_control != "github";
            if wants_seed {
                if let Err(e) = crate::templates::seed_project_workspace(&key, &p.name) {
                    log::error!("seed workspace for project {} failed: {e}", p.id);
                    if let Err(de) = registry.delete(p.id) {
                        log::error!("compensating delete for project {} failed: {de}", p.id);
                    }
                    return err_response(format!("seed project workspace failed: {e}"));
                }
            }
            // #1271 — git-mode projects get a real Forgejo repo + origin
            // remote so Deploy can snapshot per-deploy branches and promote
            // dev→prod by runtime. A git wiring failure is logged, not fatal:
            // the project still works in native mode until ALM is configured.
            // github-mode projects clone the caller's external repository into
            // the workspace instead (payload.clone_url).
            if p.source_control == "github" {
                if let Err(e) = crate::git_mode::ensure_github_clone(&p).await {
                    log::error!("github-mode wiring for project {} failed: {e}", p.id);
                }
            } else if p.source_control == "git" {
                if let Err(e) = crate::git_mode::ensure_git_repo(&p).await {
                    log::error!("git-mode wiring for project {} failed: {e}", p.id);
                }
            }
            ok_project(p)
        }
        Err(e) => err_response(e),
    }
}

async fn delete_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(lifecycle): Extension<Arc<VmLifecycle>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Owner) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    // Cascade-delete the project's VMs (rows + Incus containers) first so
    // no orphaned containers or vm_instances rows outlive the project.
    if let Err(e) = lifecycle.delete_all_for_project(id) {
        log::error!("Vibe: cascade-delete VMs for project {id} failed: {e}");
    }
    match registry.delete(id) {
        Ok(true) => deleted(),
        Ok(false) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

async fn update_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Developer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match registry.update(id, &req) {
        Ok(true) => match registry.get(id) {
            Ok(Some(p)) => ok_project(p),
            Ok(None) => err_response(format!("project {id} not found")),
            Err(e) => err_response(e),
        },
        Ok(false) => err_response(format!("project {id} not found or no changes")),
        Err(e) => err_response(e),
    }
}

async fn list_projects(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListProjectsQuery>,
) -> ApiResult {
    if user.user_id.is_nil() {
        return forbidden("forbidden: anonymous users cannot list projects".into());
    }
    // #1267 — resolve the caller's real org branch when the query does not
    // carry one, mirroring create_project. Otherwise a project created in the
    // org branch (see create) is invisible to the list, which would default
    // to the nil branch and return an empty/stale set (#931 scope mismatch).
    let query = if query.branch_id.is_none() {
        let org_id = user.organization_id.unwrap_or_else(Uuid::nil);
        ListProjectsQuery {
            branch_id: resolve_org_branch(&registry, org_id).or(query.branch_id),
            ..query
        }
    } else {
        query
    };
    match registry.list(&query) {
        Ok(list) => ok_projects(list),
        Err(e) => err_response(e),
    }
}

async fn get_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match registry.get(id) {
        Ok(Some(p)) => ok_project(p),
        Ok(None) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

pub fn projects_router(
    registry: ProjectRegistryRef,
    rbac: ProjectRbac,
    metering: Arc<VMetering>,
    lifecycle: Arc<VmLifecycle>,
) -> axum::Router {
    use axum::routing::{delete, get, post, put};
    axum::Router::new()
        .route("/api/vibe/projects", post(create_project))
        .route("/api/vibe/projects", get(list_projects))
        .route("/api/vibe/projects/:project_id", get(get_project))
        .route("/api/vibe/projects/:project_id", put(update_project))
        .route("/api/vibe/projects/:project_id", delete(delete_project))
        .route("/api/vibe/projects/:project_id/files", get(list_project_files))
        .route("/api/vibe/projects/:project_id/files/content", get(read_project_file))
        .route("/api/vibe/projects/:project_id/files", post(write_project_file))
        .route("/api/vibe/projects/:project_id/export", get(export_project))
        .route("/api/vibe/projects/:project_id/git/pr", post(create_project_pr))
        // #1192 — run the project's own custom app: stream the workspace
        // files (the LLM-generated source) so Play/Preview opens the real app
        // instead of a bundled template. Same per-project RBAC as the other
        // workspace endpoints; `?token=` is honored like the WS routes so the
        // embedded Browser iframe can load it directly.
        .route("/api/vibe/projects/:project_id/serve/*path", get(serve_project_file))
        // #1271 — Run actually starts the app as a process in the dev VM
        // (node visible in the project terminal's `ps`), exposed through a
        // host proxy device; the browser opens the returned URL instead of a
        // static workspace stream.
        .route("/api/vibe/projects/:project_id/run", post(run_project_app))
        // #1271 — same-origin preview of the running dev VM (server-side
        // fetch of the host proxy port, see `preview_vm_app`).
        .route("/api/vibe/projects/:project_id/vm-preview", get(preview_vm_app))
        // Branch combo over the real workspace repo (the old /api/git/*
        // endpoints resolve any non-/tmp repo to a fixed stub repo, so the
        // combo never showed the project's branches).
        .route("/api/vibe/projects/:project_id/branches", get(list_project_branches))
        .route(
            "/api/vibe/projects/:project_id/branches/:name",
            post(switch_project_branch),
        )
        .layer(Extension(lifecycle))
        .layer(Extension(registry))
        .layer(Extension(rbac))
        .layer(Extension(metering))
}

// ── Workspace file browser (load a project's actual agent output) ────────────
// The code editor used to read `/api/editor/files` (the bot's global editor
// workspace), which is unrelated to the per-project Vibe workspace. These
// endpoints expose the real `VIBE_WORKSPACE_ROOT/{slug}/` files so selecting
// a project in the sidebar loads its source instead of nothing.

#[derive(Debug, Deserialize)]
pub struct WorkspaceFileQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteWorkspaceFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceFilesResponse {
    pub success: bool,
    pub project_id: Option<Uuid>,
    pub workspace: Option<String>,
    pub files: Option<Vec<String>>,
    pub path: Option<String>,
    pub content: Option<String>,
    pub bytes: Option<usize>,
    pub error: Option<String>,
}

/// Canonical workspace directory key for a project: the ALM repo slug, which
/// is the same key `collect_workspace_files` (publish) and the agent's
/// `file/*` tools use for `VIBE_WORKSPACE_ROOT/{key}/`.
fn workspace_key(project: &Project) -> String {
    VmLifecycle::alm_repo(&project.name)
}

fn ws_ok(project_id: Uuid, key: String) -> WorkspaceFilesResponse {
    WorkspaceFilesResponse {
        success: true,
        project_id: Some(project_id),
        workspace: Some(key),
        files: None,
        path: None,
        content: None,
        bytes: None,
        error: None,
    }
}

fn ws_err(project_id: Option<Uuid>, key: Option<String>, msg: String) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    log::error!("Vibe workspace files API error: {msg}");
    (
        StatusCode::OK,
        Json(WorkspaceFilesResponse {
            success: false,
            project_id,
            workspace: key,
            files: None,
            path: None,
            content: None,
            bytes: None,
            error: Some(msg),
        }),
    )
}

fn ws_forbidden(msg: String) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    log::warn!("Vibe workspace files API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(WorkspaceFilesResponse {
            success: false,
            project_id: None,
            workspace: None,
            files: None,
            path: None,
            content: None,
            bytes: None,
            error: Some(msg),
        }),
    )
}

async fn list_project_files(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return ws_forbidden(e);
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::list_rel(&key, "", 0) {
        Ok(mut files) => {
            files.sort();
            let mut resp = ws_ok(id, key);
            resp.files = Some(files);
            (StatusCode::OK, Json(resp))
        }
        Err(e) => ws_err(Some(id), Some(key), e),
    }
}

async fn read_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkspaceFileQuery>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return ws_forbidden(e);
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::read_rel_file(&key, &query.path, 4 * 1024 * 1024) {
        Ok(bytes) => {
            let content = String::from_utf8_lossy(&bytes).into_owned();
            let mut resp = ws_ok(id, key);
            resp.path = Some(query.path.clone());
            resp.content = Some(content);
            resp.bytes = Some(bytes.len());
            (StatusCode::OK, Json(resp))
        }
        Err(e) => {
            let mut resp = ws_err(Some(id), Some(key), e);
            resp.1 .0.path = Some(query.path.clone());
            resp
        }
    }
}

// ── #1187: project export + external git PR creation ────────────────────────

#[derive(Debug, Serialize)]
struct ExportResponse {
    success: bool,
    project_id: Option<Uuid>,
    name: Option<String>,
    files: Option<Vec<ExportFile>>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExportFile {
    path: String,
    bytes: usize,
    // base64 content so any encoding round-trips losslessly.
    content_base64: String,
}

async fn export_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<ExportResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return (
            StatusCode::FORBIDDEN,
            Json(ExportResponse {
                success: false,
                project_id: Some(id),
                name: None,
                files: None,
                error: Some(e),
            }),
        );
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ExportResponse {
                    success: false,
                    project_id: Some(id),
                    name: None,
                    files: None,
                    error: Some(format!("project {id} not found")),
                }),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExportResponse {
                    success: false,
                    project_id: Some(id),
                    name: None,
                    files: None,
                    error: Some(e),
                }),
            );
        }
    };
    let key = workspace_key(&project);
    let paths = match harness::list_rel(&key, "", 0) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::OK,
                Json(ExportResponse {
                    success: false,
                    project_id: Some(id),
                    name: Some(project.name.clone()),
                    files: None,
                    error: Some(e),
                }),
            );
        }
    };
    let mut files: Vec<ExportFile> = Vec::new();
    for path in paths {
        if let Ok(bytes) = harness::read_rel_file(&key, &path, 4 * 1024 * 1024) {
            use base64::Engine as _;
            files.push(ExportFile {
                path,
                bytes: bytes.len(),
                content_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            });
        }
    }
    (
        StatusCode::OK,
        Json(ExportResponse {
            success: true,
            project_id: Some(id),
            name: Some(project.name.clone()),
            files: Some(files),
            error: None,
        }),
    )
}

#[derive(Debug, Deserialize)]
struct CreatePrRequest {
    title: String,
    head: String,
    #[serde(default)]
    base: String,
    #[serde(default)]
    body: String,
}

async fn create_project_pr(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreatePrRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Developer) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "success": false, "error": e })));
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "error": "project not found" })));
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": e })));
        }
    };
    let result = crate::gitflow::create_pull_request(
        &project.name,
        &req.title,
        &req.head,
        &req.base,
        &req.body,
    )
    .await;
    if result.success {
        (StatusCode::OK, Json(serde_json::json!({ "success": true, "data": result.data })))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": result.error })),
        )
    }
}

// ── #1271: run the project app as a real process in the dev VM ────────────
// Run pushes the workspace files into the dev container, starts node (or a
// generated static server) as a systemd service, exposes it through a host
// proxy device and returns the URL. The app process is then visible in the
// project terminal's `ps` — previously the browser streamed workspace files
// with nothing actually running on the VM.

#[derive(Debug, Deserialize)]
pub struct RunProjectQuery {
    /// Host port for the proxy device; stable per project when omitted.
    pub port: Option<u16>,
}

/// Deterministic per-project host port (31000-31999) so re-runs and the
/// Browser button reuse the same proxy device instead of piling up devices.
fn project_run_port(project: &Project) -> u16 {
    let offset = (project_hash(&project.name) % 1000) as u16;
    if let Ok(p) = std::env::var("VIBE_RUN_PORT_BASE") {
        if let Ok(base) = p.parse::<u16>() {
            return base + offset;
        }
    }
    31000 + offset
}

fn project_hash(name: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in name.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(1099511628211);
    }
    h
}

async fn run_project_app(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(lifecycle): Extension<Arc<VmLifecycle>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<RunProjectQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "success": false, "error": e })));
    }
    let project = match registry.get(project_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "error": "project not found" })));
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": e })));
        }
    };
    let files = match crate::publish::collect_workspace_files(&project) {
        Ok(f) => f,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    // #1271 — a project with an empty workspace must still serve an app when
    // Run is clicked (automatic project creation skips the explicit seeding in
    // `create_project`). Seed the starter (or calculator) template on the fly
    // so the Browser never opens against a blank "No web app yet" VM.
    if files.is_empty() {
        let key = workspace_key(&project);
        if let Err(e) = crate::templates::seed_project_workspace(&key, &project.name) {
            log::warn!("Vibe run {}: seed empty workspace failed: {e}", project.name);
        } else {
            log::info!(
                "Vibe run {}: seeded empty workspace before run",
                project.name
            );
        }
    }
    let files = match crate::publish::collect_workspace_files(&project) {
        Ok(f) => f,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let branch_id = resolve_org_branch(&registry, project.org_id).unwrap_or(project.branch_id);
    let vm = match lifecycle.create_project_vm(
        project_id,
        branch_id,
        &project.name,
        &crate::vm_lifecycle::CreateVmRequest {
            env: "development".to_string(),
            tier: "small".to_string(),
            runner_enabled: false,
        },
    ) {
        Ok(vm) => vm,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let port = query.port.unwrap_or_else(|| project_run_port(&project));
    match lifecycle.run_dev_app(&vm.container_name, &files, port) {
        Ok(url) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                // Same-origin route so the embedded Browser iframe can load
                // the dev VM through botserver (the generic /api/browser/proxy
                // rejects private hosts, and iframes cannot set headers).
                "url": format!("/api/vibe/projects/{project_id}/vm-preview?port={port}"),
                "host_url": url,
                "container": vm.container_name,
                "port": port,
                "project": project.name,
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": e })),
        ),
    }
}

/// Ports allocated for dev-VM proxy devices (see `project_run_port`).
const DEV_VM_PORT_MIN: u16 = 31000;
const DEV_VM_PORT_MAX: u16 = 32000;

#[derive(Debug, Deserialize)]
struct VmPreviewQuery {
    port: u16,
    path: Option<String>,
    token: Option<String>,
}

/// Resolve the default gateway IPv4 address from `/proc/net/route`.
/// The vibe-http proxy device binds on the host; a botserver running inside
/// an Incus container must reach it via the gateway rather than 127.0.0.1.
/// Returns `None` when no default route exists (e.g. running directly on the
/// host). Parsing the proc file avoids spawning any external command.
fn default_gateway_ip() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in raw.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let _iface = fields.next()?;
        let dest = fields.next()?;
        let gw = fields.next()?;
        if dest == "00000000" && gw.len() == 8 {
            // /proc/net/route stores the gateway little-endian: the first two
            // hex digits are the least-significant byte of the address.
            let bytes = gw.as_bytes();
            let mut octets = [0u8; 4];
            for (i, chunk) in bytes.chunks(2).enumerate() {
                octets[i] = u8::from_str_radix(
                    &String::from_utf8_lossy(chunk),
                    16,
                )
                .ok()?;
            }
            return Some(format!(
                "{}.{}.{}.{}",
                octets[3], octets[2], octets[1], octets[0]
            ));
        }
    }
    None
}

/// #1271 — same-origin preview of the project's dev VM. `run` starts the app
/// as a real process in the container and exposes it through a host proxy
/// device (`localhost:{port}`); the generic browser proxy refuses private
/// hosts, so this route fetches the dev-VM host address server-side and
/// streams the response back — authenticated exactly like the workspace
/// `serve` route (Bearer header from the desktop, or `?token=` from the
/// embedded iframe). Only ports from the dev-VM range are accepted; RBAC
/// still gates access.
async fn preview_vm_app(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<VmPreviewQuery>,
) -> Response {
    if let Err(e) = rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        log::warn!("Vibe vm-preview forbidden: {e}");
        return (StatusCode::FORBIDDEN, "forbidden").into_response();
    }
    let project = match registry.get(project_id) {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "project not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };
    let port = query.port;
    if !(DEV_VM_PORT_MIN..DEV_VM_PORT_MAX).contains(&port) {
        log::warn!("Vibe vm-preview {}: out-of-range port {port}", project.name);
        return (StatusCode::BAD_REQUEST, "invalid dev-vm port").into_response();
    }
    let path = query.path.as_deref().unwrap_or("/");
    // The vibe-http proxy device binds on the Incus HOST (`0.0.0.0:{port}`).
    // When botserver runs directly on the host (dev machines) `127.0.0.1` is
    // correct; when botserver runs inside an Incus container (prod bot
    // container), 127.0.0.1 is the container itself and the host is only
    // reachable via the default gateway. Probe the candidates in order.
    let mut candidates = vec![format!("http://127.0.0.1:{port}{path}")];
    if let Some(gateway) = default_gateway_ip() {
        let host_candidate = format!("http://{gateway}:{port}{path}");
        if !candidates.contains(&host_candidate) {
            candidates.push(host_candidate);
        }
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Vibe vm-preview {}: client build failed: {e}", project.name);
            return (StatusCode::INTERNAL_SERVER_ERROR, "proxy client").into_response();
        }
    };
    let mut last_error: Option<(String, String)> = None;
    let mut resp = None;
    for target in &candidates {
        match client.get(target).send().await {
            Ok(r) => {
                resp = Some(r);
                break;
            }
            Err(e) => last_error = Some((target.clone(), e.to_string())),
        }
    }
    let resp = match resp {
        Some(r) => r,
        None => {
            // Every candidate failed; log the last attempt's target and error.
            let (target, err) = last_error
                .unwrap_or_else(|| (candidates[0].clone(), "no reachable candidate".to_string()));
            log::warn!("Vibe vm-preview {}: fetch {target} failed: {err}", project.name);
            return (StatusCode::BAD_GATEWAY, "dev vm not reachable").into_response();
        }
    };
    let status = resp.status();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Vibe vm-preview {}: read response: {e}", project.name);
            return (StatusCode::BAD_GATEWAY, "read dev vm response").into_response();
        }
    };
    let body: axum::body::Body = if content_type.to_lowercase().contains("html") {
        match (std::str::from_utf8(&bytes), query.token.as_deref()) {
            (Ok(text), Some(token)) => serve_inject_token(text, token).into_bytes().into(),
            _ => bytes.into(),
        }
    } else {
        bytes.into()
    };
    (
        status,
        [
            (axum::http::header::CONTENT_TYPE, content_type),
            (axum::http::header::CACHE_CONTROL, "no-store".to_string()),
        ],
        body,
    )
        .into_response()
}

#[derive(Debug, Serialize)]
struct BranchInfo {
    name: String,
    current: bool,
}

async fn list_project_branches(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "success": false, "error": e })));
    }
    let project = match registry.get(project_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "error": "project not found" })));
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let key = workspace_key(&project);
    let cwd = match harness::ensure_workspace(&key) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let branches = harness::cmd::run(
        "git",
        &["branch".to_string(), "--format=%(refname:short)".to_string()],
        &cwd,
        30,
    );
    let current = harness::cmd::run(
        "git",
        &["rev-parse".to_string(), "--abbrev-ref".to_string(), "HEAD".to_string()],
        &cwd,
        15,
    )
    .ok()
    .filter(|o| o.exit_code == Some(0))
    .map(|o| o.stdout.trim().to_string())
    .unwrap_or_default();
    let mut out: Vec<BranchInfo> = Vec::new();
    if let Ok(b) = branches {
        if b.exit_code == Some(0) {
            for line in b.stdout.lines() {
                let name = line.trim();
                if name.is_empty() {
                    continue;
                }
                out.push(BranchInfo {
                    name: name.to_string(),
                    current: name == current,
                });
            }
        }
    }
    if out.is_empty() {
        out.push(BranchInfo {
            name: if current.is_empty() { "main".to_string() } else { current.clone() },
            current: true,
        });
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "success": true, "branches": out, "current": current })),
    )
}

async fn switch_project_branch(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, name)): Path<(Uuid, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let Err(e) = rbac.require_role(user.user_id, project_id, ProjectRole::Developer) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "success": false, "error": e })));
    }
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": "invalid branch name" })));
    }
    let project = match registry.get(project_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "success": false, "error": "project not found" })));
        }
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let key = workspace_key(&project);
    let cwd = match harness::ensure_workspace(&key) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e }))),
    };
    let exists = harness::cmd::run(
        "git",
        &[
            "rev-parse".to_string(),
            "--verify".to_string(),
            format!("refs/heads/{name}"),
        ],
        &cwd,
        15,
    )
    .map(|o| o.exit_code == Some(0))
    .unwrap_or(false);
    let cmd: Vec<String> = if exists {
        vec!["checkout".to_string(), name.clone()]
    } else {
        vec!["checkout".to_string(), "-B".to_string(), name.clone()]
    };
    match harness::cmd::run("git", &cmd, &cwd, 60) {
        Ok(out) if out.exit_code == Some(0) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "branch": name, "output": out.stdout.trim() })),
        ),
        Ok(out) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": format!("git checkout failed: {}", out.stderr.trim()) })),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": e.to_string() }))),
    }
}

// ── #1192: serve the project's own custom app (workspace static preview) ──
// Play/Preview resolves a project to this route when its workspace has source
// files, so the LLM-generated app runs in the Browser window without needing a
// VM. Files are streamed from `VIBE_WORKSPACE_ROOT/{slug}/` with the correct
// MIME type; HTML responses have relative asset URLs rewritten to carry the
// auth token (iframes cannot set headers).

#[derive(Debug, Deserialize)]
struct ServeQuery {
    token: Option<String>,
}

fn serve_mime_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "md" => "text/markdown; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Rewrite relative `src`/`href` asset URLs in an HTML document so sub-resources
/// carry the same `?token=` used for the iframe itself (embedded Browser iframes
/// cannot set an Authorization header). Absolute, root-relative, scheme-relative,
/// fragment, and data: URLs are left untouched.
fn serve_inject_token(html: &str, token: &str) -> String {
    if token.is_empty() || html.contains("?token=") {
        return html.to_string();
    }
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;
    while !rest.is_empty() {
        // Find the next `src="|href="|src='|href='` marker.
        let candidates = ["src=\"", "href=\"", "src='", "href='"]
            .into_iter()
            .filter_map(|m| rest.find(m).map(|p| (p, m)))
            .min_by_key(|(p, _)| *p);
        match candidates {
            Some((pos, marker)) => {
                out.push_str(&rest[..pos]);
                out.push_str(marker);
                let quote = &marker[marker.len() - 1..];
                let value_start = pos + marker.len();
                let value_end = rest[value_start..]
                    .find(quote)
                    .map(|p| value_start + p)
                    .unwrap_or(rest.len());
                let url = &rest[value_start..value_end];
                if !url.is_empty()
                    && !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("//")
                    && !url.starts_with('/')
                    && !url.starts_with('#')
                    && !url.starts_with("data:")
                    && !url.starts_with("blob:")
                {
                    let sep = if url.contains('?') { "&" } else { "?" };
                    out.push_str(url);
                    out.push_str(sep);
                    out.push_str("token=");
                    out.push_str(token);
                } else {
                    out.push_str(url);
                }
                out.push_str(quote);
                rest = &rest[value_end + 1..];
            }
            None => {
                out.push_str(rest);
                rest = "";
            }
        }
    }
    out
}

async fn serve_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((id, path)): Path<(Uuid, String)>,
    Query(query): Query<ServeQuery>,
) -> Response {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        log::warn!("Vibe serve forbidden: {e}");
        return (StatusCode::FORBIDDEN, "forbidden").into_response();
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "project not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };
    let key = workspace_key(&project);
    let mut rel = path.trim().to_string();
    if rel.is_empty() || rel.ends_with('/') {
        rel.push_str("index.html");
    }
    let bytes = match harness::read_rel_file(&key, &rel, 16 * 1024 * 1024) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("Vibe serve {key}/{rel}: {e}");
            return (StatusCode::NOT_FOUND, "app file not found").into_response();
        }
    };
    let mime = serve_mime_for(&rel);
    let body: axum::body::Body = if mime.starts_with("text/html") {
        match (std::str::from_utf8(&bytes), query.token.as_deref()) {
            (Ok(text), Some(token)) => serve_inject_token(text, token).into_bytes().into(),
            _ => bytes.into(),
        }
    } else {
        bytes.into()
    };
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, mime),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response()
}

async fn write_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<WriteWorkspaceFileRequest>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Developer) {
        return ws_forbidden(e);
    }
    if req.path.trim().is_empty() {
        return ws_err(Some(id), None, "path must not be empty".to_string());
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::write_rel_file(&key, &req.path, req.content.as_bytes()) {
        Ok(()) => {
            let mut resp = ws_ok(id, key);
            resp.path = Some(req.path.clone());
            resp.bytes = Some(req.content.len());
            (StatusCode::OK, Json(resp))
        }
        Err(e) => ws_err(Some(id), Some(key), e),
    }
}
