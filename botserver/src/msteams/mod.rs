use crate::core::bot::{get_default_bot, BotOrchestrator};
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botmsteams::{
    adapter::TeamsAdapter,
    channel::ChannelAdapter,
    handlers,
    session,
    state::ChannelState,
    types,
    webhook,
};

fn make_channel_state(app_state: &Arc<AppState>) -> Arc<ChannelState> {
    let conn = Arc::new(app_state.conn.clone());
    let get_default_bot_fn: botmsteams::state::GetDefaultBotFn =
        Arc::new(|c| get_default_bot(c));

    let config_state = app_state.clone();
    let get_config: botmsteams::state::GetConfigFn = Arc::new(
        move |bot_id: &uuid::Uuid, key: &str, default: Option<&str>| -> Result<String, String> {
            let config_manager = crate::core::config::ConfigManager::new(config_state.conn.clone());
            config_manager
                .get_config(bot_id, key, default)
                .map_err(|e| e.to_string())
        },
    );

    let stream_state = app_state.clone();
    let stream_response: botmsteams::state::StreamResponseFn = Arc::new(
        move |user_message: botlib::models::UserMessage, tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
            let state = stream_state.clone();
            tokio::spawn(async move {
                let orchestrator = BotOrchestrator::new(state);
                orchestrator
                    .stream_response(user_message, tx)
                    .await
                    .map_err(|e| e.to_string())
            })
        },
    );

    let attendant_broadcast = app_state.attendant_broadcast.clone();

    Arc::new(ChannelState {
        conn,
        get_default_bot: get_default_bot_fn,
        get_config,
        stream_response,
        attendant_broadcast,
    })
}

pub fn configure(app_state: Arc<AppState>) -> Router {
    botmsteams::webhook::configure().with_state(make_channel_state(&app_state))
}
