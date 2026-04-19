





































use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::messaging::register_messaging_keywords;

























































pub fn register_send_template_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering send template keywords...");


    register_messaging_keywords(state, user, engine);

    debug!("Send template keywords registered successfully");
}
