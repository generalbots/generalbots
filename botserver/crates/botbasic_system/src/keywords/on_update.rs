use diesel::prelude::*;
use log::trace;
use uuid::Uuid;

pub fn execute_on_update_registration(
    conn: &mut diesel::PgConnection,
    table_name: &str,
    script_name: &str,
    bot_uuid: Uuid,
    trigger_kind_val: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!(
        "Registering ON UPDATE OF trigger: table={}, script={}, bot_id={:?}, kind={}",
        table_name,
        script_name,
        bot_uuid,
        trigger_kind_val
    );

    use botschema::system_automations::dsl::*;

    let new_automation = (
        bot_id.eq(bot_uuid),
        kind.eq(trigger_kind_val),
        target.eq(table_name),
        param.eq(script_name),
        is_active.eq(true),
    );

    diesel::insert_into(system_automations)
        .values(&new_automation)
        .on_conflict((bot_id, kind, param))
        .do_nothing()
        .execute(conn)?;

    Ok(())
}
