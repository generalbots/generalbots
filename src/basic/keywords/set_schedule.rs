use diesel::prelude::*;
use log::{trace};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::shared::models::TriggerKind;
pub fn execute_set_schedule(conn: &mut diesel::PgConnection, cron: &str, script_name: &str, bot_uuid: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
 trace!("Scheduling SET SCHEDULE cron: {}, script: {}, bot_id: {:?}", cron, script_name, bot_uuid);
 use crate::shared::models::bots::dsl::bots;
 let bot_exists: bool = diesel::select(diesel::dsl::exists(bots.filter(crate::shared::models::bots::dsl::id.eq(bot_uuid)))).get_result(conn)?;
 if !bot_exists {
 return Err(format!("Bot with id {} does not exist", bot_uuid).into());
 }
 use crate::shared::models::system_automations::dsl::*;
 let new_automation = (
 bot_id.eq(bot_uuid),
 kind.eq(TriggerKind::Scheduled as i32),
 schedule.eq(cron),
 param.eq(script_name),
 is_active.eq(true),
 );
 let update_result = diesel::update(system_automations)
 .filter(bot_id.eq(bot_uuid))
 .filter(kind.eq(TriggerKind::Scheduled as i32))
 .filter(param.eq(script_name))
 .set((
 schedule.eq(cron),
 is_active.eq(true),
 last_triggered.eq(None::<chrono::DateTime<chrono::Utc>>),
 ))
 .execute(&mut *conn)?;
 let result = if update_result == 0 {
 diesel::insert_into(system_automations).values(&new_automation).execute(&mut *conn)?
 } else {
 update_result
 };
 Ok(json!({
 "command": "set_schedule",
 "schedule": cron,
 "script": script_name,
 "bot_id": bot_uuid.to_string(),
 "rows_affected": result
 }))
}
