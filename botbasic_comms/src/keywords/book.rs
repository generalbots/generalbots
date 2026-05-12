use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn book_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("BOOK", |title: &str, date: &str| -> String {
        format!("BOOK: {} on {} (stub)", title, date)
    });
}
