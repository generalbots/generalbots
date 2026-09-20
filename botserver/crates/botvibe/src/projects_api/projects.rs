//! `projects_api::projects` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// #1440 — DELETE /api/vibe/projects/:id query. `force` acknowledges the
/// default-bot protection (see delete_project) and deletes unconditionally.
#[derive(Debug, Deserialize)]
pub struct DeleteProjectQuery {
    #[serde(default)]
    pub force: bool,
}

/// One-column helper row for EXISTS-style lookups.
#[derive(Debug, diesel::QueryableByName)]
pub(crate) struct OneRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(crate) one: i32,
}

pub(crate) type ApiResult = (StatusCode, Json<ProjectResponse>);

pub(crate) fn ok_project(p: Project) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: Some(p),
            projects: None,
            error: None,
            code: None,
        }),
    )
}

pub(crate) fn ok_projects(list: Vec<Project>) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: Some(list),
            error: None,
            code: None,
        }),
    )
}

pub(crate) fn err_response(msg: String) -> ApiResult {
    log::error!("Vibe projects API error: {msg}");
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
            code: None,
        }),
    )
}

pub(crate) fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe projects API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
            code: None,
        }),
    )
}

pub(crate) fn deleted() -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: None,
            error: None,
            code: None,
        }),
    )
}

// ── #1187: project export + external git PR creation ────────────────────────
#[derive(Debug, Serialize)]
pub(crate) struct ExportResponse {
    pub(crate) success: bool,
    pub(crate) project_id: Option<Uuid>,
    pub(crate) name: Option<String>,
    pub(crate) files: Option<Vec<ExportFile>>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportFile {
    pub(crate) path: String,
    pub(crate) bytes: usize,
    // base64 content so any encoding round-trips losslessly.
    pub(crate) content_base64: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreatePrRequest {
    pub(crate) title: String,
    pub(crate) head: String,
    #[serde(default)]
    pub(crate) base: String,
    #[serde(default)]
    pub(crate) body: String,
}

/// Deterministic per-project host port (31000-31999) so re-runs and the
/// Browser button reuse the same proxy device instead of piling up devices.
pub(crate) fn project_run_port(project: &Project) -> u16 {
    let offset = (project_hash(&project.name) % 1000) as u16;
    if let Ok(p) = std::env::var("VIBE_RUN_PORT_BASE") {
        if let Ok(base) = p.parse::<u16>() {
            return base + offset;
        }
    }
    31000 + offset
}

pub(crate) fn project_hash(name: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in name.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// Ports allocated for dev-VM proxy devices (see `project_run_port`).
pub(crate) const DEV_VM_PORT_MIN: u16 = 31000;

pub(crate) const DEV_VM_PORT_MAX: u16 = 32000;

#[derive(Debug, Deserialize)]
pub(crate) struct VmPreviewQuery {
    pub(crate) port: u16,
    pub(crate) path: Option<String>,
    pub(crate) token: Option<String>,
}

// ── Project run history + token usage (Properties window) ────────────────────
// The Properties dialog shows the full set of runs for the selected project
// (state, intent, timestamps, error) plus token accounting: per-run and
// rolled-up total/input/output tokens, derived from the persisted telemetry
// rows (`tokens_used` and the `metadata.input_tokens` / `output_tokens`
// split recorded for `llm/chat` events).
#[derive(Debug, Serialize)]
pub(crate) struct ProjectTokenTotals {
    pub(crate) tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectRunRow {
    pub(crate) run_id: Uuid,
    pub(crate) state: String,
    pub(crate) intent: String,
    pub(crate) pipeline_mode: Option<String>,
    pub(crate) created_at: String,
    pub(crate) completed_at: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) tokens: ProjectTokenTotals,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectHistoryResponse {
    pub(crate) success: bool,
    pub(crate) runs: Vec<ProjectRunRow>,
    pub(crate) totals: ProjectTokenTotals,
    pub(crate) run_count: u64,
    pub(crate) error: Option<String>,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct HistoryRunRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(crate) run_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) state: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) intent: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) pipeline_mode: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub(crate) completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(crate) error: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub(crate) tokens_total: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub(crate) tokens_input: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub(crate) tokens_output: i64,
}

pub(crate) fn history_resp(runs: Option<Vec<ProjectRunRow>>, error: String) -> Response {
    Json(ProjectHistoryResponse {
        success: false,
        runs: runs.unwrap_or_default(),
        totals: ProjectTokenTotals {
            tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
        },
        run_count: 0,
        error: Some(error),
    })
    .into_response()
}
