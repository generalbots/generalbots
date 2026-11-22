use super::*;

pub(super) fn is_terminal(status: &str) -> bool {
    matches!(
        status,
        engine::STATUS_COMPLETED
            | engine::STATUS_FAILED
            | engine::STATUS_CANCELLED
            | engine::STATUS_TIMEOUT
    )
}

struct SseTick {
    spans: Vec<crate::models::AgentSpan>,
    total_spans: i64,
    run: Option<AgentRun>,
}

pub(super) fn poll_tick(state: &AutomationService, run_id: Uuid, emitted: usize) -> Option<SseTick> {
    use agent_runs::dsl as r;
    use agent_spans::dsl as s;
    let mut conn = match state.pool().get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("automation sse {run_id}: db pool unavailable: {e}");
            return None;
        }
    };
    let spans = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .order(s::started_at.asc())
        .offset(emitted as i64)
        .limit(RUNS_PAGE_LIMIT)
        .load::<crate::models::AgentSpan>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span poll failed: {e}");
            return None;
        }
    };
    let total = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .count()
        .get_result::<i64>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span count failed: {e}");
            return None;
        }
    };
    let run = match r::agent_runs.find(run_id).first::<AgentRun>(&mut conn).optional() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: run poll failed: {e}");
            return None;
        }
    };
    Some(SseTick { spans, total_spans: total, run })
}

/// `GET /api/automations/runs/:id/events` — Server-Sent Events stream that
/// polls the database every second, emitting newly persisted spans and a
/// terminal run-status event before closing.
/// SSE auth: EventSource cannot set headers, so the suite passes the
/// Bearer token as `?token=`. Either transport must yield a subject.
pub(super) fn sse_authorized(headers: &HeaderMap, token_param: &str) -> bool {
    let header_token = match headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(token) => token,
        None => "",
    };
    let candidate = if header_token.is_empty() { token_param } else { header_token };
    if candidate.is_empty() {
        return false;
    }
    tenant::user_id_from_claims_subject(candidate).is_some()
}
