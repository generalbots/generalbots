use diesel::prelude::*;
use diesel::sql_query;
use crate::models::BotRow;

pub fn get_bot_for_phone(
    pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>,
    _sender_id: &str,
) -> Option<String> {
    let mut conn = pool.get().ok()?;
    let bot: Option<BotRow> = sql_query(
        "SELECT id::text, name FROM bots WHERE is_public = true ORDER BY created_at LIMIT 1"
    ).get_result(&mut conn).ok();
    bot.map(|b| b.name)
}
