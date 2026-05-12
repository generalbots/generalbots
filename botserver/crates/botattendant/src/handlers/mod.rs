pub mod agents;
pub mod queues;
pub mod sessions;

use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::*;

pub(crate) fn get_bot_context(config: &crate::AttendantConfig) -> (Uuid, Uuid) {
    let Ok(mut conn) = config.pool.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = (config.get_default_bot)(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

pub(crate) fn generate_session_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("SES-{:06}", count + 1)
}

macro_rules! db_conn {
    ($config:expr) => {
        $config.pool.get().map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
        })?
    };
}

pub(crate) use db_conn;

pub use agents::*;
pub use queues::*;
pub use sessions::*;
