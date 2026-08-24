//! Planner-executor-verifier engine for adaptive agent runs (#1171).
//!
//! A run is planned by the LLM into a strict JSON step graph, executed as a
//! dependency-ordered ready-set (bounded concurrency), verified by a second
//! LLM pass with up to two repair iterations, and finalized with artifacts,
//! usage metering and optional notification delivery.

pub mod exec;
pub mod plan;
pub mod verify;

use crate::delivery;
use crate::models::{AgentRun, AgentSchedule};
use crate::schema::agent_runs;
use crate::state::AutomationService;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_COMPLETED: &str = "completed";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELLED: &str = "cancelled";
pub const STATUS_TIMEOUT: &str = "timeout";

const DEFAULT_MAX_RUNTIME_SECS: u64 = 900;

/// One node of the LLM-generated execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub name: String,
    /// One of `tool`, `llm`, `fork`, `merge`, `deliver`.
    pub kind: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub input: serde_json::Value,
}

/// Result of executing a single plan step (or fork child aggregate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutcome {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub ok: bool,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub tokens_in: i32,
    pub tokens_out: i32,
    pub vm_seconds: i32,
}

impl StepOutcome {
    pub(crate) fn skipped(step: &PlanStep, reason: String) -> Self {
        Self {
            id: step.id.clone(),
            name: step.name.clone(),
            kind: step.kind.clone(),
            ok: false,
            output: None,
            error: Some(reason),
            tokens_in: 0,
            tokens_out: 0,
            vm_seconds: 0,
        }
    }
}

/// Shared per-run context handed to executor submodules.
pub(crate) struct RunCtx {
    pub state: Arc<AutomationService>,
    pub run_id: Uuid,
    pub org_id: Option<Uuid>,
    pub schedule: Option<AgentSchedule>,
    pub cancel: Arc<AtomicBool>,
}

/// Aggregate report of the topological execution phase.
#[derive(Debug, Default)]
pub(crate) struct ExecReport {
    pub outcomes: Vec<StepOutcome>,
    pub cancelled: bool,
}

pub(crate) fn pool_conn(
    state: &AutomationService,
    context: &str,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    match state.pool().get() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!("automation {context}: db pool unavailable: {e}");
            None
        }
    }
}

pub(crate) fn load_run(state: &AutomationService, run_id: Uuid) -> Option<AgentRun> {
    let mut conn = pool_conn(state, "load run")?;
    match agent_runs::dsl::agent_runs
        .find(run_id)
        .first::<AgentRun>(&mut conn)
        .optional()
    {
        Ok(run) => run,
        Err(e) => {
            tracing::error!("automation run {run_id}: load failed: {e}");
            None
        }
    }
}

fn load_schedule(state: &AutomationService, schedule_id: Option<Uuid>) -> Option<AgentSchedule> {
    let schedule_id = schedule_id?;
    use crate::schema::agent_schedules::dsl::agent_schedules;
    let mut conn = pool_conn(state, "load schedule")?;
    match agent_schedules
        .find(schedule_id)
        .first::<AgentSchedule>(&mut conn)
        .optional()
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("automation schedule {schedule_id}: load failed: {e}");
            None
        }
    }
}

fn set_status(conn: &mut diesel::PgConnection, run_id: Uuid, new_status: &str) {
    if let Err(e) = diesel::update(agent_runs::dsl::agent_runs.find(run_id))
        .set((
            agent_runs::dsl::status.eq(new_status),
            agent_runs::dsl::started_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
    {
        tracing::error!("automation run {run_id}: mark {new_status} failed: {e}");
    }
}

fn finalize(ctx: &RunCtx, report: ExecReport, verdict: Option<serde_json::Value>, timed_out: bool) {
    let cancelled = report.cancelled || (!timed_out && ctx.cancel.load(Ordering::Relaxed));
    let any_failed = report.outcomes.iter().any(|o| !o.ok);
    let final_status = if timed_out {
        STATUS_TIMEOUT
    } else if cancelled {
        STATUS_CANCELLED
    } else if any_failed {
        STATUS_FAILED
    } else {
        STATUS_COMPLETED
    };

    let artifacts: Vec<String> = report
        .outcomes
        .iter()
        .filter_map(|o| o.output.as_deref().map(|out| out.chars().take(400).collect()))
        .collect();
    let failed_count = report.outcomes.iter().filter(|o| !o.ok).count();
    let result_summary = format!(
        "{} steps executed; {} not successful; verdict: {}",
        report.outcomes.len(),
        failed_count,
        verdict
            .as_ref()
            .and_then(|v| v.get("verdict").and_then(|x| x.as_str()))
            .unwrap_or("n/a")
    );
    let first_error = report
        .outcomes
        .iter()
        .find_map(|o| o.error.clone().filter(|_| final_status != STATUS_COMPLETED));

    let steps_json = serde_json::to_value(&report.outcomes).ok();
    let mut conn = match pool_conn(&ctx.state, "finalize") {
        Some(c) => c,
        None => return,
    };
    if let Err(e) = diesel::update(agent_runs::dsl::agent_runs.find(ctx.run_id))
        .set((
            agent_runs::dsl::status.eq(final_status),
            agent_runs::dsl::steps.eq(steps_json),
            agent_runs::dsl::result_summary.eq(Some(result_summary)),
            agent_runs::dsl::artifacts.eq(Some(serde_json::json!(artifacts))),
            agent_runs::dsl::verdict.eq(verdict),
            agent_runs::dsl::error.eq(first_error),
            agent_runs::dsl::finished_at.eq(diesel::dsl::now),
        ))
        .execute(&mut conn)
    {
        tracing::error!("automation run {}: finalize failed: {e}", ctx.run_id);
        return;
    }

    if final_status == STATUS_COMPLETED || final_status == STATUS_FAILED {
        let delivery_state = ctx.state.clone();
        let run_id = ctx.run_id;
        let schedule_ref = ctx.schedule.clone();
        tokio::spawn(async move {
            let Some(run_final) = load_run(&delivery_state, run_id) else {
                return;
            };
            let doc =
                delivery::dispatch(delivery_state.clone(), schedule_ref.as_ref(), &run_final)
                    .await;
            if let Some(mut conn) = pool_conn(&delivery_state, "delivery status") {
                let _ = delivery::save_delivery_status(&mut conn, run_id, &doc);
            }
        });
    }

    ctx.state.take_cancel_flag(ctx.run_id);
}

/// Executes one automation run end-to-end: planning, bounded-concurrency
/// execution under a runtime watchdog, verification with repair loop and
/// final persistence. Safe to spawn detached.
pub async fn execute_run(state: Arc<AutomationService>, run_id: Uuid) {
    let Some(run) = load_run(&state, run_id) else {
        tracing::error!("automation execute_run: run {run_id} not found");
        return;
    };
    let schedule = load_schedule(&state, run.schedule_id);
    match pool_conn(&state, "mark running") {
        Some(mut conn) => set_status(&mut conn, run_id, STATUS_RUNNING),
        None => return,
    }

    let cancel = state.cancel_flag(run_id);
    let goal = schedule
        .as_ref()
        .map(|s| s.goal.clone())
        .unwrap_or_else(|| "Execute the queued automation run".to_string());
    let llm = state.llm().clone();
    let plan_steps = plan::build_plan(&llm, &goal);

    if let Ok(plan_json) = serde_json::to_value(&plan_steps) {
        if let Some(mut conn) = pool_conn(&state, "save plan") {
            if let Err(e) = diesel::update(agent_runs::dsl::agent_runs.find(run_id))
                .set(agent_runs::dsl::plan.eq(Some(plan_json)))
                .execute(&mut conn)
            {
                tracing::error!("automation run {run_id}: save plan failed: {e}");
            }
        }
    }

    let ctx = Arc::new(RunCtx {
        state: state.clone(),
        run_id,
        org_id: schedule.as_ref().and_then(|s| s.org_id),
        schedule,
        cancel: cancel.clone(),
    });

    let max_runtime_secs = ctx
        .schedule
        .as_ref()
        .map(|s| s.max_runtime_secs.max(1) as u64)
        .unwrap_or(DEFAULT_MAX_RUNTIME_SECS);
    let exec_future = exec::run_plan(ctx.clone(), plan_steps);
    match tokio::time::timeout(Duration::from_secs(max_runtime_secs), exec_future).await {
        Err(_) => {
            tracing::error!(
                "automation run {run_id} exceeded the {max_runtime_secs}s watchdog"
            );
            finalize(&ctx, ExecReport::default(), None, true);
        }
        Ok(report) => {
            let cancelled = ctx.cancel.load(Ordering::Relaxed);
            let verified =
                verify::verify_and_repair(ctx.clone(), report.outcomes).await;
            finalize(
                &ctx,
                ExecReport {
                    outcomes: verified.outcomes,
                    cancelled,
                },
                Some(verified.verdict),
                false,
            );
        }
    }
}
