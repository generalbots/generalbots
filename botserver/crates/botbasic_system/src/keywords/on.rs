use std::sync::Arc;
use botlib::models::TriggerKind;
use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use diesel::prelude::*;
use log::trace;
use rhai::Dynamic;
use rhai::Engine;
use serde_json::{json, Value};
use uuid::Uuid;
pub fn register_on_keywords(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    on_keyword(&state, user, engine);
}

pub fn on_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let bot_uuid = user.bot_id;
    let branch_uuid = user.branch_id;
    let registration = engine.register_custom_syntax(
            ["ON", "$ident$", "OF", "$string$"],
            true,
            move |context, inputs| {
                let trigger_type = context.eval_expression_tree(&inputs[0])?.to_string();
                let table = context.eval_expression_tree(&inputs[1])?.to_string();
                let name = format!("{}_{}.bas", table, trigger_type.to_lowercase());
                let kind = match trigger_type.to_uppercase().as_str() {
                    "UPDATE" => TriggerKind::TableUpdate,
                    "INSERT" => TriggerKind::TableInsert,
                    "DELETE" => TriggerKind::TableDelete,
                    _ => return Err(format!("Invalid trigger type: {}", trigger_type).into()),
                };
                trace!(
                    "Starting execute_on_trigger with kind: {:?}, table: {}, param: {}",
                    kind,
                    table,
                    name
                );
                let mut conn = state_clone
                    .db_pool()
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;
                let result =
                    execute_on_trigger(&mut conn, kind, &table, &name, bot_uuid, branch_uuid)
                        .map_err(|e| format!("DB error: {}", e))?;
                if let Some(rows_affected) = result.get("rows_affected") {
                    Ok(Dynamic::from(rows_affected.as_i64().unwrap_or(0)))
                } else {
                    Err("No rows affected".into())
                }
            },
    );
    if let Err(e) = registration {
        log::error!("Failed to register ON keyword: {e}");
    }
}
pub fn execute_on_trigger(
    conn: &mut diesel::PgConnection,
    kind: TriggerKind,
    table: &str,
    param: &str,
    bot: Uuid,
    branch: Uuid,
) -> Result<Value, String> {
    use botschema::system_automations;
    // bot_id and branch_id are both NOT NULL: bot_id scopes the trigger to the bot
    // and branch_id (migration 6.5.23) scopes it to the branch. ON CONFLICT keeps
    // re-registration idempotent instead of raising a duplicate key error.
    let new_automation = (
        system_automations::bot_id.eq(bot),
        system_automations::branch_id.eq(branch),
        system_automations::kind.eq(kind as i32),
        system_automations::target.eq(table),
        system_automations::param.eq(param),
    );
    let result = diesel::insert_into(system_automations::table)
        .values(&new_automation)
        .on_conflict((
            system_automations::bot_id,
            system_automations::kind,
            system_automations::param,
        ))
        .do_nothing()
        .execute(conn)
        .map_err(|e| {
            log::error!("SQL execution error: {}", e);
            e.to_string()
        })?;
    Ok(json!({
    "command": "on_trigger",
    "trigger_type": format!("{:?}", kind),
    "table": table,
    "param": param,
    "rows_affected": result
    }))
}
