pub mod send_template;

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_messaging_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    send_template::send_template_keyword(state.clone(), user.clone(), engine);
    send_template::send_template_to_keyword(state.clone(), user.clone(), engine);
    send_template::create_template_keyword(state.clone(), user.clone(), engine);
    send_template::get_template_keyword(state, user, engine);

    debug!("Registered all messaging keywords");
}
