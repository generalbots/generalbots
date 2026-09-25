//! Split from `projects_api.rs` per #1443 (AGENTS.md 450-line rule).
//! #743 — REST surface for the Vibe project registry. Handlers are thin:
//! validation + delegation to `ProjectRegistry`; errors are sanitized.
//! #768 — every mutation is gated by per-project RBAC: create grants the
//! creator `owner`; update requires `developer+`; delete requires `owner`.

mod branches;
mod projects;
mod serve;
mod workspace;
mod workspace_2;
mod workspace_3;
mod workspace_4;

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
use botsecurity_auth::auth_api::types::{AuthenticatedUser, Role};
use crate::harness;
use crate::metering::VMetering;
use crate::projects::{
    CreateProjectRequest, ListProjectsQuery, Project, ProjectRegistryRef, UpdateProjectRequest,
};
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::vm_lifecycle::VmLifecycle;

pub use branches::{ProjectResponse};
pub(crate) use branches::{delete_project, list_project_branches, list_projects, resolve_org_branch, switch_project_branch};
pub use projects::{DeleteProjectQuery};
pub(crate) use projects::{ApiResult, CreatePrRequest, DEV_VM_PORT_MAX, DEV_VM_PORT_MIN, ExportFile, ExportResponse, HistoryRunRow, OneRow, ProjectHistoryResponse, ProjectRunRow, ProjectTokenTotals, VmPreviewQuery, deleted, err_response, forbidden, history_resp, ok_project, ok_projects, project_run_port};
pub(crate) use serve::{ServeQuery, default_gateway_ip, serve_inject_token, serve_mime_for, urlencode_path};
pub use workspace::{WorkspaceFileQuery, WorkspaceFilesResponse, WriteWorkspaceFileRequest, projects_router};
pub(crate) use workspace::{workspace_key, ws_ok};
pub use workspace_2::{RunProjectQuery};
pub(crate) use workspace_2::{create_project_pr, export_project, list_project_files, read_project_file, run_project_app, ws_err, ws_forbidden};
pub(crate) use workspace_3::{preview_vm_app, resolve_iframe_user, run_website_via_proxy};
pub(crate) use workspace_4::{project_run_history, serve_project_file, write_project_file};
