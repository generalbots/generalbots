use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_bot_keywords(state: Arc<dyn BasicRuntime>, _user: &UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("ADD_BOT", |name: &str, trigger: &str| -> String {
        format!("ADD_BOT: {} with trigger {} (stub)", name, trigger)
    });
    engine.register_fn("REMOVE_BOT", |name: &str| -> String {
        format!("REMOVE_BOT: {} (stub)", name)
    });
    engine.register_fn("LIST_BOTS", || -> String { "LIST_BOTS (stub)".to_string() });
    engine.register_fn("DELEGATE_TO", |name: &str| -> String {
        format!("DELEGATE_TO: {} (stub)", name)
    });
}
