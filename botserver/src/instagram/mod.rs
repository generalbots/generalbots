use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botinstagram::{
    adapter::InstagramAdapter,
    channel::ChannelAdapter,
    handlers,
    state::ChannelState,
    types,
    webhook,
};

fn make_channel_state(app_state: &Arc<AppState>) -> Arc<ChannelState> {
    let config_state = app_state.clone();
    let get_config: botinstagram::state::GetConfigFn = Arc::new(
        move |bot_id: &str, key: &str, default: Option<&str>| -> Result<String, String> {
            let config_manager = crate::core::config::ConfigManager::new(config_state.conn.clone());
            let bot_uuid = uuid::Uuid::parse_str(bot_id).unwrap_or(uuid::Uuid::nil());
            config_manager
                .get_config(&bot_uuid, key, default)
                .map_err(|e| e.to_string())
        },
    );

    let stream_state = app_state.clone();
    let stream_response: botinstagram::state::StreamResponseFn = Arc::new(
        move |user_message: botlib::models::UserMessage, tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
            let state = stream_state.clone();
            tokio::spawn(async move {
                let orchestrator = crate::core::bot::BotOrchestrator::new(state);
                orchestrator
                    .stream_response(user_message, tx)
                    .await
                    .map_err(|e| e.to_string())
            })
        },
    );

    let attendant_broadcast = app_state.attendant_broadcast.clone();

    Arc::new(ChannelState {
        get_config,
        stream_response,
        attendant_broadcast,
    })
}

pub fn configure(app_state: Arc<AppState>) -> Router {
    botinstagram::webhook::configure().with_state(make_channel_state(&app_state))
}
