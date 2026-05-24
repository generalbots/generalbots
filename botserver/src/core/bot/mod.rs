pub use botcorebot::*;
pub mod ws_handler;
pub mod manager;
pub mod tool_context;
pub mod multimedia;

pub use ws_handler::{websocket_handler, websocket_handler_with_bot};
use std::collections::HashMap;
use std::sync::Arc;
use axum::response::IntoResponse;
use uuid::Uuid;

pub mod channels {
    pub use botlib::traits::ChannelAdapter;

    #[derive(Debug)]
    pub struct VoiceAdapter;

    #[derive(Debug)]
    pub struct WebChannelAdapter;

    impl Default for VoiceAdapter {
    fn default() -> Self { Self::new() }
}

impl VoiceAdapter {
        pub fn new() -> Self { Self }
    }

    impl Default for WebChannelAdapter {
    fn default() -> Self { Self::new() }
}

impl WebChannelAdapter {
        pub fn new() -> Self { Self }
    }

    impl ChannelAdapter for VoiceAdapter {
        fn channel_type(&self) -> &str { "voice" }
        fn send_message(&self, to: &str, _message: &str) -> Result<(), String> {
            log::warn!("VoiceAdapter::send_message stub: to={to}");
            Ok(())
        }
    }

    impl ChannelAdapter for WebChannelAdapter {
        fn channel_type(&self) -> &str { "web" }
        fn send_message(&self, to: &str, _message: &str) -> Result<(), String> {
            log::warn!("WebChannelAdapter::send_message stub: to={to}");
            Ok(())
        }
    }

    pub mod whatsapp {
        pub struct WhatsAppAdapter;
        impl WhatsAppAdapter {
            pub fn new(_state: &botcore::shared::state::AppState, _bot_id: uuid::Uuid) -> Self { Self }
            pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                log::warn!("WhatsAppAdapter::send_message stub");
                Ok(())
            }
        }
    }
    pub mod instagram {
        pub struct InstagramAdapter;
    impl Default for InstagramAdapter {
    fn default() -> Self { Self::new() }
}

impl InstagramAdapter {
        pub fn new() -> Self { Self }
        pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            log::warn!("InstagramAdapter::send_message stub");
            Ok(())
        }
        pub async fn send_instagram_message(&self, _recipient: &str, _message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            log::warn!("InstagramAdapter::send_instagram_message stub");
            Ok(())
        }
    }
    }
    pub mod teams {
        pub struct TeamsAdapter;
        impl Default for TeamsAdapter {
    fn default() -> Self { Self::new() }
}

impl TeamsAdapter {
            pub fn new() -> Self { Self }
            pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                log::warn!("TeamsAdapter::send_message stub");
                Ok(())
            }
        }
    }
}

pub mod kb_context;

pub struct BotOrchestrator;

impl BotOrchestrator {
    pub fn new(_state: std::sync::Arc<botcore::shared::state::AppState>) -> Self { Self }
    pub fn mount_all_bots(&self) -> Result<(), String> { 
        log::info!("BotOrchestrator::mount_all_bots stub"); 
        Ok(()) 
    }
}

pub fn get_default_bot() -> (String, String) { ("default".to_string(), "Default Bot".to_string()) }
pub async fn get_bot_config(
    axum::extract::State(state): axum::extract::State<Arc<botcore::shared::state::AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    use diesel::prelude::*;
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "db error".to_string()),
    };
    use botcore::shared::models::schema::bot_configuration::dsl::*;
    use botcore::shared::models::schema::bots;

    let bot_uuid = if let Some(name) = params.get("bot_name") {
        bots::table
            .filter(bots::name.eq(name))
            .select(bots::id)
            .first::<Uuid>(&mut conn)
            .ok()
    } else {
        None
    };

    let rows: Vec<(String, String)> = if let Some(bid) = bot_uuid {
        bot_configuration
            .select((config_key, config_value))
            .filter(bot_id.eq(bid))
            .load(&mut conn)
    } else {
        bot_configuration
            .select((config_key, config_value))
            .load(&mut conn)
    }
    .unwrap_or_default();

    let map: HashMap<String, String> = rows.into_iter().collect();
    (axum::http::StatusCode::OK, serde_json::to_string(&map).unwrap_or_default())
}

pub async fn check_bot_access(state: &Arc<botcore::shared::state::AppState>, bot_name: &str, user_id: Uuid) -> Result<(), String> {
    use diesel::prelude::*;
    use botcore::shared::schema::bots::dsl as bots_dsl;
    use botcore::shared::schema::user_organizations::dsl as uo_dsl;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    let bot_record = bots_dsl::bots
        .filter(bots_dsl::name.eq(bot_name))
        .select((bots_dsl::is_public, bots_dsl::org_id))
        .first::<(bool, Option<Uuid>)>(&mut *conn)
        .optional()
        .map_err(|e| format!("DB query error: {}", e))?;

    let (is_public, org_id) = match bot_record {
        Some(record) => record,
        None => return Err("Bot not found".to_string()),
    };

    if is_public {
        return Ok(());
    }

    if let Some(org_id) = org_id {
        let is_member = uo_dsl::user_organizations
            .filter(uo_dsl::user_id.eq(user_id))
            .filter(uo_dsl::org_id.eq(org_id))
            .count()
            .get_result::<i64>(&mut *conn)
            .map_err(|e| format!("DB query error: {}", e))? > 0;

        if is_member {
            return Ok(());
        }
    }

    Err("Access denied".to_string())
}

pub async fn check_access_handler(
    axum::extract::State(state): axum::extract::State<Arc<botcore::shared::state::AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let user_id = match req.extensions().get::<Uuid>().copied() {
        Some(id) => id,
        None => return axum::http::StatusCode::UNAUTHORIZED.into_response(),
    };

    match check_bot_access(&state, &bot_name, user_id).await {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(_) => axum::http::StatusCode::FORBIDDEN.into_response(),
    }
}
