//! `projects_api::workspace_4` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) async fn serve_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((id, path)): Path<(Uuid, String)>,
    Query(query): Query<ServeQuery>,
) -> Response {
    // #1340 — an anonymous request here is the embedded-iframe transport
    // (no Authorization header possible): re-authenticate via `?token=`.
    let user = if user.is_authenticated() {
        user
    } else {
        match resolve_iframe_user(query.token.clone()) {
            Some(u) => u,
            None => return (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
        }
    };
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

pub(crate) async fn write_project_file(
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

pub(crate) async fn project_run_history(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Response {
    let conn = match registry.conn() {
        Ok(c) => c,
        Err(e) => return history_resp(None, e),
    };
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return (
            StatusCode::FORBIDDEN,
            Json(ProjectHistoryResponse {
                success: false,
                runs: Vec::new(),
                totals: ProjectTokenTotals {
                    tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                },
                run_count: 0,
                error: Some(e),
            }),
        )
            .into_response();
    }
    let want = id.to_string();
    let mut conn = conn;
    // Runs whose payload carries this project id, newest first, with token
    // usage aggregated from the persisted telemetry rows.
    let runs = diesel::sql_query(
        "SELECT r.run_id, r.state, r.intent, \
                r.config->>'pipeline_mode' AS pipeline_mode, \
                r.created_at, r.completed_at, r.error, \
                COALESCE(SUM(t.tokens_used), 0)::bigint AS tokens_total, \
                COALESCE(SUM((t.metadata->>'input_tokens')::bigint), 0)::bigint AS tokens_input, \
                COALESCE(SUM((t.metadata->>'output_tokens')::bigint), 0)::bigint AS tokens_output \
         FROM vibe_runs r \
         LEFT JOIN vibe_telemetry t ON t.run_id = r.run_id \
         WHERE r.config->>'project_id' = $1 \
         GROUP BY r.run_id \
         ORDER BY r.created_at DESC \
         LIMIT 200",
    )
    .bind::<diesel::sql_types::Text, _>(&want)
    .load::<HistoryRunRow>(&mut conn);
    let rows = match runs {
        Ok(rows) => rows,
        Err(e) => return history_resp(None, format!("run history: {e}")),
    };
    let mut totals = ProjectTokenTotals {
        tokens: 0,
        input_tokens: 0,
        output_tokens: 0,
    };
    let out: Vec<ProjectRunRow> = rows
        .into_iter()
        .map(|r| {
            totals.tokens = totals.tokens.saturating_add(r.tokens_total.max(0) as u64);
            totals.input_tokens =
                totals.input_tokens.saturating_add(r.tokens_input.max(0) as u64);
            totals.output_tokens =
                totals.output_tokens.saturating_add(r.tokens_output.max(0) as u64);
            ProjectRunRow {
                run_id: r.run_id,
                state: r.state,
                intent: r.intent,
                pipeline_mode: r.pipeline_mode,
                created_at: r.created_at.to_rfc3339(),
                completed_at: r.completed_at.map(|c| c.to_rfc3339()),
                error: r.error,
                tokens: ProjectTokenTotals {
                    tokens: r.tokens_total.max(0) as u64,
                    input_tokens: r.tokens_input.max(0) as u64,
                    output_tokens: r.tokens_output.max(0) as u64,
                },
            }
        })
        .collect();
    let run_count = out.len() as u64;
    Json(ProjectHistoryResponse {
        success: true,
        runs: out,
        totals,
        run_count,
        error: None,
    })
    .into_response()
}
