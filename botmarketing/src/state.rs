use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::PgConnection;
use std::sync::Arc;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut PgConnection) -> (uuid::Uuid, String) + Send + Sync>;

pub type SendEmailFn = Arc<dyn Fn(&str, &str, &str, uuid::Uuid, Option<&str>) -> Result<String, String> + Send + Sync>;

pub type SendWhatsAppFn = Arc<dyn Fn(uuid::Uuid, &str, &str, Option<&str>, Option<&str>) -> Result<String, String> + Send + Sync>;

pub type GetConfigFn = Arc<dyn Fn(&uuid::Uuid, &str, Option<&str>) -> Result<String, String> + Send + Sync>;

pub type LlmGenerateFn = Arc<dyn Fn(&str, &serde_json::Value, &str, &str) -> Result<String, String> + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub conn: Arc<DbPool>,
    pub get_default_bot: GetDefaultBotFn,
    pub send_email: SendEmailFn,
    pub send_whatsapp: SendWhatsAppFn,
    pub get_config: GetConfigFn,
    pub llm_generate: LlmGenerateFn,
}

impl AppState {
    pub fn new(
        conn: Arc<DbPool>,
        get_default_bot: GetDefaultBotFn,
        send_email: SendEmailFn,
        send_whatsapp: SendWhatsAppFn,
        get_config: GetConfigFn,
        llm_generate: LlmGenerateFn,
    ) -> Self {
        Self {
            conn,
            get_default_bot,
            send_email,
            send_whatsapp,
            get_config,
            llm_generate,
        }
    }

    pub fn get_bot_context(&self) -> (uuid::Uuid, uuid::Uuid) {
        use crate::schema::bots;

        let Ok(mut conn) = self.conn.get() else {
            return (uuid::Uuid::nil(), uuid::Uuid::nil());
        };
        let (bot_id, _bot_name) = (self.get_default_bot)(&mut conn);

        let org_id = bots::table
            .filter(bots::id.eq(bot_id))
            .select(bots::org_id)
            .first::<Option<uuid::Uuid>>(&mut conn)
            .unwrap_or(None)
            .unwrap_or(uuid::Uuid::nil());

        (org_id, bot_id)
    }
}
