pub use botwhatsapp::*;

use std::sync::Arc;

use axum::Router;
use botcore::shared::state::AppState;

pub fn configure(app_state: &Arc<AppState>) -> Router<()> {
    let pool = app_state.conn.clone();

    let wa_state = Arc::new(WhatsAppState {
        pool,
        send_message: Arc::new(|_phone: &str, _message: &str, _bot_name: &str| {
            Box::pin(async move { Ok(()) })
        }),
        get_default_bot: Arc::new(|_c: &mut diesel::PgConnection| {
            uuid::Uuid::nil()
        }),
        find_bot: Arc::new(|_phone: &str| (uuid::Uuid::nil(), "default".to_string())),
        get_config: Arc::new(|_key: &str| -> Result<String, String> {
            Ok("stub".to_string())
        }),
        secrets: Arc::new(|_key: &str| -> Result<String, String> { Ok("stub".to_string()) }),
        transcribe_audio: Arc::new(|_data: &[u8]| {
            Box::pin(async move { Err("Audio transcription not available".to_string()) })
        }),
        process_message: Arc::new(
            |_phone: String, _message: String, _bot_name: String| {
                Box::pin(async move { Ok(()) })
            },
        ),
        user_lookup: Arc::new(|_identifier: &str| {
            Box::pin(async move { Ok(None::<String>) })
        }),
        user_create: Arc::new(
            |_identifier: &str, _display_name: &str, _email: &str, _phone: Option<&str>| {
                Box::pin(async move { Ok("00000000-0000-0000-0000-000000000000".to_string()) })
            },
        ),
    });

    botwhatsapp::configure_whatsapp_routes().with_state(wa_state)
}
