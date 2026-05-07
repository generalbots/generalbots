use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cron::Schedule;
use diesel::prelude::*;
use log::{error, info};
use uuid::Uuid;

use crate::schema::auto_tasks;
use crate::state::TasksState;
use crate::types::{AutoTask, NewAutoTask};

pub struct SchedulerConfig {
    pub check_interval_secs: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
        }
    }
}

pub async fn start_scheduler(state: Arc<TasksState>, config: SchedulerConfig) {
    let interval = config.check_interval_secs;
    info!("Task scheduler started with interval {}s", interval);

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        match get_due_tasks(&state) {
            Ok(tasks) => {
                for task in tasks {
                    info!("Executing scheduled task: {} ({})", task.title, task.id);
                    if let Err(e) = execute_scheduled_task(&state, &task).await {
                        error!("Scheduled task {} failed: {}", task.id, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch due tasks: {}", e);
            }
        }
    }
}

fn get_due_tasks(state: &Arc<TasksState>) -> Result<Vec<AutoTask>, String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    auto_tasks::table
        .filter(auto_tasks::enabled.eq(true))
        .load::<AutoTask>(&mut conn)
        .map_err(|e| format!("Query error: {}", e))
}

async fn execute_scheduled_task(state: &Arc<TasksState>, task: &AutoTask) -> Result<(), String> {
    if let Some(ref schedule_str) = task.schedule {
        if !should_run_now(schedule_str) {
            return Ok(());
        }
    }

    let cmd_result = (state.run_command)("echo", &["scheduled task executed"]);

    match cmd_result {
        Ok(output) => {
            info!(
                "Scheduled task {} executed successfully: {}",
                task.id, output
            );
            Ok(())
        }
        Err(e) => {
            error!("Scheduled task {} execution failed: {}", task.id, e);
            Err(e)
        }
    }
}

fn should_run_now(schedule_expr: &str) -> bool {
    match schedule_expr.parse::<Schedule>() {
        Ok(schedule) => {
            let now = Utc::now();
            schedule.upcoming(Utc).next().map_or(false, |next| {
                (next - now).num_seconds() < 60
            })
        }
        Err(_) => {
            error!("Invalid cron expression: {}", schedule_expr);
            false
        }
    }
}

pub fn create_auto_task(
    state: &Arc<TasksState>,
    bot_id: Uuid,
    title: &str,
    description: Option<&str>,
    schedule: Option<&str>,
) -> Result<AutoTask, String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let new_task = NewAutoTask {
        id: Uuid::new_v4(),
        bot_id,
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        schedule: schedule.map(|s| s.to_string()),
        enabled: true,
    };

    diesel::insert_into(auto_tasks::table)
        .values(&new_task)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {}", e))?;

    auto_tasks::table
        .find(new_task.id)
        .first::<AutoTask>(&mut conn)
        .map_err(|e| format!("Fetch after insert error: {}", e))
}

pub fn update_auto_task_enabled(
    state: &Arc<TasksState>,
    task_id: Uuid,
    enabled: bool,
) -> Result<(), String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    diesel::update(auto_tasks::table.find(task_id))
        .set(auto_tasks::enabled.eq(enabled))
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {}", e))?;

    Ok(())
}

pub fn delete_auto_task(state: &Arc<TasksState>, task_id: Uuid) -> Result<(), String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    diesel::delete(auto_tasks::table.find(task_id))
        .execute(&mut conn)
        .map_err(|e| format!("Delete error: {}", e))?;

    Ok(())
}
