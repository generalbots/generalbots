use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botmarketing::triggers;

pub fn configure_marketing_routes(state: &Arc<AppState>) -> Router {
    let mkt_state = botmarketing::AppState::new(
        Arc::new(state.conn.clone()),
        Arc::new(|conn: &mut diesel::PgConnection| crate::core::bot::get_default_bot(conn)),
        Arc::new(|_to: &str, _subject: &str, _body: &str, _bot_id: uuid::Uuid, _from: Option<&str>| -> Result<String, String> {
            Err("send_email not available in marketing shim".to_string())
        }),
        Arc::new(|_bot_id: uuid::Uuid, _to: &str, _body: &str, _from: Option<&str>, _media: Option<&str>| -> Result<String, String> {
            Err("send_whatsapp not available in marketing shim".to_string())
        }),
        Arc::new(|bot_id: &uuid::Uuid, key: &str, default: Option<&str>| -> Result<String, String> {
            let config = crate::core::shared::config::ConfigManager::new(state.conn.clone());
            Ok(config.get_config(bot_id, key, default).unwrap_or_default())
        }),
        Arc::new(|_prompt: &str, _params: &serde_json::Value, _model: &str, _system: &str| -> Result<String, String> {
            Err("llm_generate not available in marketing shim".to_string())
        }),
    );
    botmarketing::routes::configure_marketing_routes()
        .with_state(Arc::new(mkt_state))
}
