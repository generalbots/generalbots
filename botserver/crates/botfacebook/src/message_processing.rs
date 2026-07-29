use std::sync::Arc;
use uuid::Uuid;
use crate::state::FacebookState;

pub async fn process_incoming_message(
    state: &Arc<FacebookState>,
    sender_id: &str,
    text: &str,
    _page_id: &str,
    _channel: &str,
) -> Result<(), String> {
    let (bot_id, bot_name) = (state.get_default_bot)();
    let session_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("fb-session:{}", sender_id).as_bytes());
    let user_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("fb:{}", sender_id).as_bytes());

    (state.process_message)(
        &bot_id.to_string(),
        sender_id,
        text,
        &session_id.to_string(),
        &bot_name,
    ).await
}
