use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn post_to_scheduled_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("POST_SCHEDULED", |_platform: &str, _message: &str, _time: &str| -> String {
        "POST_SCHEDULED (stub)".to_string()
    });
}
