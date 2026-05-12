use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_transfer_to_human_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("TRANSFER_TO_HUMAN", || -> String {
        "TRANSFER_TO_HUMAN (stub)".to_string()
    });
}
