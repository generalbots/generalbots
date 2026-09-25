//! `catalog_persistence::load` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Hydrates every persisted canvas (#921).
pub fn load_canvases(pool: &DbPool) -> Result<Vec<VibeCanvas>, String> {
    let mut conn = pool.get().map_err(|e| format!("canvas hydrate: pool get: {e}"))?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        canvas_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        project: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        content: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)]
        share_token: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: chrono::DateTime<chrono::Utc>,
    }
    let rows = diesel::sql_query(
        "SELECT canvas_id, title, project, content, share_token, created_at, updated_at \
         FROM vibe_canvases ORDER BY created_at",
    )
    .load::<Row>(&mut conn)
    .map_err(|e| format!("canvas hydrate: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeCanvas {
            canvas_id: r.canvas_id,
            title: r.title,
            project: r.project,
            content: r.content,
            share_token: r.share_token,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Hydrates every persisted issue (#921).
pub fn load_issues(pool: &DbPool) -> Result<Vec<VibeIssue>, String> {
    let mut conn = pool.get().map_err(|e| format!("issue hydrate: pool get: {e}"))?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        issue_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        body: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        labels: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)]
        state: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        assignee: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: chrono::DateTime<chrono::Utc>,
    }
    let rows = diesel::sql_query(
        "SELECT issue_id, title, body, labels, state, assignee, created_at, updated_at \
         FROM vibe_issues ORDER BY created_at",
    )
    .load::<Row>(&mut conn)
    .map_err(|e| format!("issue hydrate: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeIssue {
            issue_id: r.issue_id,
            title: r.title,
            body: r.body,
            labels: serde_json::from_value(r.labels).unwrap_or_default(),
            state: match r.state.as_str() {
                "closed" => IssueState::Closed,
                _ => IssueState::Open,
            },
            assignee: r.assignee,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Hydrates every persisted session (#921).
pub fn load_sessions(pool: &DbPool) -> Result<Vec<VibeSession>, String> {
    let mut conn = pool.get().map_err(|e| format!("session hydrate: pool get: {e}"))?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        session_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        parent_session_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        user_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        intent: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        use_case: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        budget_cents: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        run: Option<serde_json::Value>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: chrono::DateTime<chrono::Utc>,
    }
    let rows = diesel::sql_query(
        "SELECT session_id, parent_session_id, bot_id, user_id, intent, use_case, \
                budget_cents, run, created_at, updated_at \
         FROM vibe_sessions ORDER BY updated_at DESC",
    )
    .load::<Row>(&mut conn)
    .map_err(|e| format!("session hydrate: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeSession {
            session_id: r.session_id,
            parent_session_id: r.parent_session_id,
            bot_id: r.bot_id,
            user_id: r.user_id,
            intent: r.intent,
            use_case: parse_use_case(&r.use_case),
            budget_cents: r.budget_cents.max(0) as u64,
            run: r.run.and_then(|v| serde_json::from_value::<VibeRun>(v).ok()),
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Hydrates every persisted team (#921).
pub fn load_teams(pool: &DbPool) -> Result<Vec<VibeTeam>, String> {
    let mut conn = pool.get().map_err(|e| format!("team hydrate: pool get: {e}"))?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        team_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        objective: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        members: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        shared_tasks: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    }
    let rows = diesel::sql_query(
        "SELECT team_id, name, objective, members, shared_tasks, status, created_at, completed_at \
         FROM vibe_teams ORDER BY created_at",
    )
    .load::<Row>(&mut conn)
    .map_err(|e| format!("team hydrate: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeTeam {
            team_id: r.team_id,
            name: r.name,
            objective: r.objective,
            members: serde_json::from_value::<Vec<TeamMember>>(r.members).unwrap_or_default(),
            shared_tasks: serde_json::from_value::<Vec<String>>(r.shared_tasks).unwrap_or_default(),
            status: r.status,
            created_at: r.created_at,
            completed_at: r.completed_at,
        })
        .collect())
}

pub(crate) fn parse_use_case(s: &str) -> VibeUseCase {
    match s {
        "customer_support" => VibeUseCase::CustomerSupport,
        "financial_analysis" => VibeUseCase::FinancialAnalysis,
        _ => VibeUseCase::SoftwareDevelopment,
    }
}

pub(crate) fn telemetry_event_type_str(t: VibeTelemetryEventType) -> &'static str {
    match t {
        VibeTelemetryEventType::RunStarted => "run_started",
        VibeTelemetryEventType::RunCompleted => "run_completed",
        VibeTelemetryEventType::RunFailed => "run_failed",
        VibeTelemetryEventType::ToolCallStarted => "tool_call_started",
        VibeTelemetryEventType::ToolCallCompleted => "tool_call_completed",
        VibeTelemetryEventType::ToolCallFailed => "tool_call_failed",
        VibeTelemetryEventType::ApprovalRequested => "approval_requested",
        VibeTelemetryEventType::ApprovalGranted => "approval_granted",
        VibeTelemetryEventType::ApprovalDenied => "approval_denied",
    }
}

pub(crate) fn parse_telemetry_event_type(s: &str) -> VibeTelemetryEventType {
    match s {
        "run_completed" => VibeTelemetryEventType::RunCompleted,
        "run_failed" => VibeTelemetryEventType::RunFailed,
        "tool_call_started" => VibeTelemetryEventType::ToolCallStarted,
        "tool_call_completed" => VibeTelemetryEventType::ToolCallCompleted,
        "tool_call_failed" => VibeTelemetryEventType::ToolCallFailed,
        "approval_requested" => VibeTelemetryEventType::ApprovalRequested,
        "approval_granted" => VibeTelemetryEventType::ApprovalGranted,
        "approval_denied" => VibeTelemetryEventType::ApprovalDenied,
        _ => VibeTelemetryEventType::RunStarted,
    }
}

/// Write-through persistence for a single telemetry event (#921).
///
/// `vibe_telemetry.run_id` references `vibe_runs(run_id)`, so an event whose
/// run has not been persisted yet fails the FK. Telemetry is non-critical:
/// the caller treats a failed insert as a no-op rather than propagating the
/// error into the agent loop.
pub fn save_telemetry_event(pool: &DbPool, event: &VibeTelemetryEvent) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("telemetry persist: pool get: {e}"))?;
    let metadata = serde_json::to_value(&event.metadata)
        .map_err(|e| format!("telemetry persist: metadata serialize: {e}"))?;
    let tokens = event.tokens_used.map(|t| t.min(i32::MAX as u32) as i32);
    diesel::sql_query(
        "INSERT INTO vibe_telemetry \
         (event_id, run_id, event_type, tool_name, use_case, latency_ms, \
          tokens_used, estimated_cost, success, error, timestamp, metadata) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
         ON CONFLICT (event_id) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(event.event_id)
    .bind::<diesel::sql_types::Uuid, _>(event.run_id)
    .bind::<diesel::sql_types::Text, _>(telemetry_event_type_str(event.event_type))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(event.tool_name.as_deref())
    .bind::<diesel::sql_types::Text, _>(event.use_case.to_string())
    .bind::<diesel::sql_types::BigInt, _>(event.latency_ms as i64)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(tokens)
    .bind::<diesel::sql_types::Double, _>(event.estimated_cost)
    .bind::<diesel::sql_types::Bool, _>(event.success)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(event.error.as_deref())
    .bind::<diesel::sql_types::Timestamptz, _>(event.timestamp)
    .bind::<diesel::sql_types::Jsonb, _>(metadata)
    .execute(&mut conn)
    .map_err(|e| format!("telemetry persist: {e}"))?;
    Ok(())
}

/// Hydrates the most recent persisted telemetry events (#921).
///
/// Returns events in chronological order (oldest first) to match the
/// in-memory `record()` append order, bounded to `limit` rows so a large
/// archive cannot exhaust memory on startup.
pub fn load_telemetry_events(pool: &DbPool, limit: i64) -> Result<Vec<VibeTelemetryEvent>, String> {
    let mut conn = pool.get().map_err(|e| format!("telemetry hydrate: pool get: {e}"))?;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        event_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        run_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        event_type: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        tool_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        use_case: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        latency_ms: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
        tokens_used: Option<i32>,
        #[diesel(sql_type = diesel::sql_types::Double)]
        estimated_cost: f64,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        success: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        error: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        timestamp: chrono::DateTime<chrono::Utc>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        metadata: serde_json::Value,
    }
    let rows = diesel::sql_query(
        "SELECT event_id, run_id, event_type, tool_name, use_case, latency_ms, \
                tokens_used, estimated_cost, success, error, timestamp, metadata \
         FROM (SELECT * FROM vibe_telemetry ORDER BY timestamp DESC LIMIT $1) latest \
         ORDER BY timestamp ASC",
    )
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<Row>(&mut conn)
    .map_err(|e| format!("telemetry hydrate: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeTelemetryEvent {
            event_id: r.event_id,
            run_id: r.run_id,
            event_type: parse_telemetry_event_type(&r.event_type),
            tool_name: r.tool_name,
            use_case: parse_use_case(&r.use_case),
            latency_ms: r.latency_ms.max(0) as u64,
            tokens_used: r.tokens_used.map(|t| t.max(0) as u32),
            estimated_cost: r.estimated_cost,
            success: r.success,
            error: r.error,
            timestamp: r.timestamp,
            metadata: serde_json::from_value(r.metadata).unwrap_or_default(),
        })
        .collect())
}
