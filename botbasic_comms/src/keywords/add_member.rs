use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn add_member_keyword(state: Arc<dyn BasicRuntime>, _user: UserSession, engine: &mut Engine) {
    let _ = state;
    engine.register_fn("ADD_MEMBER", |group: &str, email: &str, role: &str| -> String {
        format!("ADD_MEMBER: {} to {} as {} (stub)", email, group, role)
    });
}
