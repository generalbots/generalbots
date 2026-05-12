use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::Engine;
use std::sync::Arc;

pub fn register_db_api_keywords(
    _state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    _engine: &mut Engine,
) {
}
