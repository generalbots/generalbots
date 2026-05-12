use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn post_to_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("POST_TO", |_platform: &str, _message: &str| -> String {
        "POST_TO (stub)".to_string()
    });
}
