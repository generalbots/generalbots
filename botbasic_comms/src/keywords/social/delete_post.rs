use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn delete_post_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("DELETE_POST", |platform: &str, id: &str| -> String {
        format!("DELETE_POST on {} id {} (stub)", platform, id)
    });
}
