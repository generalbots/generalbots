//! Background scheduler: fires due natural-language schedules on a fixed tick.
use crate::cron::CronExpr;
use crate::models::{AgentSchedule, NewAgentRun};
use crate::schema::agent_runs;
use crate::schema::agent_schedules;
use crate::state::AutomationService;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

const TICK_SECS: u64 = 30;

fn due_schedules(state: &AutomationService) -> Result<Vec<AgentSchedule>, String> {
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| format!("DB pool: {e}"))?;
    let now = chrono::Utc::now();
    agent_schedules::dsl::agent_schedules
        .filter(agent_schedules::dsl::enabled.eq(true))
        .filter(agent_schedules::dsl::next_run_at.le(now))
        .limit(25)
        .load::<AgentSchedule>(&mut conn)
        .map_err(|e| format!("query due schedules: {e}"))
}

fn fire_schedule(state: &Arc<AutomationService>, schedule: &AgentSchedule) {
    let mut conn = match state.pool().get() {
        Ok(c) => c,
        Err(e) => {
            error!("[automation] pool for cron fire: {e}");
            return;
        }
    };
    let new_run = NewAgentRun {
        id: Uuid::new_v4(),
        schedule_id: Some(schedule.id),
        bot_id: schedule.bot_id,
        trigger_kind: "cron".to_string(),
        status: crate::engine::STATUS_QUEUED.to_string(),
    };
    let run_id = new_run.id;
    if let Err(e) = diesel::insert_into(agent_runs::dsl::agent_runs)
        .values(&new_run)
        .execute(&mut conn)
    {
        error!("[automation] insert cron run {}: {e}", schedule.id);
        return;
    }
    let next = CronExpr::parse(&schedule.cron_expr)
        .ok()
        .and_then(|c| c.next_after(chrono::Utc::now()));
    if let Err(e) =
        diesel::update(agent_schedules::dsl::agent_schedules.find(schedule.id))
            .set(agent_schedules::dsl::next_run_at.eq(next))
            .execute(&mut conn)
    {
        error!("[automation] advance next_run_at {}: {e}", schedule.id);
    }
    drop(conn);
    tokio::spawn(crate::engine::execute_run(state.clone(), run_id));
}

pub fn spawn_scheduler(state: Arc<AutomationService>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(TICK_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            match due_schedules(&state) {
                Ok(schedules) => {
                    for schedule in schedules {
                        fire_schedule(&state, &schedule);
                    }
                }
                Err(ctx) => error!("[automation] scheduler {ctx}"),
            }
        }
    });
}
