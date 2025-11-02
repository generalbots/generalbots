use diesel::prelude::*;
use log::info;
use rhai::Dynamic;
use rhai::Engine;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::shared::models::TriggerKind;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;

pub fn set_schedule_keyword(state: &AppState, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();

    engine
        .register_custom_syntax(&["SET_SCHEDULE", "$string$", "$string$"], true, {
            move |context, inputs| {
                let cron = context.eval_expression_tree(&inputs[0])?.to_string();
                let script_name = context.eval_expression_tree(&inputs[1])?.to_string();

                let mut conn = state_clone.conn.lock().unwrap();
                let result = execute_set_schedule(&mut *conn, &cron, &script_name, user.bot_id)
                    .map_err(|e| format!("DB error: {}", e))?;

                if let Some(rows_affected) = result.get("rows_affected") {
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("No rows affected".into())
                }
            }
        })
        .unwrap();
}

pub fn execute_set_schedule(
    conn: &mut diesel::PgConnection,
    cron: &str,
    script_name: &str,
    bot_uuid: Uuid,
) -> Result<Value, Box<dyn std::error::Error>> {
    info!(
        "Starting execute_set_schedule with cron: {}, script: {}, bot_id: {:?}",
        cron, script_name, bot_uuid
    );

    use crate::shared::models::system_automations::dsl::*;

    let new_automation = (
        bot_id.eq(bot_uuid),
        kind.eq(TriggerKind::Scheduled as i32),
        schedule.eq(cron),
        param.eq(script_name),
        is_active.eq(true),
    );

    let result = diesel::insert_into(system_automations)
        .values(&new_automation)
        .on_conflict((bot_id, param))
        .do_update()
        .set((
            schedule.eq(cron),
            is_active.eq(true),
            last_triggered.eq(None::<chrono::DateTime<chrono::Utc>>),
        ))
        .execute(&mut *conn)?;

    Ok(json!({
        "command": "set_schedule",
        "schedule": cron,
        "script": script_name,
        "bot_id": bot_uuid.to_string(),
        "rows_affected": result
    }))
}
