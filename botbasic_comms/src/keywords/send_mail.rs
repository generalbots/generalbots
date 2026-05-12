use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn send_mail_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("SEND_MAIL", |_to: &str, _subject: &str, _body: &str| -> String {
        "SEND_MAIL (stub)".to_string()
    });
}
