







































use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::social::register_social_media_keywords as register_social_keywords_impl;















































pub fn register_social_media_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering social media keywords...");


    register_social_keywords_impl(state, user, engine);

    debug!("Social media keywords registered successfully");
}
