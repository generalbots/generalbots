





































use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use crate::keywords::messaging::register_messaging_keywords;

























































pub fn register_send_template_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering send template keywords...");


    register_messaging_keywords(state, user, engine);

    debug!("Send template keywords registered successfully");
}
