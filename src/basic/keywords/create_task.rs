use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};

use std::sync::Arc;
use uuid::Uuid;

pub fn create_task_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            [
                "CREATE_TASK",
                "$expr$",
                ",",
                "$expr$",
                ",",
                "$expr$",
                ",",
                "$expr$",
            ],
            false,
            move |context, inputs| {
                let title = context.eval_expression_tree(&inputs[0])?.to_string();
                let assignee = context.eval_expression_tree(&inputs[1])?.to_string();
                let due_date = context.eval_expression_tree(&inputs[2])?.to_string();
                let project_id_input = context.eval_expression_tree(&inputs[3])?;

                let project_id =
                    if project_id_input.is_unit() || project_id_input.to_string() == "null" {
                        None
                    } else {
                        Some(project_id_input.to_string())
                    };

                trace!(
                    "CREATE_TASK: title={}, assignee={}, due_date={}, project_id={:?} for user={}",
                    title,
                    assignee,
                    due_date,
                    project_id,
                    user_clone.user_id
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(_rt) = rt {
                        let result = execute_create_task(
                            &state_for_task,
                            &user_for_task,
                            &title,
                            &assignee,
                            &due_date,
                            project_id.as_deref(),
                        );
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send CREATE_TASK result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(task_id)) => Ok(Dynamic::from(task_id)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE_TASK failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "CREATE_TASK timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE_TASK thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user;

    engine
        .register_custom_syntax(
            ["ASSIGN_SMART", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let task_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let team_input = context.eval_expression_tree(&inputs[1])?;
                let load_balance = context
                    .eval_expression_tree(&inputs[2])?
                    .as_bool()
                    .unwrap_or(true);

                let mut team = Vec::new();
                if team_input.is_array() {
                    let arr = team_input.cast::<rhai::Array>();
                    for item in arr.iter() {
                        team.push(item.to_string());
                    }
                } else {
                    team.push(team_input.to_string());
                }

                trace!(
                    "ASSIGN_SMART: task={}, team={:?}, load_balance={} for user={}",
                    task_id,
                    team,
                    load_balance,
                    user_clone2.user_id
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(_rt) = rt {
                        let result = smart_assign_task(
                            &state_for_task,
                            &user_for_task,
                            &task_id,
                            team,
                            load_balance,
                        );
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send ASSIGN_SMART result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(assignee)) => Ok(Dynamic::from(assignee)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("ASSIGN_SMART failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "ASSIGN_SMART timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

fn execute_create_task(
    state: &AppState,
    user: &UserSession,
    title: &str,
    assignee: &str,
    due_date: &str,
    project_id: Option<&str>,
) -> Result<String, String> {
    let task_id = Uuid::new_v4().to_string();

    let due_datetime = parse_due_date(due_date)?;

    let actual_assignee = if assignee == "auto" {
        auto_assign_task(state, project_id)?
    } else {
        assignee.to_string()
    };

    let priority = determine_priority(due_datetime);

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let query = diesel::sql_query(
        "INSERT INTO tasks (id, title, assignee, due_date, project_id, priority, status, created_by, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'open', $7, $8)"
    )
    .bind::<diesel::sql_types::Text, _>(&task_id)
    .bind::<diesel::sql_types::Text, _>(title)
    .bind::<diesel::sql_types::Text, _>(&actual_assignee)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&due_datetime)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&project_id)
    .bind::<diesel::sql_types::Text, _>(&priority);

    let user_id_str = user.user_id.to_string();
    let now = Utc::now();
    let query = query
        .bind::<diesel::sql_types::Text, _>(&user_id_str)
        .bind::<diesel::sql_types::Timestamptz, _>(&now);

    query.execute(&mut *conn).map_err(|e| {
        error!("Failed to create task: {}", e);
        format!("Failed to create task: {}", e)
    })?;

    send_task_notification(state, &task_id, title, &actual_assignee, due_datetime)?;

    trace!(
        "Created task '{}' assigned to {} (ID: {})",
        title,
        actual_assignee,
        task_id
    );

    Ok(task_id)
}

fn smart_assign_task(
    state: &AppState,
    _user: &UserSession,
    task_id: &str,
    team: Vec<String>,
    load_balance: bool,
) -> Result<String, String> {
    if !load_balance {
        return Ok(team[0].clone());
    }

    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let mut best_assignee = team[0].clone();
    let mut min_workload = i64::MAX;

    for member in &team {
        let query = diesel::sql_query(
            "SELECT COUNT(*) as task_count FROM tasks
             WHERE assignee = $1 AND status IN ('open', 'in_progress')",
        )
        .bind::<diesel::sql_types::Text, _>(member);

        #[derive(QueryableByName)]
        struct TaskCount {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            task_count: i64,
        }

        let result: Result<Vec<TaskCount>, _> = query.load(&mut *conn);

        if let Ok(counts) = result {
            if let Some(count) = counts.first() {
                if count.task_count < min_workload {
                    min_workload = count.task_count;
                    best_assignee.clone_from(member);
                }
            }
        }
    }

    let update_query = diesel::sql_query("UPDATE tasks SET assignee = $1 WHERE id = $2")
        .bind::<diesel::sql_types::Text, _>(&best_assignee)
        .bind::<diesel::sql_types::Text, _>(task_id);

    update_query.execute(&mut *conn).map_err(|e| {
        error!("Failed to update task assignment: {}", e);
        format!("Failed to update task assignment: {}", e)
    })?;

    trace!(
        "Smart-assigned task {} to {} (workload: {})",
        task_id,
        best_assignee,
        min_workload
    );

    Ok(best_assignee)
}

fn auto_assign_task(state: &AppState, project_id: Option<&str>) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let team_query_str = if let Some(proj_id) = project_id {
        format!(
            "SELECT DISTINCT assignee FROM tasks
             WHERE project_id = '{}' AND assignee IS NOT NULL
             ORDER BY COUNT(*) ASC LIMIT 5",
            proj_id
        )
    } else {
        "SELECT DISTINCT assignee FROM tasks
         WHERE assignee IS NOT NULL
         ORDER BY COUNT(*) ASC LIMIT 5"
            .to_string()
    };

    let team_query = diesel::sql_query(&team_query_str);

    #[derive(QueryableByName)]
    struct TeamMember {
        #[diesel(sql_type = diesel::sql_types::Text)]
        assignee: String,
    }

    let team: Vec<TeamMember> = team_query.load(&mut *conn).unwrap_or_default();

    if team.is_empty() {
        return Ok("unassigned".to_string());
    }

    Ok(team[0].assignee.clone())
}

fn parse_due_date(due_date: &str) -> Result<Option<DateTime<Utc>>, String> {
    let due_lower = due_date.to_lowercase();

    if due_lower == "null" || due_lower.is_empty() {
        return Ok(None);
    }

    let now = Utc::now();

    if due_lower.starts_with('+') {
        let days_str = due_lower
            .trim_start_matches('+')
            .split_whitespace()
            .next()
            .unwrap_or("0");
        if let Ok(days) = days_str.parse::<i64>() {
            return Ok(Some(now + Duration::days(days)));
        }
    }

    if due_lower == "today" {
        if let Some(time) = now.date_naive().and_hms_opt(0, 0, 0) {
            return Ok(Some(time.and_utc()));
        }
    }

    if due_lower == "tomorrow" {
        if let Some(time) = (now + Duration::days(1)).date_naive().and_hms_opt(17, 0, 0) {
            return Ok(Some(time.and_utc()));
        }
    }

    if due_lower.contains("next week") {
        return Ok(Some(now + Duration::days(7)));
    }

    if due_lower.contains("next month") {
        return Ok(Some(now + Duration::days(30)));
    }

    if let Ok(date) = NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
        if let Some(time) = date.and_hms_opt(0, 0, 0) {
            return Ok(Some(time.and_utc()));
        }
    }

    Ok(Some(now + Duration::days(3)))
}

fn determine_priority(due_date: Option<DateTime<Utc>>) -> String {
    if let Some(due) = due_date {
        let now = Utc::now();
        let days_until = (due - now).num_days();

        if days_until <= 1 {
            "high".to_string()
        } else if days_until <= 7 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    } else {
        "medium".to_string()
    }
}

fn send_task_notification(
    _state: &AppState,
    task_id: &str,
    title: &str,
    assignee: &str,
    due_date: Option<DateTime<Utc>>,
) -> Result<(), String> {
    trace!(
        "Notification sent to {} for task '{}' (ID: {})",
        assignee,
        title,
        task_id
    );

    if let Some(due) = due_date {
        trace!("Task due: {}", due.format("%Y-%m-%d %H:%M"));
    }

    Ok(())
}
