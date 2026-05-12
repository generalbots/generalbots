use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_messaging_keywords(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("SEND_MESSAGE", |_to: &str, _msg: &str| -> String {
        "SEND_MESSAGE (stub)".to_string()
    });
}
