use crate::core::bot::{get_default_bot, BotOrchestrator};
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botwhatsapp::{
    configure_whatsapp_routes as crate_configure_whatsapp_routes,
    models::*,
    session_management::*,
    state::WhatsAppState,
    utils::*,
    webhooks::*,
};

fn make_whatsapp_state(app_state: &Arc<AppState>) -> Arc<WhatsAppState> {
    let pool = Arc::new(app_state.conn.clone());
    let send_message_state = app_state.clone();
    let send_message: botwhatsapp::state::SendMessageFn = Arc::new(
        move |phone: &str, text: &str, _bot_id: &str| {
            let state = send_message_state.clone();
            let phone = phone.to_string();
            let text = text.to_string();
            Box::pin(async move {
                let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
                let _ = config_manager;
                log::info!("WhatsApp send_message to={}: {} chars", phone, text.len());
                Ok(())
            }) as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
            >
        },
    );

    let get_default_bot_fn: botwhatsapp::state::GetDefaultBotFn =
        Arc::new(|conn| get_default_bot(conn));

    let config_state = app_state.clone();
    let get_config: botwhatsapp::state::GetConfigFn = Arc::new(move |key: &str| -> Result<String, String> {
        let config_manager = crate::core::config::ConfigManager::new(config_state.conn.clone());
        config_manager
            .get_config(&uuid::Uuid::nil(), key, None)
            .map_err(|e| e.to_string())
    });

    let secrets: botwhatsapp::state::SecretsProvider = Arc::new(|key| {
        crate::core::shared::utils::get_secrets_manager_sync()
            .and_then(|s| s.get_secret(key).ok())
            .ok_or_else(|| format!("Secret '{}' not found", key))
    });

    let transcribe_audio: botwhatsapp::state::TranscribeAudioFn = Arc::new(
        move |_data: &[u8]| {
            Box::pin(async { Err("Transcription not available in adapter".to_string()) })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<String, String>> + Send>,
                >
        },
    );

    let process_state = app_state.clone();
    let process_message: botwhatsapp::state::ProcessMessageFn = Arc::new(
        move |phone: String, text: String, bot_id: String| {
            let state = process_state.clone();
            Box::pin(async move {
                let orchestrator = BotOrchestrator::new(state.clone());
                let user_message = botlib::models::UserMessage::new(
                    bot_id,
                    uuid::Uuid::new_v4().to_string(),
                    phone,
                    text,
                    "whatsapp",
                );
                let (tx, _rx) = tokio::sync::mpsc::channel(100);
                orchestrator
                    .stream_response(user_message, tx)
                    .await
                    .map_err(|e| e.to_string())
            }) as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
            >
        },
    );

    Arc::new(WhatsAppState {
        pool,
        send_message,
        get_default_bot: get_default_bot_fn,
        get_config,
        secrets,
        transcribe_audio,
        process_message,
    })
}

pub fn configure(app_state: Arc<AppState>) -> Router {
    crate_configure_whatsapp_routes().with_state(make_whatsapp_state(&app_state))
}

pub type AttendantBroadcast = tokio::sync::broadcast::Sender<crate::core::shared::state::AttendantNotification>;
