use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_universal_messaging(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("SEND_TO", |_recipient: &str, _message: &str| -> String {
        "SEND_TO (stub)".to_string()
    });
    engine.register_fn("SEND_FILE_TO", |_recipient: &str, _file: &str| -> String {
        "SEND_FILE_TO (stub)".to_string()
    });
}
