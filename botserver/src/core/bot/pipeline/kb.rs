use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

pub async fn inject_kb(
    state: &Arc<AppState>,
    _bot_uuid: Uuid,
    session_id: Uuid,
    _user_id: Uuid,
    _bot_name: &str,
    user_text: &str,
    messages: &mut serde_json::Value,
) -> Result<(), String> {
    let bot_uuid = _bot_uuid;
    crate::core::bot::kb_context::inject_kb_context(
        &state.conn,
        session_id,
        bot_uuid,
        user_text,
        messages,
        4000,
    )
    .await;
    Ok(())
}