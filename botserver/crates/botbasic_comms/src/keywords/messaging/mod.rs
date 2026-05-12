pub mod send_template;

use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

pub fn register_messaging_keywords(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    send_template::send_template_keyword(state.clone(), user.clone(), engine);
    send_template::send_template_to_keyword(state.clone(), user.clone(), engine);
    send_template::create_template_keyword(state.clone(), user.clone(), engine);
    send_template::get_template_keyword(state, user, engine);

    debug!("Registered all messaging keywords");
}
