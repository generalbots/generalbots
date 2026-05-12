use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn get_posts_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("GET_POSTS", |platform: &str| -> String {
        format!("GET_POSTS from {} (stub)", platform)
    });
}
