







































use botbasic_types::UserSession;
use botbasic_types::BasicRuntime;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::social::register_social_media_keywords as register_social_keywords_impl;















































pub fn register_social_media_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering social media keywords...");


    register_social_keywords_impl(state, user, engine);

    debug!("Social media keywords registered successfully");
}
