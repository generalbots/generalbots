use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_sms_keywords(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("SEND_SMS", |phone: &str, message: &str| -> String {
        format!("SEND_SMS to {}: {} (stub)", phone, message)
    });
}
